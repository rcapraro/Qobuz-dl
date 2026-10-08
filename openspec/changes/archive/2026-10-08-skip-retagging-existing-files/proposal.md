# Proposal

## Why

Queuing a track whose file is already on disk skips the download, as specified. But the engine then fetches the album cover and rewrites the file's tags and embedded art anyway. Re-queuing an album you already have therefore touches every one of its files: modification times change, and any tag edits you made yourself are overwritten with Qobuz's metadata. On 2026-10-08, refreshing the README screenshots re-queued a fully downloaded 30-track album and rewrote all 30 files this way.

Skipping existing files is only safe if an existing file is always a finished one. Today it is not: the downloaded bytes are renamed into place first and tagged afterwards, so a tagging failure, a cancel during the cover fetch, or a quit at the wrong moment leaves an untagged file at the destination. Until now, re-queuing or Retry repaired such a file by tagging it again. Once existing files are skipped, it would stay untagged for good.

## What Changes

- **Tagged before it appears:** a downloaded track is tagged, with its cover art embedded when enabled, while it is still the `.partN` temp file. Only then is it renamed to its destination. If tagging fails or the batch is cancelled first, the temp file is removed and no destination file is created, so a later Retry or re-queue downloads the track again.
- **Already-downloaded files are left untouched:** when a track's destination already exists, the engine treats it as complete without fetching cover art, embedding it or writing tags. The file is neither opened for writing nor touched in any other way.
- **No Tagging step for skipped tracks:** they go straight to Done, still with their delivered-quality label and file path, so the queue, Open folder and the end-of-batch notification behave as before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `audio-download`: adds "Downloaded files are complete when they appear" and "Existing files are left untouched". The existing "skip re-downloading" rule stays as it is.

## Impact

- `crates/qobuz-core/src/download.rs`: `stream_to_file` becomes `stream_to_part`. It hands back the finished temp file (`Transfer::Downloaded(PartFile)`) instead of renaming it, or reports `Transfer::AlreadyPresent`. `PartFile::publish` does the rename.
- `crates/qobuz-core/src/tagging.rs`: `write_tags` takes the destination path as well as the file to write, because the tag container is chosen from the destination's extension, not from `.partN`.
- `crates/qobuz-core/src/engine.rs`: `download_one` tags and publishes a downloaded part, and does nothing for an already-present file. A cancel during the cover fetch now cancels the track instead of finishing it without art.
- A shared test-support module for temp files used by tests in these modules.
- `CLAUDE.md`: the per-track data flow and the `download.rs` summary.
- No GUI change: the GUI already handles a track going from queued straight to done.
