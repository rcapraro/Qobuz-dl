# downloader-gui Specification

## Purpose
TBD - created by archiving change add-qobuz-downloader. Update Purpose after archive.
## Requirements
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

### Requirement: Settings screen
The system SHALL provide a settings screen exposing Qobuz sign-in via a
`user_auth_token`, `app_id`/`app_secret`, download-directory picker, quality
selector, cover-art toggle, folder/track template fields with a live preview, and
a bounded numeric concurrency control that accepts only values in the range 1–16.
The account section SHALL NOT offer email/password login (unsupported by Qobuz for
partner/bundled accounts) and SHALL explain how to obtain the token from the Qobuz
web player.

#### Scenario: Configure and save
- **WHEN** the user fills in credentials and preferences and saves
- **THEN** the settings are persisted and the app reflects the signed-in state

#### Scenario: Token sign-in
- **WHEN** the user pastes a valid `user_auth_token` and presses Sign in
- **THEN** the token is validated, stored in the OS keyring, and the account is reported as signed in

#### Scenario: Guidance for obtaining the token
- **WHEN** the user opens the account help panel
- **THEN** the app explains that sign-in uses a `user_auth_token` and how to copy it from the web player's developer tools

#### Scenario: Concurrency is bounded
- **WHEN** the user adjusts the concurrency control
- **THEN** the value is constrained to the range 1–16 and cannot be set to a non-numeric or out-of-range value

### Requirement: Search and add screen
The system SHALL provide a screen to search the catalog and to paste a Qobuz
URL/ID, and to add resulting albums/tracks/playlists to the download queue.
Search results SHALL be grouped by type (albums, tracks, artists) in distinct card
containers, each result offering a per-row add control. Album results SHALL
display the album cover as a thumbnail, loaded asynchronously without blocking
the results list.

#### Scenario: Add search result to queue
- **WHEN** the user selects an album from search results and clicks add
- **THEN** the album's tracks are enqueued for download

#### Scenario: Add via pasted URL
- **WHEN** the user pastes a valid Qobuz URL and clicks add
- **THEN** the resolved item is enqueued for download

#### Scenario: Results grouped by type
- **WHEN** search results are displayed
- **THEN** albums, tracks, and artists appear in separate card sections, each row exposing its own add control

#### Scenario: Album cover thumbnails
- **WHEN** album results are displayed
- **THEN** each album row shows its cover thumbnail once loaded, with a placeholder shown while loading or when no cover is available, and the list remains usable before thumbnails finish loading

### Requirement: Download queue screen
The system SHALL display a download queue with per-item status
(queued/downloading/tagging/done/error) shown as a colored badge, per-item
progress bars, and overall progress. When an item has failed, the system SHALL
offer a way to relaunch that item's download without re-adding it, both as a
per-item control and as a single action that retries all failed items. Retry
controls SHALL be available only when a download batch is not currently in
progress.

#### Scenario: Live progress display
- **WHEN** downloads are in progress
- **THEN** each item shows its current status badge and progress and the overall progress updates without freezing the UI

#### Scenario: Error visibility
- **WHEN** an item fails to download
- **THEN** its row shows an error status badge with a message explaining the failure

#### Scenario: Relaunch a single failed track
- **WHEN** an item is in the error state and no batch is currently downloading
- **THEN** its row exposes a Retry control that, when activated, resets the item to queued and re-downloads only that track

#### Scenario: Retry all failed tracks
- **WHEN** one or more items are in the error state and no batch is currently downloading
- **THEN** the queue header exposes a "Retry failed (N)" control that re-downloads all failed items, and the error count updates as they complete

#### Scenario: Retry disabled during download
- **WHEN** a download batch is in progress
- **THEN** the per-item Retry controls and the "Retry failed" control are disabled

### Requirement: Auto-detect credentials control
The Settings screen SHALL provide a control that triggers automatic discovery
of the Qobuz `app_id` and `app_secret`, populates the credential fields with the
result, and communicates progress and outcome to the user.

#### Scenario: User triggers auto-detection
- **WHEN** the user activates the auto-detect control in Settings
- **THEN** the app runs discovery without blocking the UI and indicates that
  detection is in progress

#### Scenario: Fields populated on success
- **WHEN** discovery succeeds
- **THEN** the `app_id` and `app_secret` fields are filled with the discovered
  values and a success message is shown

#### Scenario: Error surfaced on failure
- **WHEN** discovery fails
- **THEN** the app shows a clear error message and the credential fields keep
  their previous contents so the user can still enter values manually

