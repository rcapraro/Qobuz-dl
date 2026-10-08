# Design

## Context

See proposal.md for motivation and the three delta specs for requirements.

`queue_row` (`app/view/queue.rs`) builds a row as:
- a title/performer column;
- a status `Badge` (lowercase text, monospace, padded percentage);
- for done rows, a `quality_badge` with the delivered label;
- compact Retry / Show in folder / Remove buttons;
- a full-width progress bar below, for every state.

`group_view` builds the header: toggle, cover, title/artist, `N / M done`, Open folder and Remove. The three row buttons send `Message::RetryTrack`, `RevealTrack` and `RemoveTrack`. `RevealTrack` is the only user of `open::Target::File`, which reveals a file with `open -R` or `explorer /select,`.

The delivered label is built in core by `engine::describe_delivered` as `"{tier label} {bit}bit/{rate}kHz"`, e.g. `FLAC 24/≤96 24bit/96kHz`, and travels to the GUI as `JobEvent::Done { delivered: String }`.

## Goals / Non-Goals

**Goals:**
- A row is one line of text plus at most one badge, and one more only for a downgraded track.
- One quality vocabulary: the same short label in the group header and on any differing row.

**Non-Goals:**
- Hover-revealed row actions. Rows have no actions at all, by decision.
- A per-track retry. "Retry failed (N)" covers one failed track as well as many.
- Changing group or overall progress, collapsing, or the queue header controls.

## Decisions

### Short delivered label, built in core
`describe_delivered` returns:
- `MP3 320` for the MP3 tier, whatever the response reports;
- `FLAC {bit}/{rate}` when the bit depth and sampling rate are present, with the rate formatted through `f64`'s `Display` so 96.0 reads `96` and 44.1 reads `44.1`;
- otherwise the tier label, as today.

The engine's existing tests (24/96.0 and 16/44.1) are updated, and an MP3 case is added.

*Alternative:* reformatting the string in the GUI. Rejected because the label is opaque text by the time it reaches the GUI, and parsing it back would couple the GUI to core's format.

### Group quality is the most common done label
A pure helper in `queue.rs`, `group_quality(&[&QueueItem]) -> Option<&str>`, counts the `Done(label)` values. It returns the most frequent one, with ties going to the label reached first in queue order so the result is stable. The header shows it as a `quality_badge` before the done count. `queue_row` receives the group label and shows a row's `quality_badge` only when the row is `Done(label)` with a different label. Unit tests cover:
- all tracks the same;
- one track downgraded;
- none done;
- a tie.

### Row anatomy
`queue_row(it, group_quality)`:
- **Left:** a title/performer column at `Fill`.
- **Right:** the optional differing quality badge, then the status badge.
- **Status text:** `Queued`, `Downloading {:>3.0}%` (still monospace so the badge doesn't jitter), `Tagging`, `Done` and `Failed: {reason}`. Colours stay those of `badge_palette`.
- **Progress bar:** drawn only for `Downloading | Tagging`. Other rows get an empty `Space` of the same height (`PROGRESS_HEIGHT`), so rows don't change height, and the list doesn't shift, as tracks start and finish. This was added after code review; it costs about 12px per row.

The `downloading`/`can_remove` parameters go. With no failed row drawing a bar, the split between `row_fraction` (failed = 0) and `batch_fraction` (failed = 1) has no visible effect, so both fold back into one `item_fraction` where failed counts as settled.

### Removed messages and code
- `Message::RetryTrack`, `RemoveTrack` and `RevealTrack` and their handlers go.
- `open::Target` goes entirely: its `File` variant had no caller left, and a one-variant enum only wrapped a path. `open::command`, `tasks::open_path` and `App::open_folder` take the folder path directly. The project CLAUDE.md mention of `open -R` / `explorer /select,` goes too.
- `removable` and `App::can_remove` stay, because group Remove uses them.
- `QueueItem.path` stays, because `album_folder` derives the shared folder from it.

Of the four `RemoveTrack` tests, three behaviours were already covered by `RemoveGroup` tests:
- done, failed and queued rows go while a row in progress stays;
- a batch's own rows stay;
- a track added mid-batch goes.

The collapsed-state test already used `RemoveGroup`. One test is added: removing a group drops its failures from what "Retry failed" relaunches.

## Risks / Trade-offs

- [The user can no longer remove one track from an album] → Accepted by decision. Untick it in the album view before adding, or remove the group.
- [No per-file reveal] → Open folder opens the album's folder, where the file is. Only the selection highlight is lost.
- [The delivered label changes text] → Nothing parses it. Notifications count tracks, not labels.
- [A tie between two labels picks one arbitrarily] → It is deterministic (queue order), and the other label then shows on its rows, so nothing is hidden.
