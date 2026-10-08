# Proposal

## Why

Queue rows are the densest thing in the app. A finished track carries a lowercase `done` badge, a quality badge that says the tier twice (`FLAC 24/≤96 24bit/96kHz`), "Show in folder" and "Remove". Long titles wrap to three lines to make room. Every row also keeps a progress bar, full or empty, under the group's bar and the overall bar, so a finished album is a stack of identical blue stripes.

Group-level actions (Open folder, Remove, Retry failed, Clear queue) are enough in practice, by decision on 2026-10-08. This is the third change of the 2026-10-08 polish pass (settings-forms → chrome-and-layout → queue-row-redesign → search-affordances).

## What Changes

- **No row controls:** queue rows have no buttons. Per-row Retry, Remove and Show in folder are dropped.
  - Retry failed (N) relaunches failed tracks, including a single one.
  - A group's Remove takes out its settled tracks.
  - Open folder on the group opens where its files are.
- **Status:** shown in Title Case: Queued, Downloading 42%, Tagging, Done, Failed: <reason>. The colour roles are unchanged.
- **Progress bars:** a row shows one only while it is downloading or tagging. Queued, done and failed rows have none. The group and overall bars are unchanged.
- **Quality:** shown once per album. The group header carries the delivered quality most of its done tracks share. A row shows its own only when it differs from its group's, such as a track the service downgraded.
- **Shorter delivered label:** `FLAC 24/96`, `FLAC 16/44.1` or `MP3 320`, built from the actual bit depth and sample rate, instead of the tier name followed by the same numbers. This is a **BREAKING** change for anything parsing the old label text; nothing in the repository does.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `downloader-gui`:
  - "Download queue screen": rows offer no controls; status words, row progress bars and the retry/remove clauses are rewritten. All scenario names are kept.
  - "Remove a group's settled tracks": gains the collapsed-state scenario from the removed per-row requirement.
  - Adds "Delivered quality per album group".
  - Removes "Remove settled tracks" and "Show a track's file".
- `gui-theming`: "Consistent control sizing": the compact-variant example no longer cites queue rows.
- `audio-download`: adds "Delivered quality label", the short form reported for each finished track. "Select download quality" is unchanged.

## Impact

- `crates/qobuz-core/src/engine.rs`: `describe_delivered` and its tests.
- `crates/qobuz-gui/src/app/view/queue.rs`: `queue_row`, the group header quality, and the badge wording.
- `crates/qobuz-gui/src/app.rs`:
  - `Message::RetryTrack`, `RemoveTrack` and `RevealTrack`, their handlers and tests;
  - the per-row behaviours those tests covered move to group-level tests where not already covered.
- `crates/qobuz-gui/src/app/open.rs`, `tasks.rs`: the "reveal a file" path, if nothing else uses it.
- The README queue screenshot and its alt text ("with open-folder actions") need refreshing.
