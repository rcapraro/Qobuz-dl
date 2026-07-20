## Why

Removing queued items one at a time is tedious when the user wants to start
over. There is no single action to empty the queue, so resetting means many
per-row clicks or restarting the app.

## What Changes

- Add a **Clear queue** control in the queue header that removes every item
  from the queue at once.
- The control is shown only when the queue is non-empty and no download batch
  is in progress (mirroring the existing Retry/Remove gating), so an active
  batch is never disrupted.

## Capabilities

### New Capabilities

<!-- None: refines the existing queue screen. -->

### Modified Capabilities

- `downloader-gui`: the Download queue screen offers a header control to clear
  the entire queue, disabled during an active batch.

## Impact

- **qobuz-gui** — `app.rs` (new `Message::ClearQueue` variant + update arm that
  empties `self.queue`) and `app/view/queue.rs` (a "Clear queue" header button,
  built like the existing "Retry failed" control).
- No changes to `qobuz-core`, the download engine, config, or the network
  layer. Backwards compatible; no breaking changes.
