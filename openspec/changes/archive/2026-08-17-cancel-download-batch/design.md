## Context

See proposal.md — Why.

The mechanics that constrain the design, all confirmed against the current
code:

- `engine::download_all` builds a `Vec<JoinHandle<_>>` and `tokio::spawn`s
  **every** job immediately — a 200-track album is 200 live tasks, all but
  `config.concurrency` of them parked on `Semaphore::acquire()`. The semaphore
  gates work *inside* an already-spawned task.
- Those handles are detached on drop. Dropping the `download_all` future does
  not stop anything: transfers keep streaming, `.part` files keep growing,
  renames and `write_tags` still run, and `JobEvent`s keep being pushed into
  `Sender` clones the tasks still hold.
- The GUI's stream future ends only because `download_all` returning drops the
  last `Sender`, which closes `rx` and lets the drain loop finish. If a cancel
  path let `download_all` return while job tasks still held `Sender` clones,
  the drain would never see `None`, `future::join` would never complete,
  `Message::DownloadsFinished` would never fire, and `app.downloading` would
  stay `true` forever — permanently blocking every future batch through the
  guard at the top of `spawn_downloads`. This is the failure mode to design
  against.
- `download::with_retry` sleeps between attempts (~0.5s, 1s, 2s, 4s, or an
  arbitrary server-supplied `Retry-After`). A cancel that does not interrupt
  that sleep looks like a hang.
- `stream_to_file` removes its `.partN` temp file only on the `Err` branch.
  There is no `Drop` guard, so a transfer abandoned mid-stream orphans the file
  permanently, and `part_path`'s process-unique counter means retries
  accumulate `track.part0`, `track.part1`, …
- `JobEvent`'s terminal outcomes are `Done` and `Failed` only; `Error` has no
  cancellation variant, and `Config(String)` is the only free-form variant —
  semantically wrong for this.

## Goals / Non-Goals

**Goals:**

- Stop a batch promptly and completely, with nothing left running.
- Keep the normal end-of-batch path intact, so `DownloadsFinished` still fires
  and `downloading` still clears.
- Make a cancelled track visibly different from a failed one.
- Leave no `.part` files behind, whatever way a transfer ends.

**Non-Goals:**

- Cancelling a single track. Cancellation is batch-wide; per-track control
  stays Remove (queued) and Retry (failed).
- Resuming a partially transferred file with a `Range` request. A cancelled
  track restarts from zero, as retries already do.
- Rolling back tracks that already finished, or deleting their files.
- Making Clear queue available mid-batch. Cancel first, then clear.
- Changing concurrency, retry classification, or backoff policy.

## Decisions

### `CancellationToken` over a hand-rolled flag

`tokio-util = "0.7"` joins `[workspace.dependencies]`, and `qobuz-core` takes
it and re-exports `CancellationToken` so the GUI needs no dependency of its
own. The crate is already in `Cargo.lock` at 0.7.18 as a transitive dep of
`reqwest`/`h2`, so no new version enters the graph. No feature flag is needed —
`tokio_util::sync` is always compiled; the crate's optional features cover
`io`/`codec`/`net`/etc., and asking for a `sync` feature is in fact a
resolution error.

It is the right shape for this: cheap clones, `is_cancelled()` for the
zero-cost pre-flight check, and `cancelled()` as a future to `select!` against.

*Alternative considered — `tokio::sync::watch::Receiver<bool>`*, which needs no
new dependency at all (tokio's `sync` feature is already on). Rejected on
ergonomics: every await site would need `changed()` plus a `borrow()` re-check
to avoid missing an edge, and the token expresses "cancelled" as a latched
one-way state, which is exactly the semantics wanted.

### Cancel at three points, not everywhere

```rust
pub async fn download_all(
    client: QobuzClient,
    config: Config,
    jobs: Vec<Job>,
    events: mpsc::Sender<JobEvent>,
    cancel: CancellationToken,
)
```

1. **Right after `Semaphore::acquire()`**, before any network I/O. This is
   where the bulk of a large batch is parked, so it is where cancellation is
   nearly free: emit `JobEvent::Cancelled { track_id }` and return.
2. **Around the whole retry loop** in `download_with_progress`:

   ```rust
   let result = tokio::select! {
       biased;
       r = download::with_retry(MAX_ATTEMPTS, || { … }) => r,
       _ = cancel.cancelled() => Err(Error::Cancelled),
   };
   drop(tx);
   let _ = forward.await;
   result
   ```

   One `select!` covers both the in-flight transfer and the backoff sleep,
   because both live inside that future. Crucially this needs **no signature
   change to `with_retry` or `stream_to_file`** — losing the race simply drops
   the transfer future, and the drop guard below handles the temp file. The
   `drop(tx)` / `forward.await` cleanup still runs, so the progress-forwarder
   task is joined rather than leaked.

   `biased` polls the **download branch first**, deliberately. The transfer's
   last step renames the temp file into place, and that side effect completes
   on the blocking pool before the future reports ready — so letting cancel win
   a tie would discard a download already sitting complete at `dest`, leaving an
   untagged file the caller was told never finished. Ordering the download first
   costs nothing in responsiveness: when it isn't ready it returns pending and
   the cancel branch is polled immediately after.
3. **Around the cover fetch** in `download_one`, falling back to no artwork. By
   that point the audio is already on disk, so cancelling outright would only
   strand an untagged file; instead the track finishes and cancellation merely
   gives up on the *artwork*. Without this, a cancel could block on an
   unresponsive cover host for the full 30-second `fetch_bytes` timeout — times
   `concurrency` tracks — before the batch stopped. `write_tags` itself is local
   and fast, so it is left to run.
4. **`run_job`** maps `Err(Error::Cancelled)` to `JobEvent::Cancelled` instead
   of `JobEvent::Failed`, so cancelling does not paint the queue red.

*Alternative considered — threading a token parameter through `with_retry` and
`stream_to_file` and checking it per chunk.* Rejected: it changes two public
signatures and sprinkles checks through the hot loop to buy nothing, since the
chunk loop awaits constantly and the enclosing `select!` already lands within
one chunk.

### `JoinSet` instead of `Vec<JoinHandle>`

`JoinSet` aborts its tasks on drop, which closes the "detached zombie
downloads" hole structurally rather than by convention — even a future drop
from a path nobody anticipated can no longer leave downloads running. It needs
no new dependency (tokio's `rt` feature is already on) and `join_next()` in a
loop replaces the sequential `h.await`. `download_all` still drains the set
before returning, so the last `Sender` clone is gone by the time it does and
the GUI's drain loop terminates exactly as it does today.

### Cancelled rows go back to `Queued`

`apply_event` handles `JobEvent::Cancelled` by setting the row to
`ItemStatus::Queued` with `downloaded = 0, total = None`. No new `ItemStatus`
variant, so the three exhaustive matches in `queue.rs` (`item_fraction`,
`badge_palette`, the row label) are untouched, and `startable` reports true
again the moment the batch stops — which is exactly the "resume by pressing
Start" behaviour wanted.

The partial file is deleted, so a resumed track restarts from zero rather than
being wrongly skipped by the already-exists check.

*Alternative considered — an `ItemStatus::Cancelled` badge.* Rejected as
churn for its own sake: it breaks three exhaustive matches, needs a decision
about whether `startable` counts it, and the information it adds ("this one was
interrupted") stops being true the moment the user resumes.

### The GUI keeps one token per batch

`App` gains `cancel: Option<CancellationToken>`. `spawn_downloads` creates a
**fresh** token per batch — reusing one would start the next batch
pre-cancelled — stores a clone, and moves the other into `download_all`.
`Message::CancelDownloads` calls `token.cancel()` and sets a "Cancelling…"
status; it does not touch `downloading` or the queue. `DownloadsFinished`
remains the single place that clears `downloading`, reads the token's
`is_cancelled()` to choose between the normal and the cancelled status text,
and then drops the token.

That ordering is the point: cancellation is a *request*, and the batch still
ends through its normal completion path. Nothing about the "who clears
`downloading`" invariant changes.

The Cancel button renders in the queue header only while `app.downloading`,
beside the disabled "Downloading…" indicator, and disables itself once the
token reports cancelled so it cannot be pressed twice.

## Risks / Trade-offs

- **Archive ordering.** This change's `downloader-gui` delta is written against
  the `hide-start-when-idle` version of the "Download queue screen"
  requirement, not the version currently in `openspec/specs/`. Archive
  `hide-start-when-idle` first, or its Start-control text is silently dropped.
  Flagged here because both changes are active at once.
- **Cancellation is not instantaneous** — a task inside `write_tags` or a
  filesystem rename finishes that step first. Bounded and short, and stopping
  mid-tag would be worse than finishing it. The UI says "Cancelling…" for that
  window.
- **A track can complete between the click and the stop**, so the final count
  may exceed what the user saw when they cancelled. Honest, and the status text
  reports the actual number.
- **`Error::Cancelled` must not be classified transient**, or `with_retry`
  would retry a cancellation. `is_transient()` matches only `RateLimited`,
  `Network`, and 5xx `Http`, so a new variant is non-transient by default —
  asserted by a test rather than left to inference.
- **Adding a `JobEvent` variant is a public API break** for anything matching
  it exhaustively. The GUI's `apply_event` is the only such site in the repo,
  and it must extract `track_id` for the new variant too.
- **The drop guard uses sync `std::fs::remove_file`**, since `Drop` cannot
  await. A single unlink on the runtime thread is acceptable; it runs only on
  the interrupted path.

## Migration Plan

Library callers of `engine::download_all` must pass a `CancellationToken`; the
GUI in this workspace is the only one. No persisted state, no config schema, no
on-disk format changes, so rollback is reverting the commits.
