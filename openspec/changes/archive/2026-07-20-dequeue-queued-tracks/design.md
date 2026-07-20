## Context

The download queue is `App.queue: Vec<QueueItem>` in `crates/qobuz-gui/src/app.rs`.
Each `QueueItem` has a `track_id: i64` and a `status: ItemStatus`
(`Queued | Downloading | Tagging | Done | Error`). Rows are rendered by
`queue_row(it, downloading)` in `crates/qobuz-gui/src/app/view/queue.rs`, which
already conditionally shows a per-row **Retry** button for error items using
`on_press_maybe((!downloading).then_some(Message::RetryTrack(id)))`. Queue rows
are looked up by linear scan on `track_id`.

A download batch is launched by `spawn_downloads`, which hands the full job list
to `engine::download_all` at once; `self.downloading` tracks whether a batch is
live. Because the engine already owns the jobs once a batch starts, removing a
row mid-batch cannot cancel in-flight work — so removal is restricted to when no
batch is running.

## Goals / Non-Goals

**Goals:**
- Remove a still-queued item from the queue with a single per-row control.
- Reuse the existing per-row button pattern and the `!downloading` gating.

**Non-Goals:**
- No cancellation of in-progress, tagging, done, or errored downloads.
- No "clear all" / bulk removal (can follow later if wanted).
- No persistence — the queue is already in-memory only.
- No core/engine changes.

## Decisions

- **New message `Message::DequeueTrack(i64)`.** Its update arm removes the
  matching row only if it is still `Queued`:
  `self.queue.retain(|it| !(it.track_id == id && matches!(it.status, ItemStatus::Queued)))`.
  Guarding on status inside the arm keeps the state authoritative even if a
  stale message arrives. A short status line ("Removed from queue.") gives
  feedback, consistent with other queue actions.
- **Per-row Remove control**, shown in `queue_row` next to the title only when
  `matches!(it.status, ItemStatus::Queued)`. Gated with
  `on_press_maybe((!downloading).then_some(Message::DequeueTrack(it.track_id)))`
  so it greys out during an active batch, exactly like Retry. Built with the
  same `button(text("Remove").size(TEXT_SM))` + `button::secondary` styling as
  the Retry control for visual consistency.
- **Placement:** append to the row's top line after the status badge (Retry and
  Remove are mutually exclusive since they target different statuses, so they
  never appear together).

## Risks / Trade-offs

- **Removing while downloading:** disallowed via the `!downloading` gate, which
  matches user expectation and avoids desyncing the UI from the engine's job
  list. A queued item added during a live batch simply waits until the batch
  ends to become removable.
- **Linear scan:** `retain` is O(n) like the existing `item_mut` lookups —
  negligible for realistic queue sizes and consistent with current code.
