# catalog-browsing Specification

## Purpose
TBD - created by archiving change add-qobuz-downloader. Update Purpose after archive.
## Requirements
### Requirement: Search the catalog
The system SHALL allow the user to search the Qobuz catalog by query and view
results grouped as albums and tracks. Album and track results SHALL expose a
hi-res quality indicator derived from the Qobuz search response's hi-res flags,
defaulting to non-hi-res when the response omits them. Track results SHALL
expose a preview image derived from the track's album cover when available.

#### Scenario: Search returns results
- **WHEN** the user enters a non-empty search query while signed in
- **THEN** the system calls the Qobuz search endpoint and displays matching albums and tracks

#### Scenario: Artists not surfaced
- **WHEN** the search response includes artist matches
- **THEN** the system does not present artists among the results

#### Scenario: Empty results
- **WHEN** a search query matches nothing
- **THEN** the system displays a "no results" state rather than an error

#### Scenario: Hi-res quality is surfaced
- **WHEN** the search response marks an album or track as hi-res-streamable (or hi-res)
- **THEN** that result is exposed as hi-res

#### Scenario: Missing hi-res flags default to non-hi-res
- **WHEN** the search response omits the hi-res flags for a result
- **THEN** that result is treated as non-hi-res rather than causing an error

#### Scenario: Track preview image is surfaced
- **WHEN** a track result carries an album cover in the search response
- **THEN** that cover is exposed as the track's preview image, and a track without one is exposed with no preview image rather than causing an error

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

### Requirement: Paged search results
The system SHALL fetch album and track search results as independent pages,
each request naming its result type, query, page size, and starting offset.
Each page SHALL report the total number of matches for its type when the
Qobuz response provides one.

#### Scenario: First pages of a search
- **WHEN** the user searches for a non-empty query
- **THEN** the system requests the first page of album results and the first page of track results for that query

#### Scenario: Next page from an offset
- **WHEN** a further page of one result type is requested after N results of that type were received
- **THEN** the system requests that type only, starting at offset N, and leaves the other type's results untouched

#### Scenario: Total reported
- **WHEN** a page response includes the total number of matches
- **THEN** that total is exposed with the page

#### Scenario: Total missing
- **WHEN** a page response omits the total
- **THEN** the page is still returned, with no total, rather than causing an error
