## 1. Dependency and error type

- [x] 1.1 Add `tokio-util = "0.7"` to `[workspace.dependencies]` in the root `Cargo.toml`, and `tokio-util.workspace = true` to `crates/qobuz-core/Cargo.toml`. No feature flag — `tokio_util::sync` is always compiled, and requesting a `sync` feature is a resolution error. The GUI does not need it directly; it gets `CancellationToken` re-exported from `qobuz-core`.
- [x] 1.2 In `crates/qobuz-core/src/error.rs`, add `#[error("download cancelled")] Cancelled` to `Error`.
- [x] 1.3 Extend the existing `classifies_permanent_errors` test in `error.rs` to assert `!Error::Cancelled.is_transient()` — a cancellation that `with_retry` retried would defeat the whole feature.
- [x] 1.4 Re-export `CancellationToken` from `crates/qobuz-core/src/lib.rs` alongside the other public items, so the GUI does not need its own `tokio-util` dependency.

## 2. Temp-file cleanup on drop (pre-existing leak)

- [x] 2.1 In `crates/qobuz-core/src/download.rs`, add a small RAII guard holding the `.partN` `PathBuf` whose `Drop` calls `std::fs::remove_file` (sync — `Drop` cannot await), with a `disarm`/`into_inner` method to defuse it after a successful rename.
- [x] 2.2 Use it in `stream_to_file`: arm the guard right after `part_path(dest)`, disarm it after `tokio::fs::rename(&tmp, dest)` succeeds. The existing explicit `remove_file` in the `Err` branch becomes redundant — remove it so there is one cleanup path, not two.
- [x] 2.3 Add a test that the guard removes its file when dropped, and another that it leaves the file alone after being disarmed. Use `std::env::temp_dir()` with a unique name; do not rely on network or on the Qobuz client.
- [x] 2.4 Confirm the existing `skips_when_destination_exists` and `part_paths_for_the_same_dest_are_unique` tests still pass unchanged.

## 3. Core cancellation

- [x] 3.1 In `crates/qobuz-core/src/engine.rs`, add `Cancelled { track_id: i64 }` to `JobEvent`.
- [x] 3.2 Add a `cancel: CancellationToken` parameter to `download_all` and update its doc comment to state that cancellation is cooperative and that the function still drains all work before returning.
- [x] 3.3 Replace the `Vec<JoinHandle<_>>` with a `tokio::task::JoinSet`, draining it with `join_next()` in place of the sequential `h.await` loop, keeping the existing `tracing::error!` on a panicking task. This makes drop abort the tasks instead of detaching them.
- [x] 3.4 In the spawned job body, check `cancel.is_cancelled()` immediately after `permit_sem.acquire()` and before any network I/O: emit `JobEvent::Cancelled { track_id }` and return. This is where most of a large batch is parked.
- [x] 3.5 In `download_with_progress`, wrap the `with_retry(...)` await in `tokio::select!` with `biased;`, racing `cancel.cancelled() => Err(Error::Cancelled)` against the retry future. Keep the existing `drop(tx); let _ = forward.await;` cleanup after the select so the progress-forwarder task is still joined. Do **not** change the signatures of `with_retry` or `stream_to_file`.
- [x] 3.6 In `run_job`, match `Err(Error::Cancelled)` and emit `JobEvent::Cancelled { track_id }` instead of `JobEvent::Failed`.
- [x] 3.7 Thread `cancel` from `download_all` through `run_job` and `download_one` to `download_with_progress`.
- [x] 3.8 Add an engine test: with an already-cancelled token, `download_all` over a couple of `sample_job()`s emits exactly one `JobEvent::Cancelled` per job, emits no `Done`/`Failed`, makes no network request, and returns (closing the event channel).

## 4. GUI wiring

- [x] 4.1 In `crates/qobuz-gui/src/app.rs`, add `cancel: Option<CancellationToken>` to `App`, initialised `None` in `App::new`.
- [x] 4.2 Add `Message::CancelDownloads` to the `Message` enum under the Downloads group.
- [x] 4.3 In `spawn_downloads`, create a **fresh** `CancellationToken` per batch (never reuse — a reused token would start the next batch already cancelled), store a clone in `self.cancel`, and pass the other into `engine::download_all`.
- [x] 4.4 Handle `Message::CancelDownloads`: call `cancel()` on the stored token if present and set `self.status = "Cancelling…"`. Do **not** touch `self.downloading` or the queue here — the batch still ends through its normal completion path.
- [x] 4.5 In `apply_event`, extract `track_id` for the new `JobEvent::Cancelled` variant and handle it by setting the row to `ItemStatus::Queued` with `downloaded = 0, total = None`.
- [x] 4.6 In `Message::DownloadsFinished`, read `is_cancelled()` from the stored token before clearing it, and use it to choose the status text (a cancelled batch reports how many completed rather than "All downloads finished."). Then set `self.cancel = None`. Keep `self.downloading = false` here as the single place it is cleared.

## 5. Queue header control

- [x] 5.1 In `crates/qobuz-gui/src/app/view/queue.rs`, render a "Cancel" button in the header while `app.downloading`, beside the disabled "Downloading…" indicator, styled like the other secondary header buttons.
- [x] 5.2 Disable the Cancel button once the token reports cancelled (label it "Cancelling…") so it cannot be pressed twice. Expose whatever minimal accessor on `App` this needs rather than making the field public.
- [x] 5.3 Confirm the existing header conditions are untouched: Retry failed and Clear queue stay hidden while `app.downloading`, and become available again once the cancelled batch has stopped.

## 6. Verify

- [x] 6.1 `cargo test --workspace` passes.
- [x] 6.2 `cargo clippy --workspace --all-targets` and `cargo fmt --check` are clean.
- [x] 6.3 `cargo run -p qobuz-gui`: queue an album larger than `config.concurrency`, start it, and press Cancel mid-batch. In-flight and pending rows return to "queued", completed rows stay done, and the status line reports the cancellation.
- [x] 6.4 Confirm cancellation is prompt — the batch stops in about a second, not after a full retry backoff — and that "Downloading…" gives way to the idle controls.
- [x] 6.5 Check the destination directory for orphaned `.part*` files after cancelling; there must be none. Cancel and resume several times and confirm they still do not accumulate.
- [x] 6.6 Press Start after cancelling and confirm the requeued tracks download to completion, then Clear queue and confirm the queue empties.
- [x] 6.7 `openspec validate cancel-download-batch` passes.
