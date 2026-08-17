## Context

See proposal.md — Why.

The state that shapes the fix:

- `ItemStatus` (`crates/qobuz-gui/src/app.rs`) is
  `Queued | Downloading | Tagging | Done(String) | Error(String)`. Only
  `Queued` means "never attempted": `spawn_downloads` forces every row it is
  about to run back to `Queued` before starting, so the state is also used
  transiently mid-batch.
- `Message::StartDownloads` selects `Queued | Error(_)` via
  `App::jobs_with(pred)`, the only status-filtering helper on `App`. There is
  no "is there anything to start" predicate anywhere.
- `app.downloading` flips true only inside `spawn_downloads` and false only in
  `Message::DownloadsFinished`, which the event stream sends after the engine
  and the drain future both complete. There is no per-item signal for "the
  batch is over".
- `queue_view` already conditionally pushes "Retry failed" (`failed > 0 &&
  !app.downloading`) and "Clear queue" (`!app.queue.is_empty() &&
  !app.downloading`), so a conditional header control is an established
  pattern in this file. The Start button is the one that is pushed
  unconditionally.
- `App::new()` performs config and keyring IO and returns a `Task`, so `App`
  is not constructible in a unit test. The testable seam is a free function
  over `&[QueueItem]`, which is how the queue view's progress helpers are
  already tested.

## Goals / Non-Goals

**Goals:**

- Offer Start only when activating it would do something.
- Have one predicate decide both whether Start is shown and what it acts on,
  so the two cannot drift.
- Give the empty queue an honest presentation.

**Non-Goals:**

- Reworking the retry controls, the clear control, or their `!downloading`
  gating.
- Adding an empty-state to the Search screen (it has none either; out of
  scope).
- Any change to `qobuz-core`, to `spawn_downloads`, or to the event stream.
- Introducing a muted-text color token to `style::Accents`.

## Decisions

### One shared predicate, defined next to the model

```rust
/// Tracks a fresh Start acts on: ones that have never been attempted.
/// Shared by `Message::StartDownloads` and the Queue header, so the button is
/// only offered when pressing it would do something. Failed tracks are
/// deliberately excluded — the Retry controls own those.
fn startable(queue: &[QueueItem]) -> bool {
    queue.iter().any(|it| matches!(it.status, ItemStatus::Queued))
}
```

It lives in `app.rs` beside `QueueItem`/`ItemStatus`, not in the view: it is a
question about model state. No visibility modifier is needed — a private item
in `crate::app` is reachable from its descendant `crate::app::view::queue` as
`super::super::startable`.

*Alternative considered — a method on `App`.* Rejected: `App` cannot be built
in a test, so the predicate would be untestable. A free function over the slice
is testable with the `item(...)` helper the queue tests already have.

*Alternative considered — reusing `App::jobs_with(pred).is_empty()`.* Rejected:
it clones a `Job` per matching row just to answer a yes/no question, and it
would still leave the view and the message handler writing the predicate twice.

### Start acts on `Queued` only

`Message::StartDownloads` narrows from `Queued | Error(_)` to `Queued`, so its
action is exactly `startable`'s question. The alternative — keep the wider
action but show the button on the narrower condition — was rejected precisely
because a visibility predicate that disagrees with the action predicate is the
drift that produced the overall-progress bug fixed in the preceding change.

The user-visible consequence is that Start no longer sweeps up failed tracks.
That is acceptable because the per-item Retry and "Retry failed (N)" controls
already cover failures, are visible whenever failures exist, and name the count
explicitly. The `jobs.is_empty()` early return in the handler stays as a guard
even though the button can no longer reach it.

### Visibility is `downloading || startable`

```rust
if app.downloading || super::super::startable(&app.queue) { … }
```

The `downloading` arm is load-bearing rather than defensive: during a batch,
rows leave `Queued` as they start, so by the time the last item is downloading
`startable` is already false — without this arm the button would blink out
mid-batch and reappear on the next queue. Keeping it visible also preserves
the disabled "Downloading…" label as the screen's running indicator. The
existing `on_press_maybe((!app.downloading).then_some(...))` gate is unchanged,
so the button is still inert while a batch runs.

### Empty queue short-circuits the whole view

`queue_view` returns early when `app.queue.is_empty()`, before building the
header, with a centered container holding a two-line hint. This drops the
"0/0 complete" counter and the empty progress bar along with the button —
showing a 0%-wide bar for a queue that does not exist is the part that reads as
broken. "Clear queue" and "Retry failed" are already absent in this state, so
the early return loses nothing.

No muted-color token exists in `style::Accents` (`base`, `surface0..2`, `text`,
`on_accent`, and the named accents — no `subtext`), so the hint uses the
default text color with `style::TEXT_BODY` for the headline and
`style::TEXT_SM` for the sub-line rather than inventing a palette entry for one
screen.

## Risks / Trade-offs

- **A user who relied on Start to also retry failures now needs a second
  click** → Mitigated: "Retry failed (N)" is visible whenever failures exist
  and states the count. Spelled out in the proposal as a behaviour break.
- **`Queued` doubles as a transient mid-batch state**, so `startable` is true
  for rows a running batch already owns → Harmless: the `downloading` arm makes
  the button visible in that window anyway, and `on_press_maybe` keeps it
  inert, so the predicate's value cannot start a second batch.
- **The empty-state early return means two layouts to keep in step** → Small
  and contained; the empty branch has no controls, so there is little to drift.
- **Hiding rather than disabling the button moves the header controls** as
  Start appears and disappears → Consistent with how "Retry failed" and "Clear
  queue" already behave in this header.

## Migration Plan

None required — GUI-only, no persisted state, no API surface. Rollback is
reverting the two source files.
