# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Cross-platform desktop app (Rust + `iced`) for downloading Qobuz music. Cargo workspace with two crates under `crates/`:
- **qobuz-core** — UI-agnostic library: Qobuz API client + download engine. No GUI deps.
- **qobuz-gui** — `iced` desktop app; produces the binary named `qobuz-dl` (`src/main.rs`).

## Commands

```bash
cargo run -p qobuz-gui              # Run the GUI (debug)
cargo build --workspace            # Build everything
cargo build --release -p qobuz-gui # Size-optimized release binary
cargo test -p qobuz-core           # Run core tests
cargo test -p qobuz-core <name>    # Run a single test by name
cargo fmt                          # Format (no rustfmt.toml — uses defaults)
cargo clippy --workspace           # Lint (no clippy.toml — uses defaults)
```

No justfile/Makefile/CI. `Cargo.lock` is gitignored.

Packaging (from `crates/qobuz-gui/`, config in `[package.metadata.packager]` in
`crates/qobuz-gui/Cargo.toml` — a standalone `Packager.toml` is NOT auto-detected
in this workspace):
```bash
cargo install cargo-packager --locked
cargo packager --release   # dmg / nsis / deb / appimage
```
CI builds releases automatically on a `v*` tag (`.github/workflows/release.yml`).

## Architecture

**Separation of concerns:** the GUI never touches HTTP or the filesystem directly. It builds a `QobuzClient` via `App::client()` and delegates all network/IO to `qobuz-core`. The only interface between the two is the functions re-exported in `crates/qobuz-core/src/lib.rs` plus the `JobEvent` progress channel. The core is concrete structs + free functions — no trait abstractions or dyn dispatch.

**qobuz-core** (`crates/qobuz-core/src/`):
- `engine.rs` — the hub. `resolve(Reference) -> Vec<Job>` flattens metadata into per-track jobs; `download_all` runs jobs concurrently under a `Semaphore` (`config.concurrency`), isolating per-item failures, emitting `JobEvent`s.
- `client.rs` — `QobuzClient`: async reqwest JSON client. Holds `app_id`, `app_secret`, optional `token`. Cloneable.
- `signature.rs` — MD5 request signing (see quirks below).
- `auth.rs` — stores `user_auth_token` in the OS keyring only; never in config.
- `download.rs` — streams to a `.part` temp file then atomic rename; `with_retry` uses exponential backoff on transient errors only (429/network/5xx), fails fast on permanent ones.
- `tagging.rs` — audio tags + cover-art embedding via `lofty` (container chosen by file extension).
- `template.rs` — `{placeholder}` path templating; each path segment sanitized independently.
- `quality.rs`, `catalog.rs`, `config.rs`, `models.rs`, `error.rs` — quality/format mapping, URL/ID parsing into `Reference`, persisted JSON settings, serde API models, central `thiserror` `Error`.

**qobuz-gui** (`crates/qobuz-gui/src/`): `main.rs` is a thin entry (tracing setup → `app::run()`). `app.rs` (~790 lines) is the whole app in classic iced Elm architecture — `struct App` (state) / `enum Message` / update / view, with three screens (`enum Screen { Settings, Search, Queue }`). Async core calls are wrapped in `Task::perform`; download progress bridges a `tokio::sync::mpsc` channel of `JobEvent`s into `Message::Download` via `iced::stream::channel`. The queue is keyed by `track_id` (`index: HashMap<i64, usize>`).

**Data flow:** auth (email/password `login`, or pasted raw token `login_with_token`) → `search` or paste URL/ID (`catalog::parse_input`) → `engine::resolve` → `engine::download_all`. Per track: request a fresh signed file URL just-in-time → determine *delivered* quality from the response `format_id` → build path from templates → stream to `.part` → embed art (cached per-album) → write tags.

## Domain quirks (non-obvious)

- **User-supplied credentials:** `app_id` and `app_secret` are NOT bundled — the user extracts them from Qobuz's web player and enters them in Settings. `app_id` goes in header `x-app-id`; `app_secret` is used only for signing.
- **Request signing** (`signature.rs`): `request_sig = MD5(object + method + sorted(name+value) + request_ts + app_secret)`, params sorted alphabetically, `app_id`/`token` excluded. Only `track/getFileUrl` is signed. The signed-string shape can drift between Qobuz web-player releases — cross-check `streamrip`/`qopy.py` if signing breaks (surfaces as `Error::InvalidSignature`).
- **Quality downgrade:** requested quality may be silently downgraded by the API. Always derive the real file extension and "delivered" label from the response `format_id`/`bit_depth`/`sampling_rate`, not the request. Tiers: MP3-320 (5), FLAC-CD 16/44.1 (6), FLAC-24/≤96 (7, default), FLAC-Hi-Res 24/≤192 (27).
- **Robust deserialization:** models use `#[serde(default)]` throughout and ignore unknown fields; playlist fetch paginates past the 500-item page size.
- **Auth token** is sent as header `x-user-auth-token`, stored in the OS keyring, and deliberately excluded from serialized `Config` (there's a test asserting this).

## Workflow

This project uses **OpenSpec** (`openspec/`) for spec-driven changes, not Cursor/Copilot rules. Existing specs/changes live under `openspec/changes/` and `openspec/specs/`. Use the OpenSpec skills/slash-commands when proposing or applying spec changes.
