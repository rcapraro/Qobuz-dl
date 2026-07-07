## ADDED Requirements

### Requirement: Tabbed navigation

The system SHALL present the Settings, Search/Add, and Queue sections as a tab bar,
with exactly one section visible at a time and the active tab visually indicated.
Global controls (theme toggle and sign-in indicator) SHALL remain visible
independent of the selected tab.

#### Scenario: Switch section via tab

- **WHEN** the user selects a different tab
- **THEN** the corresponding section is shown, the previous section is hidden, and the selected tab is marked active

#### Scenario: Global controls persist across tabs

- **WHEN** the user switches between any tabs
- **THEN** the theme toggle and the signed-in/out indicator remain visible

## MODIFIED Requirements

### Requirement: Settings screen
The system SHALL provide a settings screen exposing Qobuz login (email/password
or raw token), `app_id`/`app_secret`, download-directory picker, quality
selector, cover-art toggle, folder/track template fields with a live preview, and
a bounded numeric concurrency control that accepts only values in the range 1–16.

#### Scenario: Configure and save
- **WHEN** the user fills in credentials and preferences and saves
- **THEN** the settings are persisted and the app reflects the signed-in state

#### Scenario: Concurrency is bounded
- **WHEN** the user adjusts the concurrency control
- **THEN** the value is constrained to the range 1–16 and cannot be set to a non-numeric or out-of-range value

### Requirement: Search and add screen
The system SHALL provide a screen to search the catalog and to paste a Qobuz
URL/ID, and to add resulting albums/tracks/playlists to the download queue.
Search results SHALL be grouped by type (albums, tracks, artists) in distinct card
containers, each result offering a per-row add control.

#### Scenario: Add search result to queue
- **WHEN** the user selects an album from search results and clicks add
- **THEN** the album's tracks are enqueued for download

#### Scenario: Add via pasted URL
- **WHEN** the user pastes a valid Qobuz URL and clicks add
- **THEN** the resolved item is enqueued for download

#### Scenario: Results grouped by type
- **WHEN** search results are displayed
- **THEN** albums, tracks, and artists appear in separate card sections, each row exposing its own add control

### Requirement: Download queue screen
The system SHALL display a download queue with per-item status
(queued/downloading/tagging/done/error) shown as a colored badge, per-item
progress bars, and overall progress.

#### Scenario: Live progress display
- **WHEN** downloads are in progress
- **THEN** each item shows its current status badge and progress and the overall progress updates without freezing the UI

#### Scenario: Error visibility
- **WHEN** an item fails to download
- **THEN** its row shows an error status badge with a message explaining the failure
