# Tasks

## 1. Delivered label in core

- [x] 1.1 Change `engine::describe_delivered` to return `MP3 320` for the MP3 tier, `FLAC {bit}/{rate}` when both values are present, and the tier label otherwise. Update the existing 24/96 and 16/44.1 tests and add an MP3 case and a missing-values case. Verify with `cargo test -p qobuz-core engine`.

## 2. Queue rows and group header

- [x] 2.1 Add `group_quality` to `app/view/queue.rs` (most frequent `Done` label, ties to the first in queue order) with unit tests for: all the same, one downgraded, none done, and a tie. Verify with `cargo test -p qobuz-gui group_quality`.
- [x] 2.2 Rewrite `queue_row(it, group_quality)`: no buttons; Title Case status text; a quality badge only for a done row whose label differs from the group's; a progress bar only while downloading or tagging. Show the group quality badge in `group_view`'s header. Verify in the rebuilt app (`cargo build -p qobuz-gui`, confirming the app window is in front before any click) with an album queued: rows show `Queued` and no bar or buttons, the header has Open folder and Remove only, and no row wraps for lack of room.
- [x] 2.3 Remove `Message::RetryTrack`, `RemoveTrack`, `RevealTrack` and their handlers, `open::Target` (callers pass the folder path), and the CLAUDE.md mention of the reveal commands. Rewrite the `RemoveTrack`-based tests against `RemoveGroup` where a group test doesn't already cover the behaviour. Verify with `cargo test -p qobuz-gui` and a grep showing no remaining references.

## 3. Checks

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.

## Workflow follow-up

- Refresh the README queue screenshot and its alt text ("with open-folder actions").
- Archive the change (`/opsx:archive`) before proposing `search-affordances`.
