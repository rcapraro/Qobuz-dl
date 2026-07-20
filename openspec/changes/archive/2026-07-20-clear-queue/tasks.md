## 1. Message + update logic

- [x] 1.1 In `crates/qobuz-gui/src/app.rs`, add a `ClearQueue` variant to `enum Message`, in the `// Downloads.` group near `RetryFailed`.
- [x] 1.2 Add an update arm: `self.queue.clear();` and set a "Queue cleared." status message, returning `Task::none()`.

## 2. Header control

- [x] 2.1 In `crates/qobuz-gui/src/app/view/queue.rs::queue_view`, when `!app.queue.is_empty() && !app.downloading`, push a "Clear queue" button into the header (before "Start downloads"), built like the existing "Retry failed" button (`button(text("Clear queue")).padding([SPACE_XS, SPACE_MD]).height(CONTROL_HEIGHT).style(button::secondary).on_press(Message::ClearQueue)`).

## 3. Verify

- [x] 3.1 `cargo fmt` and `cargo clippy --workspace --tests` clean.
- [x] 3.2 `cargo build --workspace` clean.
- [x] 3.3 `cargo run -p qobuz-gui`: add several tracks, confirm a "Clear queue" control appears and empties the queue in one click, is hidden when the queue is empty, and is hidden/disabled while a batch is downloading.
