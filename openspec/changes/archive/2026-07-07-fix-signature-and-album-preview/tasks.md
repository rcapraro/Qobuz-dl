## 1. Fix the download signature bug (qobuz-core)

- [x] 1.1 In `QobuzClient::get` (`client.rs`), make signature-failure detection case-insensitive: extracted `is_signature_error(body)` — lowercases the body and matches "signature" plus ("invalid" or "request_sig")
- [x] 1.2 Added unit tests: capital-I "Invalid Request Signature parameter (request_sig)" is classified as a signature error, and 401/app_id errors are not misclassified

## 2. Album cover thumbnails (qobuz-gui)

- [x] 2.1 `SearchPayload.albums` is now `Vec<AlbumResult { id, label, cover: Option<String> }>`; `do_search` fills `cover` from `album.image` (prefers small → thumbnail → large)
- [x] 2.2 Added `thumbnails: HashMap<String, image::Handle>` to `App` and initialized it
- [x] 2.3 Added `Message::ThumbnailLoaded(String, Result<Vec<u8>, ()>)` and `fetch_thumbnail` (delegates to new core `qobuz_core::fetch_bytes` — keeps the GUI free of direct HTTP)
- [x] 2.4 On `SearchDone`, batch-fetch each uncached album cover (deduped); `ThumbnailLoaded(Ok)` inserts `Handle::from_bytes` into the cache, errors ignored
- [x] 2.5 `album_result_row` renders a 52px cover `image` when cached, else a `thumb_placeholder` box of the same size, so the list is usable while thumbnails stream in

## 3. Verification

- [x] 3.1 `cargo fmt`, `cargo clippy --workspace`, and `cargo test -p qobuz-core` (37) all pass
- [x] 3.2 Verified end-to-end against the live API: candidate secrets 1–3 return "Invalid Request Signature", candidate 4 (`abb21364…`) returns a valid `getFileUrl` — confirming the fix makes `file_url` reach the working secret. GUI launches clean; live in-app download left for the user to confirm.
