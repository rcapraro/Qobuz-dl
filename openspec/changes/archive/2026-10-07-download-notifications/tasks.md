# Tasks

## 1. Setting

- [x] 1.1 Add `Config.notify_on_finish` (default `true`) in `qobuz-core`, and verify `defaults_are_sane` asserts it and a config JSON without the field loads with it on
- [x] 1.2 Add the "Notify when downloads finish" checkbox (`NotifyToggled`) to the Options card and one line to the options help panel, and verify with an update test that toggling changes `config.notify_on_finish`

## 2. Deciding and posting

- [x] 2.1 Add a pure `notification(outcome, enabled, focused)` returning the summary and body, and verify unit tests for: all downloaded, some failed, singular counts, cancelled, disabled, and focused, each yielding no notification where expected
- [x] 2.2 Add `notify-rust = "4.18"` to `qobuz-gui` (with `Cargo.lock` in the same change) and `tasks::notify` posting via `spawn_blocking` and logging failures, and verify `cargo build --workspace` succeeds
- [x] 2.3 Track focus with a `subscription()` on `Window(Focused/Unfocused)` → `WindowFocus(bool)`, and post from `DownloadsFinished` using the outcome it already computes, and verify update tests that a focused or cancelled finish posts nothing (the decision function's result is asserted, not the OS call)
- [x] 2.4 On macOS, set the delivering application once in `app::run` from `get_bundle_identifier_or_default("Qobuz-dl")`, and verify it compiles on macOS (`cargo build`) and is gated by `#[cfg(target_os = "macos")]`

## 3. Verification and documentation

- [x] 3.1 Run the dev build, start a batch, switch to another app, and verify a notification appears (attributed to Finder on the dev build), that none appears while the app is focused or after Cancel, and that turning the setting off suppresses it
- [x] 3.2 Update the README (feature bullet, and a note that dev builds post as Finder on macOS) and `CLAUDE.md` (notifications in `app/tasks.rs`, set-once macOS identity), and verify the text matches the code
- [x] 3.3 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`, and verify all pass

## Workflow follow-up

- Archive and sync this change (it creates the `desktop-notifications` spec).
