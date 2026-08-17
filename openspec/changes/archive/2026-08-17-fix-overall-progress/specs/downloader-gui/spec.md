## MODIFIED Requirements

### Requirement: Download queue screen
The system SHALL display a download queue with per-item status
(queued/downloading/tagging/done/error) shown as a colored badge, per-item
progress bars, and overall progress. Overall progress SHALL express how far
the whole queue has advanced: every item in the queue counts toward it,
whether or not that item has started downloading. An item that has not started
counts as no progress; an item that is downloading counts by its own progress;
an item that has reached a terminal state — tagging, done, or failed — counts
as fully advanced, so overall progress reaches completion exactly when no item
is left to process. Overall progress SHALL NOT decrease while a batch runs,
except where an item's own progress is reset by a retry. When an item has
failed, the system SHALL offer a way to relaunch that item's download without
re-adding it, both as a per-item control and as a single action that retries
all failed items. When an item is still queued, the system SHALL offer a
per-item control to remove it from the queue. When the queue is non-empty, the
system SHALL offer a header control to clear the entire queue. Retry, remove,
and clear controls SHALL be available only when a download batch is not
currently in progress.

#### Scenario: Live progress display
- **WHEN** downloads are in progress
- **THEN** each item shows its current status badge and progress and the overall progress updates without freezing the UI

#### Scenario: Pending items hold overall progress back
- **WHEN** some items have finished but others are still queued and have never started downloading
- **THEN** overall progress is below completion and reflects the finished share of the entire queue, consistent with the "N/M complete" counter shown in the header

#### Scenario: Overall progress does not move backwards as later items start
- **WHEN** a batch downloads more items than it processes concurrently, so items start in successive waves
- **THEN** overall progress rises steadily across the whole batch and does not drop when a new wave of items begins downloading

#### Scenario: Items completed without transferring bytes still count
- **WHEN** an item completes without any bytes being transferred, because its destination file already exists and the download is skipped
- **THEN** that item counts as fully advanced in overall progress

#### Scenario: Overall progress completes despite failures
- **WHEN** every item in the queue has reached a terminal state and at least one of them failed
- **THEN** overall progress shows the batch as complete, while the failed items keep their error badges and are counted by the "Retry failed (N)" control

#### Scenario: A failed item's own bar stays empty
- **WHEN** an item is in the error state
- **THEN** that item's own progress bar shows no progress, even though the item counts as settled for overall progress

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

#### Scenario: Clear the entire queue
- **WHEN** the queue is non-empty and no batch is currently downloading
- **THEN** the queue header exposes a "Clear queue" control that, when activated, removes all items from the queue

#### Scenario: Remove disabled during download
- **WHEN** a download batch is in progress
- **THEN** the per-item Remove controls and the "Clear queue" control are disabled

#### Scenario: Retry disabled during download
- **WHEN** a download batch is in progress
- **THEN** the per-item Retry controls and the "Retry failed" control are disabled
