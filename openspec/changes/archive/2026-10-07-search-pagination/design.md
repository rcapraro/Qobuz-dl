# Design

## Context

- `QobuzClient::search(query, limit)` calls `catalog/search` with `query` and
  `limit` only, and `tasks::do_search` asks for 25. The response's `albums` and
  `tracks` lists carry `total` (and, for tracks, `offset`/`limit`), but the GUI
  drops them when it flattens results into `SearchPayload { albums, tracks }`.
- `catalog/search` returns both types from one request, so paging one section
  through it would refetch the other. Whether it accepts `offset` at all is not
  confirmed by any reference client.
- streamrip (`streamrip/client/qobuz.py`) searches through per-type endpoints,
  `f"{media_type}/search"` (`album/search`, `track/search`), with `query`,
  `limit`, and `offset`. It pages by reading `total`/`limit`/`offset` from the
  response object keyed `albums` or `tracks`.
- `qobuz-core` has no HTTP mocking. The only integration test is a live,
  env-gated one.

## Goals / Non-Goals

**Goals:**
- Page each section independently, with the paging decisions in pure
  functions that unit tests can cover.
- No new dependency.

**Non-Goals:**
- Infinite scroll or automatic loading on reaching the bottom. An explicit
  control is predictable and avoids firing requests while the user scrolls.
- Changing the page size or making it configurable. It stays at 25.
- Playlist or artist search.

## Decisions

### Per-type endpoints for every search request

The core gains:

```rust
pub async fn search_albums(&self, query: &str, limit: u32, offset: u32) -> Result<AlbumList>
pub async fn search_tracks(&self, query: &str, limit: u32, offset: u32) -> Result<TrackList>
```

Each calls `album/search` or `track/search` and deserializes into the existing
`SearchResults`, taking its `albums` or `tracks` field and defaulting to an
empty list when the field is absent. `AlbumList` gains `offset` and `limit`
(`#[serde(default)]`), matching `TrackList`. `QobuzClient::search` is removed
rather than kept beside them, so the app has exactly one way to search.

The first page of both types also goes through these endpoints, as two
concurrent requests joined with `iced::futures::future::try_join`, the
re-export the GUI already uses for `future::join`. That way page 2 continues
the same ordering as page 1.

*Alternative:* keep `catalog/search` for the first page and use the per-type
endpoints only for "Show more". Rejected because two endpoints may rank results
differently, so page 2 could skip or repeat results from page 1.

### One section type, one paging rule

```rust
struct Section<T> {
    items: Vec<T>,
    next_offset: u32,      // offset of the last request + items it returned
    total: Option<u32>,
    last_page_len: usize,
    loading: bool,
}
```

`SearchPayload` becomes `{ albums: Section<AlbumResult>, tracks: Section<TrackResult> }`.
Two pure functions decide everything:

- `has_more(section, PAGE_SIZE)`: with a total, `next_offset < total`.
  Without one, `last_page_len == PAGE_SIZE`.
- `append(section, page, id_of)`: advances `next_offset` by the page's raw
  length, then pushes only items whose id is not already present.

`next_offset` counts what the API returned, not what survived de-duplication.
Otherwise a page full of repeats would request the same offset forever.

### Stale-page guard by search generation

`App` holds `search_generation: u64`, incremented on every submitted search.
`SearchDone`, `MoreAlbums`, and `MoreTracks` carry the generation they were requested under,
and a mismatch is dropped. A generation is used instead of comparing query
text, because resubmitting the same query is still a new search. The guard
also covers the existing race where an older `SearchDone` lands after a newer
one.

### Messages and loading state

`Message::ShowMore(Kind)` sets that section's `loading` and requests
`Kind::Albums | Kind::Tracks` from `next_offset`.
`Message::MoreAlbums(u64, Result<Page<AlbumResult>, String>)` and its
`MoreTracks` twin each carry a page of their own row type. A single
`MoreLoaded(Kind, …)` would need an enum of page types and a second match on
the kind. Both messages go through one generic `more_loaded` helper, which
clears `loading`, then either appends or returns the error, keeping the
existing items. Covers for
appended rows are fetched like the first page's. Cache eviction still happens
only when a new search starts, so appending never drops covers already on
screen.

### Section header shows the count

The card header becomes `Albums · 25 of 140`, or `Albums · 25` when the total
is unknown. The header is built with `card_el` because the title is now an
owned string. "Show more" is a secondary button at the foot of the section's
rows. While loading it reads "Loading…" and is disabled.

## Risks / Trade-offs

- [The per-type endpoints could return different fields from `catalog/search`,
  for example no `hires_streamable` or `image.small`] → The models are already
  `#[serde(default)]` throughout. A live search check in tasks confirms the
  badges and thumbnails still appear.
- [Two requests per search instead of one] → Two small JSON GETs, run
  concurrently. A 429 surfaces as "Search failed", as a failed search does
  today.
- [If either first-page request fails, the whole search fails] → This is the
  same outcome as today's single request failing, and it avoids half-populated
  results that look complete.
- [Removing `QobuzClient::search` breaks the core's public API] → The only
  consumer is this workspace's GUI. The core has no external users or semver
  promise.
