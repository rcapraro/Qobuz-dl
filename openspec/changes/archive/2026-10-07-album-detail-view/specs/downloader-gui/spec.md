# Spec Delta

## ADDED Requirements

### Requirement: Open an album's detail from search
Activating an album search result, other than its add control, SHALL open
that album's detail within the Search/Add screen in place of the results
list. The detail SHALL show the album's cover, title, artist, year, label,
genre, track count, total duration, and a Hi-Res badge when the album is
hi-res. The search field SHALL remain visible.

#### Scenario: Open from a result
- **WHEN** the user activates an album result's title or cover
- **THEN** the results list is replaced by that album's detail, and nothing is added to the queue

#### Scenario: Row add control still adds the whole album
- **WHEN** the user activates an album result's add control
- **THEN** every track of the album is enqueued without opening the detail

#### Scenario: Loading state
- **WHEN** the detail is opened and the album's tracks have not arrived yet
- **THEN** the detail shows the title, artist, and cover already known from the result, with a loading indicator in place of the track list

#### Scenario: Load failure
- **WHEN** fetching the album fails
- **THEN** the detail shows the error in place of the track list with a control to retry, and the Back control remains available

### Requirement: Album track list
The album detail SHALL list every track with its number, title, duration,
and a Hi-Res badge when hi-res, and SHALL show a track's performer when it
differs from the album artist. On an album with more than one disc, tracks
SHALL be grouped under a heading per disc.

#### Scenario: Track rows
- **WHEN** the album's tracks are shown
- **THEN** each row shows the track number, title, and duration as minutes and seconds

#### Scenario: Guest performer shown
- **WHEN** a track's performer differs from the album artist
- **THEN** the row shows that performer under the title, and rows whose performer matches the album artist show none

#### Scenario: Multi-disc grouping
- **WHEN** the album has more than one disc
- **THEN** the tracks are grouped under a "Disc N" heading for each disc, in disc order

### Requirement: Select album tracks to add
Each track in the album detail SHALL have a checkbox, all checked when the
detail opens. The detail SHALL offer controls to check all and to uncheck all
tracks, and one primary control that adds the checked tracks to the queue,
labelled with how many are checked. That control SHALL be disabled when no
track is checked.

#### Scenario: Add a subset
- **WHEN** the user unchecks some tracks and activates the add control
- **THEN** only the checked tracks are enqueued, in album order, and tracks already in the queue are not added twice

#### Scenario: Count follows the selection
- **WHEN** the user checks or unchecks a track
- **THEN** the add control's label updates to the number of checked tracks

#### Scenario: Select none disables adding
- **WHEN** the user activates "Select none"
- **THEN** every track is unchecked and the add control is disabled until a track is checked again

#### Scenario: Select all
- **WHEN** the user activates "Select all"
- **THEN** every track is checked

### Requirement: Return from an album's detail
The album detail SHALL offer a Back control that returns to the search results
it was opened from, unchanged and at the scroll position they had. Submitting
a new search or a URL from the field SHALL also leave the detail.

#### Scenario: Back restores results
- **WHEN** the user scrolls the results, opens an album, and activates Back
- **THEN** the same results are shown at the scroll position they had before the album was opened

#### Scenario: New search leaves the detail
- **WHEN** the user submits a new search from the field while an album's detail is open
- **THEN** the detail closes and the new search's results are shown

#### Scenario: Late album response ignored
- **WHEN** the user activates Back before the album has loaded, and the album's response then arrives
- **THEN** the response is discarded and the results stay shown
