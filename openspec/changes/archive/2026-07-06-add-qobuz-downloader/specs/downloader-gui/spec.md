## ADDED Requirements

### Requirement: Settings screen
The system SHALL provide a settings screen exposing Qobuz login (email/password
or raw token), `app_id`/`app_secret`, download-directory picker, quality
selector, cover-art toggle, and folder/track template fields with a live
preview.

#### Scenario: Configure and save
- **WHEN** the user fills in credentials and preferences and saves
- **THEN** the settings are persisted and the app reflects the signed-in state

### Requirement: Search and add screen
The system SHALL provide a screen to search the catalog and to paste a Qobuz
URL/ID, and to add resulting albums/tracks/playlists to the download queue.

#### Scenario: Add search result to queue
- **WHEN** the user selects an album from search results and clicks add
- **THEN** the album's tracks are enqueued for download

#### Scenario: Add via pasted URL
- **WHEN** the user pastes a valid Qobuz URL and clicks add
- **THEN** the resolved item is enqueued for download

### Requirement: Download queue screen
The system SHALL display a download queue with per-item status
(queued/downloading/tagging/done/error), per-item progress bars, and overall
progress.

#### Scenario: Live progress display
- **WHEN** downloads are in progress
- **THEN** each item shows its current status and progress and the overall progress updates without freezing the UI

#### Scenario: Error visibility
- **WHEN** an item fails to download
- **THEN** its row shows an error status with a message explaining the failure
