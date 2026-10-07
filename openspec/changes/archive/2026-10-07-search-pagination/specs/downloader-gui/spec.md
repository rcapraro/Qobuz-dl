# Spec Delta

## ADDED Requirements

### Requirement: Show more search results
Each search results section SHALL show how many results it holds and, when
known, the total available. While more results of that type remain, the
section SHALL offer a "Show more" control that appends the next page to that
section only. A result already shown SHALL NOT appear twice, and a page that
arrives after the user started a different search SHALL be discarded.

#### Scenario: Count and total shown
- **WHEN** the albums section holds 25 results and the total reported is 140
- **THEN** the section shows that 25 of 140 albums are displayed

#### Scenario: Show more appends to one section
- **WHEN** the user activates "Show more" in the albums section
- **THEN** the next page of albums is appended below the existing albums, and the tracks section is unchanged

#### Scenario: No control once everything is shown
- **WHEN** a section displays every result of the reported total
- **THEN** that section offers no "Show more" control

#### Scenario: Unknown total
- **WHEN** the response gave no total and the last page received was full
- **THEN** the section offers "Show more", and stops offering it after a page arrives with fewer results than the page size

#### Scenario: Loading state
- **WHEN** a "Show more" request is in flight for a section
- **THEN** that section's control indicates loading and cannot be activated again until the page arrives or fails

#### Scenario: Duplicates are skipped
- **WHEN** a later page includes a result already displayed in that section
- **THEN** the result is shown once

#### Scenario: Stale page discarded
- **WHEN** the user submits a new search while a "Show more" request for the previous search is in flight
- **THEN** the late page is discarded and does not appear in the new results

#### Scenario: Show more fails
- **WHEN** a "Show more" request fails
- **THEN** the existing results stay as they were, an error is reported, and the control can be activated again
