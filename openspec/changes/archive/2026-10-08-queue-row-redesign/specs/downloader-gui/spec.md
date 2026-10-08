# Spec Delta

## MODIFIED Requirements

### Requirement: Download queue screen
The system SHALL display a download queue with per-item status
(Queued, Downloading with its percentage, Tagging, Done, Failed with its reason)
shown as a colored badge in Title Case, a per-item progress bar only while
that item is downloading or tagging, and overall progress. Queue rows SHALL
offer no controls; every action on tracks is offered by a group header or the
queue header. Overall progress SHALL express how far
the whole queue has advanced: every item in the queue counts toward it,
whether or not that item has started downloading. An item that has not started
counts as no progress; an item that is downloading counts by its own progress;
an item that has reached a terminal state — tagging, done, or failed — counts
as fully advanced, so overall progress reaches completion exactly when no item
is left to process. Overall progress SHALL NOT decrease while a batch runs,
except where an item's own progress is reset by a retry or by cancellation.
The system SHALL offer a control that starts downloading the queue. That
control SHALL act on items that have never been attempted, and SHALL be
offered only when acting on it would start work — that is, when at least one
such item is present, or while a batch is already running. It SHALL NOT be
offered when the queue is empty or when every item has already been attempted.
While a batch is running, the system SHALL offer a control that cancels it,
and SHALL indicate that cancellation is under way once it has been requested.
Cancelling SHALL return every item that was downloading or still waiting to
the queued state, discarding its recorded progress, and SHALL leave completed
and failed items as they are. Once a cancelled batch has stopped, the controls
that require an idle queue SHALL become available again, so the queue can then
be cleared or started afresh. When an item has failed, the system SHALL offer
a single header action that retries all failed items without re-adding them;
relaunching failed items is the responsibility of that control and not of
the start control. When the queue is non-empty, the
system SHALL offer a header control to clear the entire queue. Retry
and clear controls SHALL be available only when a download batch is not
currently in progress. When the queue is empty, the system SHALL present a
message saying so and how to add tracks, in place of the progress counter and
the overall progress bar.

#### Scenario: Live progress display
- **WHEN** downloads are in progress
- **THEN** each item shows its current status badge, each downloading item also shows its own progress bar, and the overall progress updates without freezing the UI

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
- **THEN** overall progress shows the batch as complete, while the failed items keep their Failed badges and are counted by the "Retry failed (N)" control

#### Scenario: A failed item's own bar stays empty
- **WHEN** an item is in the error state
- **THEN** its row shows no progress bar of its own, even though the item counts as settled for overall progress

#### Scenario: Start offered for never-attempted items
- **WHEN** the queue holds at least one item that has never been attempted and no batch is running
- **THEN** the queue header offers the start control, and activating it downloads those items

#### Scenario: Start withheld from an empty queue
- **WHEN** the queue is empty, whether because nothing has been added or because the queue was just cleared
- **THEN** the queue header offers no start control

#### Scenario: Start withheld once every item is done
- **WHEN** every item in the queue has downloaded successfully
- **THEN** the queue header offers no start control

#### Scenario: Start withheld when only failures remain
- **WHEN** every item in the queue has been attempted and one or more of them failed
- **THEN** the queue header offers no start control, and "Retry failed (N)" is the way to relaunch those items

#### Scenario: Start stays visible for the whole batch
- **WHEN** a download batch is in progress, including while its last items leave the queued state
- **THEN** the start control remains visible, labelled to show a batch is running, and cannot be activated

#### Scenario: Empty queue explains itself
- **WHEN** the queue is empty
- **THEN** the screen shows a message that nothing is queued and how to add tracks, and shows neither the completion counter nor the overall progress bar

#### Scenario: Cancel offered only while a batch runs
- **WHEN** a download batch is in progress
- **THEN** the queue header offers a cancel control, and it is absent whenever no batch is running

#### Scenario: Cancelling stops the batch and requeues its items
- **WHEN** the user cancels a running batch
- **THEN** the items that were downloading or waiting return to the queued state with their progress reset, while completed and failed items keep their outcome

#### Scenario: Cancellation is acknowledged immediately
- **WHEN** the user activates the cancel control and the batch has not finished stopping yet
- **THEN** the screen indicates that cancellation is under way and the control cannot be activated a second time

#### Scenario: A cancelled batch can be resumed
- **WHEN** a cancelled batch has stopped and requeued items remain
- **THEN** the start control is offered again and activating it downloads those items

#### Scenario: Clearing is possible after cancelling
- **WHEN** a cancelled batch has stopped
- **THEN** the "Clear queue" control is available again, so the queue can be emptied without waiting for downloads that are no longer running

#### Scenario: Error visibility
- **WHEN** an item fails to download
- **THEN** its row shows a Failed status badge with a message explaining the failure

#### Scenario: Relaunch a single failed track
- **WHEN** exactly one item is in the error state and no batch is currently downloading
- **THEN** the queue header offers "Retry failed (1)", which resets that item to queued and re-downloads only that track

#### Scenario: Retry all failed tracks
- **WHEN** one or more items are in the error state and no batch is currently downloading
- **THEN** the queue header exposes a "Retry failed (N)" control that re-downloads all failed items, and the error count updates as they complete

#### Scenario: Remove a queued track
- **WHEN** an item is in the queued state and no batch is currently downloading
- **THEN** its row offers no remove control, and its group header's remove control takes it out of the queue together with the group's other settled tracks

#### Scenario: Clear the entire queue
- **WHEN** the queue is non-empty and no batch is currently downloading
- **THEN** the queue header exposes a "Clear queue" control that, when activated, removes all items from the queue

#### Scenario: Remove disabled during download
- **WHEN** a download batch is in progress
- **THEN** the "Clear queue" control is unavailable, and group remove controls take out none of that batch's tracks

#### Scenario: Retry disabled during download
- **WHEN** a download batch is in progress
- **THEN** the "Retry failed" control is unavailable

#### Scenario: Rows carry no controls
- **WHEN** the queue shows rows in any state
- **THEN** no row offers a retry, remove, show-in-folder or other button

#### Scenario: Progress bar only while a track is in progress
- **WHEN** a row is queued, done or failed
- **THEN** it shows no progress bar, while keeping the same height as a row that has one so the list does not shift, and a downloading or tagging row shows one

### Requirement: Remove a group's settled tracks
A group header SHALL offer a control that removes all of that group's queued,
done, and failed tracks that are not part of the running batch from the queue
list, without deleting files. It SHALL NOT be offered when the group has no
such track.

#### Scenario: Remove a finished album
- **WHEN** a group holds done, failed, and queued tracks and the user activates its remove control
- **THEN** all of them leave the queue, their files stay on disk, and other groups are untouched

#### Scenario: Group disappears when emptied
- **WHEN** the user removes every track of a group
- **THEN** the group no longer appears

#### Scenario: Running batch's tracks are kept
- **WHEN** a download batch is running and the user removes a group holding tracks of that batch and tracks added after it started
- **THEN** only the tracks added after it started leave the queue

#### Scenario: Not offered for a group entirely in the batch
- **WHEN** a download batch is running and every settled track of a group belongs to it
- **THEN** that group does not offer the remove control

#### Scenario: Removed album forgets its collapsed state
- **WHEN** the user collapses a group, removes it, and later adds that album again
- **THEN** the new group is shown expanded

## ADDED Requirements

### Requirement: Delivered quality per album group
A queue group header SHALL show the delivered quality shared by most of its done tracks, once at least one track is done. A done row SHALL show its own delivered quality only when it differs from its group's, so a track the service delivered at another tier stays visible on its row while its group is expanded.

#### Scenario: Album delivered at one quality
- **WHEN** every done track of a group was delivered as FLAC 24/96
- **THEN** the group header shows FLAC 24/96 once and no row shows a quality

#### Scenario: One track downgraded
- **WHEN** a group's done tracks were delivered as FLAC 24/96 except one delivered as FLAC 16/44.1
- **THEN** the header shows FLAC 24/96 and only the downgraded track's row shows FLAC 16/44.1

#### Scenario: Nothing done yet
- **WHEN** none of a group's tracks is done
- **THEN** the group header shows no quality

## REMOVED Requirements

### Requirement: Remove settled tracks
**Reason**: Queue rows no longer carry controls; per-track removal is replaced by the group header's remove control and "Clear queue".
**Migration**: Use the group's Remove, which takes out the group's queued, done and failed tracks outside the running batch. The collapsed-state scenario moves to "Remove a group's settled tracks".

### Requirement: Show a track's file
**Reason**: Queue rows no longer carry controls.
**Migration**: Use the group's "Open folder", which opens the folder holding the album's downloaded files.
