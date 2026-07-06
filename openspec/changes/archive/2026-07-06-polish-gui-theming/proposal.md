## Why

The current GUI relies entirely on default `iced` styling with ad-hoc, inconsistent sizing, spacing, and alignment across the Settings, Search, and Queue screens. It looks unpolished and offers no way to switch between light and dark appearances. A cohesive, professional theme with consistent controls and a user-controllable light/dark switch makes the app feel trustworthy and comfortable to use in any environment.

## What Changes

- Introduce a centralized visual design system (shared spacing, padding, control-size, and typography constants) so buttons and text inputs have consistent dimensions and alignment across all screens.
- Align form fields and their labels/buttons on a consistent grid: labels, inputs, and action buttons share widths and baselines within each row.
- Standardize button sizing (primary/secondary variants with consistent height, padding, and minimum width) and text-input sizing (consistent height and width behavior).
- Apply a clean, sleek custom theme (custom `iced::Theme` palette + styled containers/buttons/inputs) replacing raw default styling.
- Add a light/dark theme toggle in the UI, defaulting to a sensible initial theme and applying the choice app-wide via `App::theme()`.
- Persist the selected theme preference so it is restored on next launch.

## Capabilities

### New Capabilities
- `gui-theming`: Application-wide visual theming and appearance — a cohesive light/dark theme, a user-facing theme switch, persisted theme preference, and consistency requirements for control sizing, spacing, and field/button alignment across all screens.

### Modified Capabilities
<!-- None: openspec/specs/ has no synced main specs yet; GUI behavior specs live only in the unarchived add-qobuz-downloader change. This introduces a new, self-contained theming capability. -->

## Impact

- **Code:** `crates/qobuz-gui/src/app.rs` (view functions `settings_view`/`search_view`/`queue_view`, `struct App` state, `Message` enum, `update`, and app setup incl. a new `theme()` handler). Likely a new small styling module (e.g. `crates/qobuz-gui/src/theme.rs` or `style.rs`) for palette + widget style helpers and layout constants.
- **Persistence:** a theme-preference field added to persisted settings — either in `qobuz-core`'s `Config` (`crates/qobuz-core/src/config.rs`) or a GUI-local persisted preference; decided in design.md.
- **Dependencies:** none expected beyond existing `iced` 0.13 capabilities; no new crates anticipated.
- **No functional/behavioral change** to authentication, search, download, tagging, or file organization — this is presentation-layer only.
