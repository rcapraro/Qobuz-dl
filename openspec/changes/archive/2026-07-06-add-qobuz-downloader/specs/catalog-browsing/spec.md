## ADDED Requirements

### Requirement: Search the catalog
The system SHALL allow the user to search the Qobuz catalog by query and view
results grouped as albums, tracks, and artists.

#### Scenario: Search returns results
- **WHEN** the user enters a non-empty search query while signed in
- **THEN** the system calls the Qobuz search endpoint and displays matching albums, tracks, and artists

#### Scenario: Empty results
- **WHEN** a search query matches nothing
- **THEN** the system displays a "no results" state rather than an error

### Requirement: Resolve URLs and IDs
The system SHALL parse a pasted Qobuz URL (e.g. `open.qobuz.com/...`,
`play.qobuz.com/...`) or a bare numeric ID into a typed reference (album, track,
or playlist) and fetch its metadata.

#### Scenario: Album URL resolved
- **WHEN** the user pastes a Qobuz album URL
- **THEN** the system extracts the album ID, fetches album metadata, and lists its tracks

#### Scenario: Playlist paginated
- **WHEN** the user resolves a playlist with more than 500 tracks
- **THEN** the system paginates via increasing offsets and returns the complete track list

#### Scenario: Unrecognized input
- **WHEN** the user pastes text that is neither a valid Qobuz URL nor a numeric ID
- **THEN** the system reports that the input could not be recognized

### Requirement: Fetch item metadata
The system SHALL fetch album, track, artist, and playlist metadata needed for
downloading and tagging (titles, artists, track/disc numbers, year, cover art
URL, ISRC, container/quality availability).

#### Scenario: Metadata available for tagging
- **WHEN** the user selects an album to download
- **THEN** the system has retrieved per-track metadata sufficient to name files and write tags
