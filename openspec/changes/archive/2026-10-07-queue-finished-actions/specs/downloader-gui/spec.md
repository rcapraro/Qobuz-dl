# Spec Delta

## ADDED Requirements

### Requirement: Remove settled tracks
Every queue row that is queued, done, or failed SHALL offer a control that
removes the track from the queue list without deleting any file from disk.
Rows that are downloading or tagging SHALL NOT offer it, and the control
SHALL be disabled while a batch is running, as the queued rows' control is
today.

#### Scenario: Remove a done track
- **WHEN** no batch is running and the user activates Remove on a done row
- **THEN** that row leaves the queue and its downloaded file stays on disk

#### Scenario: Remove a failed track
- **WHEN** no batch is running and the user activates Remove on a failed row
- **THEN** that row leaves the queue and the "Retry failed (N)" count drops by one

#### Scenario: Disabled during a batch
- **WHEN** a download batch is running
- **THEN** the remove control on queued, done, and failed rows is disabled

#### Scenario: Not offered on tracks in progress
- **WHEN** a row is downloading or tagging
- **THEN** it offers no remove control

### Requirement: Remove a group's settled tracks
While no batch is running, a group header SHALL offer a control that removes
all of that group's queued, done, and failed tracks from the queue list,
without deleting files. It SHALL NOT be offered while a batch is running or
when the group has no such track.

#### Scenario: Remove a finished album
- **WHEN** a group holds done, failed, and queued tracks and the user activates its remove control
- **THEN** all of them leave the queue, their files stay on disk, and other groups are untouched

#### Scenario: Group disappears when emptied
- **WHEN** the user removes every track of a group
- **THEN** the group no longer appears

#### Scenario: Not offered during a batch
- **WHEN** a download batch is running
- **THEN** no group offers the remove control

### Requirement: Open an album's folder
A queue group with at least one done track SHALL offer a control that opens
the folder holding that album's downloaded files in the system file manager.
When the files are in different folders, it SHALL open the nearest folder
they share.

#### Scenario: Open after downloading
- **WHEN** a group has done tracks and the user activates Open folder
- **THEN** the folder containing those tracks' files opens in the file manager

#### Scenario: No done track yet
- **WHEN** none of a group's tracks is done
- **THEN** the group offers no Open folder control

#### Scenario: Folder gone
- **WHEN** the user activates Open folder after the folder was moved or deleted
- **THEN** the status line reports that the folder no longer exists, and nothing else changes

### Requirement: Show a track's file
Each done row SHALL offer a control that opens the file's folder in the
system file manager with the file selected, or the folder alone where the
platform has no standard way to select a file.

#### Scenario: Reveal a downloaded track
- **WHEN** the user activates Show in folder on a done row
- **THEN** the file manager opens on that track's folder, with the file selected where the platform supports it

#### Scenario: File gone
- **WHEN** the user activates Show in folder after the file was moved or deleted
- **THEN** the status line reports that the file no longer exists

### Requirement: Open the download folder from Settings
The Settings screen SHALL offer, next to the download directory, a control
that opens that directory in the system file manager.

#### Scenario: Open the download root
- **WHEN** the user activates Open next to the download directory
- **THEN** that directory opens in the file manager

#### Scenario: Directory missing
- **WHEN** the download directory does not exist
- **THEN** the status line reports it rather than opening anything

## REMOVED Requirements

### Requirement: Remove a group's queued tracks
**Reason**: It required that a group's done and failed tracks stay when the
group's remove control is used. This change widens that control to every
settled track.
**Migration**: Replaced by "Remove a group's settled tracks", which keeps the
same placement and the same no-batch rule, and also removes done and failed
tracks.
