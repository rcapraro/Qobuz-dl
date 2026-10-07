# Design

## Context

- `App.queue: Vec<QueueItem>` is flat, in enqueue order. Each item keeps its
  `job: Job`, so `job.album` gives the album `id`, `title`, `artist_name()`,
  and `image`. `job.track` gives `track_number`, `title`, and `artist_name()`.
  `QueueItem.title` is precomputed as `"Artist — Title"`.
- `view/queue.rs` renders a header (counter, Retry failed, Clear, Cancel,
  Start), the overall bar from `overall_progress(&[QueueItem])`, then one
  `queue_row` per item. Item lookups are linear scans on `track_id`, and the
  CLAUDE.md notes that this is intended.
- Thumbnails live in `App.thumbnails: HashMap<url, Handle>`. On `SearchDone`
  it is trimmed to the new results' covers, which would drop any cover the
  queue shows.
- Inter has ▶ (U+25B6) and ▼ (U+25BC), but not ▸ or ▾.

## Goals / Non-Goals

**Goals:**
- Group at render time. `queue` stays flat, so every existing handler
  (events, retry, dequeue, cancel, clear, start) and every existing test is
  untouched.
- Group summaries reuse `overall_progress`, so a group's bar follows exactly
  the rules the spec already sets for the overall bar.

**Non-Goals:**
- Per-group Start, Retry, or reordering. The global controls cover these.
- Automatically collapsing finished groups.
- Persisting the collapsed state across restarts. The queue itself is not
  persisted.

## Decisions

### Render-time grouping

```rust
struct Group<'a> { album: &'a Album, items: Vec<&'a QueueItem> }
fn groups(queue: &[QueueItem]) -> Vec<Group<'_>>   // by job.album.id, first-appearance order
```

This is a linear pass with a small `Vec` of groups searched by album id, in
the same no-index style as the rest of the queue. A track queued later for an
album already present joins that group, because grouping follows album id
and not insertion runs.

*Alternative:* store groups in `App` (`Vec<AlbumGroup { items }>`). Rejected
because every handler that finds an item by `track_id` would have to search
nested groups. Event application, retry, dequeue, and the startable or
cancelled checks all rely on the flat list today.

### Group summary

`GroupSummary { done, failed, total, fraction }`, where
`fraction = overall_progress(&items)`. `overall_progress` and the item-fraction
helpers currently take `&[QueueItem]`, so they change to accept an iterator or
slice of `&QueueItem`. The existing tests keep passing through a thin wrapper.

### State and messages

- `App.collapsed: HashSet<String>` holds collapsed album ids. Removing a
  group's last track leaves a stale id behind, which is harmless and cleared
  on `ClearQueue`.
- `ToggleGroup(String)` flips membership.
- `RemoveGroup(String)` retains items unless `job.album.id == id && status ==
  Queued`. It is guarded by `!self.downloading`, like `DequeueTrack`, and
  reports "Removed N track(s)".

### Covers for queued albums

`Resolved(Ok(jobs))` collects each new group's `album.image` (small, then
thumbnail, then large, as search does) and calls `fetch_missing_covers`. The
`SearchDone` trim keeps the union of the new results' covers and the queue's
album covers, so a search never blanks the queue. A small helper,
`queue_covers(&self)`, feeds both.

### Rows inside a group

`queue_row` keeps its badge, bar, Retry, and Remove, but takes its label from
the job: `"{n}. {title}"` (just the title when there is no number), with a
second muted line for the performer when it differs from the album artist.
`QueueItem.title` is then unused and is removed.

### Group header layout

On a `style::surface` panel: ▶/▼ toggle (compact, `TEXT_SM`), a 40 px cover,
bold album title over the muted artist, a `7 / 12 done · 1 failed` count
(muted, with "failed" in the error accent), a compact "Remove" when it
applies, and the group's progress bar under the row. The track rows sit
indented under the header. This reuses the album view's vocabulary: muted
secondary text, `surface`, and compact buttons.

## Risks / Trade-offs

- [Rendering re-groups on every frame] → One pass over the queue per view.
  Queues are hundreds of tracks at most. This matches the existing per-frame
  `overall_progress` and the done and failed counts.
- [Grouping by album id splits an album whose tracks report different album
  ids, as some compilations do] → Each id gets its own correctly titled
  group, which is accurate if slightly fragmented.
- [Removing `QueueItem.title` touches the queue test helper] → It is a
  mechanical change, covered by the compiler.
