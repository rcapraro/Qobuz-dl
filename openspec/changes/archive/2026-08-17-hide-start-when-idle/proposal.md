## Why

The Queue screen's "Start downloads" button is rendered unconditionally — only
its `on_press` is gated, and only on whether a batch is already running. So it
stays visible and live with an empty queue (including right after "Clear
queue") and with a queue where every track is already downloaded. Pressing it
then does nothing but set the status line "Nothing queued to download.": a
dead control advertising an action the app cannot take.

The empty queue has no empty state either — it shows a bare "0/0 complete"
counter, a full-width empty progress bar, and a blank list, with that live
button as the only affordance. It reads as broken rather than as empty.

## What Changes

- "Start downloads" is offered only when pressing it would start work: when at
  least one track in the queue has never been attempted, or while a batch is
  already running (so the disabled "Downloading…" label stays put for the whole
  batch instead of vanishing as the last rows leave the queued state).
- **BREAKING (behaviour)**: Start now acts on never-attempted tracks only,
  dropping its current "queued *or* previously errored" selection. Relaunching
  failures becomes the exclusive job of the existing per-item Retry and
  "Retry failed (N)" controls, which already cover it. This removes the overlap
  where two buttons silently did the same thing, and lets the button's
  visibility and its action share one predicate so they cannot drift apart.
- An empty queue shows a short centered hint instead of the "0/0 complete"
  counter, the empty progress bar, and the blank list.
- Not changed: the per-item Retry, "Retry failed (N)", "Clear queue", and
  Remove controls; their existing "only when no batch is in progress" gating;
  the download engine; and everything in `qobuz-core`.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `downloader-gui`: the "Download queue screen" requirement gains the
  previously unspecified semantics of the Start control — when it is offered,
  what it acts on — plus the empty-queue presentation.

## Impact

- `crates/qobuz-gui/src/app.rs` — a shared `startable(&[QueueItem])` predicate,
  and `Message::StartDownloads` narrowed to match it.
- `crates/qobuz-gui/src/app/view/queue.rs` — the header guard around the Start
  button, the empty-queue branch in `queue_view`, and unit tests.
- `openspec/specs/downloader-gui/spec.md` — via the delta spec.
- No change to `qobuz-core`, to persisted config, or to any public API. Users
  who relied on Start to sweep up failed tracks alongside queued ones now press
  "Retry failed (N)" for those.
