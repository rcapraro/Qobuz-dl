# Tasks

## 1. Detail state and pure helpers

- [x] 1.1 Add `AlbumDetail` / `DetailState` and `App.album`, plus pure `discs`, `selected_jobs`, `format_duration`, and `guest_performer`, and verify unit tests for: single-disc vs multi-disc grouping in album order, selection preserving album order, durations below and above an hour, and a guest performer shown only when it differs
- [x] 1.2 Add `OpenAlbum`, `AlbumLoaded`, and `RetryAlbum` with the stale-response guard, and verify update tests that opening sets Loading without touching `results`, that a response for a closed or different album is dropped, and that a loaded album starts with every track selected
- [x] 1.3 Add `ToggleTrack`, `SelectAllTracks`, `SelectNoTracks`, and `AddSelected` (reusing `Resolved`), and verify update tests that a subset is enqueued in album order, that duplicates already in the queue are skipped, and that an empty selection adds nothing

## 2. Navigation and scroll

- [x] 2.1 Give the results scrollable an `Id` and `on_scroll` → `ResultsScrolled`, add `CloseAlbum` returning `scroll_to` to the stored offset, and clear the detail and offset on `SearchSubmit`, and verify with an update test that `SearchSubmit` closes the detail. Verify in the running app that Back returns to the same scroll position

## 3. Views

- [x] 3.1 Make the album row's cover and title block a borderless button emitting `OpenAlbum`, keeping the separate compact Add, and verify in the running app that clicking the title opens the detail while Add still queues the whole album
- [x] 3.2 Add `view/album.rs` with the header, toolbar (Back, Select all/none, `Add N tracks`), loading and failure-with-Retry states, and track rows with disc headings, and verify in the running app on a single-disc album and a multi-disc album in both themes

## 4. Documentation and checks

- [x] 4.1 Update the README usage step for finding music and the GUI paragraph in `CLAUDE.md` (album detail in `app/view/album.rs`, built from resolved jobs), and verify the text matches the app
- [x] 4.2 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`, and verify all pass

## Workflow follow-up

- Archive and sync this change before proposing `queue-grouping`.
