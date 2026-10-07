# Tasks

## 1. Grouping logic

- [x] 1.1 Add a pure `groups(queue)` (by `job.album.id`, first-appearance order, queue order within) and a `GroupSummary` (done, failed, total, fraction via `overall_progress`), generalising `overall_progress` to take item references, and verify unit tests for: one album, a later track joining an earlier group, a playlist spread across albums, and a group with a failure settling to complete
- [x] 1.2 Derive the row label from the job (`"{n}. {title}"`, plus the performer when it differs from the album artist), remove `QueueItem.title`, and verify unit tests for the label with and without a track number and a guest performer

## 2. State and messages

- [x] 2.1 Add `App.collapsed` with `ToggleGroup`, and `RemoveGroup` guarded by `!downloading`, and verify update tests that a toggle flips collapsed state, that remove drops only that group's queued tracks, and that remove is ignored while downloading
- [x] 2.2 Fetch covers for newly queued albums on `Resolved`, and keep queue covers through the `SearchDone` trim, and verify with an update test that a search keeps a queued album's cached cover

## 3. View

- [x] 3.1 Render groups in `view/queue.rs` with the header (toggle, cover, title/artist, count with failures, Remove when allowed, group bar) and indented rows, and verify in the running app with an album, a single track from the same album added afterwards, and a playlist, in both themes

## 4. Documentation and checks

- [x] 4.1 Update the README queue step and the queue notes in `CLAUDE.md` (render-time grouping, flat `queue` kept), and verify the text matches the app
- [x] 4.2 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`, and verify all pass

## Workflow follow-up

- Archive and sync this change before proposing `download-notifications`.
