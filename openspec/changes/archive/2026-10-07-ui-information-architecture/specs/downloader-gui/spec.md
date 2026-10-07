# Spec Delta

## MODIFIED Requirements

### Requirement: Tabbed navigation

The system SHALL present the Search/Add, Queue, and Settings sections as a tab
bar, in that order, with exactly one section visible at a time and the active
tab visually indicated. The application SHALL open on the Search/Add section.
While the queue holds tracks still to process (queued, downloading, or
tagging), the Queue tab label SHALL show their count; otherwise it SHALL show
no count. Global controls (theme toggle and sign-in indicator) SHALL remain
visible independent of the selected tab.

#### Scenario: Switch section via tab

- **WHEN** the user selects a different tab
- **THEN** the corresponding section is shown, the previous section is hidden, and the selected tab is marked active

#### Scenario: Global controls persist across tabs

- **WHEN** the user switches between any tabs
- **THEN** the theme toggle and the signed-in/out indicator remain visible

#### Scenario: Tab order

- **WHEN** the tab bar is displayed
- **THEN** the tabs appear in the order Search/Add, Queue, Settings

#### Scenario: Launch on Search

- **WHEN** the application starts
- **THEN** the Search/Add section is the active tab

#### Scenario: Queue tab counts remaining tracks

- **WHEN** the queue holds tracks that are queued, downloading, or tagging
- **THEN** the Queue tab label shows the number of those tracks, and the number excludes tracks that are done or failed

#### Scenario: Queue tab without remaining tracks

- **WHEN** the queue is empty, or every track in it is done or failed
- **THEN** the Queue tab label shows no count

### Requirement: Search and add screen
The system SHALL provide a screen with a single input field that both searches
the catalog and accepts a Qobuz URL/ID, and SHALL let the user add resulting
albums/tracks/playlists to the download queue. Submitting a Qobuz URL SHALL add
the referenced item; submitting any other text SHALL search the catalog. When
submitted text is not a URL but is also a valid bare Qobuz ID, the system SHALL
search and additionally offer an explicit control to add that ID, rather than
adding it without asking. Search results SHALL be grouped by type (albums,
tracks) in distinct card containers, each result offering a per-row add
control; artists SHALL NOT be shown. Album and track result rows SHALL display
the title and the artist as separate, distinctly styled elements, and SHALL
show a "Hi-Res" badge when the item is hi-res. Album and track result rows
SHALL display a cover thumbnail (the track's album cover for tracks), loaded
asynchronously without blocking the results list.

#### Scenario: Add search result to queue
- **WHEN** the user selects an album from search results and clicks add
- **THEN** the album's tracks are enqueued for download

#### Scenario: Add via pasted URL
- **WHEN** the user pastes a valid Qobuz URL into the field and submits it
- **THEN** the resolved item is enqueued for download and no search is run

#### Scenario: Text is searched
- **WHEN** the user submits text that is not a Qobuz URL
- **THEN** the system searches the catalog for that text

#### Scenario: Bare ID offered, not assumed
- **WHEN** the user submits text that is not a URL but is a valid bare Qobuz ID, such as a single word or number
- **THEN** the system searches for that text and also shows a control to add it as an ID, and nothing is enqueued unless the user activates that control

#### Scenario: Unrecognised URL
- **WHEN** the user submits a URL that is not a recognisable Qobuz album, track, or playlist link
- **THEN** the system reports an error and does not search for the URL text

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

## ADDED Requirements

### Requirement: Setup prompt on the Search screen
While app credentials are missing or no account is signed in, the Search/Add
screen SHALL show a prompt stating what is missing and offering a control that
opens Settings, in place of the search results area. The search field SHALL
remain visible.

#### Scenario: Missing credentials
- **WHEN** the user opens the Search/Add screen and `app_id` or `app_secret` is not set
- **THEN** the screen shows a prompt that app credentials are needed, with a control that opens Settings

#### Scenario: Signed out
- **WHEN** app credentials are set but no account is signed in
- **THEN** the screen shows a prompt that sign-in is needed, with a control that opens Settings

#### Scenario: Setup complete
- **WHEN** app credentials are set and an account is signed in
- **THEN** no setup prompt is shown

### Requirement: Typed status messages
The status area SHALL classify each message as info, progress, success, or
error, and SHALL render each kind with its own accent color and icon. An error
message SHALL remain until another message replaces it or the user dismisses
it.

#### Scenario: Error is distinguishable
- **WHEN** an operation fails and reports a message
- **THEN** the status area shows it with the error accent and icon, distinct from success and progress messages

#### Scenario: Progress then outcome
- **WHEN** a long operation starts and later completes
- **THEN** the status area first shows a progress message and then replaces it with a success or error message

#### Scenario: Dismiss an error
- **WHEN** an error message is shown and the user activates its dismiss control
- **THEN** the error is cleared from the status area
