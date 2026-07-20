## 1. Message + update logic

- [x] 1.1 In `crates/qobuz-gui/src/app.rs`, add a `DequeueTrack(i64)` variant to `enum Message`, in the `// Downloads.` group near `RetryTrack`.
- [x] 1.2 Add an update arm that removes the matching row only if it is still queued: `self.queue.retain(|it| !(it.track_id == id && matches!(it.status, ItemStatus::Queued)))`, and set a "Removed from queue." status message.

## 2. Queue row control

- [x] 2.1 In `crates/qobuz-gui/src/app/view/queue.rs::queue_row`, when `matches!(it.status, ItemStatus::Queued)`, push a Remove button onto the row's top line after the badge, built like the existing Retry control (`button(text("Remove").size(style::TEXT_SM))` + `button::secondary`) and gated with `on_press_maybe((!downloading).then_some(Message::DequeueTrack(it.track_id)))`.

## 3. Verify

- [x] 3.1 `cargo fmt` and `cargo clippy --workspace --tests` clean.
- [x] 3.2 `cargo build --workspace` clean.
- [x] 3.3 `cargo run -p qobuz-gui`: add several tracks, confirm queued rows show a Remove control that drops only that row; confirm the control is disabled while a batch is downloading and absent on downloading/done/error rows.
