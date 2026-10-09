## Why

While an album's tag editor is open, it takes the queue list's place, so albums added or downloading meanwhile are hidden behind a single overall progress bar. Adding an album from Search switches to the Queue tab and lands on the editor, and the new album's progress is not visible anywhere. Also, if the edited album goes back into the queue, Save is disabled and nothing says why.

## What Changes

- While the tag editor is open, the Queue tab shows a compact strip of in-progress albums between the queue header and the editor. Each line shows the album's cover, title, done count and progress bar. An album appears while any of its tracks is queued, downloading, tagging or in the running batch, and leaves the strip when none is. The strip is hidden when no album is in progress.
- The strip shows at most three albums at a time and scrolls beyond that, so the editor keeps its space. Its lines carry no track rows, actions or quality badge.
- When the edited album has a track back in the queue, the editor says why Save is unavailable: the album is waiting on its download.
- Unchanged: the group quality badge still appears only once a track is done, and adding tracks still switches to the Queue tab.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`: the "Edit an album's tags from the queue" requirement now shows the editor below a strip of in-progress albums instead of hiding them; a new requirement defines that strip.
- `tag-editing`: a new requirement makes the editor explain why Save is unavailable while its album is back in the queue.

## Impact

- `crates/qobuz-gui/src/app/view/queue.rs`: the editor branch of `queue_view` adds the strip; reuses `groups()` and `summarize()`.
- `crates/qobuz-gui/src/app/view/tag_editor.rs`: the editor's status line covers the album being back in the queue.
- `crates/qobuz-gui/src/app.rs`: possibly a small helper exposing whether a group is in progress (reusing `group_editable`'s notion of the running batch).
- No core, dependency or config changes.
