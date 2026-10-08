# Tasks

## 1. Unsaved-changes tracking

- [x] 1.1 Derive `PartialEq` on `Config` in `crates/qobuz-core/src/config.rs` and verify `cargo build --workspace` succeeds with no serde change (the existing config tests still pass).
- [x] 1.2 Add `saved_config` to `App`, initialized from the loaded config in `from_parts`, plus a `settings_dirty()` accessor. Verify with a unit test that a freshly built `App` is not dirty.
- [x] 1.3 Route every save through one helper that refreshes `saved_config` on success. The helper replaces the direct `config.save()` calls in `SaveSettings` and `CredentialsDetected`, and `save_config()` wraps it. Verify with unit tests: an edit (e.g. `QualitySelected`) makes the app dirty, and reverting it clears dirty. Implicit saves are not unit-tested because `Config::save` writes the real user config file. They are covered by `persist_config` being the only call site of `config.save()`, which is verified by grep.

## 2. Settings view

- [x] 2.1 Save row: Save settings is enabled only when `settings_dirty()`, followed by an "Unsaved changes" hint in the `progress()` accent role. Verify by running `cargo run -p qobuz-gui`: edit a field, see the hint and the enabled button; save, and see both clear.
- [x] 2.2 API credentials card:
  - stacked `labeled_row`s `App ID:` and `App secret:`;
  - button row indented to the input column, with the helper text on its own line beneath it;
  - Check signing as a secondary button;
  - helper text covering both buttons.

  Verify in the running app that the filled fields keep their labels and only Auto-detect is primary.
- [x] 2.3 Account card: `labeled_row("Token:", …)` with Sign in (primary) and Sign out (secondary) after the input, with the existing enable/disable rules unchanged. Verify in the running app that the token input aligns with the credential and template inputs.
- [x] 2.4 Options card: Quality and Concurrency on the first line, the two checkboxes on the second line indented to the input column. Verify in the running app at the default window size.

## 3. Checks

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.

## Workflow follow-up

- Refresh `docs/screenshots/settings.png` for the README.
- Archive the change (`/opsx:archive`) before proposing `chrome-and-layout`.
