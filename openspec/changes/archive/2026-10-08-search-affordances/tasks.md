# Tasks

## 1. Queued results and quality column

- [x] 1.1 Add `App::queued()`, a per-render `Queued` index with `track(&str)` and `album(&str)`, with unit tests: a queued track, a non-queued track, an album with one queued track, and an emptied queue. Verify with `cargo test -p qobuz-gui queued_`.
- [x] 1.2 In `add_row`, take an `added` flag and render `compact_button("Added ✓")` without a press handler when set. Always push a right-aligned slot of `HIRES_SLOT_WIDTH` holding the Hi-Res badge or nothing. Verify in the rebuilt app (confirming the app window is in front before any click) that adding an album turns its Add into "Added ✓" on returning to Search, that Clear queue turns it back, and that badges and buttons form straight columns.

## 2. Keyboard shortcuts

- [x] 2.1 Add `Shortcut` (`FocusSearch`, `NextTab`, `PreviousTab`, `Escape`, `AddSelected`) and a pure `shortcut(key, modifiers, status) -> Option<Shortcut>` mapping: Ctrl+Tab / Ctrl+Shift+Tab and Escape whatever the status; `/` and ⌘↵ only when `Ignored`; no ⌘+character combinations. Unit tests cover each key, the captured cases, ⌘+characters mapping to nothing, and wrap-around. Verify with `cargo test -p qobuz-gui shortcut`.
- [x] 2.2 Map key presses to `Message::Shortcut` in the existing `listen_with` subscription. Handle them in one `update` arm with the guards from the design, and give the search field an id for `text_input::focus`. Unit tests for the guards: Escape with no album is a no-op, and AddSelected off the Search tab is a no-op. Verify in the rebuilt app: `/` focuses the field from Settings, Ctrl+Tab while typing switches tabs without adding a character to the query, Esc leaves an album's detail, and ⌘↵ adds its selection when the field isn't focused.
- [x] 2.3 Add the shortcuts to the README Features list, and verify the list renders as one bullet.

## 3. Checks

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass with no new warnings.

## Workflow follow-up

- Refresh all four README screenshots once this lands.
- Archive the change (`/opsx:archive`).
