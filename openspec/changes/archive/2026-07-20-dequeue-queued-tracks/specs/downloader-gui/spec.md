## MODIFIED Requirements

### Requirement: Download queue screen
The system SHALL display a download queue with per-item status
(queued/downloading/tagging/done/error) shown as a colored badge, per-item
progress bars, and overall progress. When an item has failed, the system SHALL
offer a way to relaunch that item's download without re-adding it, both as a
per-item control and as a single action that retries all failed items. When an
item is still queued, the system SHALL offer a per-item control to remove it
from the queue. Retry and remove controls SHALL be available only when a
download batch is not currently in progress.

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

#### Scenario: Remove a queued track
- **WHEN** an item is in the queued state and no batch is currently downloading
- **THEN** its row exposes a Remove control that, when activated, removes that item from the queue while leaving other items untouched

#### Scenario: Remove disabled during download
- **WHEN** a download batch is in progress
- **THEN** the per-item Remove controls are disabled

#### Scenario: Retry disabled during download
- **WHEN** a download batch is in progress
- **THEN** the per-item Retry controls and the "Retry failed" control are disabled
