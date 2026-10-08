# Proposal

## Why

The window chrome (header, status line, tab bar, bordered pane) takes about 200px before any content appears, and it is louder than what it frames:
- A routine "50 results." gets the same coloured frame as an error.
- The theme toggle is a full 130px button for something used about once.
- The tabs are three solid slabs with the active one filled blue.
- Every tab's content sits inside an extra bordered pane.

Content also has no shared left edge: scrolled content is inset by a scrollbar gutter on both sides, so the Search field and the result cards (or the Queue header and its groups) start at different positions.

This is the second change of the 2026-10-08 polish pass (settings-forms → chrome-and-layout → queue-row-redesign → search-affordances).

## What Changes

- **Status line:** only an error is outlined in its accent colour. Info, progress and success keep their coloured icon on a plain neutral surface. The status line stays a persistent record of the last message, with no auto-expiry and no toasts.
- **Theme toggle:** it becomes a compact control instead of a full-size secondary button.
- **Tab bar:** lighter treatment. Inactive tabs have no fill, and the active tab is marked by the blue accent on a raised neutral surface rather than a solid blue slab.
- **Search tab label:** "Search / Add" becomes "Search". The omnibox already makes adding part of searching.
- **Tab content:** no longer enclosed in a bordered pane. It starts at the same left edge as the header, status line and tab bar on every tab.
- **Scrollbars:** scrollable content reserves a gutter on the right only, so it lines up with the fixed controls above it.

No behaviour changes. Status message lifetime, theme switching and persistence, tab order and the Queue count all stay as they are.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`: "Tabbed navigation" names the first tab "Search". Its order, counting and global-control scenarios are kept.
- `gui-theming`:
  - Modifies "Consistent control sizing" so the compact variant also covers small header and inline controls, such as the theme toggle and the status dismiss control.
  - Adds requirements for status-line emphasis by kind, tab-bar emphasis, and a shared content edge with unframed tab content.

## Impact

- `crates/qobuz-gui/src/app.rs`: the header (theme toggle), tab label text, and `tab_pane` (no border, no inner padding).
- `crates/qobuz-gui/src/style.rs`: `tab_bar` and `status_surface` styles. `panel` is removed if `tab_pane` was its only user.
- `crates/qobuz-gui/src/app/view/mod.rs`: `status_bar` outline per kind, and `gutter_padding` becomes right-only.
- All four README screenshots go out of date.
