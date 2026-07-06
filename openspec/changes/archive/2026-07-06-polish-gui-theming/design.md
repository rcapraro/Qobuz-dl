## Context

`qobuz-gui` is an `iced` 0.13 app built with the functional builder (`iced::application(title, App::update, App::view).theme(...).run_with(App::new)` at `app.rs:17-21`). Today:

- Theme is hardcoded: `.theme(|_| Theme::Dark)` (`app.rs:19`). `Theme` is imported but otherwise unused.
- Styling is almost entirely framework default. The only `.style(...)` in the codebase is `nav_button` selecting `button::secondary` for inactive tabs (`app.rs:681-688`).
- Sizing/spacing/alignment are ad-hoc: column spacing varies (root `10`, settings `18`/sections `8`, search `12`/results `6`, queue `12`/list `8`); action buttons have no `.width()` so they size to content and don't line up; `Space::with_width(16)` is used as manual gaps; `FillPortion(2)` mixes with `Fill`; `.align_y(Center)` is applied to some rows but not the auth/creds/search input rows; text sizes are ad-hoc (`18`, `13`, default).
- `Config` (`crates/qobuz-core/src/config.rs:16-53`) has `#[serde(default)]` on the struct and an `impl Default`, loaded at `app.rs:121` and saved via `self.config.save()` (`app.rs:218-224`, `258`). It has no UI/theme field.

## Goals / Non-Goals

**Goals:**
- A single centralized visual design system (spacing scale, control sizes, typography, palette) applied across all three screens.
- Consistent button and text-input dimensions; aligned labels/fields/buttons.
- A sleek, professional custom theme for both light and dark modes.
- A user-facing light/dark toggle that applies instantly and is persisted across launches.

**Non-Goals:**
- No changes to app behavior (auth, search, download, tagging, file organization).
- No new screens, no per-widget user-configurable colors, no system-theme auto-detection (may be a follow-up).
- No new third-party crates.

## Decisions

### 1. New `style` module for the design system
Add `crates/qobuz-gui/src/style.rs` (declared `mod style;` in `main.rs`) holding:
- **Spacing scale** constants: `SPACE_XS = 4`, `SPACE_SM = 8`, `SPACE_MD = 12`, `SPACE_LG = 18`, `SPACE_XL = 24` — replace all literal spacings/paddings in `app.rs` with these.
- **Control sizing** constants: `CONTROL_HEIGHT` (shared button/input height), `BUTTON_MIN_WIDTH`, `INPUT_PADDING`, `LABEL_WIDTH` (fixed width so labels form a column).
- **Typography** constants: `TEXT_SM`, `TEXT_BODY`, `TEXT_SECTION` (replaces the `18`/`13` literals).
- **Helper builders** so call sites stay uniform: `action_button(label, msg)` and `secondary_button(label, msg)` (apply variant + `.padding` + `.width(Length::Fixed(BUTTON_MIN_WIDTH))`), `field_input(...)` (apply `.padding(INPUT_PADDING).size(TEXT_BODY)`), and `labeled_row(label, control)` (fixed-width label + control, `.align_y(Center)`).

*Why:* a module of constants + helpers is the smallest change that guarantees consistency and mirrors the existing `nav_button`/`section` helper pattern. Alternative — scattering tuned literals inline — reproduces the current inconsistency and was rejected.

### 2. Custom light + dark themes via `Theme::custom` + `Palette`
Build two `iced::Theme` values from hand-tuned `iced::theme::Palette` structs (background, text, primary/accent, success, danger). iced auto-derives the extended palette (`palette::Extended`) from these five colors, so buttons/inputs/containers pick up cohesive styling for free. Store them and select by the active mode.

*Why custom over built-ins:* gives the "sleek, professional" branded look the request asks for and keeps full control of the accent color. *Alternative considered:* map the toggle to two curated built-in themes (e.g. `Theme::Light` / `Theme::TokyoNight`) — simpler but generic. We go custom; the palette lives in `style.rs` so swapping to built-ins later is a one-line change.

### 3. Theme state on `App`, applied via `App::theme`
- Add a field to `struct App` (`app.rs:56-79`), `dark_mode: bool` (source of truth = persisted `Config`).
- Add `fn theme(&self) -> Theme` returning `style::theme(self.dark_mode)` and change `app.rs:19` from `.theme(|_| Theme::Dark)` to `.theme(App::theme)`.
- Add `Message::ToggleTheme` (near `Navigate` in the enum, `app.rs:81-117`); its `update` arm flips `self.dark_mode`, updates `self.config.dark_mode`, persists via `self.config.save()`, and returns `Task::none()`.

*Why:* matches the existing `Navigate(Screen)` UI-state precedent and iced's `.theme(&self)` re-render model, so the switch applies app-wide instantly.

### 4. Persist theme in `qobuz-core` `Config`
Add `pub dark_mode: bool` to `Config` (`config.rs:16-37`) with a value in `impl Default` (`config.rs:39-53`, default `true` to preserve today's dark appearance). Read it in `App::new` (`app.rs:120-150`).

*Why:* `Config` already has `#[serde(default)]` (backward-compatible with existing config files) and an established load/save path, so the preference rides along with no new persistence machinery. A theme preference is non-secret and belongs with the other UI settings (quality, templates). *Alternative* — a separate GUI-local pref file — adds a second persistence mechanism for no benefit.

### 5. Theme toggle placement
Place the toggle in the nav row (`app.rs:441-453`), right-aligned near the signed-in status, as a compact `button` showing a sun/moon glyph (e.g. `☀`/`🌙`) emitting `Message::ToggleTheme`. Keeps it globally reachable from every screen without a new settings row.

### 6. Alignment & sizing normalization pass
Apply across `settings_view`/`search_view`/`queue_view`: `.align_y(Center)` on every row mixing inputs and buttons; replace `Space::with_width(16)` gaps with row `.spacing(SPACE_SM)`; standardize field widths (use `Length::Fill` for single main inputs, one `FillPortion` scheme where two share a row); route every button through the `action_button`/`secondary_button` helpers; use `labeled_row` for label+field rows so labels form an aligned column; unify the two `progress_bar` heights.

## Risks / Trade-offs

- **Hand-tuned palette may look off in one mode** → keep palette values centralized in `style.rs`; verify both modes visually by running the app; iterate on the five base colors only.
- **Fixed `LABEL_WIDTH`/`BUTTON_MIN_WIDTH` could truncate longer labels** → choose widths from the longest existing label ("Concurrency:", "Save settings"); labels are static so this is bounded.
- **Adding a `Config` field touches `qobuz-core`** → low risk: `#[serde(default)]` + `Default` value make it backward-compatible; there is a config test in `config.rs` to update/extend.
- **iced 0.13 default widget styles already respond to `Theme`** → most theming comes free from the palette; custom `.style` closures only needed where defaults look wrong, keeping the diff small.

## Migration Plan

No data migration. Existing `config.json` files load unchanged (missing `dark_mode` defaults to `true` = current behavior). Rollback = revert the change; older configs with a `dark_mode` field are ignored harmlessly by the previous binary (unknown field).

## Open Questions

- Exact accent/base colors for the custom palettes — resolve during implementation by eye while running the app (default accent: a muted blue/teal consistent with a music app).
- Default theme on first run: proposed **dark** (preserves current look); flip the `Default` value if a light default is preferred.
