## Why

When a track fails during a queue download, the app marks the row `error: <msg>` and moves on with no way to relaunch just that track — the user must re-add the whole album. Worse, the internal stream-retry loop reuses the **same signed file URL** across attempts; because Qobuz signed URLs are short-lived, once a URL expires every retry keeps failing on a stale URL, so transient failures often become permanent. This change makes retries reliable and gives the user a way to relaunch failed tracks.

## What Changes

- **Re-sign the file URL on every stream attempt** so a retry never reuses an expired signed URL (the primary reliability fix).
- **Expose a public single-job entry point** (`download_job`) extracted from `download_all`'s per-job closure, so one track can be re-downloaded without re-resolving the whole reference.
- **Promote error classification** to a `pub fn Error::is_transient(&self)` method (429 / network / HTTP 5xx), replacing the private helper in `download.rs`.
- **Clean up on failure & be idempotent**: delete the stale `.part` temp file when a download attempt errors, and skip streaming when the destination file already exists (so relaunching a partially-completed batch doesn't re-download finished tracks).
- **Better backoff**: add jitter to the exponential backoff and honor a `Retry-After` header on 429 responses when present. Retry limits stay internal (hardened defaults; no new Settings control).
- **GUI retry affordance**: retain resolved `Job`s per queue row, add a per-row **Retry** button for errored tracks and a **Retry failed (N)** button in the Queue header, both disabled while a batch is downloading.

## Capabilities

### New Capabilities
<!-- None; this change hardens and extends existing capabilities. -->

### Modified Capabilities
- `audio-download`: the "Concurrency and retry" requirement gains behavior for re-signing the file URL on retry, cleaning up partial files on failure, skipping already-downloaded destinations, and jittered backoff / `Retry-After`.
- `downloader-gui`: the "Download queue screen" requirement gains behavior for relaunching a single failed track and retrying all failed tracks.

## Impact

- **Core** (`crates/qobuz-core/src/`): `engine.rs` (re-sign on retry, extract `download_job`), `download.rs` (`.part` cleanup, skip-existing, jitter/`Retry-After`), `error.rs` (add `is_transient()`), `lib.rs` (re-export `download_job`).
- **GUI** (`crates/qobuz-gui/src/app.rs`): `QueueItem` gains `track_id`/`job` fields; new `RetryTrack`/`RetryFailed` messages, handlers, and buttons.
- **Config/persistence**: none — no new config fields, no keyring changes.
- **Public API**: additive (`download_job`, `Error::is_transient`); no breaking changes.
