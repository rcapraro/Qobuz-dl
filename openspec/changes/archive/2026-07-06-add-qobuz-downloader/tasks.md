## 1. Scaffold workspace

- [x] 1.1 Create root `Cargo.toml` workspace with members `crates/qobuz-core` and `crates/qobuz-gui`, plus release profile (`lto=true`, `codegen-units=1`, `strip=true`, `panic="abort"`)
- [x] 1.2 Create `crates/qobuz-core` lib crate and `crates/qobuz-gui` bin crate (gui depends on core)
- [x] 1.3 Add `.gitignore` (target/, secrets), and a README with build/run instructions
- [x] 1.4 Verify `cargo build --workspace` succeeds on the empty scaffold

## 2. Core: config, models, errors

- [x] 2.1 Define `error.rs` with a `thiserror` error type covering network, auth, signature, IO, and tagging failures
- [x] 2.2 Define serde `models.rs` for album/track/artist/playlist/search responses and `getFileUrl` (temporary URL + actual `bit_depth`/`sampling_rate`)
- [x] 2.3 Implement `config.rs`: serde settings (download dir, quality, folder/track templates, embed-art toggle, concurrency, app_id, app_secret) with load/save via `directories` and sensible defaults
- [x] 2.4 Ensure `user_auth_token`/password are never written to the config file

## 3. Core: client and authentication

- [x] 3.1 Implement `client.rs` reqwest client sending `X-App-Id` and `X-User-Auth-Token` headers
- [x] 3.2 Implement `login(email, password)` → `user_auth_token`, rejecting free/ineligible accounts with a clear error
- [x] 3.3 Implement `login_with_token(user_id, token)` direct-token path
- [x] 3.4 Store/retrieve the token via the `keyring` crate; fail gracefully when the keyring is unavailable

## 4. Core: signing, getFileUrl, quality

- [x] 4.1 Implement `quality.rs` `Quality` enum ↔ `format_id` (5/6/7/27)
- [x] 4.2 Implement `signature.rs`: `request_ts` + MD5 `request_sig` (verify exact string shape against live streamrip/qopy.py reference)
- [x] 4.3 Implement signed `track/getFileUrl` request returning the temporary CDN URL + delivered quality
- [x] 4.4 Implement graceful downgrade: request chosen tier, accept/report the actually delivered tier

## 5. Core: search and metadata

- [x] 5.1 Implement metadata fetch: `album/get`, `track/get`, `playlist/get?extra=tracks`, `artist/get` with offset pagination (+=500)
- [x] 5.2 Implement catalog search returning albums/tracks/artists
- [x] 5.3 Implement URL/ID parser for `open.qobuz.com`/`play.qobuz.com` URLs and bare IDs → typed album/track/playlist reference

## 6. Core: download engine

- [x] 6.1 Implement `download.rs` streaming a CDN URL to disk (reqwest `stream`, no full-buffer)
- [x] 6.2 Emit progress over `tokio::sync::mpsc`
- [x] 6.3 Add bounded concurrency (configurable semaphore) and exponential-backoff retry on rate limit
- [x] 6.4 Isolate per-item failures so one failed track doesn't abort the batch

## 7. Core: templating and tagging

- [x] 7.1 Implement `template.rs` folder/track rendering with placeholders and zero-padding, plus per-segment sanitization and multi-disc subfolders
- [x] 7.2 Implement `tagging.rs` with `lofty`: write title/artist/album/albumartist/track/disc/year/genre/ISRC/composer/explicit for FLAC/MP3/M4A
- [x] 7.3 Fetch album cover image and embed it (respecting the embed-art toggle)

## 8. Core: tests

- [x] 8.1 Unit tests: `signature` (known MD5 vector), `template` (render + sanitize edge cases), `quality` mapping, URL/ID parser
- [x] 8.2 Integration test of the album→files flow (mocked HTTP, or a live test gated behind an env-var + real account)
- [x] 8.3 `cargo test -p qobuz-core` and `cargo clippy --workspace` pass clean

## 9. GUI: settings screen

- [x] 9.1 Build iced settings screen: email/password + raw-token login, app_id/app_secret fields
- [x] 9.2 Add download-directory picker (`rfd`), quality dropdown, embed-art toggle
- [x] 9.3 Add folder/track template fields with a live rendered-path preview; persist via core config

## 10. GUI: search / add screen

- [x] 10.1 Build search box + results list (albums/tracks/artists) with an "add to queue" action
- [x] 10.2 Add a paste-URL/ID field that resolves and enqueues album/track/playlist
- [x] 10.3 Handle empty results and unrecognized-input states

## 11. GUI: download queue

- [x] 11.1 Build the queue view with per-item status (queued/downloading/tagging/done/error) and per-item + overall progress bars
- [x] 11.2 Bridge the core `mpsc` progress channel into an iced `Subscription`/`Task` so the UI updates without blocking
- [x] 11.3 Display per-item error messages

## 12. Packaging and verification

- [x] 12.1 Add `cargo-packager` config for macOS (`.dmg`/`.app`), Windows (`.msi`/`.exe`), Linux (`.AppImage`/`.deb`)
- [ ] 12.2 Manual end-to-end: configure credentials, search an album, download at FLAC 24/96, confirm templated path + embedded cover art + correct tags
- [ ] 12.3 Cross-platform smoke test: `cargo run -p qobuz-gui` opens, picker works, an MP3-320 download completes; finalize README docs

> 12.2 and 12.3 require a real Qobuz subscriber account (app_id/app_secret + credentials)
> and an interactive display session, so they're left for the user to run.
> A gated automated version of 12.2 exists: `crates/qobuz-core/tests/integration.rs`
> (`QOBUZ_APP_ID/SECRET/TOKEN/ALBUM_ID` env vars). README build/run docs are finalized.
