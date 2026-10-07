# Proposal

## Why

The queue is one flat list of "Artist — Title" rows. Queue three albums and it
becomes forty near-identical lines with no album context, no covers, and no
way to see how far each album has got. Finding a failed track, or clearing
one album that is not wanted after all, means scanning the whole list.

## What Changes

- Queued tracks are grouped under their album, always, whether they were
  added as an album, a single track, a playlist, or a track selection. A track
  added later joins its album's existing group. Groups appear in the order
  their first track was queued, and tracks keep queue order inside a group.
- Each group has a header: cover, album title, artist, a "done of total"
  count with a failure count when any failed, and the group's own progress
  bar, computed the same way as the overall bar but over that group only.
- A group can be collapsed to its header and expanded again. Groups start
  expanded.
- While no batch is running, a group header offers "Remove" for its tracks
  that are still queued. Tracks in other states stay.
- Rows inside a group drop the repeated album artist: they show the track
  number and title, plus the performer only when it differs from the album
  artist. Per-row badges, bars, Retry, and Remove are unchanged.
- The queue header, overall progress, Start, Cancel, "Retry failed (N)", and
  "Clear queue" are unchanged and still act on the whole queue.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`: new requirements for album groups in the queue, group
  progress and collapsing, and removing a group's queued tracks. The existing
  "Download queue screen" requirement is not modified. Its per-item rows,
  controls, and overall progress all still hold inside the groups.

## Impact

- `qobuz-gui` only. `app/view/queue.rs` (grouped rendering, group header),
  `app.rs` (collapsed-group state, `ToggleGroup` / `RemoveGroup` messages, and
  cover thumbnails for queued albums, which a new search's cache cleanup must
  keep), plus a pure grouping helper with tests.
- `qobuz-core`: no change. `Job.album` already carries each track's album id,
  title, artist, and cover.
- Dependencies: none added.
- Follows `album-detail-view`, now archived, and precedes
  `download-notifications`.
