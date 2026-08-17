## 1. Shared per-item fraction

All work is in `crates/qobuz-gui/src/app/view/queue.rs`.

- [x] 1.1 Add `fn item_fraction(it: &QueueItem, settled_on_error: bool) -> f32` next to `overall_progress`: `Queued` → `0.0`; `Downloading` → `it.downloaded as f32 / t as f32` clamped to `0.0..=1.0` when `it.total` is `Some(t)` with `t > 0`, else `0.0`; `Tagging` and `Done(_)` → `1.0`; `Error(_)` → `1.0` when `settled_on_error`, else `0.0`. Document why the error case differs between the two callers.
- [x] 1.2 Add the two thin wrappers `batch_fraction(it)` (`settled_on_error = true`) and `row_fraction(it)` (`settled_on_error = false`).

## 2. Overall progress

- [x] 2.1 Replace the body of `overall_progress` with: return `0.0` for an empty queue, otherwise `queue.iter().map(batch_fraction).sum::<f32>() / queue.len() as f32`. Delete the byte-sum fold and the done-count fallback branch.
- [x] 2.2 Rewrite the doc comment on `overall_progress` to describe coverage of the whole queue (pending items count as 0, terminal items as complete) instead of the old known-totals-only rationale.

## 3. Per-item row

- [x] 3.1 In `queue_row`, keep the existing match for the status *label* only (including the padded `downloading {:>3.0}%` text, which must use the same fraction it renders) and take the bar value from `row_fraction(it)`, so the label and the bar cannot diverge.
- [x] 3.2 Confirm the `Error` row still renders an empty bar and the `Tagging`/`Done` rows still render a full one.

## 4. Tests

Rework the `#[cfg(test)] mod tests` block at the bottom of `queue.rs`; the `item(...)` helper stays as-is.

- [x] 4.1 Remove `falls_back_to_done_fraction_without_totals` and rewrite `unknown_totals_do_not_inflate_progress` — both assert the old byte-subset contract.
- [x] 4.2 Add `pending_items_count_toward_denominator`: 4 × `Done` + 16 × `Queued` → `0.2` (the regression test for the reported bug; the old code returned `1.0` or `0.25` depending on totals).
- [x] 4.3 Add `partial_download_is_averaged`: one `Downloading` item at `total: Some(100), downloaded: 50` plus one `Queued` item → `0.25`.
- [x] 4.4 Add `skipped_items_count_as_complete`: a single `Done` item with `total: None, downloaded: 0` → `1.0`.
- [x] 4.5 Add `failed_items_are_settled`: one `Done` + one `Error` → `1.0`.
- [x] 4.6 Add `unknown_total_while_downloading_contributes_nothing`: one `Downloading` item with `total: None, downloaded: 999` plus one `Done` item → `0.5`.
- [x] 4.7 Add `row_bar_is_empty_for_failed_item`: `row_fraction` of an `Error` item → `0.0` while `batch_fraction` of the same item → `1.0`.
- [x] 4.8 Add `overdownload_is_clamped_per_item`: one item with `total: Some(100), downloaded: 150` plus one `Queued` item → `0.5`, not more.
- [x] 4.9 Keep `empty_queue_is_zero`.

## 5. Verify

- [x] 5.1 `cargo test -p qobuz-gui` passes.
- [x] 5.2 `cargo clippy --workspace` and `cargo fmt --check` are clean.
- [x] 5.3 `cargo run -p qobuz-gui`: queue an album with more tracks than `config.concurrency`, start downloads, and confirm the overall bar rises steadily, never reads full while rows are still `queued`, never snaps backwards as new rows start, and tracks the header's `N/M complete` counter.
- [x] 5.4 Re-run the same album so every file is skipped as already-present, and confirm the overall bar completes.
- [x] 5.5 `openspec validate fix-overall-progress` passes.
