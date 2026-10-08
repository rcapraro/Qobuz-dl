# Design

## Context

See proposal.md for motivation and specs/downloader-gui/spec.md for requirements.

`add_row` (`app/view/search.rs`) lays out a result: cover and title, an optional `quality_badge("Hi-Res")`, then `compact_button("Add")` sending `Message::Add(reference)`. `Message::Resolved(Ok(jobs))` already skips tracks whose id is queued, then switches to the Queue tab. `TrackResult.id` and `AlbumResult.id` are strings; `QueueItem.track_id` is an `i64`, and `QueueItem.job.album.id` is a string.

The only subscription is `iced::event::listen_with`, which maps window focus events. In iced 0.13:
- `keyboard::on_key_press` drops events a widget captured.
- A focused `text_input` captures every key press except Tab and the up/down arrows, including command combinations and Escape. On Escape it also unfocuses itself.
- Enter on a focused `text_input` publishes its `on_submit` whatever the modifiers.

## Goals / Non-Goals

**Goals:**
- Queue membership visible on results without new state.
- Shortcuts that work while typing in the search field.

**Non-Goals:**
- Marking already-queued tracks inside an album's detail.
- Arrow-key navigation of results, or rebindable shortcuts.
- Shortcuts for queue actions (Start, Cancel, Retry).

## Decisions

### "Added ✓" derived from the queue
`App::queued()` builds a `Queued` index, a `HashSet` of queued track ids and one of album ids, once per `results()` render. Its `track(&str)` parses the result id; `album(&str)` matches any queued track's album. The first cut scanned the queue once per result row; code review pointed out that, with a large playlist queued, that repeats on every download progress repaint. The index is still derived on each render, so it can't go stale.

`add_row` takes an `added: bool`:
- When `added` is true, the button is `compact_button("Added ✓")` with no press handler, which draws it disabled.
- Otherwise it is the usual Add.

Unit tests cover the index: a queued track, a non-queued or non-numeric id, an album with one queued track, and an emptied queue.

*Alternative:* a `HashSet` of added references. Rejected because it would need clearing whenever the queue changes (remove, clear, group remove), while the derived check is always right.

### Fixed quality slot
`add_row` always pushes a container `Length::Fixed(HIRES_SLOT_WIDTH)` wide, right-aligned, holding the badge or nothing. The width is a named constant sized to the "Hi-Res" badge, confirmed on screen. The Add / "Added ✓" button gets a fixed `ADD_BUTTON_WIDTH` that fits the longer label. On screen, the size-to-label compact button pushed an added row's badge left of the column.

### One `Shortcut` message
The `listen_with` closure maps a `Keyboard(KeyPressed { key, modifiers, .. })` with its `status` to `Message::Shortcut(Shortcut)`. `Shortcut` is a small enum: `FocusSearch`, `NextTab`, `PreviousTab`, `Escape` and `AddSelected`.

**Mapping:** pure and testable, in `app/shortcut.rs`.
- Ctrl+Tab and Ctrl+Shift+Tab map whatever the status. A focused field ignores Tab and never types it, because its text is a control character.
- Escape maps whatever the status.
- ⌘↵ (`modifiers.command()`) maps only when `status` is `Ignored`, because a focused field has already turned Enter into a search submit.
- `/` maps only when `Ignored`, so typing a `/` into the field stays typing. Shift is allowed because some layouts, such as AZERTY, need it for `/`. Ctrl+Alt is allowed because that is how Windows reports AltGr. A plain ⌘ or Ctrl chord is not.

No shortcut is a ⌘+character combination. The first cut used ⌘F and ⌘1/2/3. On screen, ⌘2 in the focused field switched tabs and also typed "2", because on macOS a ⌘+character press still carries its character and iced's `text_input` inserts it. That is iced's own behaviour, and no handler can undo it reliably.

**Guards,** all in one `update` arm:
- `FocusSearch` sets `screen = Search` and returns `text_input::focus(search_input_id())`, with the id set on the field in `search_view`. It does nothing from Settings: iced_aw's `NumberInput` (the concurrency field) leaves a rejected `/` uncaptured, so there a `/` typed into a field can't be told from one pressed in no field;
- `NextTab` / `PreviousTab` navigate in tab-bar order, wrapping around;
- `Escape` and `AddSelected` act only when the album is actually on screen: `screen == Search`, an album is open, and `view::search::setup_gap` reports nothing missing. Otherwise the Search tab shows the setup prompt in the album's place.

Unit tests cover:
- each key's mapping, including ⌘↵ and `/` with a captured status, and ⌘+characters mapping to nothing;
- the wrap-around order;
- the guards.

*Alternative:* `keyboard::on_key_press`. Rejected because it never sees keys pressed while the search field has focus, which is most of the time.

## Risks / Trade-offs

- [Ctrl+Tab may be taken by the OS or a window manager before the app sees it] → Not bound globally on macOS, which is checked on screen. On Windows and Linux it is untested here; the tab bar stays clickable either way.
- [`/` is less discoverable than ⌘F] → It's the convention many apps use for focusing search, and the README lists it.
- [Esc in a focused field both unfocuses it and closes the album] → Intended: one press leaves the album, the way a dismiss key usually works.
- [An "Added ✓" album can't be re-added from the results to pick up its missing tracks] → Opening the album and adding the rest still works, and dedup skips what's queued.
