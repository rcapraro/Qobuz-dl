# downloader-gui Specification

## Purpose
TBD - created by archiving change add-qobuz-downloader. Update Purpose after archive.

## Requirements

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
a way to relaunch that item's download without re-adding it, both as a
per-item control and as a single action that retries all failed items;
relaunching failed items is the responsibility of those controls and not of
the start control. When an item is still queued, the system SHALL offer a
per-item control to remove it from the queue. When the queue is non-empty, the
system SHALL offer a header control to clear the entire queue. Retry, remove,
and clear controls SHALL be available only when a download batch is not
currently in progress. When the queue is empty, the system SHALL present a
message saying so and how to add tracks, in place of the progress counter and
the overall progress bar.

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

### Requirement: Account card shows stored-token status
The Settings screen's Account card SHALL display the stored-token status line
(token saved with masked preview and session origin, or no token saved),
serving as the detailed counterpart to the header's at-a-glance signed-in
indicator.

#### Scenario: Status line with a saved token
- **WHEN** the user opens Settings while a token is stored
- **THEN** the Account card shows that a token is saved in the system keyring, a masked preview, and whether it was restored at startup or validated this session

#### Scenario: Status line without a token
- **WHEN** the user opens Settings while no token is stored
- **THEN** the Account card states that no token is saved

### Requirement: Token actions follow token state
The Account card SHALL enable Sign out only while a token is stored, and SHALL
enable Sign in only while the token input is non-empty.

#### Scenario: Sign out disabled without a token
- **WHEN** no token is stored
- **THEN** the Sign out button is disabled

#### Scenario: Sign in disabled with an empty input
- **WHEN** the token input field is empty
- **THEN** the Sign in button is disabled

#### Scenario: Actions re-enable as state changes
- **WHEN** the user pastes a token into the input, or a sign-in stores a token
- **THEN** Sign in (respectively Sign out) becomes enabled accordingly

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
