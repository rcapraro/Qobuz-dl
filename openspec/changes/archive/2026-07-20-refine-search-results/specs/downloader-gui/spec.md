## MODIFIED Requirements

### Requirement: Search and add screen
The system SHALL provide a screen to search the catalog and to paste a Qobuz
URL/ID, and to add resulting albums/tracks/playlists to the download queue.
Search results SHALL be grouped by type (albums, tracks) in distinct card
containers, each result offering a per-row add control; artists SHALL NOT be
shown. Album and track result rows SHALL display the title and the artist as
separate, distinctly styled elements, and SHALL show a "Hi-Res" badge when the
item is hi-res. Album and track result rows SHALL display a cover thumbnail
(the track's album cover for tracks), loaded asynchronously without blocking the
results list.

#### Scenario: Add search result to queue
- **WHEN** the user selects an album from search results and clicks add
- **THEN** the album's tracks are enqueued for download

#### Scenario: Add via pasted URL
- **WHEN** the user pastes a valid Qobuz URL and clicks add
- **THEN** the resolved item is enqueued for download

#### Scenario: Results grouped by type
- **WHEN** search results are displayed
- **THEN** albums and tracks appear in separate card sections, each row exposing its own add control, and no artists section is shown

#### Scenario: Title and artist shown separately
- **WHEN** album or track results are displayed
- **THEN** each row shows the title emphasised with the artist on a separate secondary line

#### Scenario: Hi-Res badge on hi-res results
- **WHEN** an album or track result is hi-res
- **THEN** the row shows a "Hi-Res" badge, and non-hi-res rows show no such badge

#### Scenario: Album and track cover thumbnails
- **WHEN** album or track results are displayed
- **THEN** each row shows its cover thumbnail once loaded, with a placeholder shown while loading or when no cover is available, and the list remains usable before thumbnails finish loading
