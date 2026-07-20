## 1. Display models

- [x] 1.1 In `crates/qobuz-gui/src/app.rs`, remove the `artists` field from `SearchPayload`.
- [x] 1.2 Add `cover: Option<String>` to `TrackResult`.

## 2. Populate results

- [x] 2.1 In `crates/qobuz-gui/src/app/tasks.rs::do_search`, delete the `r.artists` loop.
- [x] 2.2 Populate `TrackResult.cover` from the track's album image with the same small→thumbnail→large preference used for albums: `t.album.as_ref().and_then(|al| al.image.as_ref()).and_then(|i| i.small.clone().or_else(|| i.thumbnail.clone()).or_else(|| i.large.clone()))`.

## 3. Thumbnail fetch + result count

- [x] 3.1 In `crates/qobuz-gui/src/app.rs` `Message::SearchDone(Ok(..))`, change the result count to `payload.albums.len() + payload.tracks.len()`.
- [x] 3.2 Extend the `wanted` cover-URL set to also include `payload.tracks.iter().filter_map(|t| t.cover.clone())` so track thumbnails are fetched and cached (eviction/fetch logic unchanged).

## 4. View

- [x] 4.1 In `crates/qobuz-gui/src/app/view/search.rs`, remove the Artists card block from `search_view` (and drop now-unused imports/refs).
- [x] 4.2 Render track rows with a cover thumbnail: reuse the 52×52 cover element (cached `image::Handle` or `thumb_placeholder`) from `album_result_row` for tracks, keeping the title/artist/Hi-Res badge. Factor the shared cover-element construction so it isn't duplicated.

## 5. Verify

- [x] 5.1 `cargo fmt` and `cargo clippy --workspace --tests` clean.
- [x] 5.2 `cargo build --workspace` clean.
- [x] 5.3 `cargo run -p qobuz-gui`: run a search and confirm no Artists section appears, track rows show a preview thumbnail (placeholder when absent), and album rows/badges still render correctly.
