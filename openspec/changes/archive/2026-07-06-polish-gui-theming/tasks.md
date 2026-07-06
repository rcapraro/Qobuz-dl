## 1. Persistence (qobuz-core)

- [x] 1.1 Add `pub dark_mode: bool` field to `Config` in `crates/qobuz-core/src/config.rs` (struct at 16-37)
- [x] 1.2 Set `dark_mode: true` in `impl Default for Config` (39-53) to preserve current dark appearance
- [x] 1.3 Update/extend the existing config test in `config.rs` to cover the new field (round-trip + default); run `cargo test -p qobuz-core`

## 2. Design system module (qobuz-gui)

- [x] 2.1 Create `crates/qobuz-gui/src/style.rs` and declare `mod style;` in `main.rs`
- [x] 2.2 Add spacing scale constants: `SPACE_XS=4`, `SPACE_SM=8`, `SPACE_MD=12`, `SPACE_LG=18`, `SPACE_XL=24`
- [x] 2.3 Add control-sizing constants: `CONTROL_HEIGHT`, `BUTTON_MIN_WIDTH`, `INPUT_PADDING`, `LABEL_WIDTH` (size `LABEL_WIDTH`/`BUTTON_MIN_WIDTH` from longest labels: "Concurrency:", "Save settings")
- [x] 2.4 Add typography constants: `TEXT_SM`, `TEXT_BODY`, `TEXT_SECTION`
- [x] 2.5 Define light + dark `iced::theme::Palette` values (background, text, primary/accent, success, danger) with a muted, professional accent
- [x] 2.6 Add `pub fn theme(dark: bool) -> iced::Theme` building a custom `Theme::custom` from the palette for the active mode

## 3. Theme state & toggle (qobuz-gui app.rs)

- [x] 3.1 Add `dark_mode: bool` field to `struct App` (56-79); initialize from `self.config.dark_mode` in `App::new` (120-150)
- [x] 3.2 Add `Message::ToggleTheme` variant near `Navigate` (81-117)
- [x] 3.3 Add `fn theme(&self) -> Theme` on `App` returning `style::theme(self.dark_mode)`
- [x] 3.4 Change `.theme(|_| Theme::Dark)` (app.rs:19) to `.theme(App::theme)`
- [x] 3.5 Add the `ToggleTheme` `update` arm: flip `self.dark_mode`, set `self.config.dark_mode`, call `self.config.save()`, return `Task::none()`
- [x] 3.6 Add a compact sun/moon toggle button (emitting `ToggleTheme`) to the nav row (441-453), right-aligned near the signed-in status

## 4. Reusable widget helpers (style.rs)

- [x] 4.1 Add `action_button(label, msg)` (primary variant + `.padding` + `.width(Length::Fixed(BUTTON_MIN_WIDTH))` + `CONTROL_HEIGHT`)
- [x] 4.2 Add `secondary_button(label, msg)` (same sizing, `button::secondary` style)
- [x] 4.3 Add `field_input(...)` helper applying `INPUT_PADDING` + `TEXT_BODY` size (and consistent height)
- [x] 4.4 Add `labeled_row(label, control)` helper: fixed-width (`LABEL_WIDTH`) label + control, `.align_y(Center)`

## 5. Normalize layout across views (app.rs)

- [x] 5.1 `settings_view` (472-562): route all buttons through the button helpers; use `labeled_row` for label+field rows; replace `Space::with_width(16)` gaps in `options_row` with row spacing; unify field widths; apply `.align_y(Center)` to auth/creds input rows
- [x] 5.2 `search_view` (588-636): apply button helpers + `.align_y(Center)` to the search/url bars; use the spacing scale
- [x] 5.3 `queue_view` (638-678): apply button helpers; unify the two `progress_bar` heights (overall vs per-item at `Fixed(8.0)`)
- [x] 5.4 Root `view()` (440-470) and `section()`/`result_row()`/`queue_row()` helpers: replace all literal spacings/paddings/text sizes with the `style` constants

## 6. Verification

- [x] 6.1 `cargo build --workspace` and `cargo clippy --workspace` clean
- [x] 6.2 `cargo run -p qobuz-gui`: confirm buttons/inputs share sizes, labels/fields align, and spacing is uniform across Settings/Search/Queue
- [x] 6.3 Toggle theme in-app: verify instant light↔dark switch app-wide; restart and confirm the preference is restored from config
- [x] 6.4 Confirm an existing `config.json` without `dark_mode` still loads (defaults to dark)
