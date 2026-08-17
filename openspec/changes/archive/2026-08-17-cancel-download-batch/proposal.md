## Why

Once a batch starts there is no way to stop it. The only controls that touch
the queue — Retry, Remove, Clear queue — are all hidden while a batch runs,
because mutating the queue cannot stop work that is already in flight. A user
who starts a 200-track album by mistake, picks the wrong quality, or needs the
bandwidth has to quit the app. Two archived changes explicitly deferred this
and listed "no cancellation of an in-progress batch" as a non-goal; this change
takes it on.

The engine cannot simply be dropped to stop it. `download_all` spawns one
detached `tokio::spawn` per job (plus one progress forwarder per in-flight
job), so dropping its future detaches the tasks rather than stopping them:
downloads would keep streaming, renaming, and tagging in the background with
the UI showing nothing. Cancellation has to be cooperative and reach into
`qobuz-core`.

## What Changes

- A Cancel control appears on the Queue screen while a batch is running, and
  stops it: no further tracks start, in-flight transfers stop mid-stream, and
  the pending retry backoff is interrupted rather than waited out.
- Cancelling returns interrupted and not-yet-started tracks to the queued
  state; tracks that already completed stay done. The Start control reappears,
  so a cancelled batch is resumed by starting it again. Clear queue also
  becomes available again once the batch has stopped — cancel first, then
  clear.
- `qobuz-core` gains cooperative cancellation: `download_all` takes a
  cancellation token, and a new `JobEvent::Cancelled` and `Error::Cancelled`
  distinguish a cancelled track from a failed one, so cancelling does not paint
  the queue red.
- **BREAKING (library API)**: `engine::download_all` takes an additional
  cancellation argument. The GUI is the only caller.
- **Bug fix, required by the above**: a partial `.partN` file is currently only
  removed on the error branch of `stream_to_file`, never when the future is
  dropped mid-stream — so an interrupted transfer orphans it permanently, and
  the sequence number means repeated attempts accumulate `track.part0`,
  `track.part1`, … This already contradicts the `audio-download` requirement
  that "a failed or aborted download SHALL NOT leave a stale partial file
  behind"; cancellation would turn a latent leak into a routine one. The temp
  file gains a cleanup-on-drop guard.
- **Hardening, required by the above**: the per-job tasks move to a `JoinSet`
  so the batch can no longer outlive its own future as detached work.
- Not changed: the download engine's concurrency, retry classification and
  backoff policy, tagging, path templating, or anything about how a normal
  (uncancelled) batch behaves.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `audio-download`: gains a requirement that a batch can be cancelled
  cooperatively and promptly, and that cancellation is distinguishable from
  failure. The existing "Concurrency and retry" requirement is tightened so its
  no-stale-partial-file guarantee holds when a transfer is interrupted, not
  only when it returns an error.
- `downloader-gui`: the "Download queue screen" requirement gains the Cancel
  control, what cancelling does to each row, and which controls return
  afterwards.

## Impact

- `Cargo.toml` (workspace) and `crates/qobuz-core/Cargo.toml` — a new
  `tokio-util` dependency for `CancellationToken`, which lives in that crate's
  always-available `sync` module and needs no feature flag. It is already in
  `Cargo.lock` at 0.7.18 via `reqwest`/`h2`, so no new version enters the graph.
  `qobuz-core` re-exports `CancellationToken` so the GUI does not need its own
  dependency on it.
- `crates/qobuz-core/src/engine.rs` — `download_all` signature, `JoinSet`,
  cancellation checks, `JobEvent::Cancelled`.
- `crates/qobuz-core/src/download.rs` — the temp-file drop guard.
- `crates/qobuz-core/src/error.rs` — `Error::Cancelled`.
- `crates/qobuz-gui/src/app.rs` — the token on `App`, `Message::CancelDownloads`,
  the `JobEvent::Cancelled` arm, and the end-of-batch status text.
- `crates/qobuz-gui/src/app/view/queue.rs` — the Cancel button.
- No change to persisted config, to the on-disk layout of completed downloads,
  or to authentication.
