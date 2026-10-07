# Tasks

## 1. State and pure helpers

- [x] 1.1 Add `QueueItem.path`, set from `JobEvent::Done` and cleared when an item is reset to queued, and verify with an update test that a Done event stores the path and a Cancelled one clears it
- [x] 1.2 Add pure `removable(status)` and `shared_folder(paths)`, and verify unit tests for each status, one album in one folder, an album split across folders, and no done track (None)

## 2. Removal

- [x] 2.1 Rename `DequeueTrack` to `RemoveTrack` and widen it and `RemoveGroup` to `removable` statuses, still ignored while downloading, and verify update tests that done and failed rows are removed (files untouched), a downloading row is not, and nothing is removed during a batch
- [x] 2.2 Show Remove on every removable row (disabled during a batch) and the group Remove when any item is removable, and verify in the running app

## 3. Opening

- [x] 3.1 Add `app/open.rs` (`open_dir`, `reveal` per OS, existence check) and `tasks::open_path` (spawn_blocking, reap, ignore exit codes), and verify `cargo build` and a unit test that a missing path returns `NotFound` without running a command
- [x] 3.2 Add `OpenAlbumFolder`, `RevealTrack`, `OpenDownloadDir`, and `Opened(Result)`, the group **Open folder** (when a track is done), the row **Show in folder** (done rows), and the Settings **Open**, and verify in the running app that each opens the right place, and that a moved file reports an error in the status line

## 4. Documentation and checks

- [x] 4.1 Update the README queue step and the Settings mention, and the `CLAUDE.md` notes (`QueueItem.path`, `app/open.rs` with no dependency, ignored exit codes), and verify the text matches the code
- [x] 4.2 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`, and verify all pass

## Workflow follow-up

- Archive and sync this change.
