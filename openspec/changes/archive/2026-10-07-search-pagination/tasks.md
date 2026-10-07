# Tasks

## 1. Core: per-type paged search

- [x] 1.1 Add `offset` and `limit` (`#[serde(default)]`) to `AlbumList` and verify with a deserialization test that an `album/search`-shaped body (`{"albums": {"items": [...], "total": 140, "offset": 25, "limit": 25}}`) yields those values, and that a body without `total` yields `None`
- [x] 1.2 Add `search_albums(query, limit, offset)` and `search_tracks(query, limit, offset)` calling `album/search` / `track/search`, defaulting to an empty list when the type's field is absent, remove `QobuzClient::search`, and verify `cargo build --workspace` succeeds with no remaining caller
- [x] 1.3 Run a live check of both endpoints (the env-gated pattern in `tests/integration.rs`, or a signed-in app search) and verify page 2 of a broad query returns different items than page 1 and that `total` is populated

## 2. GUI: paging state

- [x] 2.1 Introduce `Section<T>` with pure `has_more` and `append` in the GUI, make `SearchPayload` hold two sections, and verify unit tests for: total known with more / exhausted, total unknown with a full / short last page, duplicate ids skipped, and `next_offset` advancing by the raw page length even when every item is a duplicate
- [x] 2.2 Fetch the first album and track pages concurrently in `tasks::do_search` via `try_join`, and verify that a search still shows both sections, badges, and thumbnails in the running app
- [x] 2.3 Add `search_generation`, carry it on `SearchDone`, and drop mismatched results, and verify with an update test that an older `SearchDone` does not replace newer results

## 3. GUI: Show more

- [x] 3.1 Add `Message::ShowMore(Kind)` / `MoreAlbums(u64, ..)` / `MoreTracks(u64, ..)` with the loading flag, append, error, and stale-generation handling, and fetch covers for appended rows without evicting, and verify update tests for: append to one section only, a stale page discarded, and a failure keeping existing items and clearing `loading`
- [x] 3.2 Render the `Albums · N of T` header and the "Show more" / "Loading…" control per section, and verify in the running app that a broad query pages albums and tracks independently and the control disappears at the end

## 4. Documentation and checks

- [x] 4.1 Update the README feature bullet and the `client.rs` line in `CLAUDE.md` (per-type search endpoints with `offset`), and verify the text matches the code
- [x] 4.2 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`, and verify all pass

## Workflow follow-up

- Archive and sync this change before proposing `album-detail-view`.
