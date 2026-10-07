# Proposal

## Why

A search returns at most 25 albums and 25 tracks, with no way to see more.
Broad queries (a prolific artist, a common word) bury the wanted release past
that cut-off, so the user has to guess a narrower query or fall back to
finding a URL elsewhere.

## What Changes

- Albums and tracks are fetched as independent pages. The first page of each is
  requested when the user searches, as today.
- Each results section shows how many results it holds out of the total and
  offers a "Show more" control while more remain. Activating it appends the
  next page to that section only.
- Results that a later page repeats are not shown twice.
- A page that arrives after the user started a different search is discarded.
- The search requests move from the combined `catalog/search` endpoint to the
  per-type `album/search` and `track/search` endpoints, which take an `offset`.
  `QobuzClient::search` is replaced by `search_albums` and `search_tracks`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `catalog-browsing`: a new requirement for fetching search results per type
  in pages, with a reported total.
- `downloader-gui`: a new requirement for the per-section result count and
  "Show more" control. The existing "Search and add screen" requirement is
  not modified.

## Impact

- `qobuz-core`: `client.rs` (search methods), `models.rs` (`AlbumList` gains
  `offset`/`limit`, matching `TrackList`), `lib.rs` re-exports if needed.
  `QobuzClient::search` is removed. Its only caller is the GUI.
- `qobuz-gui`: `app/tasks.rs` (two page requests), `app.rs` (per-section
  paging state, load-more messages, stale-page guard), `app/view/search.rs`
  (section count and "Show more").
- Dependencies: none added.
- Follows `ui-information-architecture`, now archived, and precedes
  `album-detail-view`.
