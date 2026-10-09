## Context

`queue_view` (`app/view/queue.rs`) returns early when `App.tag_editor` is set, rendering only `queue_header(app, true)` and the editor. Album groups are built at render time by `groups()` and summarized by `summarize()` (done, failed, total, fraction), both private to `queue.rs`. Whether a group is settled is decided by `App::group_editable`: every track is `Done` or `Error` and not in `App.batch` while `downloading`. The editor's Save is gated by `App::save_ready`, which includes `tags_editable`, but the editor's status line (`progress()` in `app/view/tag_editor.rs`) does not explain that gate.

## Goals / Non-Goals

**Goals:**
- Reuse the queue's existing grouping and summary, so the strip and the full list can never disagree on counts or progress.
- Use one "in progress" predicate for the strip line and the Save notice.

**Non-Goals:**
- No change to the queue list, group headers, quality badge, or tab switching on add.
- No actions on strip lines (expand, cancel, edit); the editor stays the focus.
- No re-reading of the edited files when the album's download ends.

## Decisions

**In progress = not settled.** An album is in the strip exactly when its group is not settled in `group_editable`'s sense, ignoring the "at least one done" part: some track is queued, downloading, tagging, or in the running batch. Factoring that check out of `group_editable` (for example as `group_in_progress`) gives the strip and the Save notice the same answer that already gates Edit tags. *Alternative:* a separate status-only check in the view; rejected because it would ignore `App.batch` and drop an album from the strip while its batch is still running, which disagrees with when Edit tags returns.

**Strip built from `groups()` in `queue.rs`.** The editor branch of `queue_view` filters `groups(&app.queue)` with that predicate and renders each with a new `strip_line`: `cover` at a small size, title on one line (`Wrapping::None` in a fill-width cell), the `summarize` count text, and a progress bar of `summary.fraction`. Placing it in `queue.rs` keeps `groups`/`summarize` private. *Alternative:* render it from `view/tag_editor.rs`; rejected because it would need those helpers exported for a single caller.

**Height cap by a fixed-height scrollable.** The strip is a `scrollable` whose height is `min(n, 3) × line height + spacing`, computed from style constants, so up to three lines show without a scrollbar and the editor starts at a stable offset beyond that. *Alternative:* `Length::Shrink` with a max height; the computed height gives an exact cap without relying on a max-height API (check iced 0.14 for one during implementation).

**Covers.** Strip lines use the same thumbnail URLs as group headers; `queue_covers()` / `covers_in_use()` already include every queued album, so no cache change is needed. This should be confirmed during implementation (tasks 1.3).

**Save notice in the editor's status line.** `progress()` gains a case before "Unsaved changes": when `!app.tags_editable(&editor.album_id)` and the editor is not saving or looking up, it reads "Back in the queue — save once its download ends". It takes `app` (or that bool) as an argument. Saving and lookup messages keep priority, since they describe work in flight. *Alternative:* a separate banner; rejected because the status line already exists for exactly this and keeps the header's height constant.

## Risks / Trade-offs

- [The strip pushes the editor down by up to three lines] → Lines are compact (small cover, one text row plus a thin bar), and the cap keeps the cost bounded.
- [A line's title is long] → Fill-width with no wrapping; it is clipped rather than growing the line, keeping the computed height exact.
- [Edited album re-queued and its files rewritten by the download] → Out of scope: an existing destination is skipped by the engine, so done files are not rewritten; newly added tracks of that album are not in the open editor, as today.

## Decisions added during implementation

**The strip replaces the overall bar.** With the editor open, the queue-wide bar sat directly above the strip's album bars and, with one album downloading, read as the same progress drawn twice; it also mostly counts finished albums, including the edited one. `queue_header` takes an `overall_bar` flag, false exactly when the strip is shown; the "N / M complete" count stays, so the overall figure is still there. *Alternative:* keep the overall bar and drop the strip's bars for a "· 40 %" label; rejected because the per-album bar is the progress worth watching while editing.
