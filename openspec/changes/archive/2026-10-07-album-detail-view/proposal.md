# Proposal

## Why

Adding an album from search queues every track on it, sight unseen. The user
cannot check the track list, edition, or year before committing, and cannot
take only some tracks: a deluxe edition with bonus discs, or the one song
they wanted from a long compilation. Today the only ways around that are
removing tracks from the queue one by one, or finding each track's URL.

## What Changes

- Activating an album search result opens that album's detail inside the
  Search screen, in place of the results list. The row's existing "Add"
  control still adds the whole album directly.
- The detail shows the cover, title, artist, year, label, genre, track count,
  total duration, and a Hi-Res badge, followed by the track list: number,
  title, the performer when it differs from the album artist, duration, and a
  per-track Hi-Res badge, grouped by disc on multi-disc albums.
- Every track has a checkbox, and all start checked. "Select all" and
  "Select none" adjust the selection, and one primary control adds the
  checked tracks, labelled with their count.
- A Back control returns to the same results at the same scroll position. A
  new search from the field also leaves the detail.
- While the album loads, the detail shows what the result row already knew
  (cover, title, artist) with a loading indicator. A failure is shown in place
  with a Retry control.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gui-theming`: "Semantic accent colors" gains lavender as the accent for a
  page's identity panel (the album header), keeping its existing scenarios.
- `downloader-gui`: new requirements for opening an album's detail from
  search, selecting tracks to add, and returning to results. The existing
  "Search and add screen" requirement is not modified. Its per-row add
  control keeps adding the whole album.

## Impact

- `qobuz-gui` only. `app.rs` (detail state, open/back/select/add messages,
  scroll-offset tracking), a new `app/view/album.rs`, `app/view/search.rs`
  (clickable album rows, results scrollable id and `on_scroll`).
- `qobuz-core`: no change. The detail is built from `engine::resolve` on the
  album reference, whose jobs already carry the album metadata and tracks, so
  adding a selection reuses those jobs without fetching the album again.
- Dependencies: none added.
- Follows `search-pagination`, now archived, and precedes `queue-grouping`.
