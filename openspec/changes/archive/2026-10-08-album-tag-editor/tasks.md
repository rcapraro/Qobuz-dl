# Tasks

Requires `cover-resize-and-tag-fields` applied first.

## 1. Verify before building

- [x] 1.1 Confirm from the lofty 0.24.0 source in the cargo registry how to: read `FlacFile`/`MpegFile`/`Mp4File` with their native tags; insert, replace and remove items on `VorbisComments`, `Id3v2Tag` and `Ilst` (including `TXXX:ITUNESADVISORY`, `rtng`, `TRCK`/`TPOS` totals and front-cover pictures); and save one native tag without touching others. Record the calls, and any difference from design.md, in the design.

## 2. Core

- [x] 2.1 Add `TagFields`, `TagEdits` (Set/Clear per field; cover Replace/Remove plus an optional resize `CoverSize`) and `tagging::fields_from`. Make download tagging write from `fields_from`, with the existing tagging tests still passing.
- [x] 2.2 Implement `tag_edit::read_fields` for FLAC, MP3 and M4A, with read-back tests on fixtures written by `write_tags`.
- [x] 2.3 Implement `tag_edit::apply_edits` on native tags. Tests:
  - Set and Clear for each field
  - an unlisted item (e.g. `REPLAYGAIN_TRACK_GAIN`) survives an edit
  - replacing the cover leaves exactly one front cover
  - removing the cover removes all front covers and keeps other picture types
  - resize: a 600 px embedded cover at 400 px becomes 400×400 with other tags unchanged; two files with different covers each keep their own, reduced; a 450 px cover at 500 px leaves the file unwritten; a replacement plus a size is resized once and embedded in every file
- [x] 2.4 Add `PartFile::copy_of(path)` and `publish_sync(dest)`, and `tag_edit::save_files(Vec<(path, TagEdits)>) -> Vec<Result<()>>`, which saves sequentially. Tests: a read-only file is reported and unchanged, with no `.part` left; a successful save replaces the content at the same path; files without edits are not written (modification time unchanged).
- [x] 2.5 Re-export the editing API in `lib.rs`. Verify with `cargo test -p qobuz-core`.

## 3. GUI

- [x] 3.1 Add `app/tag_editor.rs`: editor state; field state derived from text vs original (Keep, Set or Clear; mixed fields with explicit Clear); album-to-track expansion; Apply to all; validation; and building `TagEdits`. Unit-test each.
- [x] 3.2 Add the Edit tags group-header control with the gating from the `downloader-gui` delta, the messages, and `tasks.rs` blocking wrappers for read, cover processing and save.
- [x] 3.3 Add `app/view/tag_editor.rs`: header actions, album form on the label column, track rows with expand, the mixed placeholder, invalid-field styling, and save progress and report. Use `style.rs` builders and semantic accents only.
- [x] 3.4 Reset to Qobuz from each item's `Job` via `fields_from`. Replace the cover through `rfd`, checking that the image decodes; Remove cover. Add the Resize cover pick list (Don't resize, 400 px, 500 px, 600 px): disabled while Remove is chosen, Don't resize by default, and set to `config.cover_size` when a replacement is picked. Unit-test those defaults.
- [x] 3.5 The cover cache is trimmed to `covers_in_use()`: add the edited album's cover so its header thumbnail stays loaded.

## 4. Docs

- [x] 4.1 Update `CLAUDE.md`: add `tag_edit.rs` and the editor files to the architecture, and the native-tag / copy-and-rename rules.

## 5. Checks

- [x] 5.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.
- [x] 5.2 In the running app, edit an album's title, year and one track title; save; and check with a third-party tagger (e.g. `metaflac --list`, `ffprobe`) that only those fields changed.
