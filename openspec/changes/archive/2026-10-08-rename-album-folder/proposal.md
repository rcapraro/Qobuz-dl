# Proposal

## Why

A downloaded album's folder name is fixed when its tracks are resolved, from the folder template and Qobuz's metadata. After editing an album's tags with `album-tag-editor`, or just to tidy one folder, the user has to leave the app and rename it in the file manager. That also breaks the queue's Open folder, which still points at the old path.

Decided on 2026-10-08: renaming is a separate action from editing tags. The new name comes either from a rename template kept in Settings, separate from the download folder template, or from free text.

## What Changes

- **Rename folder:** a new album group control shows a field prefilled with the album's rename template, rendered from its files' current tags. The user can accept it or type any name, then confirm.
- **Rename template setting:** a new template in Settings' File organization, separate from the download folder template, with the same placeholder syntax and a live preview. It produces one folder name; the default is `{albumartist} - {album} ({year})`.
- **Rename in place:** the folder keeps its parent directory, with its name sanitized like any path segment. The rename is refused when the target exists, when the folder is the download directory or outside it, and when another album in the queue has files inside it.
- **Queue stays correct:** after a rename, every queue item whose file was inside the folder points at its new path, so Open folder and the tag editor keep working.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `file-organization`: adds renaming an album folder and the rules that make it safe.
- `app-configuration`: "Persist settings" includes the rename template, and "Live template preview" covers it.
- `downloader-gui`: "Settings screen" includes the rename template field, and the group header gains the Rename folder control.
- `template-help`: "Template syntax help" covers the rename template and the fact that it has no `/`.

## Impact

- `crates/qobuz-core/src/config.rs`: `rename_format` with a default.
- New `crates/qobuz-core/src/rename.rs`: render a folder name from `TagFields` (from `album-tag-editor`) and the file's audio properties, check the target, and rename the folder.
- `crates/qobuz-gui/src/app.rs`, `app/view/queue.rs`, `app/view/settings.rs`, `app/help.rs`: the inline rename field, path rewriting in `App.queue`, the Settings field and preview, and help text.
- Order: apply after `cover-resize-and-tag-fields` and `album-tag-editor`. Its `Persist settings` and `Settings screen` deltas include the scenarios added by `cover-resize-and-tag-fields`, so archive that change first.
- Known effect: a renamed folder no longer matches the download folder template, so re-queuing the album downloads it again under the template's folder.
