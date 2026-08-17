## Why

The Queue screen's overall progress bar reports a number that does not
describe the batch. `overall_progress` sums bytes only over items whose total
size is already known, and a track's total only becomes known once it has
received its first `JobEvent::Progress` — which, because `download_all` gates
jobs behind a concurrency semaphore, never happens for tracks still waiting
their turn. Pending tracks are therefore dropped from both the numerator and
the denominator, so the bar measures the current concurrency window instead of
the queue.

Concretely, with a 20-track album at `concurrency = 4`: when the first four
tracks finish, the bar reads **100% while 16 tracks are still queued**, then
snaps backwards to ~50%, ~33%, and so on as the next batches start. It is not
monotonic, it disagrees with the header's `4/20 complete` counter, and in the
common case where files already exist on disk (the download is skipped and no
progress event is emitted at all) it ignores completed work entirely.

## What Changes

- Overall batch progress becomes **track-weighted**: the mean of each item's
  own progress fraction over *every* item in the queue, instead of a byte sum
  over the subset of items that happen to have started.
- Not-yet-started items now count toward the denominator as 0% rather than
  being excluded, so the bar can no longer reach 100% while work is pending
  and can no longer move backwards within a batch.
- Terminal item states count as settled in the aggregate: tagging, done, and
  **failed** items each contribute 100%, so a batch that ends with failures
  still completes the bar (the red badge and the "Retry failed (N)" counter
  remain the signal for failure). Items completed by the already-on-disk skip
  path, which emit no progress events, are counted through their `done` status
  rather than through bytes.
- Per-item row bars keep their current behaviour, including a failed row
  rendering as empty; only the batch aggregate treats failure as settled.
- The per-item fraction used by the row bars and by the aggregate becomes a
  single shared function, so the two can no longer drift apart.
- Not changed: byte counters, `JobEvent` payloads, the download engine, retry
  behaviour, and progress-event emission rate. This is a GUI-side computation
  fix only.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `downloader-gui`: the "Download queue screen" requirement gains explicit
  semantics for what overall progress means — coverage of the whole queue,
  monotonic within a batch, pending items counted as incomplete, terminal
  items counted as complete.

## Impact

- `crates/qobuz-gui/src/app/view/queue.rs` — `overall_progress` rewritten, a
  shared per-item fraction helper extracted from `queue_row`, and the unit
  tests updated (the existing `unknown_totals_do_not_inflate_progress` and
  `falls_back_to_done_fraction_without_totals` tests encode the current
  byte-subset contract and must be reworked).
- `openspec/specs/downloader-gui/spec.md` — via the delta spec.
- No change to `qobuz-core`, no change to persisted config, no change to any
  public API. User-visible behaviour change is limited to the number the
  overall progress bar displays.
