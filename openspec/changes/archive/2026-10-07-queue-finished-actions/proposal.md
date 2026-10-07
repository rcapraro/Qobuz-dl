# Proposal

## Why

Once tracks finish, the queue can't be tidied short of "Clear queue", which
also drops what is still waiting. Finished and failed rows pile up across
batches. After a download, finding the files means opening a file manager and
working out where the folder template put them.

## What Changes

- Every settled row (queued, done, or failed) offers **Remove**, which takes
  the track out of the list. Removing never deletes files from disk.
  Downloading and tagging tracks cannot be removed.
- The album group's **Remove** widens from "its queued tracks" to "all its
  settled tracks" (queued, done, and failed). The group disappears once empty.
- As today, no removal is possible while a batch is running.
- A group with at least one done track offers **Open folder**, which opens the
  folder holding that album's downloaded files. When the template spreads them
  over several folders, it opens the nearest folder they share.
- Each done row offers **Show in folder**, which opens its folder with the file
  selected. On Linux, where file managers have no standard "select", it opens
  the folder.
- Settings gains **Open** next to "Download to:", which opens the download root.
- If a file or folder no longer exists, the status line reports it instead of
  the control silently doing nothing.
- The header keeps its current controls. There is no "Clear completed".

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`:
  - Removing done and failed tracks per row is a new requirement. "Download
    queue screen" is not modified: it requires a Remove on queued rows and does
    not forbid one elsewhere.
  - "Remove a group's queued tracks" is removed. It says done and failed tracks
    stay, which this change reverses. It is replaced by "Remove a group's
    settled tracks".
  - Opening a group's folder, revealing a track's file, and opening the
    download folder from Settings are new requirements.

## Impact

- `qobuz-gui` only:
  - `app.rs`: `QueueItem.path` kept from `JobEvent::Done`, widened removal
    messages, open/reveal messages.
  - A small `app/open.rs` for the per-OS commands.
  - `app/view/queue.rs` and `app/view/settings.rs`: new controls.
- `qobuz-core`: no change. `JobEvent::Done` already carries the final path,
  including when an existing file was skipped.
- Dependencies: none added. Opening uses each OS's own command through
  `std::process::Command`.
