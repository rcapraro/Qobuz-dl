# Proposal

## Why

Once a track is downloaded, its tags can only be changed in another program. Since v1.4.0, re-queuing an album deliberately leaves existing files alone, so there is no way left inside the app to fix or refresh a downloaded album's tags. Users want to correct metadata the way a tagger does: fix one track's title, or set the album title, year or genre on a whole album at once.

Scope decided on 2026-10-08: only albums in the current session's queue, only their done tracks, and a fixed list of fields. This builds on `cover-resize-and-tag-fields`, which adds the field list to downloads and the cover resizing pipeline.

## What Changes

- **Edit tags:** each queue album group with done tracks gets an Edit tags control. It opens an editor in place of the queue list showing those files' current tags, read from disk.
- **Album fields, edited in batch:** album, album artist, date, genre, label, copyright, total discs, total tracks, compilation and cover are shown once for the album. A field whose value differs between tracks shows as mixed and is left alone unless edited.
- **Track fields:** title, artist, track number, disc number, composer, ISRC, explicit and comment can be edited per track, or set on every track at once.
- **Clear a field:** a field can be cleared explicitly. That's separate from leaving it untouched.
- **Reset to Qobuz:** fills the fields from the Qobuz metadata held in the queue. Nothing is written until saved.
- **Cover:** replace it with a local image file, or remove it.
- **Resize the cover:** shrink the embedded cover to 400, 500 or 600 px, using the same high-quality downscaling as downloads (Lanczos3 in linear light, 4:4:4 JPEG). It applies to each file's own cover or to a replacement, and never enlarges.
- **Save:** writes only the edited fields and keeps every other tag in the file. Each file is replaced atomically, so a failure leaves it exactly as it was.

## Capabilities

### New Capabilities

- `tag-editing`: viewing and editing the tags of downloaded files, in batch and per track, and saving them safely.

### Modified Capabilities

- `downloader-gui`: adds an Edit tags control to the album group header, available only while the group has no track in progress.

## Impact

- New `crates/qobuz-core/src/tag_edit.rs`: read a file's fields; apply edits to a copy of the file through each container's native tag; atomic replace. Re-exported in `lib.rs`.
- `crates/qobuz-core/src/download.rs`: `PartFile` gains a way to be created as a copy of an existing file, next to its original.
- `crates/qobuz-core/src/artwork.rs` (from `cover-resize-and-tag-fields`): `prepare_cover` reused to resize embedded and replacement covers.
- New `crates/qobuz-gui/src/app/tag_editor.rs` (state and pure logic) and `app/view/tag_editor.rs` (view); `app.rs` messages; `app/tasks.rs` blocking wrappers; `rfd` (already a dependency) for the image picker.
- The audio-download rule "Existing files are left untouched" is unchanged. It covers downloading, and the editor writes only when the user saves.
