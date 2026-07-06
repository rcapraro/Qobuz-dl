# Qobuz-dl

A cross-platform desktop application (Rust + [iced](https://iced.rs)) to
download music from Qobuz for a subscriber's own offline use — with control over
quality, cover art, file organization, and tags.

> Intended for downloading content you are entitled to via a valid paid Qobuz
> account. You are responsible for complying with Qobuz's terms of service.

## Features

- Authenticate with **email/password** or a **raw `user_auth_token`**.
- Choose download **quality**: MP3 320, FLAC 16/44.1, FLAC 24/≤96, FLAC 24/≤192.
- **Embed cover art** into downloaded files.
- Configurable **download directory** and **folder/track path templates**.
- Full audio **tag** writing (FLAC / MP3 / M4A).
- Find music by **search** or by pasting a **Qobuz URL / ID** (album, track, playlist).
- **Download queue** with per-item progress, bounded concurrency, and retry.

## Architecture

Cargo workspace:

- `crates/qobuz-core` — API client + download engine (no UI dependencies).
- `crates/qobuz-gui` — the `iced` desktop application.

## Prerequisites

- Rust (stable) — `rustup` recommended.
- A Qobuz `app_id` and `app_secret` (entered in **Settings**). These are the web
  player's API credentials; obtain them and paste them into the app.
- Linux: a Secret Service provider (e.g. GNOME Keyring) for secure token storage,
  plus GTK for native file dialogs.

## Build & run

```bash
# Run the GUI (debug)
cargo run -p qobuz-gui

# Build everything
cargo build --workspace

# Run core tests
cargo test -p qobuz-core

# Optimized release build
cargo build --release -p qobuz-gui
```

## Packaging

Install [`cargo-packager`](https://github.com/crabnebula-dev/cargo-packager) and
run it against `qobuz-gui` to produce macOS `.dmg`/`.app`, Windows `.msi`/`.exe`,
and Linux `.AppImage`/`.deb` bundles. See `crates/qobuz-gui/Packager.toml`.

## Configuration

Non-secret settings persist as JSON under your platform config directory
(via the `directories` crate). The `user_auth_token` is stored in the OS keyring
(macOS Keychain / Windows Credential Manager / Linux Secret Service) and is
**never** written to the config file.
