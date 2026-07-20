## Context

The queue is `App.queue: Vec<QueueItem>` in `crates/qobuz-gui/src/app.rs`; the
queue header is built in `crates/qobuz-gui/src/app/view/queue.rs::queue_view`,
which already conditionally renders a "Retry failed (N)" button
(`if failed > 0 && !app.downloading`) and the "Start downloads" button. The
recently added per-row **Remove** control (message `DequeueTrack`) removes a
single queued item and is gated with `!downloading`. A batch is tracked by
`self.downloading`; because `spawn_downloads` hands the full job list to the
engine at once, mutating the queue mid-batch cannot cancel in-flight work — so
bulk removal, like the other controls, is only offered when no batch is running.

## Goals / Non-Goals

**Goals:**
- One header action that empties the entire queue.
- Reuse the header-button pattern and the `!downloading` gating.

**Non-Goals:**
- No cancellation of an in-progress batch.
- No confirmation dialog (removal is in-memory and easily redone by re-adding).
- No core/engine changes.

## Decisions

- **New message `Message::ClearQueue`.** Its update arm empties the queue:
  `self.queue.clear();` and sets a "Queue cleared." status line for feedback,
  consistent with other queue actions. Search thumbnails (`self.thumbnails`)
  are unrelated to the queue and left untouched.
- **Header control**, rendered in `queue_view` only when
  `!app.queue.is_empty() && !app.downloading`, placed alongside "Retry failed".
  Built manually like the "Retry failed" button (owned `String` label not
  required, but the same `button(text("Clear queue")).padding(...).height(CONTROL_HEIGHT).style(button::secondary).on_press(Message::ClearQueue)`
  construction) for visual consistency. Using `on_press` (not `on_press_maybe`)
  is fine because the button only exists when actionable.
- **Placement:** before "Start downloads" in the header row so destructive and
  primary actions read left-to-right, matching the existing "Retry failed" →
  "Start downloads" order.

## Risks / Trade-offs

- **Accidental clear:** no confirmation, but the queue is in-memory only and
  the control is hidden during downloads, so the blast radius is limited to
  not-yet-started work the user can re-add. Matches the low-friction feel of
  the per-row Remove.
- **Gating consistency:** hiding (rather than disabling) the button when the
  queue is empty or a batch is running keeps the header uncluttered and is
  consistent with how "Retry failed" already appears conditionally.
