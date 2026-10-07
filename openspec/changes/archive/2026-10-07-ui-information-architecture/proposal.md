# Proposal

## Why

The app works, but its layout reads as a prototype: the one-time Settings
screen is the first tab and the launch screen, search and add-by-URL are two
separate bars for one task, every status message looks the same whether it
reports progress, success, or failure, and accent colors carry no stable
meaning (card headers pick blue, green, mauve, teal, or peach arbitrarily while
green also means "done"). This change fixes the information architecture and
makes the Catppuccin styling consistent, as the foundation for the follow-up
changes `search-pagination`, `album-detail-view`, and `queue-grouping`.

## What Changes

- Reorder the tabs to Search, Queue, Settings and open the app on Search.
- Show the number of tracks still to process on the Queue tab label.
- Replace the two Search-screen bars with a single field: a Qobuz URL is added
  to the queue, anything else is searched. Input that is also a valid bare ID
  additionally offers an explicit "add by ID" action rather than silently
  hijacking the search.
- When credentials or sign-in are missing, the Search screen shows a setup
  prompt that leads to Settings, instead of failing on use.
- Status messages become typed (info, progress, success, error), each with a
  distinct accent and icon. Errors stay until replaced or dismissed.
- Assign each Catppuccin accent a single meaning (brand, primary action,
  success, in progress, error, quality). Card headers become neutral surfaces.
- Every button comes from the shared style builders. A compact variant is
  added for per-row actions, replacing the hand-built buttons on the Queue
  screen.
- Fixes: the search placeholder no longer mentions artists, and the theme
  toggle uses a plain text label instead of a mismatched glyph.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`: tab order, launch screen, and Queue tab count (Tabbed
  navigation); the single search-or-add field (Search and add screen); new
  requirements for the setup prompt and typed status messages.
- `gui-theming`: a new requirement fixing accent colors to semantic roles; the
  control-sizing requirement gains a compact row-action variant and requires
  every button to use a shared variant.

## Impact

- Code: `crates/qobuz-gui` only. `app.rs` (screen order and initial screen,
  status type, omnibox message handling), `app/view/{search,queue,settings}.rs`,
  `app/help.rs`, `style.rs` (semantic color helpers, compact button builder,
  status styles).
- `qobuz-core`: no change. The omnibox reuses `catalog::parse_input`.
- Dependencies: none added.
- Follow-up changes `search-pagination` and `album-detail-view` will write their
  `downloader-gui` deltas against the spec as synced after this change is
  archived.
