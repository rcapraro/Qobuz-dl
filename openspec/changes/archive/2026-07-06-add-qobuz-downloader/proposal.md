## Why

Qobuz subscribers have no first-party desktop app to download the lossless
tracks they are entitled to for offline use, with control over quality, tags,
cover art, and file organization. This change delivers a cross-platform Rust
desktop app that fills that gap (analogous to `youtube-dl`, modeled on the
public open-source `streamrip` / `qobuz-dl` projects).

## What Changes

- New cross-platform desktop application written in **Rust**, split into a
  reusable core engine (`qobuz-core`) and an `iced` GUI (`qobuz-gui`).
- **Authentication** with Qobuz via email/password login **or** a pasted raw
  `user_auth_token`; the resulting token is stored securely in the OS keyring.
- **Manual `app_id` / `app_secret`** entry in settings (no web-player scraping).
- **Quality selection**: MP3 320, FLAC CD 16/44.1, FLAC 24/≤96, FLAC 24/≤192,
  with graceful downgrade and reporting of the actually delivered quality.
- **Cover art embedding** into downloaded FLAC/MP3/M4A files.
- **Configurable download directory** and **path/filename templates** for folder
  and track naming, plus full audio **tag** writing (title, artist, album,
  albumartist, track/disc no., year, genre, ISRC, composer, explicit).
- **Two ways to choose content**: in-app search (album/track/artist) and pasting
  a Qobuz URL or bare ID (album/track/playlist).
- **Download queue** with per-item and overall progress, bounded concurrency,
  and retry with backoff on rate limiting.

## Capabilities

### New Capabilities
- `qobuz-authentication`: Authenticate to the Qobuz API via credentials or a raw token, manage `app_id`/`app_secret`, and store the auth token securely.
- `catalog-browsing`: Search the Qobuz catalog and resolve pasted URLs/IDs into album/track/playlist/artist metadata.
- `audio-download`: Request signed file URLs at a chosen quality and stream audio to disk with progress, concurrency, and retry.
- `file-organization`: Render configurable folder/track path templates, sanitize paths, write audio tags, and embed cover art.
- `app-configuration`: Persist and edit user settings (download dir, quality, templates, credentials, concurrency, embed-art toggle).
- `downloader-gui`: Desktop UI (settings, search/add, download queue) built on `iced`.

### Modified Capabilities
<!-- None — greenfield project, no existing specs. -->

## Impact

- **New codebase**: Cargo workspace with `crates/qobuz-core` and `crates/qobuz-gui`.
- **Dependencies**: `iced`, `rfd`, `tokio`, `reqwest`, `serde`, `lofty`,
  `keyring`, `md-5`, `directories`, `thiserror`, `tracing`.
- **External**: depends on the undocumented Qobuz JSON API (`api.json/0.2/`);
  `request_sig` string shape and secrets drift between web-player releases and
  must be verified against live reference implementations.
- **Legal/usage**: intended for downloading content a Qobuz subscriber is
  entitled to; requires a valid paid account.
- **Packaging**: `cargo-packager` bundles for macOS, Windows, Linux.
