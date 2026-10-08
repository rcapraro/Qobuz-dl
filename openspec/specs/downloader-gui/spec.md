# downloader-gui Specification

## Purpose
TBD - created by archiving change add-qobuz-downloader. Update Purpose after archive.

## Requirements

### Requirement: Tabbed navigation

The system SHALL present the Search/Add, Queue, and Settings sections as a tab
bar, in that order, with exactly one section visible at a time and the active
tab visually indicated. The Search/Add section's tab SHALL be labelled
"Search". The application SHALL open on the Search/Add section.
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

#### Scenario: Search tab label

- **WHEN** the tab bar is displayed
- **THEN** the Search/Add section's tab reads "Search"

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
a bounded numeric concurrency control that accepts only values in the range 1–10.
A persisted concurrency above 10 SHALL be treated as 10.
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
- **THEN** the value is constrained to the range 1–10 and cannot be set to a non-numeric or out-of-range value

#### Scenario: Saved concurrency above the range
- **WHEN** the app starts with a persisted concurrency above 10
- **THEN** the concurrency control shows 10, downloads run at most 10 tracks at once, and no unsaved-changes hint is shown

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

### Requirement: Queue grouped by album
The Queue screen SHALL group queued tracks under their album, however they
were added. Groups SHALL appear in the order their first track was queued,
and tracks SHALL keep queue order within a group. Each track row SHALL show
the track number and title, and the performer only when it differs from the
album artist.

#### Scenario: Album added whole
- **WHEN** the user adds an album
- **THEN** its tracks appear under one group headed by that album

#### Scenario: Single track joins its album
- **WHEN** a track is queued whose album already has a group
- **THEN** the track is listed in that existing group rather than in a new one

#### Scenario: Playlist tracks group by album
- **WHEN** the user adds a playlist whose tracks come from several albums
- **THEN** each track is listed under its own album's group

#### Scenario: Group order
- **WHEN** tracks from album A are queued, then from album B, then another track from album A
- **THEN** album A's group comes first and holds both of its tracks, followed by album B's group

#### Scenario: Rows drop the repeated artist
- **WHEN** a track's performer is the album artist
- **THEN** its row shows the track number and title without the artist, and a track with a different performer shows that performer

### Requirement: Album group header
Each queue group SHALL have a header with the album's cover, title, artist,
how many of its tracks are done out of its total, how many failed when any
did, and a progress bar for the group computed the same way as the overall
progress but over that group's tracks only.

#### Scenario: Group count
- **WHEN** a group holds 12 tracks of which 7 are done and 1 failed
- **THEN** its header shows that 7 of 12 are done and that 1 failed

#### Scenario: Group progress settles on failure
- **WHEN** every track in a group has finished and at least one failed
- **THEN** the group's bar shows the group as complete, as the overall bar does for the whole queue

#### Scenario: Cover shown
- **WHEN** a group's album has a cover
- **THEN** the header shows it once loaded, with a placeholder until then, and the cover stays shown after the user runs a new search

### Requirement: Collapse an album group
The user SHALL be able to collapse a queue group to its header and expand it
again. Groups SHALL start expanded. Collapsing SHALL NOT change any track's
state or the overall progress.

#### Scenario: Collapse and expand
- **WHEN** the user collapses a group and then expands it
- **THEN** its track rows are hidden while collapsed and shown again after, with their states unchanged

#### Scenario: Collapsed group still reports progress
- **WHEN** a collapsed group's tracks are downloading
- **THEN** its header count and progress bar keep updating

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

### Requirement: Open an album's folder
A queue group with at least one done track SHALL offer a control that opens
the folder holding that album's downloaded files in the system file manager.
When the files are in different folders, it SHALL open the nearest folder
they share.

#### Scenario: Open after downloading
- **WHEN** a group has done tracks and the user activates Open folder
- **THEN** the folder containing those tracks' files opens in the file manager

#### Scenario: No done track yet
- **WHEN** none of a group's tracks is done
- **THEN** the group offers no Open folder control

#### Scenario: Folder gone
- **WHEN** the user activates Open folder after the folder was moved or deleted
- **THEN** the status line reports that the folder no longer exists, and nothing else changes

### Requirement: Open the download folder from Settings
The Settings screen SHALL offer, next to the download directory, a control
that opens that directory in the system file manager.

#### Scenario: Open the download root
- **WHEN** the user activates Open next to the download directory
- **THEN** that directory opens in the file manager

#### Scenario: Directory missing
- **WHEN** the download directory does not exist
- **THEN** the status line reports it rather than opening anything

### Requirement: Unsaved settings indicator
The Settings screen SHALL show an "Unsaved changes" hint next to the Save settings control whenever the current settings differ from the persisted ones. Save settings SHALL be enabled only while there are unsaved changes. Any successful save, explicit or implicit, SHALL clear the hint.

#### Scenario: Editing a setting shows the hint
- **WHEN** the user changes any setting (credentials, directory, templates, quality, cover art, notifications, or concurrency) without saving
- **THEN** the "Unsaved changes" hint is shown and Save settings is enabled

#### Scenario: Saving clears the hint
- **WHEN** the user activates Save settings and the save succeeds
- **THEN** the hint disappears and Save settings is disabled

#### Scenario: Reverting an edit clears the hint
- **WHEN** the user changes a setting and then restores its persisted value
- **THEN** the hint disappears and Save settings is disabled

#### Scenario: Implicit save clears the hint
- **WHEN** an action that persists settings on its own succeeds (theme toggle, sign-in, credential auto-detection, or adopting a working fallback secret after a signing check or at the end of a download batch)
- **THEN** all current settings are persisted and the hint disappears

#### Scenario: Failed save keeps the hint
- **WHEN** a save fails
- **THEN** the error is reported on the status line and the hint stays shown

#### Scenario: Nothing to save at startup
- **WHEN** the app starts with its persisted settings
- **THEN** no hint is shown and Save settings is disabled

#### Scenario: Unreadable settings file stays replaceable
- **WHEN** the app starts with defaults because the persisted settings could not be read
- **THEN** the hint is shown and Save settings is enabled, so the defaults can replace the unreadable file

### Requirement: Labeled settings fields
Every text input on the Settings screen SHALL have a visible label that remains shown when the field holds a value. Each row's first input SHALL start on one fixed-width label column shared with the File organization fields. A second input on the same row, such as `app_secret` after `app_id`, SHALL carry its own inline label.

#### Scenario: Filled credentials keep their labels
- **WHEN** the `app_id` and `app_secret` fields contain values
- **THEN** each field still shows its label beside it

#### Scenario: Token input is labeled
- **WHEN** the user views the Account card
- **THEN** the token input has a visible label beside it

#### Scenario: Inputs align across cards
- **WHEN** the user views the Settings screen
- **THEN** the `app_id`, token, folder and track inputs all start at the same horizontal position

#### Scenario: Credentials share a row
- **WHEN** the user views the API credentials card
- **THEN** `app_id` and `app_secret` are on one row, `app_id` on the label column and `app_secret` after its inline label

### Requirement: One primary action per settings card
Each Settings card SHALL present at most one control in the primary button style, and that control SHALL be the card's main action. Other controls in the card SHALL use the secondary style. When a card explains its buttons, the explanation SHALL be in the card's help panel and SHALL describe every button in that card.

#### Scenario: Credentials card emphasis
- **WHEN** the user views the API credentials card
- **THEN** Auto-detect is the only primary button, Check signing is secondary, and no explanatory text sits under the buttons

#### Scenario: Credentials help explains both buttons
- **WHEN** the user opens the API credentials help panel
- **THEN** it explains what Auto-detect and Check signing each do

#### Scenario: Account card emphasis
- **WHEN** the user views the Account card
- **THEN** Sign in is the only primary button and Sign out is secondary

### Requirement: Single-line Options card
The Options card SHALL lay out its controls on one line: Quality on the label column, then Concurrency, then the Embed cover art and Notify checkboxes.

#### Scenario: Options on one line
- **WHEN** the user views the Options card at the default window size
- **THEN** Quality, Concurrency and both checkboxes share one line, with Quality starting on the label column

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

### Requirement: Queued results are marked
A search result's add control SHALL read "Added ✓" and SHALL be disabled while the result is in the queue: a track result while that track is queued, an album result while any of that album's tracks is queued. It SHALL return to its normal state once those tracks leave the queue.

#### Scenario: Track already queued
- **WHEN** a track result's track is in the queue
- **THEN** its add control reads "Added ✓" and cannot be activated

#### Scenario: Album partly queued
- **WHEN** at least one track of an album result is in the queue
- **THEN** the album result's add control reads "Added ✓" and cannot be activated, and the album can still be opened to add more of its tracks

#### Scenario: Removed from the queue
- **WHEN** the queued tracks of a result are removed or the queue is cleared
- **THEN** that result's add control reads "Add" and can be activated again

### Requirement: Aligned result quality column
Every search result row SHALL reserve the same width for the hi-res indicator before its add control, whether or not the release is hi-res, so the indicators and the add controls each form a straight column.

#### Scenario: Mixed hi-res results
- **WHEN** a results list shows hi-res and non-hi-res releases
- **THEN** the Hi-Res badges line up in one column, and the rows without one keep an empty slot of the same width

### Requirement: Navigation shortcuts
Ctrl+Tab SHALL show the next tab and Ctrl+Shift+Tab the previous one, in tab-bar order and wrapping around, whether or not a text field has focus. On the Search and Queue tabs, when no text field has focus, `/` SHALL show the Search tab and focus its search field. No navigation shortcut SHALL type a character into a focused text field.

#### Scenario: Focus search
- **WHEN** no text field has focus and the user presses `/` on the Search or Queue tab
- **THEN** the Search tab is shown and its search field has focus

#### Scenario: Switch tabs
- **WHEN** the user presses Ctrl+Tab while typing in the search field on the Search tab
- **THEN** the Queue tab is shown and no character is added to the search field

#### Scenario: Tabs wrap around
- **WHEN** the Settings tab is shown and the user presses Ctrl+Tab
- **THEN** the Search tab is shown, and Ctrl+Shift+Tab from Search shows Settings

#### Scenario: Slash while typing
- **WHEN** the search field has focus and the user types `/`
- **THEN** a `/` is added to the query, as with any other character

#### Scenario: Slash on Settings
- **WHEN** the Settings tab is shown and the user presses `/`
- **THEN** the Settings tab stays shown, whichever of its fields has focus

### Requirement: Album detail shortcuts
While an album's detail is shown on the Search tab, Escape SHALL close it and return to the results, and command+Enter SHALL add the selected tracks as "Add selected" does, unless the search field has focus. In every other case these keys SHALL do nothing beyond their usual effect on a focused text field.

#### Scenario: Leave an album's detail
- **WHEN** an album's detail is open on the Search tab and the user presses Escape
- **THEN** the results are shown again at the scroll position they had

#### Scenario: Escape elsewhere
- **WHEN** no album's detail is open, or another tab is shown, and the user presses Escape
- **THEN** nothing changes, apart from a focused text field losing focus

#### Scenario: Album hidden behind the setup prompt
- **WHEN** an album's detail is open but the Search tab shows the setup prompt instead, because credentials are missing or nobody is signed in, and the user presses Escape or command+Enter
- **THEN** nothing is added to the queue and the album stays open for when setup is done

#### Scenario: Add the selection
- **WHEN** an album's detail is open on the Search tab with tracks selected, the search field does not have focus, and the user presses command+Enter
- **THEN** the selected tracks are added to the queue as if "Add selected" had been activated

#### Scenario: Command+Enter in the search field
- **WHEN** the search field has focus and the user presses command+Enter
- **THEN** the search is submitted as with Enter, and nothing is added
