## MODIFIED Requirements

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
