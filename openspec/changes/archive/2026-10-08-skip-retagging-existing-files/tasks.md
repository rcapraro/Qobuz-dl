# Tasks

## 1. Core

- [x] 1.1 Add a `#[cfg(test)]` `test_support` module with a scratch-path helper, a minimal-FLAC helper and a guard that deletes the file on drop, and use it in the existing `download.rs` temp-file tests in place of their local `scratch`.
- [x] 1.2 Rename `download::stream_to_file` to `stream_to_part`, returning `Transfer::{Downloaded(PartFile), AlreadyPresent}` without renaming. Make `PartFile` public, with `path()` and `publish(dest)`. Update `skips_when_destination_exists` to assert `AlreadyPresent`, and add a test that `publish` moves the file to `dest` and leaves no temp file. Verify with `cargo test -p qobuz-core download`.
- [x] 1.3 Change `tagging::write_tags` to `write_tags(file, dest, tags)`, choosing the container from `dest`. Add a test that tagging a `.partN` file for a `.flac` destination writes Vorbis comments. Verify with `cargo test -p qobuz-core tagging`.
- [x] 1.4 In `engine.rs`, carry the `Transfer` out of `download_with_progress` and remove its `biased` race and that race's explanation. In `download_one`, call `finish_download(part, &dest, …)` for `Downloaded` and do nothing for `AlreadyPresent`. `finish_download` sends `Tagging`, fetches the cover, returning `Error::Cancelled` if cancelled, writes the tags into the part and publishes it. Replace the earlier `finish` tests with: tagged and published, tagging failure leaves no file, and cancel during cover fetch leaves no file. Verify with `cargo test -p qobuz-core engine`.

## 2. Docs

- [x] 2.1 Update `CLAUDE.md`: the `download.rs` summary (temp file handed back, published after tagging) and the per-track data flow (tag the `.part`, then rename; an existing destination is left untouched).

## 3. Checks

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.

## Workflow follow-up

- Archive the change (`/opsx:archive`).
