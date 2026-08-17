## Context

See proposal.md — Why.

The constraints that shape the fix:

- `QueueItem` (`crates/qobuz-gui/src/app.rs`) carries `status: ItemStatus`,
  `downloaded: u64`, `total: Option<u64>`. `total` comes from the HTTP
  `Content-Length` of the streaming response (`download.rs`, `resp.content_length()`)
  and is therefore **unknowable before a track starts**. Enqueue and batch
  (re)launch both initialise items to `downloaded: 0, total: None`.
- `engine::download_all` runs jobs under `Semaphore::new(config.concurrency)`,
  so at any moment only a few items have a known total. This is the root of
  the bug: any denominator built from known totals is a denominator over the
  concurrency window, not the queue.
- Three code paths finish an item without ever emitting a `JobEvent::Progress`
  with a real total: the already-on-disk skip in `download.rs`, an immediate
  failure before the body starts streaming, and a response with no
  `Content-Length`.
- `apply_event` in `app.rs` never normalises the counters on
  `Tagging`/`Done`/`Failed`; an item's last-seen `downloaded`/`total` is
  whatever the final chunk reported.
- The Queue screen has no stored aggregate state — `overall_progress` is
  recomputed from `&[QueueItem]` on every frame, so the fix is confined to
  that pure function and is directly unit-testable.

## Goals / Non-Goals

**Goals:**

- Make the overall bar a function of the whole queue, monotonic within a batch
  and complete exactly when the batch ends.
- Keep it a pure function of `&[QueueItem]` — no new state on `App`, no new
  events, no core changes.
- Remove the possibility of the row bars and the aggregate disagreeing about
  what "this item's progress" means.

**Non-Goals:**

- Estimating sizes for not-yet-started items to smooth the bar (see Decisions).
- Making per-item progress resilient to retries restarting at byte 0.
- Throttling the per-chunk `JobEvent::Progress` emission rate.
- Any change to `JobEvent`, `engine.rs`, or `download.rs`.

## Decisions

### Track-weighted aggregate instead of byte-weighted

Overall progress becomes the mean of the per-item fractions over the entire
queue:

```rust
fn overall_progress(queue: &[QueueItem]) -> f32 {
    if queue.is_empty() {
        return 0.0;
    }
    queue.iter().map(batch_fraction).sum::<f32>() / queue.len() as f32
}
```

Every item contributes exactly `1/len` of the bar, so pending items hold it
back and no item can be silently dropped. The old function's two branches
(byte sum, and a done-count fallback when no totals were known) collapse into
this one — the former fallback is now the general case, refined by live
byte progress inside the currently-downloading items.

*Alternative considered — byte-weighted with an estimated size for pending
items* (e.g. the mean of the known totals). It yields a smoother bar because
big tracks move it more than small ones. Rejected: the estimate shifts every
time a real total arrives, which reintroduces non-monotonic jumps — the exact
defect being fixed — and it cannot handle the skip path, where a completed
item legitimately has no size at all. Track weighting is slightly coarser but
is always truthful; a queue of tracks from the same album has fairly uniform
sizes anyway.

*Alternative considered — normalising the counters in `apply_event`* (set
`downloaded = total` on `Done`, synthesise a total for skipped items). Rejected:
it puts display arithmetic into the event handler, invents byte counts that
were never transferred, and still leaves not-yet-started items with no total.

### Failure counts as settled in the batch, but not in the row

A failed item is finished — it will not progress further without an explicit
retry — so it contributes `1.0` to the aggregate. Otherwise a batch with one
permanent failure leaves the bar stuck short of full forever, which reads as
"still working" when nothing is running. Failure is already communicated by
the red badge, the error message, and the "Retry failed (N)" header control.

The item's **own** bar keeps rendering `0.0` for the error state: a full green
bar under an error badge would be actively misleading. This asymmetry is
deliberate and is the one place where the row and the batch differ, so it is
expressed as two thin wrappers over one shared match rather than duplicated
logic:

```rust
/// How far this item has advanced, in `0.0..=1.0`.
/// `settled_on_error` distinguishes the batch aggregate (a failed item is
/// finished, so it counts as advanced) from the item's own bar (a failed row
/// must not render as full).
fn item_fraction(it: &QueueItem, settled_on_error: bool) -> f32 { … }

fn batch_fraction(it: &QueueItem) -> f32 { item_fraction(it, true) }
fn row_fraction(it: &QueueItem) -> f32 { item_fraction(it, false) }
```

`Queued` → `0.0`; `Downloading` → `downloaded / total` clamped to `0.0..=1.0`,
or `0.0` when the total is unknown or zero; `Tagging` and `Done` → `1.0`;
`Error` → `settled_on_error as f32`.

`queue_row` keeps its own match for the status *label* (it needs the
percentage text and the delivered-quality string) but takes its bar value from
`row_fraction`, so label and bar cannot drift.

### Clamp per item, not just at the end

`downloaded > total` is possible in principle (a total from one retry attempt
paired with bytes from another, since `with_retry` reuses a single progress
forwarder across attempts). Clamping inside `item_fraction` stops one item
from borrowing headroom from the rest of the batch; the existing
`.clamp(0.0, 1.0)` at the `progress_bar` call sites stays as a cheap backstop.

## Risks / Trade-offs

- **A long track and a short track move the bar equally** → Accepted. Queues
  are usually one album or playlist, where track sizes are within a small
  factor of each other. Truthful coverage beats smoothness.
- **The bar advances in visible steps when concurrency is low** → Accepted;
  within each step the currently-downloading items still contribute continuous
  byte progress, so the bar is never fully static while data is moving.
- **A retry rewinds the aggregate** → Reduced, not eliminated: a retry now
  costs at most `1/len` of the bar instead of a whole file's bytes out of a
  window-sized denominator. The spec allows this explicitly.
- **Failed items make the bar read "100%" on a batch that partly failed** →
  Mitigated by the unchanged error badges and the "Retry failed (N)" control,
  which is the established place users read failure counts.
- **Existing tests encode the old contract** → `unknown_totals_do_not_inflate_progress`
  and `falls_back_to_done_fraction_without_totals` in `queue.rs` assert the
  byte-subset behaviour and will fail; they are rewritten as part of the change
  rather than adjusted numerically, so the new contract is what is asserted.

## Migration Plan

None required — the change is a pure display computation with no persisted
state, no schema, and no API surface. Rollback is reverting the single source
file.
