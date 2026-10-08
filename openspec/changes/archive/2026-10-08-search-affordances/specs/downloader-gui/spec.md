# Spec Delta

## ADDED Requirements

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
