# Tasks

## 1. In-progress strip above the editor

- [x] 1.1 Factor an "in progress" check out of `App::group_editable` in `app.rs` (some track queued, downloading, tagging, or in the running batch) and keep `group_editable`'s behaviour; verify with a unit test covering queued, downloading, done-in-running-batch, and settled groups, plus the existing `tags_editable` tests passing
- [x] 1.2 In `app/view/queue.rs`, render the strip in `queue_view`'s editor branch from `groups()` filtered by that check: one `strip_line` per album with cover, title, `summarize` done count and progress bar, no actions or quality badge; omit it when empty; verify with a unit test on the filtering (the album being added shows up, a settled album does not) and by running the app
- [x] 1.3 Cap the strip at three visible lines with a fixed computed-height `scrollable`, and confirm `covers_in_use()` keeps strip covers cached; verify in the running app with five albums in progress while the editor is open
- [x] 1.5 Leave the overall progress bar out of the queue header while the strip is shown (`queue_header`'s `overall_bar`), since the strip's bars would read as a double of it; verify in the running app that one album downloading while editing shows a single bar
- [x] 1.4 Update the `qobuz-gui` section of `CLAUDE.md` where it says the editor replaces the list, to mention the strip; verify the text matches the implementation

## 2. Save notice when the album is back in the queue

- [x] 2.1 In `app/view/tag_editor.rs`, extend the status line so that, when the album is not editable and the editor is neither saving nor looking up, it reads that the album is back in the queue and can be saved once its download ends; verify with a unit test on the status text for each case (saving, lookup, back in queue, unsaved, idle)
- [x] 2.2 Verify in the running app: open the editor, make an edit, add a track of the same album from Search, see the notice and a disabled Save, let the download finish, and see Save enabled with the edit kept

## 3. Checks

- [x] 3.1 Run `cargo fmt`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, all clean
- [x] 3.2 Run `openspec validate show-downloads-beside-tag-editor --strict` and confirm it passes

## Workflow follow-up

- Archive the change after review, syncing the `downloader-gui` and `tag-editing` deltas into the main specs.
