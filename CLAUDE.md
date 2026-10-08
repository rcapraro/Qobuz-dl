# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Cross-platform desktop app (Rust + `iced`) for downloading Qobuz music, tagging it on download, and then editing a finished album's tags and renaming its folder from the queue. Cargo workspace with two crates under `crates/`:
- **qobuz-core** — UI-agnostic library: Qobuz API client + download engine, plus tag reading/editing and folder renaming. No GUI deps.
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
- `client.rs` — `QobuzClient`: async reqwest JSON client. Holds `app_id`, `app_secret`, optional `token`. Cloneable. Search uses the per-type `album/search` and `track/search` endpoints (`search_albums`/`search_tracks`, with `offset`), not `catalog/search`, so each type pages on its own with a stable ranking.
- `signature.rs` — MD5 request signing (see quirks below).
- `auth.rs` — stores `user_auth_token` in the OS keyring only; never in config.
- `download.rs` — `stream_to_part` streams to a process-unique `.partN` temp file and hands it back as a `PartFile` (or reports `Transfer::AlreadyPresent` when the destination exists); the engine tags it and only then `publish`es it by atomic rename, so a file at its destination is always a finished one — which is what makes skipping an existing file safe. The `PartFile` drop guard deletes the temp file however things end before that, since a cancelled transfer is dropped mid-stream and never returns an error for an `Err` branch to clean up after. `with_retry` uses exponential backoff on transient errors only (429/network/5xx), fails fast on permanent ones. `fetch_bytes` (whole-body GET with status check) also backs cover-art/thumbnail fetches.
- `bootstrap.rs` — auto-detects `app_id`/`app_secret` candidates from the Qobuz web player bundle (`discover_app_credentials`).
- `tagging.rs` — audio tags + cover-art embedding via `lofty` (container chosen by file extension). lofty 0.24's generic `ItemKey` has no custom key, so FLAC's explicit flag (`ITUNESADVISORY`) is set on the native `VorbisComments` after converting the generic tag; pictures are inserted on that native tag too, because the conversion drops any picture whose header lofty can't parse.
- `artwork.rs` — `CoverSize` (400 / 500 / 600 px; 600 is Qobuz's own `large` size, so it embeds the cover unchanged) and `prepare_cover`: a cover above the cap is downscaled with Lanczos3 in linear light and re-encoded as a 4:4:4 JPEG (`jpeg-encoder`; `image`'s own encoder is fixed at 4:2:2); one that fits is returned byte for byte, as is anything that fails to decode.
- `tag_edit.rs` — the tag editor's core: a `Field` enum (the 17 editable fields, each with a kind for validation) over normalized string values, read with `read_fields` and changed with `save_file`. `tagging::fields_from` and download tagging use the same `set_field` mapping onto lofty's generic tag. Edits go through each container's native tag via lofty's `SplitTag`/`MergeTag`, so tags outside the list (ReplayGain etc.) survive. An edit that already matches the file is dropped, and a file left with no change is not written at all. Otherwise it is saved through `PartFile::copy_of` → edit the copy → `publish_sync` over the original, so its path never holds a half-written file. FLAC pictures live on `FlacFile`, not its `VorbisComments`, and lofty's FLAC write replaces every PICTURE block, so FLAC covers are edited on the file object. ID3's `TPUB` reads back as `Publisher`, not `Label`, so setting a label clears both.
- `rename.rs` — renaming an album folder in place. `suggest` renders `Config::rename_format` (one segment, so `/` is sanitized) from a file's tags and lofty audio properties with the same keys the engine sets; `rename_folder` refuses an empty name, a folder equal to or outside the download dir (both canonicalized), a folder with a subfolder holding none of the album's files (an `{albumartist}` folder holds other albums), and a target that is a different entry. Same-entry is checked by inode on Unix because macOS canonicalizes in the case given, which is what lets a case-only rename through on APFS.
- `template.rs` — `{placeholder}` path templating; each path segment sanitized independently.
- `quality.rs`, `catalog.rs`, `config.rs`, `models.rs`, `error.rs` — quality/format mapping, URL/ID parsing into `Reference`, persisted JSON settings, serde API models, central `thiserror` `Error`.

**qobuz-gui** (`crates/qobuz-gui/src/`): `main.rs` is a thin entry (tracing setup → `app::run()`). Classic iced Elm architecture — `app.rs` holds `struct App` (state) / `enum Message` / update, with three tabs in the order `Search, Queue, Settings` (opens on Search); the per-screen views live in `app/view/{search,queue,settings}.rs` (shared widget helpers, the status bar and badges in `app/view/mod.rs`); an album opened from search renders from `app/view/album.rs` in place of the results, its state and pure helpers in `app/album.rs` — it holds the jobs `engine::resolve` returned, so adding a track selection enqueues those directly without refetching; an album's tag editor (`App.tag_editor`, an `EditorSlot` that is `Loading` while files are read, then `Open`) replaces the list on the Queue tab, its pure state in `app/tag_editor.rs` (album fields shown once with a mixed state, per-track fields, edits derived by comparing typed text with the file's value, save sequencing) and its view in `app/view/tag_editor.rs`; it is offered only by `App::tags_editable` (a done track, nothing queued, downloading or in the running batch), and Save runs `tasks::save_tags` one file at a time, then reads the files back; Rename folder (`App.rename`, an inline field in the group header) uses that gate plus no running download and no failed track (`App::renamable`; another album may be writing into the folder, and a retry would land under the old name), closes via `drop_stale_rename` once that stops holding, renames `App::rename_target` (the album folder, above `Disc N` when only one disc is done), refuses when another album's done files are inside it, and on success rewrites every `QueueItem.path` under the old folder, so Open folder and Edit tags keep working; the typed status line (`Status`/`StatusKind`) in `app/status.rs`, the Search field's URL-vs-query classification in `app/omnibox.rs` (shape-based: `parse_input` accepts any single word as a bare album ID, so it cannot decide alone), static help panels in `app/help.rs`, and the `Task::perform` async wrappers around core calls in `app/tasks.rs` (including `open_path`, which opens folders through each OS's own opener built in `app/open.rs` — `open`, `explorer`, `xdg-open` — with no dependency and exit codes ignored because `explorer` exits 1 on success; and `notify`, which posts desktop notifications via `notify-rust` on a blocking thread; whether to notify is decided in pure `app/notify.rs` from the finished batch's own track ids in `App.batch`). On macOS the notification sender is set lazily, once, in `tasks::notify` just before the first notification from `get_bundle_identifier_or_default("Qobuz-dl")` (an AppleScript lookup, kept off startup) — it can be set only once per process, so never hard-code the identifier: a dev binary has no registered bundle and posts as Finder. `style.rs` is the design system (spacing/typography constants, Catppuccin palettes, widget builders). Text renders in the bundled Inter, which is the default font, so a non-ASCII glyph must exist in Inter or it shows as a box: it has `▲ ▼ ▶ ◀ × … ← ·` but not the small triangles `▴ ▾ ▸ ◂` or `✕`. Views color by semantic role (`Accents::primary/success/progress/error/quality/brand`), never by hue, and every button comes from a `style.rs` builder (`compact_button` for row actions and small header or inline controls). Download progress bridges a `tokio::sync::mpsc` channel of `JobEvent`s into `Message::Download` via `iced::stream::channel`. Queue rows are looked up by linear scan on `track_id` (no index map); `App.queue` stays flat (each `QueueItem` keeps its downloaded file in `path`, taken from `JobEvent::Done`) and album groups are built only at render time (`groups()` in `app/view/queue.rs`, keyed by `job.album.id`), so no handler ever searches nested groups — the cover cache is shared by every view and trimmed to `covers_in_use()`, so a new view that shows covers must add them there; `signed_in`/theme are derived from `token`/`config`, never stored twice.

**Data flow:** auth (paste a raw token → `client::login_with_token`; there is no email/password login) → `search_albums`/`search_tracks` or paste URL/ID (`catalog::parse_input`) → `engine::resolve` → `engine::download_all`. Per track: request a fresh signed file URL just-in-time → determine *delivered* quality from the response `format_id` → build path from templates → stream to `.part` → embed art (the `large` rendition, sized by `prepare_cover` on a blocking thread and cached per album) and write tags into the `.part` (container chosen from the destination's extension) → rename into place. A destination that already exists is left untouched: no cover fetch, no tagging. The tags written are `tagging::fields_from` (15 of the 17 `tag_edit::Field`s; Compilation and Comment are only set by the editor) plus the cover. After a batch, a done album can be edited (`tag_edit::read_fields` → edits → `tag_edit::save_file` per file) and its folder renamed (`rename::suggest` → `rename::rename_folder` → queue paths rewritten); both work on the files on disk, never on Qobuz metadata, except the editor's Reset to Qobuz, which refills from `fields_from`.

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
