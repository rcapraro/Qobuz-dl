## Context

Greenfield cross-platform desktop app in Rust to download Qobuz music for a
subscriber's own use. It must handle authentication (credentials or token),
signed API access, quality selection, streamed downloads, tag/cover-art
embedding, and configurable file organization. There is no existing code. The
Qobuz JSON API (`https://www.qobuz.com/api.json/0.2/`) is undocumented; we model
behavior on the public open-source `streamrip`, `qobuz-dl`, and
`qobuz-api-rust` projects.

## Goals / Non-Goals

**Goals:**
- Reusable, UI-agnostic core engine that a CLI could later reuse.
- Correct signed `getFileUrl` flow with just-in-time URL fetching.
- Robust, non-blocking download queue with progress, concurrency, and retry.
- Correct multi-format tagging + cover-art embedding (FLAC/MP3/M4A).
- Secure token storage; no plaintext secrets in config.
- Cross-platform builds (macOS, Windows, Linux).

**Non-Goals:**
- Automatic `app_id`/`app_secret` scraping from the web player (manual entry).
- Audio playback/streaming inside the app.
- Playlist/library management beyond downloading.
- Mobile platforms.

## Decisions

### Cargo workspace: `qobuz-core` + `qobuz-gui`
Split the API/download engine from the UI so the core is testable without a GUI
and reusable by a future CLI. Alternative (single crate) rejected: couples UI to
engine and complicates testing.

### GUI: `iced`
Chosen for pure-Rust single-language development, an official async
download-with-progress example matching the queue model, a built-in
`progress_bar` widget, and idiomatic `Task`/`Subscription` async integration.
Alternatives: `egui` (faster to ship but less polished — kept as fallback);
`Tauri` (best packaging but requires a second JS stack — rejected for a solo
Rust project).

### Async engine: `tokio` + `reqwest`
`reqwest` with `stream` feature streams large FLAC files to disk without full
in-memory buffering; `json` feature deserializes API responses. Progress flows
from core to UI over a `tokio::sync::mpsc` channel bridged into an iced
`Subscription`.

### Signing: `md-5`, just-in-time
`request_sig = md5("trackgetFileUrl" + alphabetically-sorted-params + request_ts + app_secret)`.
File URLs are short-lived, so `getFileUrl` is called immediately before each
download, never at enqueue time. The exact string shape is verified against live
reference sources at implementation time (see Risks).

### Tagging: `lofty`
One unified crate reads/writes FLAC (Vorbis + PICTURE), MP3 (ID3v2), and
MP4/M4A (atoms) with a common `Picture` type for cover art — covers all target
formats with a single dependency. Alternatives (`id3` + `metaflac` + `mp4ameta`)
rejected as three single-format deps.

### Credentials: `keyring` + `directories`
`user_auth_token` stored in the OS keyring (Keychain / Credential Manager /
Secret Service); non-secret settings persisted as serde config under the
platform config dir via `directories`. Password is never persisted.

### Templating: `HashMap`-driven replacer + sanitizer
Folder and track templates rendered from placeholder maps, each segment
sanitized (strip `/ \ : * ? " < > |`, trim length) for cross-platform safety;
multi-disc emits per-disc subfolders.

### Quality: enum → `format_id` with graceful downgrade
`Quality::{Mp3=5, FlacCd=6, Flac24=7, FlacHiRes=27}`. Request the chosen tier;
if unavailable, accept the delivered tier and report actual
`bit_depth`/`sampling_rate`.

## Risks / Trade-offs

- **`request_sig` / `app_secret` shape drifts between web-player releases** →
  Verify the exact signing string against live `streamrip`/`qopy.py` at
  implementation time; surface signature errors clearly and let the user
  re-enter `app_secret`.
- **Short-lived CDN URLs** → Fetch `getFileUrl` just-in-time per download; retry
  on expiry.
- **Undocumented API changes / rate limiting** → Centralize API calls in one
  client module; exponential-backoff retry; bounded concurrency.
- **Manual `app_id`/`app_secret` is a setup burden** → Accepted trade-off for
  robustness; clear settings UI and validation feedback mitigate it.
- **Linux keyring (Secret Service) may be unavailable in some environments** →
  Fail gracefully with an actionable message; token can be re-entered.
- **iced learning curve / thinner docs** → Lean on the official
  `download_progress` example as the queue blueprint.

## Migration Plan

Greenfield — no migration. Rollout: scaffold workspace → build core bottom-up
(config/models → client/auth → signing/quality → search → download →
templating/tagging → tests) → build GUI screens → package. Rollback is trivial
(new project, nothing to revert).

## Open Questions

- Exact current `request_sig` string ordering and secret-decoding offsets —
  resolve by checking live reference implementations during implementation.
- Whether to persist a download history/log for resumability (deferred; not in
  scope for the initial version).
