## Why

The `qobuz-gui` UI is hand-rolled from base `iced` widgets: section navigation is
a row of buttons, concurrency is a free-text field parsed and clamped on every
keystroke, the template help is a manual show/hide `column`, and queue status is
plain text. This reads as utilitarian and carries avoidable state (a `String` for
a number, a bool flag for a panel). Adopting the official `iced_aw` companion crate
provides purpose-built widgets that map almost one-to-one onto these pieces,
improving legibility while removing custom plumbing.

## What Changes

- Add the `iced_aw = 0.12.2` dependency (the only line compatible with the
  project's `iced 0.13.1`) and enable iced's `advanced` feature it requires.
- Replace the button-row navigation with a real **tab bar** (`iced_aw::Tabs`) for
  the Settings / Search-Add / Queue sections; move the theme toggle and sign-in
  indicator to a slim header row above the tabs.
- Replace the free-text concurrency field with a bounded **number input**
  (`iced_aw::number_input`, range 1–16), removing the `String` state and the
  parse-and-clamp-on-keystroke handler.
- Present the template syntax help inside a **card** (`iced_aw::Card`) while keeping
  the existing show/hide toggle and the live path preview.
- Show per-item queue status as a colored **badge** (`iced_aw::Badge`) instead of
  plain text; keep progress bars.
- Group search results (Albums/Tracks/Artists) and Settings sections in **cards**,
  keeping the per-row add controls.

## Capabilities

### New Capabilities
<!-- None: this change enhances presentation of existing capabilities. -->

### Modified Capabilities
- `downloader-gui`: navigation becomes a tab bar; concurrency edited via a bounded
  numeric input; queue status shown as colored badges; search results grouped in
  cards with per-row add controls.
- `template-help`: syntax help rendered inside a card container while remaining
  toggleable (toggle, copy, and apply behavior unchanged).

## Impact

- **Crate:** `qobuz-gui` only. `qobuz-core` is untouched (no API, config, or
  behavior changes).
- **Dependencies:** adds `iced_aw` (feature-gated: `tabs`, `number_input`, `card`,
  `badge`); adds the `advanced` feature to the existing `iced` dependency.
- **Code:** `crates/qobuz-gui/src/app.rs` (view + navigation + concurrency handling,
  queue/search rendering) and `crates/qobuz-gui/src/style.rs` (new iced_aw style
  helpers matching the existing custom light/dark palette).
- **No breaking changes** to persisted config, the download engine, or the
  `qobuz-core` interface.
