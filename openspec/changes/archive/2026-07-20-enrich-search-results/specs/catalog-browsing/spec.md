## MODIFIED Requirements

### Requirement: Search the catalog
The system SHALL allow the user to search the Qobuz catalog by query and view
results grouped as albums, tracks, and artists. Album and track results SHALL
expose a hi-res quality indicator derived from the Qobuz search response's
hi-res flags, defaulting to non-hi-res when the response omits them.

#### Scenario: Search returns results
- **WHEN** the user enters a non-empty search query while signed in
- **THEN** the system calls the Qobuz search endpoint and displays matching albums, tracks, and artists

#### Scenario: Empty results
- **WHEN** a search query matches nothing
- **THEN** the system displays a "no results" state rather than an error

#### Scenario: Hi-res quality is surfaced
- **WHEN** the search response marks an album or track as hi-res-streamable (or hi-res)
- **THEN** that result is exposed as hi-res

#### Scenario: Missing hi-res flags default to non-hi-res
- **WHEN** the search response omits the hi-res flags for a result
- **THEN** that result is treated as non-hi-res rather than causing an error
