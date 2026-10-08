# Spec Delta

## MODIFIED Requirements

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
