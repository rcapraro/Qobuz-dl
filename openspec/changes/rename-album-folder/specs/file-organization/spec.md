# Spec Delta

## ADDED Requirements

### Requirement: Rename an album folder
The system SHALL rename a downloaded album's folder, the nearest folder holding all of its done files, to a name the user confirms, keeping it in the same parent directory. The name SHALL be sanitized like any rendered path segment, and SHALL NOT contain path separators.

#### Scenario: Folder renamed
- **WHEN** an album's files are in `Qobuz/Miles Davis - Kind of Blue (1959) [FLAC] [24B-96kHz]` and the user confirms the name `Miles Davis - Kind of Blue (1959)`
- **THEN** the folder is now `Qobuz/Miles Davis - Kind of Blue (1959)` with the same contents

#### Scenario: Separators sanitized
- **WHEN** the user confirms the name `AC/DC - Back in Black`
- **THEN** the folder is renamed to one sanitized name in the same parent directory, not moved into a subfolder

#### Scenario: Multi-disc album
- **WHEN** an album's files are in `Album/Disc 1` and `Album/Disc 2` and the user renames it
- **THEN** `Album` is renamed and both disc subfolders move with it

### Requirement: Folder rename safety
The system SHALL refuse a rename, change nothing, and say why when: a different file or folder already exists at the target; the folder is the download directory or lies outside it; or the folder holds done files of another album in the queue. Renaming that only changes letter case SHALL be allowed.

#### Scenario: Target exists
- **WHEN** a folder with the confirmed name already exists in the same parent
- **THEN** nothing is renamed and the status line reports that the name is taken

#### Scenario: Album directly in the download directory
- **WHEN** the folder template is empty, so an album's files sit directly in the download directory
- **THEN** renaming is refused and the download directory keeps its name

#### Scenario: Folder shared with another album
- **WHEN** another queued album has done files in the same folder
- **THEN** renaming is refused, since it would move that album's files too

#### Scenario: Case-only rename
- **WHEN** the user renames `kind of blue` to `Kind of Blue` on a case-insensitive filesystem
- **THEN** the folder's name changes case

### Requirement: Queue follows a renamed folder
After a successful rename, every queue item whose file was inside the renamed folder SHALL refer to its file at the new location, so opening the album's folder and editing its tags keep working.

#### Scenario: Open folder after renaming
- **WHEN** the user renames an album's folder and then activates Open folder on its group
- **THEN** the renamed folder opens

### Requirement: Rename template
The system SHALL provide a rename template, separate from the download folder template, that suggests an album folder's new name. It SHALL use the same placeholders and modifiers as the path templates, with values taken from the album's files: tag values from its first track in disc and track order, and `container`, `bit_depth` and `sampling_rate` from that file's audio format.

#### Scenario: Suggested from edited tags
- **WHEN** the album title tag was edited to "Kind of Blue (Remaster)" and the rename template is `{albumartist} - {album} ({year})`
- **THEN** the suggested name is `Miles Davis - Kind of Blue (Remaster) (1959)`

#### Scenario: Template unchanged by downloads
- **WHEN** the user changes the rename template
- **THEN** download folder and track paths are unaffected
