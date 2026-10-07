# Design

## Context

- `JobEvent::Done { track_id, path, delivered }` already carries each track's
  final path, including the skip path where the file already existed
  (`download_one` returns `dest`). `App::apply_event` keeps only `delivered`
  (`ItemStatus::Done(delivered)`) and drops the path.
- `ItemStatus::Done(String)` is matched in 13 places: badges, counts,
  `startable`, group summaries, and tests.
- Removal today:
  - `DequeueTrack(id)` removes a queued item.
  - `RemoveGroup(album_id)` removes a group's queued items.
  - Both are ignored while `downloading`. The row's Remove is disabled then,
    and the group's Remove is hidden.
- `App.batch` and the end-of-batch notification count a batch by scanning the
  queue when it ends. Removal stays blocked during a batch, so no batch row
  can vanish before it is counted.
- Settings renders "Download to:" as `labeled_row` holding the path text and
  `Choose…`.

## Goals / Non-Goals

**Goals:**
- No change to `ItemStatus` or `qobuz-core`.
- No new dependency: opening goes through each OS's own command.
- The pure parts are unit-tested: which statuses are removable, and an album's
  shared folder.

**Non-Goals:**
- Deleting files, and "Clear completed".
- Removal during a batch.
- Watching the filesystem. Existence is checked only when a control is
  pressed.

## Decisions

### Keep the path beside the status, not inside it

`QueueItem` gains `path: Option<PathBuf>`, set on `JobEvent::Done` and
cleared wherever an item is reset to queued (`spawn_downloads`, `Cancelled`).

*Alternative:* `ItemStatus::Done { delivered, path }`. Rejected because it
touches all 13 matches for no behavioural gain, since only the open and
reveal controls read the path.

### One removal rule

`fn removable(status) -> bool` is true for `Queued | Done(_) | Error(_)`.
- `DequeueTrack` is renamed `RemoveTrack`, and `RemoveGroup` uses `removable`
  in place of `Queued`. Both stay ignored while `downloading`.
- The row view offers Remove when `removable`, disabled while downloading.
- The group view offers Remove when any of its items is `removable` and no
  batch runs.
- The status line reports "Removed N track(s) from the queue." in both cases.

### Opening, per OS, fire-and-forget

`app/open.rs`:

```rust
pub(super) fn open_dir(dir: &Path) -> io::Result<()>
pub(super) fn reveal(file: &Path) -> io::Result<()>
```

| OS | `open_dir` | `reveal` |
|---|---|---|
| macOS | `open <dir>` | `open -R <file>` |
| Windows | `explorer <dir>` | `explorer /select,<file>` |
| other unix | `xdg-open <dir>` | `xdg-open <parent>` |

Each function checks existence first and returns `NotFound` with a clear
message. The command runs in `tasks::open_path` under `spawn_blocking`, which
waits so the child is reaped. Only a failure to start the command counts as
an error. Exit codes are ignored, because `explorer` returns 1 even when it
succeeds. The result arrives as `Message::Opened(Result<(), String>)`, and an error becomes
`Status::error`. On Windows, `explorer /select,<file>` must be a single
argument, so it is built as one `OsString`.

*Alternative:* the `opener` crate. Rejected because the project rule is to
add a dependency only when the platform cannot do the job, and three commands
do it here.

### An album's folder

`fn shared_folder(paths) -> Option<PathBuf>` takes the done items' files and
returns the longest common ancestor of their parent folders. It is `None`
when no item is done, which hides the control. For an album in one folder,
that folder is the result. When a template splits an album, the result is
their nearest common folder.

### Settings

"Download to:" gains a secondary **Open** after **Choose…**. It sends
`OpenDownloadDir` and opens `config.download_dir` through `open_dir`. A
missing directory is reported, not created.

### Messages

- `RemoveTrack(i64)` replaces `DequeueTrack`.
- `RemoveGroup` is widened.
- New: `OpenAlbumFolder(String)` (album id), `RevealTrack(i64)`,
  `OpenDownloadDir`, and `Opened(Result<(), String>)`. A single result message,
  the same shape as `SearchDone` and `Resolved`, replaces the planned
  `OpenFailed` plus a no-op `Opened`.

## Risks / Trade-offs

- [Linux has no standard "reveal"] → It opens the folder, as the spec allows.
- [`xdg-open` may be missing on minimal Linux setups] → Starting it fails,
  which the status line reports.
- [The path stays in memory after the user moves files] → The control checks
  existence when pressed and reports a missing file. It never acts on a stale
  path silently.
