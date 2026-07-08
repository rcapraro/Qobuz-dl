## Context

The core download path (`qobuz-core`) already isolates per-track failures and retries
transient errors internally (`MAX_ATTEMPTS = 4`, exponential backoff). Two shortcomings
motivate this change:

1. **Stale-URL-on-retry bug.** `engine::download_one` fetches a signed file URL once
   (step 1), then wraps only the *stream* in `with_retry`. Qobuz signed URLs expire
   quickly, so a retry that fires after expiry re-uses a dead URL and fails permanently.
2. **No relaunch path.** The GUI drops all `Job`s via `std::mem::take(&mut self.pending_jobs)`
   when a batch starts, and `QueueItem` stores neither `track_id` nor `Job`. A failed row
   therefore cannot reconstruct its own download. `JobEvent::Failed` only carries
   `track_id` + a stringified error.

Constraints: the GUI must not touch HTTP/FS directly — it delegates through the re-exports
in `lib.rs` and the `JobEvent` channel. Config uses `#[serde(default)]` throughout.

## Goals / Non-Goals

**Goals:**
- A retry that needs a signed URL always fetches a fresh one.
- Transient vs permanent error classification is a reusable, testable `Error` method.
- Failed downloads leave no stale `.part`; already-complete destinations are skipped.
- Backoff has jitter and honors `Retry-After` on 429.
- The user can relaunch a single failed track and retry all failed tracks from the Queue.

**Non-Goals:**
- HTTP Range / resume of a partially transferred file (retries restart from byte 0).
- Making retry limits or timeouts user-configurable (no new Settings control, no Config fields).
- Changing concurrency behavior or the `Semaphore` model.
- Enriching `JobEvent` with structured error types (keep the stringified error).

## Decisions

**1. Re-sign inside the retry unit.** Restructure `download_one` so the retried operation
encompasses *both* the `client.file_url(...)` fetch and `stream_to_file(...)`. Each attempt
gets a fresh signed URL. Alternative considered: wrap the whole `download_one` in
`with_retry` — rejected because tagging/cover-art fetch would then re-run on every attempt;
scoping the retry to URL+stream keeps side effects minimal.

**2. Extract `pub async fn download_job(client, config, job, events)`.** The body of the
per-job closure in `download_all` (emit `Started` → `download_one` → `Done`/`Failed`) becomes
a public function; `download_all` calls it under the semaphore. Re-export from `lib.rs`.
Rationale: gives a clean single-track entry point and keeps `download_all` a thin fan-out.
The GUI retry can call either `download_job` per track or `download_all` with a filtered
`Vec<Job>`; we standardize on reusing the existing `download_all` stream bridge with a
filtered vec so the GUI has exactly one download code path.

**3. `Error::is_transient(&self) -> bool`.** Move the classifier out of `download.rs` onto
`Error`. `with_retry` calls `e.is_transient()`. Enables a direct unit test and future reuse.

**4. Cleanup + idempotency in `stream_to_file`.** On any error after the `.part` file is
created, remove it (best-effort). Before streaming, if the final destination already exists,
return early as success. Rationale: makes relaunching a partially-completed batch cheap and
avoids orphaned temp files.

**5. Jitter + `Retry-After`.** Add bounded random jitter to the backoff delay; when the error
is a rate-limit and a `Retry-After` value is available, prefer it. Keep `MAX_ATTEMPTS`
internal. A small jitter source is needed — use the `rand` crate if already in the tree, else
derive jitter from a cheap entropy source without adding a heavy dependency.

**6. GUI retains jobs in `QueueItem`.** Add `track_id: i64` and `job: Job` to `QueueItem`
(`Job: Clone`). Start downloads by cloning jobs from the queue instead of `mem::take`, so the
rows keep their jobs. `Message::RetryTrack(i64)` resets one row to `Queued` and streams that
one job; `Message::RetryFailed` does the same for every `Error` row. Both reuse the
`StartDownloads` stream bridge and the existing `DownloadsFinished` error recount, and are
gated on `!downloading && signed_in`.

## Risks / Trade-offs

- **Re-signing adds an extra `file_url` request per retry.** → Only occurs on the retry path
  (already the slow path); the correctness win far outweighs the extra call.
- **Skip-if-exists could skip a genuinely wanted re-download.** → Acceptable: the destination
  path is fully templated per track/quality, so an existing file at that exact path is the
  same track already downloaded. A future "force re-download" toggle could override this.
- **Jitter dependency.** → Prefer an existing dependency; if none, use a minimal entropy
  source rather than pulling in `rand` solely for jitter.
- **`Retry-After` parsing.** → Header may be absent or in either delta-seconds or HTTP-date
  form; fall back to computed backoff when it can't be parsed.
- **GUI cloning jobs increases retained memory.** → `Job` holds track+album metadata only
  (no audio); negligible for realistic queue sizes.

## Migration Plan

Additive and backward-compatible: new public functions (`download_job`, `Error::is_transient`)
and new GUI messages/fields. No Config schema change, no persisted-state migration, no keyring
change. Rollback is a straight revert.

## Open Questions

- Jitter source: reuse an existing transitive dependency vs. a tiny inline entropy source —
  decide during implementation based on `Cargo.lock` contents.
