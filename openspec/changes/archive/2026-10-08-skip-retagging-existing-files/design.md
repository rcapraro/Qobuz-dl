# Design

## Context

See proposal.md for motivation and specs/audio-download/spec.md for the requirements.

`download::stream_to_file` returns `Ok(())` early when `dest` exists, the idempotency skip. Otherwise it streams to a `.partN` temp file, guarded by `PartFile`, which deletes the file on drop, and renames it to `dest`. `engine::download_with_progress` wraps it in `with_retry`, races that against cancellation, and returns `(FileUrl, PathBuf, Quality)`. `engine::download_one` then always:
1. sends `JobEvent::Tagging`;
2. fetches the cover through the per-album cache when `embed_art` is set, racing cancellation, which only gives up the artwork;
3. calls `tagging::write_tags(&dest, …)`;
4. builds the delivered label.

So a file reaches `dest` before it is tagged. Nothing tells `download_one` whether the bytes were just written or were already there.

## Goals / Non-Goals

**Goals:**
- A file at its destination is a finished file: downloaded in full and tagged.
- A file found at its destination is never opened for writing.
- A track whose tagging fails or is cancelled leaves no destination file, so a later attempt downloads it again.

**Non-Goals:**
- Verifying an existing file (size, checksum, tags) or repairing one left by an older app version that tagged after the rename.
- An option to force re-tagging. It could be a later setting if wanted.
- Retrying a failed cover fetch. A cover host error still caches "no cover" for the album and its tracks are tagged without art, as today.

## Decisions

### `stream_to_part` hands back the temp file instead of renaming it
`stream_to_file` becomes `stream_to_part` and returns `Result<Transfer>`, where `enum Transfer { Downloaded(PartFile), AlreadyPresent }`. `PartFile` becomes public: `path()` exposes the temp file, and `publish(self, dest)` renames it into place and disarms the guard. An unpublished `PartFile` that is dropped still deletes its file, so every way out of the engine before `publish`, whether an error, a cancel or a panic, leaves nothing behind. `download_with_progress` carries the `Transfer` out of the retry closure alongside the file URL, path and delivered quality.

*Alternative:* passing a tagging callback into `stream_to_file` so it tags before its own rename. Rejected: it would make the download module depend on tagging, cover fetching and cancellation, which belong to the engine.

*Alternative:* checking `dest.exists()` again in `download_one`. Rejected because only the download step knows whether it found the file or wrote it.

### `write_tags` takes the destination for the container choice
lofty identifies the file type from its content, so writing to `.partN` works. But `tag_type_for` picks Vorbis comments, MP4 ilst or ID3v2 from the extension, and `.partN` would fall back to ID3v2 for a FLAC file. `write_tags(file, dest, tags)` writes into `file` and chooses the container from `dest`.

### `download_one` tags and publishes only what it downloaded
```
match transfer {
    Transfer::Downloaded(part) => finish_download(part, &dest, …).await?,
    Transfer::AlreadyPresent => {}
}
```
`finish_download` sends `Tagging`, fetches the cover, writes the tags into the part file and publishes it. An already-present file gets no `Tagging` event, no cover fetch and no `write_tags`. That also saves one cover request per album when the whole album is already on disk. `download_all` then sends `Done` as usual. The GUI already accepts `Downloading → Done`, as the existing "Items completed without transferring bytes still count" scenario shows.

### A cancel during the cover fetch cancels the track
Before, the bytes were already at `dest` when the cover was fetched, so a cancel only gave up the artwork, to avoid stranding an untagged file. Now nothing is at `dest` yet. A cancel returns `Error::Cancelled`, the dropped `PartFile` removes the temp file, and the track reports Cancelled. A re-queue downloads it again with its art. The cover fetch stays raced against cancellation, so a slow cover host still cannot hold up a cancel.

### The `biased` race in `download_with_progress` loses its reason
It polled the download first because the download's last step, the rename, had already landed on disk by the time the future was ready. The retry future now ends with an unpublished temp file, so cancel winning a tie just drops it. The `biased` and its explanation are removed.

### Tests
- `download.rs`: `skips_when_destination_exists` asserts `Transfer::AlreadyPresent`. A new test checks that `publish` moves the file to `dest` and leaves no temp file.
- `tagging.rs`: writing to a `.partN` path whose destination is `.flac` produces Vorbis comments.
- `engine.rs`, through `finish_download`, which runs without the network:
  - a valid part is tagged, sends `Tagging` and is published at `dest`;
  - a part that cannot be tagged returns an error, and neither the temp file nor `dest` exists;
  - a cancel during the cover fetch returns `Error::Cancelled`, and neither the temp file nor `dest` exists.
- Temp-file helpers move into a `#[cfg(test)]` `test_support` module shared by these tests. A guard deletes each file however the test ends.

## Risks / Trade-offs

- [Files tagged by an older version after the rename may be untagged if that tagging failed] → They are skipped like any existing file. Deleting the file and re-queuing repairs it. Accepted as a non-goal.
- [A cancel during the cover fetch discards bytes already downloaded] → At most `concurrency` tracks are lost per cancel, and they are downloaded again on re-queue. In return, a cancelled track never leaves a file without its art.
- [A cover host error still leaves a published file without art] → Unchanged from today and listed as a non-goal.
