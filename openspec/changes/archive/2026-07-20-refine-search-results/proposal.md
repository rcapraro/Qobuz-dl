## Why

Artist results in search are rarely useful for downloading — an artist link
isn't directly downloadable (the resolver rejects it and tells the user to pick
an album), so the Artists card is dead weight. Meanwhile track rows show no
cover, making them harder to recognise than album rows. Dropping artists and
adding a track preview thumbnail makes the results list cleaner and more
scannable.

## What Changes

- **Remove the Artists category** from search results entirely — the search
  screen shows only Albums and Tracks.
- **Show a preview thumbnail on track rows**, using the track's album cover
  from the search response, loaded asynchronously like album thumbnails (with
  a placeholder while loading or when no cover is available).
- No API/endpoint change — the search call is unchanged; artist items in the
  response are simply not surfaced.

## Capabilities

### New Capabilities

<!-- None: refines existing search behavior. -->

### Modified Capabilities

- `catalog-browsing`: search surfaces only albums and tracks (not artists), and
  track results expose a preview image.
- `downloader-gui`: the Search/Add screen omits the Artists card and shows a
  cover thumbnail on track rows.

## Impact

- **qobuz-gui** — `app.rs` (`SearchPayload` loses its `artists` field;
  `TrackResult` gains `cover: Option<String>`; the `SearchDone` handler also
  collects track covers into the thumbnail-fetch set), `app/tasks.rs::do_search`
  (drop the artists loop, populate track cover from the track's album image),
  and `app/view/search.rs` (remove the Artists card; render track rows with a
  thumbnail).
- No changes to `qobuz-core`, networking, the download engine, or config.
  Backwards compatible; no breaking changes.
