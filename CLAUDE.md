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
cargo test --workspace             # All tests (core + GUI view logic)
cargo test -p qobuz-core <name>    # Run a single test by name
cargo fmt                          # Format (no rustfmt.toml — uses defaults)
cargo clippy --workspace --all-targets  # Lint incl. test code (no clippy.toml)
```

No justfile/Makefile — invoke cargo directly. `Cargo.lock` is committed, so a
dependency change belongs in the same commit as the `Cargo.toml` edit. GitHub
Actions runs `.github/workflows/ci.yml` on pushes and PRs to `main`.

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
- `engine.rs` — the hub. `resolve(Reference) -> Vec<Job>` flattens metadata into per-track jobs (deduped by track id); `download_all` runs jobs in a `JoinSet` under a `Semaphore` (`config.concurrency`), isolating per-item failures, emitting `JobEvent`s. It takes a `CancellationToken`: cancelling is cooperative (checked after the permit is acquired, and raced against the retry future), reports `JobEvent::Cancelled` rather than `Failed`, and still drains every task before returning — that drain is what drops the last event `Sender` and closes the channel, so any path returning early would hang the GUI's drain loop and strand `downloading = true`.
- `client.rs` — `QobuzClient`: async reqwest JSON client. Holds `app_id`, `app_secret`, optional `token`. Cloneable.
- `signature.rs` — MD5 request signing (see quirks below).
- `auth.rs` — stores `user_auth_token` in the OS keyring only; never in config.
- `download.rs` — streams to a process-unique `.partN` temp file then atomic rename; a drop guard deletes that temp file however the transfer ends, since a cancelled transfer is dropped mid-stream and never returns an error for an `Err` branch to clean up after. `with_retry` uses exponential backoff on transient errors only (429/network/5xx), fails fast on permanent ones. `fetch_bytes` (whole-body GET with status check) also backs cover-art/thumbnail fetches.
- `bootstrap.rs` — auto-detects `app_id`/`app_secret` candidates from the Qobuz web player bundle (`discover_app_credentials`).
- `tagging.rs` — audio tags + cover-art embedding via `lofty` (container chosen by file extension).
- `template.rs` — `{placeholder}` path templating; each path segment sanitized independently.
- `quality.rs`, `catalog.rs`, `config.rs`, `models.rs`, `error.rs` — quality/format mapping, URL/ID parsing into `Reference`, persisted JSON settings, serde API models, central `thiserror` `Error`.

**qobuz-gui** (`crates/qobuz-gui/src/`): `main.rs` is a thin entry (tracing setup → `app::run()`). Classic iced Elm architecture — `app.rs` holds `struct App` (state) / `enum Message` / update, with three screens (`enum Screen { Settings, Search, Queue }`); the per-screen views live in `app/view/{settings,search,queue}.rs` (shared widget helpers in `app/view/mod.rs`), static help panels in `app/help.rs`, and the `Task::perform` async wrappers around core calls in `app/tasks.rs`. `style.rs` is the design system (spacing/typography constants, Catppuccin palettes, widget builders). Download progress bridges a `tokio::sync::mpsc` channel of `JobEvent`s into `Message::Download` via `iced::stream::channel`. Queue rows are looked up by linear scan on `track_id` (no index map); `signed_in`/theme are derived from `token`/`config`, never stored twice.

**Data flow:** auth (paste a raw token → `client::login_with_token`; there is no email/password login) → `search` or paste URL/ID (`catalog::parse_input`) → `engine::resolve` → `engine::download_all`. Per track: request a fresh signed file URL just-in-time → determine *delivered* quality from the response `format_id` → build path from templates → stream to `.part` → embed art (cached per-album) → write tags.

## Domain quirks (non-obvious)

- **User-supplied credentials:** `app_id` and `app_secret` are NOT bundled — the user extracts them from Qobuz's web player and enters them in Settings. `app_id` goes in header `x-app-id`; `app_secret` is used only for signing.
- **Request signing** (`signature.rs`): `request_sig = MD5(object + method + sorted(name+value) + request_ts + app_secret)`, params sorted alphabetically, `app_id`/`token` excluded. Only `track/getFileUrl` is signed. The signed-string shape can drift between Qobuz web-player releases — cross-check `streamrip`/`qopy.py` if signing breaks (surfaces as `Error::InvalidSignature`).
- **Quality downgrade:** requested quality may be silently downgraded by the API. Always derive the real file extension and "delivered" label from the response `format_id`/`bit_depth`/`sampling_rate`, not the request. Tiers: MP3-320 (5), FLAC-CD 16/44.1 (6), FLAC-24/≤96 (7, default), FLAC-Hi-Res 24/≤192 (27).
- **Robust deserialization:** models use `#[serde(default)]` throughout and ignore unknown fields; playlist fetch paginates past the 500-item page size.
- **Auth token** is sent as header `x-user-auth-token`, stored in the OS keyring, and deliberately excluded from serialized `Config` (there's a test asserting this).

## Workflow

This project uses **OpenSpec** (`openspec/`) for spec-driven changes, not Cursor/Copilot rules. Existing specs/changes live under `openspec/changes/` and `openspec/specs/`. Use the OpenSpec skills/slash-commands when proposing or applying spec changes.

Archive order matters when two unarchived changes `MODIFY` the same requirement:
a MODIFIED requirement replaces the entire block, so archive the older change
first and make sure the newer delta is a superset of its scenarios — otherwise
syncing the second silently deletes what the first added.
