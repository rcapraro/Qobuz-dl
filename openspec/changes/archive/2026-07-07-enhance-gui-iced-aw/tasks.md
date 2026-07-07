## 1. Dependencies

- [x] 1.1 Add `"advanced"` to the existing `iced` feature list in `crates/qobuz-gui/Cargo.toml` (keep `"tokio"`)
- [x] 1.2 Add `iced_aw = { version = "0.12", default-features = false, features = ["tabs", "number_input", "card", "badge"] }` to `crates/qobuz-gui/Cargo.toml`
- [x] 1.3 Run `cargo build -p qobuz-gui` to confirm iced + iced_aw versions resolve and compile (iced_aw 0.12.2 resolved against iced 0.13.1)

## 2. Styling helpers

- [x] 2.1 iced_aw widgets follow the custom light/dark `Palette` via iced_aw's default `Catalog` impls (our `Theme::custom` derives the extended palette they read) — no bespoke tab/card/number-input closures needed
- [x] 2.2 Add a helper mapping `ItemStatus` → badge style (`badge_style` in `app.rs`: success for Done, danger for Error, primary/info/secondary for Downloading/Tagging/Queued)

## 3. Tabbed navigation

- [x] 3.1 Replace the `nav` button row + `Screen` dispatch in `view` with `iced_aw::Tabs` keyed by `Screen`, using `Message::Navigate` as the on-select handler
- [x] 3.2 Add a slim header row above the tabs holding the theme toggle and sign-in indicator
- [x] 3.3 Remove the now-unused `nav_button` helper if no longer referenced

## 4. Concurrency number input

- [x] 4.1 Replace the concurrency text input in the Settings view with `iced_aw::number_input` bound to `self.config.concurrency` (range 1..=16)
- [x] 4.2 Change `Message::ConcurrencyChanged` to carry `usize` and set `self.config.concurrency` directly
- [x] 4.3 Remove the `concurrency: String` field from `App` and its initialization, and delete the parse/clamp branch in `update`

## 5. Template help card

- [x] 5.1 Wrap the `template_help()` content in an `iced_aw::Card`, keeping the `show_template_help` toggle and the live path preview above the inputs
- [x] 5.2 Verify Copy and Apply example controls still function and the preview updates (message paths unchanged; wrapping is presentation-only)

## 6. Queue badges

- [x] 6.1 Render per-item status as a colored `iced_aw::Badge` in `queue_row` using the status→color helper; keep per-item and overall progress bars

## 7. Card grouping for results and settings

- [x] 7.1 Wrap the Albums/Tracks/Artists result sections in `iced_aw::Card`s, keeping per-row Add buttons
- [x] 7.2 Group the Settings sub-sections (credentials, auth, file organization, options) in `iced_aw::Card`s

## 8. Quality gates

- [x] 8.1 `cargo fmt`
- [x] 8.2 `cargo clippy --workspace` (no warnings)
- [x] 8.3 `cargo build --release -p qobuz-gui`

## 9. Manual verification

- [x] 9.1 Smoke-launched the release binary: app starts, opens its window, and renders the Settings tab (tabs + cards + number input built) without panic
- [ ] 9.2 Confirm the concurrency spinner enforces 1–16 *(needs hands-on interaction at the window)*
- [ ] 9.3 Toggle the template help card; confirm Copy/Apply work and the live preview updates *(needs hands-on interaction)*
- [ ] 9.4 Run a search; confirm results are grouped in cards with working Add buttons; enqueue an item and confirm queue rows show colored status badges *(needs a signed-in session)*
- [ ] 9.5 Toggle light/dark and confirm all iced_aw widgets follow the theme *(needs hands-on interaction)*
- [x] 9.6 `openspec validate enhance-gui-iced-aw --strict`
