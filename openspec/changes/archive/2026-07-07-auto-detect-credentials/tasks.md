## 1. Core: credential discovery module

- [x] 1.1 Add `regex` as a direct dependency of `qobuz-core` in its `Cargo.toml` (also added `base64` for secret decoding)
- [x] 1.2 Create `crates/qobuz-core/src/bootstrap.rs` with an `AppCredentials { app_id, app_secrets }` struct (secret candidate list per the chosen strategy)
- [x] 1.3 Implement `async fn discover_app_credentials() -> Result<AppCredentials>`: fetch `play.qobuz.com/login`, locate the `bundle.js` URL, and fetch the bundle (builds its own `reqwest` client so the GUI needs no reqwest dep)
- [x] 1.4 Parse the `app_id` from the bundle via regex (with a fallback pattern for layout drift)
- [x] 1.5 Parse and assemble the candidate `app_secret`s from the seed/info/extras segments and base64-decode them
- [x] 1.6 Add an `Error::CredentialDiscovery(String)` variant in `error.rs` and return it with context on any fetch/parse failure
- [x] 1.7 Register the module in `lib.rs` and re-export `discover_app_credentials` and `AppCredentials`
- [x] 1.8 (strategy) `QobuzClient` holds `app_secrets: Vec<String>` + `with_secret_candidates`; `file_url` tries each candidate until `getFileUrl` is accepted, caching the working one. `Config` gains `app_secret_candidates`.

## 2. Core: tests

- [x] 2.1 Add a unit test that parses `app_id`/`app_secret` from a small synthetic bundle fixture (offline) + a bundle-path test
- [x] 2.2 Add a test asserting a malformed/empty bundle yields `Error::CredentialDiscovery`

## 3. GUI: settings integration

- [x] 3.1 Add `Message::AutoDetectCredentials` and `Message::CredentialsDetected(Result<AppCredentials, String>)` variants
- [x] 3.2 In `settings_view`, add an "Auto-detect" button next to the `app_id`/`app_secret` fields
- [x] 3.3 Handle `AutoDetectCredentials` in `update`: set a "Detecting…" status and run `discover_app_credentials` via `Task::perform`
- [x] 3.4 Handle `CredentialsDetected`: on success fill the fields (first secret visible, rest as candidates), save config, show success; on failure show the error and leave existing values untouched. `AppSecretChanged` clears candidates so manual entry stays authoritative.
- [x] 3.5 Reword the existing `credentials_help()` text to present manual entry as the fallback path

## 4. Verification

- [x] 4.1 `cargo fmt`, `cargo clippy --workspace`, and `cargo test -p qobuz-core` all pass
- [ ] 4.2 Manually run the GUI, click Auto-detect, and confirm the fields populate and a subsequent download succeeds — requires live network to play.qobuz.com and a real Qobuz account; left for the user to verify.
