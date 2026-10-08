# Proposal

## Why

Three small frictions remain on the Search screen:
- **No sign of what's queued.** An Add button looks the same before and after its album or track was queued. Adding again is harmless (queued tracks are skipped), but nothing on the result says it's already there.
- **A jagged right edge.** The Hi-Res badge appears on some result rows and not others, so the Add buttons sit at the same x, but the column before them has no edge to read along.
- **Mouse-only use.** Focusing the search, switching tabs, leaving an album's detail and adding a selection all need the mouse.

This is the last change of the 2026-10-08 polish pass (settings-forms → chrome-and-layout → queue-row-redesign → search-affordances).

## What Changes

- **Queued results are marked:** a result's Add button reads "Added ✓" and is disabled while that track, or any track of that album, is in the queue. It turns back into Add once those tracks leave the queue. This is derived from the queue, with nothing new stored. To add more of a partly added album, open it and pick tracks as today.
- **Aligned quality column:** every result row reserves the Hi-Res badge's width, filled or empty, and Add / "Added ✓" share one fixed width, so rows line up whether or not a release is hi-res or queued.
- **Keyboard shortcuts:**

  | Shortcut | Action |
  |---|---|
  | `/` | Focus the search field, on the Search tab (when not already typing) |
  | Ctrl+Tab / Ctrl+Shift+Tab | Next / previous tab, wrapping around |
  | Esc | Close an open album's detail, back to the results |
  | ⌘↵ (Ctrl+Enter elsewhere) | Add the selected tracks while an album's detail is open, unless the search field has focus (there, Enter submits the search) |

  None of them types a character into a focused field. ⌘F and ⌘1/2/3 were tried first, but on macOS a ⌘+character press still carries its character, which iced's text field types in.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`: adds "Queued results are marked", "Aligned result quality column", "Navigation shortcuts" and "Album detail shortcuts". "Search and add screen" is unchanged.

## Impact

- `crates/qobuz-gui/src/app.rs`:
  - a `Message::Shortcut` carrying a small `Shortcut` enum, handled in one place;
  - the keyboard mapping added to the existing `event::listen_with` subscription;
  - a queue membership helper for results.
- `crates/qobuz-gui/src/app/view/search.rs`: the Add/Added button, the fixed quality slot, and the search field's widget id.
- The README Features list gains the shortcuts.
