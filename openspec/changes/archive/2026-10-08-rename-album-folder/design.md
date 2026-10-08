# Design

## Context

See proposal.md for motivation and the specs for requirements. This change assumes `album-tag-editor` is applied, for `tag_edit::read_fields` and `TagFields`, and the shared group-header gating.

- **Folder lookup:** `App::album_folder` derives an album's folder with `open::shared_folder` over its done items' `path`s. For a multi-disc album that's the album folder, the parent of `Disc N`.
- **Template rendering:** `template::render_segment` renders one sanitized segment from a `TemplateContext`. `engine` fills the context from a `Job`.
- **Settings:** templates live in the File organization card, with a live preview built from a sample context.

## Goals / Non-Goals

**Goals:**
- Renaming never moves, merges or overwrites anything except the one album folder.
- The queue stays consistent with the disk after a rename.

**Non-Goals:**
- Renaming track files after `track_format` (a possible follow-up using the same mechanics).
- Moving an album to another parent directory.
- Renaming albums outside this session's queue.
- Keeping re-queue from downloading the album again into the template's folder: accepted, and stated in the proposal.

## Decisions

### Suggestion rendered from the files, not the `Job`
`rename::suggest(template, first_done_path) -> Result<Option<String>>` (`None` when the template renders nothing, so the field keeps the current name rather than offering `_`):
1. reads `TagFields` for the album's first done track in disc and track order, plus the file's audio properties (lofty `FileProperties`: bit depth, sample rate);
2. builds a `TemplateContext` with the same keys the engine sets;
3. renders `rename_format` with `render_segment`, so a `/` in the template is sanitized rather than nesting.

Using the files means a name edited in the tag editor is what gets suggested. The read runs on a blocking task when the rename field opens; until it returns, the field shows the album folder's current name.

*Alternative:* render from `Job`. Rejected: it would ignore tag edits, the main reason to rename.

### `rename::rename_folder(download_dir, folder, album_files, new_name) -> Result<PathBuf>`
The function works through these steps:
1. **Clean the name:** `sanitize_segment(new_name)`. Reject a name that comes out as `_`, which `sanitize_segment` returns when nothing is left.
2. **Check the folder:** it must be strictly inside `download_dir`. Both paths are canonicalized first, so symlinks and `..` can't escape it.
3. **Check its subfolders:** every subfolder must hold one of `album_files`. With a `{albumartist}` folder template the album's folder is the artist's, and it may hold albums from earlier sessions that the queue doesn't know about. Loose files (scans, logs) move with the folder.
4. **Check the target:** target = `folder.parent()/name`.
   - If the target exists and is not the same directory entry (compared by inode on Unix, since macOS canonicalizes a path in the case given; by canonical path on Windows), the rename is refused as "name taken".
   - If it is the same directory entry and the name is identical, nothing happens and the result is success.
5. **Rename:** `std::fs::rename(folder, target)`, which is atomic within one parent directory.

On macOS (APFS, case-insensitive), a case-only rename through `rename` changes the case directly. Windows also accepts it. A test covers it on the CI platforms.

The "folder shared with another album" check needs the queue, so it lives in the GUI: refuse when any other album's done `path` starts with `folder`.

### Queue path rewrite
On success, every `QueueItem.path` that `starts_with(old)` is rebuilt as `new.join(path.strip_prefix(old))`, across all groups. It's a linear scan, consistent with the flat queue. If a tag editor is open, it can't be for this album, because the editor replaces the queue list where the control lives.

### Inline UI
`App.rename: Option<RenameState>` holds the album id, the text, and a loading flag. Only one rename field is open at a time. The group header swaps its action buttons for:
- a `text_input`
- Rename (primary)
- Cancel

Enter confirms. The gate, `App::group_renamable`, is Edit tags' `App::group_editable` plus two conditions: no download is running at all, because another album in the queue may be writing `.part` files into the same folder before any of its paths is known; and none of the album's tracks failed, because Retry failed rebuilds the destination from the folder template, under the old name. The field closes (`drop_stale_rename`) when the queue is cleared, the album removed, or a download starts. Results go to the typed status line.

### Config and Settings
`Config::rename_format` defaults to `{albumartist} - {album} ({year})` via `#[serde(default)]`. The File organization card adds a Rename row under Track, on the label column, with its own one-line preview rendered from the same sample context as the existing preview. It's covered by the unsaved-changes hint like the other templates.

## Risks / Trade-offs

- **A file is open in another program on Windows** → the folder rename fails, nothing moves, and the OS error is reported.
- **The album folder holds unrelated files the user added** (scans, logs) → they move with the folder, which is what a folder rename means.
- **Re-queue after a rename downloads again** → documented in the proposal and in the template help. A later change could look for an album's existing files by tag instead.
- **The first track's tags are used for the suggestion** → on albums with mixed album-level tags, the suggestion follows track 1. The user can always type a name.
