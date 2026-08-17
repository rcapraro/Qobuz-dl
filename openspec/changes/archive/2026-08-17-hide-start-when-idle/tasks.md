## 1. Shared predicate

- [x] 1.1 In `crates/qobuz-gui/src/app.rs`, add a free function `fn startable(queue: &[QueueItem]) -> bool` beside the `QueueItem`/`ItemStatus` definitions, returning `queue.iter().any(|it| matches!(it.status, ItemStatus::Queued))`. No visibility modifier — descendant modules can already see it. Document that failed items are deliberately excluded because the Retry controls own them.
- [x] 1.2 In `Message::StartDownloads`, narrow `jobs_with(|s| matches!(s, ItemStatus::Queued | ItemStatus::Error(_)))` to `Queued` only, so the action matches `startable`. Update the stale comment "Download everything not yet done (fresh + previously errored)". Keep the `jobs.is_empty()` early return and its status message as a guard.

## 2. Header control

- [x] 2.1 In `crates/qobuz-gui/src/app/view/queue.rs`, wrap the `header.push(styled_button(…))` block in `if app.downloading || super::super::startable(&app.queue) { … }`. Leave the button's label logic and its `on_press_maybe` gate untouched.
- [x] 2.2 Add a comment explaining why `app.downloading` is part of the condition: rows leave `Queued` as they start, so without it the button would disappear mid-batch and take the "Downloading…" indicator with it.

## 3. Empty state

- [x] 3.1 In `queue_view`, early-return when `app.queue.is_empty()`: a `container` filling both axes and centered on both, holding a column with "Nothing queued yet." at `style::TEXT_BODY` and "Search for an album or paste a Qobuz URL to add tracks." at `style::TEXT_SM`. Use the default text color — there is no muted token in `style::Accents`.
- [x] 3.2 Confirm the empty branch renders neither the "0/0 complete" counter nor the overall progress bar, and that the non-empty branch is otherwise unchanged.

## 4. Tests

Extend the existing `#[cfg(test)] mod tests` in `queue.rs`, reusing its `item(total, downloaded, status)` helper and the `done()` helper; import `super::super::startable`.

- [x] 4.1 Add `startable_with_a_queued_track`: one `Queued` item → `true`.
- [x] 4.2 Add `not_startable_when_queue_is_empty`: `&[]` → `false`.
- [x] 4.3 Add `not_startable_when_all_done`: two `Done` items → `false`.
- [x] 4.4 Add `not_startable_when_only_failures_remain`: `Done` + `Error` → `false` (encodes the "Retry owns failures" decision).
- [x] 4.5 Add `startable_when_a_queued_track_sits_beside_failures`: `Queued` + `Error` → `true`.
- [x] 4.6 Add `not_startable_while_a_lone_item_downloads`: one `Downloading` item → `false`, documenting that the button stays visible through the `app.downloading` arm rather than through this predicate.
- [x] 4.7 Confirm the existing progress tests still pass unchanged.

## 5. Verify

- [x] 5.1 `cargo test -p qobuz-gui` passes.
- [x] 5.2 `cargo clippy --workspace --all-targets` and `cargo fmt --check` are clean.
- [x] 5.3 `cargo run -p qobuz-gui`: with an empty queue the Queue tab shows the hint and no Start button, no counter, no progress bar.
- [x] 5.4 Add an album → Start appears; press it → the label becomes "Downloading…" and the button stays visible for the whole batch; on a clean finish Start is gone and "Clear queue" remains.
- [x] 5.5 Force a failure → Start is gone, "Retry failed (N)" is present, and pressing it re-runs the failed tracks.
- [x] 5.6 Press "Clear queue" → the screen returns to the empty-state hint with no Start button.
- [x] 5.7 `openspec validate hide-start-when-idle` passes.
