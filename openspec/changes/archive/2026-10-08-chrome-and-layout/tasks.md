# Tasks

## 1. Header and status line

- [x] 1.1 Replace the header's theme `secondary_button` with a `compact_button` labelled with the target theme ("Light" / "Dark"). Add a ☀/☾ glyph only if both render in the running app. Verify the toggle still switches and persists the theme with `cargo run -p qobuz-gui`.
- [x] 1.2 Change `style::status_surface` to take an optional outline. Pass the error accent for errors and nothing for the other kinds, which then use a neutral border of the same width. Verify in the running app: a search shows a progress then a success message with no coloured outline, and an error (e.g. Check signing while signed out) is outlined in red, with the bar's height and position unchanged.

## 2. Tabs and content edge

- [x] 2.1 Restyle `style::tab_bar`:
  - no bar background or border;
  - active label: `surface1` with `primary()` text;
  - hovered label: `surface0`;
  - inactive label: transparent with regular text.

  Rename the first tab's label to "Search" and update the `search.rs` module doc. Verify in the running app in both themes.
- [x] 2.2 Remove the border and inner padding from `tab_pane`, keeping a `SPACE_LG` gap below the tab bar. Delete `style::panel` once it has no users. Verify in the running app that no pane outline remains on any tab.
- [x] 2.3 Make `gutter_padding()` right-only. Verify in the running app that the Search field and result cards, the Queue header and its groups, the album view's Back button and track table, and the Settings cards all start at the tab bar's left edge.

## 3. Checks

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.

## Workflow follow-up

- Refresh the four README screenshots (`docs/screenshots/*.png`).
- Archive the change (`/opsx:archive`) before proposing `queue-row-redesign`.
