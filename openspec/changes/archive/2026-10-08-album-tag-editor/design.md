# Design

## Context

See proposal.md for motivation and `specs/tag-editing/spec.md` for requirements. This change assumes `cover-resize-and-tag-fields` is applied: the extra fields are written on download, and `artwork::prepare_cover` exists.

- **Queue:** each `QueueItem` holds its `Job` (the Qobuz `Track` and `Album`) and, once done, its file in `path`. Groups are built only at render time by album id. `can_remove` and the running `App.batch` already tell whether a track is settled.
- **Album detail precedent:** the album detail from Search replaces the results list with state in `app/album.rs` and a view in `app/view/album.rs`. The editor follows that pattern on the Queue tab.
- **Tag writing today:** `tagging::write_tags` builds a generic lofty `Tag` and saves it. That replaces the container's whole tag, which suits a fresh download but would drop unlisted tags if used to edit.
- **Safe replacement:** `download::PartFile` is a temp file deleted on drop until `publish` renames it into place. Its `publish` is async (`tokio::fs::rename`).

## Goals / Non-Goals

**Goals:**
- Edits change exactly the fields the user touched, and nothing else in the file.
- A file is never seen half-written.
- One mapping between fields and each container's tags, shared by download, Reset to Qobuz and save.

**Non-Goals:**
- Editing tracks that aren't done, files outside this session's queue, or custom fields.
- Undo after saving: the user can Reset to Qobuz, or re-edit.
- Multiple editors open at once.

## Decisions

### Fields model in core: `TagFields` with one `Option` per field
`tag_edit.rs` defines `TagFields`: one `Option` per listed field, with the cover as `Option<Vec<u8>>` (the front cover's bytes, so a resize can start from each file's own image). It provides two functions:
- `read_fields(path) -> Result<TagFields>`
- `apply_edits(path, &TagEdits) -> Result<()>`, where `TagEdits` maps each field to `Set(value)` or `Clear`. An absent field means Keep. The cover can be `Replace(bytes)` or `Remove`, plus an optional resize (see "Cover resizing at save").

`tagging.rs` gains `fields_from(track, album, disc_track_total) -> TagFields`. Download tagging then writes from it, and Reset to Qobuz fills the editor from it, so both produce the same values.

### Edit native tags, not the generic `Tag`
Converting a file's native tag to lofty's generic `Tag` and back can lose items that have no `ItemKey`. `apply_edits` therefore opens the file through its concrete type, chosen by extension as `tag_type_for` does today:
- `FlacFile` → its `VorbisComments`
- `MpegFile` → its `Id3v2Tag`
- `Mp4File` → its `Ilst`

It changes only the edited items on that native tag and saves only that tag, so every other item is left as it was. Each field maps to its native key with the same table as downloads, including the explicit-flag special cases from `cover-resize-and-tag-fields`.

Combined fields such as ID3's `TRCK` "n/total" go through lofty's accessors (`set_track`, `set_track_total` and similar), never through hand-built strings.

*Alternative:* converting to the generic `Tag` and back with `From`. Rejected because that round-trip is lossy.

### Verified lofty 0.24 calls (task 1.1)
Checked in the lofty 0.24.0 and lofty_attr 0.12.0 sources:
- **Reading:** `FlacFile`, `MpegFile` and `Mp4File` implement `AudioFile::read_from(&mut File, ParseOptions)`. Each native tag gets generated `x()`, `x_mut()`, `set_x(tag)` and `remove_x()` accessors (`vorbis_comments`, `id3v2`, `ilst`).
- **Lossless editing:** each native tag implements `SplitTag::split_tag(self) -> (Remainder, Tag)`. The remainder keeps everything the generic `Tag` can't represent, and `MergeTag::merge_tag(remainder, tag)` rebuilds the native tag. So `apply_edits` uses split → change the generic `Tag` through `Accessor` and `ItemKey` → merge. One field table then serves downloads and edits, with no hand-built ID3 frames or MP4 atoms.
- **Explicit flag:** for ID3 and MP4 it is the generic `ParentalAdvisory`. The MP4 merge turns `"1"` into an integer `rtng`. For Vorbis it stays in the remainder, so it is read and written natively with `VorbisComments::get`/`insert`/`remove("ITUNESADVISORY")` after the merge.
- **Label in ID3:** `TPUB` maps to both `Publisher` and `Label`, and splits as `Publisher`. Reading accepts either key. Setting or clearing the label removes both keys first, so a merge never writes two `TPUB` frames.
- **Pictures:**
  - **FLAC:** pictures live on `FlacFile`, not on its `VorbisComments`. When lofty writes a FLAC file, it deletes every existing PICTURE block and writes the tag's pictures plus the file's. So the FLAC cover is edited on the `FlacFile` through `OggPictureStorage` (`remove_picture_type`, `insert_picture` with `PictureInformation::from_picture(..).unwrap_or_default()`), and the whole `FlacFile` is saved.
  - **ID3:** `APIC` frames split into the generic tag's pictures and merge back unchanged.
  - **MP4:** `covr` atoms have no picture type; the merge marks them `Other`. Every `covr` picture counts as the cover.
- **Saving:** `AudioFile::save_to_path(&self, path, WriteOptions)` on the concrete file, written into the `.partN` copy.

### Field model refinement
- **Fields:** rather than one struct field per tag, `TagFields` maps a `Field` enum (the 17 listed fields) to a normalized string: numbers as digits, dates as `YYYY[-MM[-DD]]`, flags as `"1"` when on and absent when off, plus the cover bytes. Each `Field` knows its kind (text, number, date or flag) for validation. That makes the editor, validation and edits generic over fields.
- **Flags:** "off" is the absence of the flag, so turning explicit or compilation off is a `Clear`.
- **Cover edits are album-wide**, as the spec says, so they aren't part of each file's `TagEdits`. A `CoverPlan` built once from the `CoverEdit` (resizing a Replace there) is passed to every `save_file(path, edits, plan)`.
- **The GUI drives the save loop:** core exposes `save_file` for one file, and the editor sends the next file after each one finishes, rather than calling a `save_files` that saves all files in one blocking call. That keeps one file in flight, as designed, and gives the progress count ("Saving 4 of 20…") without a channel.
- **Skipping unchanged files:** a field edit whose value already matches the file is dropped. A file left with no change, cover included, is not copied or written.

### Atomic save: copy, edit the copy, rename over
For each file with at least one edit:
1. `PartFile::copy_of(path)` copies the file to a process-unique `.partN` next to it, on the same filesystem so the rename is atomic. `std::fs::copy` keeps the permissions, so a read-only file fails at the write step and stays untouched.
2. `apply_edits` runs on the copy. The container is chosen from the original path's extension, as `write_tags` already does for part files.
3. The copy is published over the original. `rename` replaces the target on Unix and on Windows. A failure at any step drops the `PartFile`, which removes the copy.

Saving runs on one `spawn_blocking` task, **one file at a time**, so a hi-res album never needs more than one extra file's worth of disk space. `PartFile` gains a blocking `publish_sync` for this path; the async download path keeps `publish`.

The result is one entry per track: either saved or the error text. The GUI reports it in the editor, for example "Saved 11 of 12 — 1 failed: …", and never changes `QueueItem.status`.

*Alternative:* lofty's in-place save. Rejected: a crash during the rewrite can corrupt a finished file.

### Editor state and field semantics (GUI)
`App.tag_editor: Option<TagEditor>` holds:
- the album id
- per track: `track_id`, `path`, the `TagFields` read from disk, and the text per field
- the album-field texts
- a cover choice: Keep, `Replace(image bytes)` or Remove
- a cover resize choice: `None` or a `CoverSize` (400 / 500 / 600 px)
- a saving flag and the last save report

Whether a field is touched is derived by comparing its text with the original:
- **Non-mixed field:** text equal to the original means Keep. Different non-empty text means Set. Emptied text means Clear.
- **Mixed album field:** shown empty with a "(mixed)" placeholder. Empty means Keep. Clearing it needs the field's explicit Clear (×) button, so "untouched" and "delete everywhere" can't be confused.
- **Explicit and compilation:** yes/no toggles. A mixed value shows as a third, indeterminate state until changed.

Album fields expand to every track's edits at save time. Each track field in a track's expanded row has an "Apply to all" control, which copies that value into the same field of every track.

Validation is pure and unit-tested. Save stays disabled while any field is invalid, while saving, or while the album gains an active track again (for example after being re-queued while the editor is open).

### Opening and reading
Edit tags reads every done track's file on one blocking task. A file that fails to read is listed as not editable, with the reason. Choosing a replacement cover uses `rfd`'s async file dialog filtered to JPEG/PNG. The editor checks that the image decodes, then stores its bytes unchanged; resizing happens at save.

### Cover resizing at save
`TagEdits` carries the cover as two independent parts:
- **What to embed:** Keep, `Replace(bytes)` or Remove.
- **Resize:** `Option<CoverSize>`.

At save, per file, on its blocking task:
1. Take the cover that file would end up with: its own embedded front cover for Keep, or the replacement bytes for Replace.
2. If a size is chosen, pass it through `artwork::prepare_cover`. That's the same Lanczos3, linear-light, 4:4:4 JPEG routine as downloads, which already returns a cover within the size byte for byte.
3. Write the front cover only if the result differs from what the file holds.

Because of step 3, Keep + 500 px on a file whose cover is already 450 px leaves the file unwritten, as `Save writes only edited fields` requires. Each file keeps its own cover, because resizing reads every file's cover rather than one album-wide image. With Replace, the image is processed once and the result shared by every file.

When a replacement is picked, the resize choice defaults to the Settings Cover art size (`config.cover_size`, kept even when Cover art is Off). This matches what a download would embed, and the user can change it before saving. With Keep, the default is Don't resize, so opening the editor never changes covers by itself.

### Layout
The editor uses the Queue tab's content area:
- **Header:** cover thumbnail, album title, then Reset to Qobuz, Save (the primary action) and Close. Close reads "Discard changes" while edits exist.
- **Album form:** label column aligned with Settings.
- **Track list:** one row per track with disc, number, title and artist inputs. An expand control reveals composer, ISRC, explicit and comment.

No Escape shortcut, because it would discard edits silently. All buttons come from `style.rs` builders, colored by role.

## Risks / Trade-offs

- **lofty native APIs differ from expectations** → task 1.1 confirms them before anything is built on them. If saving only one native tag isn't possible, the fallback is to read the whole file, change the tag, and save the whole file to the copy. Since the copy is a temp file, that's still safe.
- **Copying is I/O-heavy:** about 100 MB per hi-res track, so a 20-track album means 2 GB of copying. Saving shows progress ("Saving 4 of 20…") and only writes files that have edits.
- **Modification time and extended attributes** (e.g. macOS Finder tags) are not carried over by copy-and-rename → accepted, and noted in the help text. Tag editors that rewrite files behave the same.
- **Another program has a file open on Windows** → the rename fails, the copy is removed, and the failure is reported per file.
- **Re-queuing an edited album** downloads nothing: its files exist and are left untouched (v1.4.0 rule), so the edits survive.
