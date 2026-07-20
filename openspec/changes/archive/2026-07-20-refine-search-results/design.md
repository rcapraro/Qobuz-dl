## Context

Search results are reduced to display models in `app/tasks.rs::do_search`, then
stored in `App.results: SearchPayload` (`app.rs`) and rendered by
`app/view/search.rs`. Today `SearchPayload { albums: Vec<AlbumResult>,
tracks: Vec<TrackResult>, artists: Vec<(String, String)> }`; `TrackResult` has
`{ id, title, artist, hires }` and carries no cover.

Album thumbnails already work end to end: `do_search` picks a small cover URL
per album; the `SearchDone` update handler builds a `wanted` set of album cover
URLs, evicts stale entries from `App.thumbnails`, and spawns a
`tasks::fetch_thumbnail` task per new URL; `search.rs::album_result_row` renders
the cached `image::Handle` (or a `thumb_placeholder` while loading). The core
`Track` model already exposes its album via `Track.album: Option<Album>`, whose
`Image` has `small`/`thumbnail`/`large` — so a track preview URL needs no API
change.

Artist links are already non-downloadable (`engine::resolve` returns an error
for `Reference::Artist`), which is why the Artists card adds little value.

## Goals / Non-Goals

**Goals:**
- Stop surfacing artist results in the search screen.
- Give track rows a cover thumbnail sourced from the track's album image,
  reusing the existing thumbnail cache and fetch pipeline.

**Non-Goals:**
- No change to the search API call or to `qobuz-core`.
- No change to album rows beyond what already exists.
- No removal of `Reference::Artist` parsing (pasted artist URLs still error as
  before) — only the search *display* drops artists.

## Decisions

- **Drop `artists` from `SearchPayload`** and remove the artists loop in
  `do_search`. The result count in `SearchDone` becomes
  `albums.len() + tracks.len()`.
- **`TrackResult` gains `cover: Option<String>`**, populated in `do_search`
  from the track's album image with the same small→thumbnail→large preference
  used for albums:
  `t.album.as_ref().and_then(|al| al.image.as_ref()).and_then(|i| i.small.clone().or_else(|| i.thumbnail.clone()).or_else(|| i.large.clone()))`.
- **Thumbnail fetch set includes track covers.** In the `SearchDone` handler,
  extend the `wanted` set (currently album covers only) to also include
  `payload.tracks.iter().filter_map(|t| t.cover.clone())`. Eviction and
  per-URL fetch logic are unchanged, so track thumbnails load and cache exactly
  like album thumbnails. A track whose album cover URL equals an album's is
  naturally deduped by the `HashSet`.
- **Render track rows with a thumbnail.** Generalise the search-view row so a
  track row shows its cover (or the loading placeholder) the same way
  `album_result_row` does — reuse the existing 52×52 cover element construction
  rather than duplicating it. Track rows keep their title/artist/Hi-Res badge.
- **Remove the Artists card** block from `search_view`.

## Risks / Trade-offs

- **Missing track album image:** some track search items may lack an embedded
  album image; `cover` is then `None` and the row shows the placeholder — the
  same graceful degradation album rows already have.
- **More thumbnails fetched:** track covers add to the fetch set, but they are
  small images, deduped against album covers, and loaded lazily off the UI
  thread — consistent with current behavior and bounded by the 25-result page.
