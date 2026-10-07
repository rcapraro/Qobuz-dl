# Spec Delta

## ADDED Requirements

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
