# Tasks

## 1. Concurrency range in core

- [x] 1.1 Add `MAX_CONCURRENCY = 10` to `qobuz-core/src/config.rs` and clamp `concurrency` to `1..=MAX_CONCURRENCY` in `Config::load`. Verify with a unit test that a parsed config with concurrency 16 (and one with 0) comes out as 10 (and 1), via the same parse-and-clamp path `load` uses, without touching the real config file. Then run `cargo test -p qobuz-core config`.

## 2. Settings view and help

- [x] 2.1 Put App ID (fixed `APP_ID_WIDTH`) and "App secret:" + the secret input on one `labeled_row`, keep the button row indented, and remove the helper sentence. Add a "Check signing" entry to `help::credentials_help`. Verify in the running app (`cargo build -p qobuz-gui` first) that both credentials share a row with labels, that the card shows no sentence under the buttons, and that its `?` panel explains Check signing.
- [x] 2.2 Put Options on one `labeled_row("Quality:", …)`: pick list, "Concurrency:", `NumberInput` bounded by `1..=MAX_CONCURRENCY` at `CONCURRENCY_WIDTH`, then "Embed cover art" and "Notify when done". Update `help::options_help` to say 1–10 and "Notify when done". Verify in the running app at the default window size that the line fits without clipping, that the stepper stops at 10, and whether Save settings is now visible without scrolling.

- [x] 2.3 Add a `sapphire` palette colour, an `Accents::info()` role (covered by the accent-role test) and a `style::help_panel` container (sapphire wash and border, like the `hero` identity panel). Wrap all four Settings help panels in it, with the template help's card replaced by the same panel titled "Template help", and remove the unused `view::card`. Verify in the running app that every `?` panel is visibly set apart from its card in both themes.

## 3. Checks

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.

## Workflow follow-up

- Refresh `docs/screenshots/settings.png`, possibly unscrolled now.
- Archive the change (`/opsx:archive`). Its deltas don't overlap with `chrome-and-layout`, so either can be archived first.
