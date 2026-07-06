## ADDED Requirements

### Requirement: Configurable path templates
The system SHALL render the destination folder and file name from user-defined
templates supporting placeholders including `{albumartist}`, `{artist}`,
`{title}`, `{album}`, `{year}`, `{tracknumber}` (with zero-padding, e.g.
`{tracknumber:02}`), `{bit_depth}`, `{sampling_rate}`, `{container}`, and
`{explicit}`.

#### Scenario: Template rendered
- **WHEN** the folder template is `{albumartist} - {album} ({year})` and the track template is `{tracknumber:02}. {title}`
- **THEN** a track is written to a matching path such as `Artist - Album (2020)/01. Song.flac`

#### Scenario: Multi-disc handling
- **WHEN** an album has more than one disc
- **THEN** the system organizes tracks into per-disc subfolders

### Requirement: Path sanitization
The system SHALL sanitize each rendered path segment by removing or replacing
characters illegal on target filesystems (`/ \ : * ? " < > |`) and trimming
overly long segments.

#### Scenario: Illegal characters removed
- **WHEN** a track title contains characters like `:` or `?`
- **THEN** the rendered path segment has those characters stripped or replaced so the file writes successfully on macOS, Windows, and Linux

### Requirement: Choose download directory
The system SHALL let the user select the base download directory via a native
directory picker.

#### Scenario: Directory selected
- **WHEN** the user picks a download directory in settings
- **THEN** subsequent downloads are written under that directory using the configured templates

### Requirement: Write audio tags
The system SHALL write metadata tags to downloaded files, including title,
artist, album, album artist, track number, disc number, year, genre, ISRC,
composer, and explicit flag, for FLAC, MP3, and M4A containers.

#### Scenario: Tags written
- **WHEN** a track finishes downloading
- **THEN** the file contains the correct title, artist, album, track/disc numbers, and year tags

### Requirement: Embed cover art
The system SHALL, when embedding is enabled, fetch the album cover image and
embed it into each downloaded audio file.

#### Scenario: Cover embedded
- **WHEN** cover-art embedding is enabled and an album has a cover image
- **THEN** each downloaded file includes the embedded cover art

#### Scenario: Embedding disabled
- **WHEN** cover-art embedding is disabled
- **THEN** downloaded files contain no embedded cover art and downloading still succeeds
