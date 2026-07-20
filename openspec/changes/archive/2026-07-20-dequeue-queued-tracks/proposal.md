## Why

Once a track is added to the download queue there is no way to take it back
out — the only per-item control is Retry (error state only). A user who adds
the wrong album or changes their mind must download it anyway or restart the
app. Letting them remove still-queued tracks is a small, expected affordance.

## What Changes

- Add a per-row **Remove** control on queued items that drops the item from
  the queue.
- The control is available only for items in the *queued* state and only when
  no download batch is in progress (mirroring the existing Retry-during-download
  rule), so an in-flight or completed download is never yanked out mid-batch.
- No core, networking, or persistence changes — this is in-memory queue state
  only.

## Capabilities

### New Capabilities

<!-- None: this refines the existing queue screen. -->

### Modified Capabilities

- `downloader-gui`: the Download queue screen lets the user remove a queued
  item from the queue via a per-row control, disabled during an active batch.

## Impact

- **qobuz-gui** — `app.rs` (new `Message::DequeueTrack(i64)` variant + update
  arm that removes the matching queued row) and `app/view/queue.rs` (a Remove
  button on queued rows, built like the existing per-row Retry control).
- No changes to `qobuz-core`, the download engine, config, or the network
  layer. Backwards compatible; no breaking changes.
