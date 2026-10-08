# Tasks

Requires `cover-resize-and-tag-fields` and `album-tag-editor` applied first.

## 1. Core

- [ ] 1.1 Add `Config::rename_format` with the default, plus a test that a config without it loads the default. Verify with `cargo test -p qobuz-core config`.
- [ ] 1.2 Add `rename.rs` with `suggest` (tags plus audio properties → `TemplateContext` → `render_segment`). Test with a tagged FLAC fixture, including a `/` in the template.
- [ ] 1.3 Add `rename::rename_folder` with the checks from design.md. Tests:
  - successful rename keeps the contents
  - target exists → refused, nothing changed
  - folder equal to or outside `download_dir` → refused
  - a name that sanitizes to nothing → refused
  - a case-only rename succeeds

  Re-export it in `lib.rs`. Verify with `cargo test -p qobuz-core rename`.

## 2. GUI

- [ ] 2.1 Extract the Edit tags gating into one group predicate shared with Rename folder.
- [ ] 2.2 Add `RenameState`, the messages, the blocking `tasks.rs` wrappers for `suggest` and `rename_folder`, the shared-folder check against other albums, and the queue path rewrite. Unit-test the rewrite and the shared-folder refusal.
- [ ] 2.3 Add the inline header field in `app/view/queue.rs`: text input, Rename (primary, disabled when empty) and Cancel; Enter confirms; results go to the status line.
- [ ] 2.4 Add the Settings Rename row with a preview, the unsaved-changes hint, and the template help text for the rename template.

## 3. Docs

- [ ] 3.1 Update `CLAUDE.md`: add `rename.rs` and the rule that queue paths are rewritten after a folder rename.

## 4. Checks

- [ ] 4.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.
- [ ] 4.2 In the running app, rename a finished album's folder (suggested name, then a free name), and check that Open folder and Edit tags still work afterwards.

## Workflow follow-up

- Archive after `cover-resize-and-tag-fields` and `album-tag-editor`, because this change's `Persist settings` and `Settings screen` deltas include theirs.
