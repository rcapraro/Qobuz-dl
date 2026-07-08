## 1. Core: error classification

- [x] 1.1 Add `pub fn is_transient(&self) -> bool` to `Error` in `crates/qobuz-core/src/error.rs` (429 / `Network` / HTTP 5xx), matching the current `download.rs` logic
- [x] 1.2 Update `download.rs::with_retry` to call `e.is_transient()` and remove the private `is_transient` free function
- [x] 1.3 Add a unit test asserting transient vs permanent classification for representative `Error` variants

## 2. Core: download reliability

- [x] 2.1 In `download.rs::stream_to_file`, return early (as success) when the destination file already exists
- [x] 2.2 In `download.rs::stream_to_file`, remove the `.part` temp file (best-effort) on any error after it was created
- [x] 2.3 In `download.rs::with_retry`, add bounded jitter to the backoff delay (reuse an existing dependency for randomness, or a minimal inline entropy source)
- [x] 2.4 Honor a `Retry-After` header on rate-limit (429) responses when present, falling back to computed backoff when absent/unparseable
- [x] 2.5 Add/extend unit tests: skip-if-exists behavior and permanent-error-not-retried

## 3. Core: fresh signed URL on retry + single-job entry point

- [x] 3.1 In `engine.rs::download_one`, restructure so the retried unit re-fetches a fresh signed `file_url` before each streaming attempt (no reuse of an expired URL)
- [x] 3.2 Extract the per-job closure body from `download_all` into `pub async fn download_job(client, config, job, events)`; have `download_all` call it under the existing semaphore
- [x] 3.3 Re-export `download_job` from `crates/qobuz-core/src/lib.rs`
- [x] 3.4 Run `cargo test -p qobuz-core` and `cargo clippy --workspace`; confirm existing tests still pass

## 4. GUI: retain jobs per queue item

- [x] 4.1 Add `track_id: i64` and `job: Job` fields to `QueueItem` in `crates/qobuz-gui/src/app.rs`; populate them when the queue is built from resolved jobs
- [x] 4.2 Replace `std::mem::take(&mut self.pending_jobs)` in `StartDownloads` with cloning the jobs to download from the queue, so rows retain their `Job`

## 5. GUI: retry messages and handlers

- [x] 5.1 Add `Message::RetryTrack(i64)` and `Message::RetryFailed` to the `Message` enum
- [x] 5.2 Handle `RetryTrack`: reset the targeted row's status to `Queued` and spawn the existing download-stream bridge with just that job; gate on `!downloading && signed_in`
- [x] 5.3 Handle `RetryFailed`: reset every `Error` row to `Queued` and spawn the bridge with all those jobs; gate on `!downloading && signed_in`
- [x] 5.4 Confirm `DownloadsFinished` recomputes the error count correctly after a retry batch

## 6. GUI: retry affordances in the view

- [x] 6.1 In `queue_row`, render a per-row Retry button shown only for `ItemStatus::Error(_)`, disabled while `downloading`
- [x] 6.2 In `queue_view` header, render a "Retry failed (N)" button shown when any row is `Error`, disabled while `downloading`, using existing button helpers and `on_press_maybe`
- [x] 6.3 Ensure `queue_row` has access to the row's `track_id` to emit `RetryTrack`

## 7. Verification

- [x] 7.1 `cargo fmt` and `cargo clippy --workspace` clean
- [x] 7.2 `cargo test -p qobuz-core` passes (including new tests)
- [ ] 7.3 `cargo run -p qobuz-gui`: enqueue a multi-track album, force a failure, confirm the row shows a Retry button and the header shows "Retry failed (N)"; restore connectivity, retry, confirm the track completes and the error count clears — **requires interactive/manual confirmation**
- [x] 7.4 `openspec validate harden-download-retry` passes
