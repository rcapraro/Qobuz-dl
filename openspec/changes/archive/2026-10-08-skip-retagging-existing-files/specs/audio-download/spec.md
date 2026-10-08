# Spec Delta

## ADDED Requirements

### Requirement: Downloaded files are complete when they appear
The system SHALL write a downloaded track's tags, and embed its cover art when cover embedding is enabled and the art is available, before the file appears at its destination path. If tagging fails, or the batch is cancelled before the file has been moved to its destination, the system SHALL NOT create the destination file and SHALL remove the partial file, so a later attempt downloads the track again.

#### Scenario: New file tagged before it appears
- **WHEN** a track's destination does not exist and it is downloaded
- **THEN** the file at its destination already has its tags and, when cover embedding is enabled, its cover art

#### Scenario: Tagging failure leaves no file
- **WHEN** a track's bytes have been downloaded but writing its tags fails
- **THEN** the track is reported as failed, no file exists at its destination, and retrying it downloads it again

#### Scenario: Cancelled while fetching cover art
- **WHEN** a batch is cancelled while a downloaded track is waiting for its album's cover art
- **THEN** the track is reported as cancelled, no file exists at its destination, and its partial file is removed

### Requirement: Existing files are left untouched
When a track's destination file already exists at the time its download begins, the system SHALL complete the track without writing to that file: it SHALL NOT write tags, SHALL NOT embed cover art, and SHALL NOT fetch the album's cover art for that track. The track SHALL still be reported as complete with its delivered quality and its file path.

#### Scenario: Re-queuing a downloaded album
- **WHEN** an album whose files are all on disk is queued and downloaded again
- **THEN** every track completes, and none of the files' contents or modification times change

#### Scenario: User-edited tags survive
- **WHEN** the user has edited the tags of a downloaded file and its track is downloaded again
- **THEN** the file keeps the user's tags

#### Scenario: Skipped track still reports its outcome
- **WHEN** a track is completed because its file already exists
- **THEN** it is reported as done with its delivered quality and file path, so the queue can show it and open its folder
