## MODIFIED Requirements

### Requirement: Edit an album's tags from the queue
A queue group SHALL offer an Edit tags control when it has at least one done track and none of its tracks is queued, downloading, tagging, or part of a running batch. Activating it SHALL show that album's tag editor in place of the queue list, on the Queue tab, below the strip of albums in progress.

#### Scenario: Offered for a finished album
- **WHEN** every track of a group is done or failed and no batch includes them
- **THEN** its header offers Edit tags

#### Scenario: Not offered while downloading
- **WHEN** any track of a group is queued, downloading or tagging
- **THEN** its header does not offer Edit tags

#### Scenario: Not offered without done tracks
- **WHEN** none of a group's tracks is done
- **THEN** its header does not offer Edit tags

#### Scenario: Editor replaces the list
- **WHEN** the user activates Edit tags on a group
- **THEN** the Queue tab shows that album's tag editor instead of the queue list, until the editor is closed, with the strip of albums in progress above it

## ADDED Requirements

### Requirement: Albums in progress beside the tag editor
While the tag editor is open, the Queue tab SHALL show, above it, one line for each album with a track that is queued, downloading, tagging or part of the running batch, in queue order. Each line SHALL show the album's cover, title, done count and progress bar, and nothing else. At most three lines SHALL be visible at once; the rest SHALL be reachable by scrolling. With no album in progress, the strip SHALL not be shown.

#### Scenario: Album added while editing
- **WHEN** the tag editor is open and the user adds an album from Search
- **THEN** the Queue tab shows the editor, and above it a line for the added album showing 0 done and an empty progress bar

#### Scenario: Progress shown while editing
- **WHEN** the tag editor is open and a batch downloads another album
- **THEN** that album's line shows its done count and progress bar advancing as its tracks finish

#### Scenario: Album leaves the strip
- **WHEN** every track of an album in the strip is done or failed and the batch has ended
- **THEN** its line is removed, and the strip is hidden once no line remains

#### Scenario: Many albums in progress
- **WHEN** five albums are in progress while the editor is open
- **THEN** three lines are visible, the other two are reachable by scrolling the strip, and the editor's position does not depend on how many albums are in progress beyond three

#### Scenario: Nothing in progress
- **WHEN** the tag editor is open and no track in the queue is queued, downloading, tagging or in a running batch
- **THEN** no strip is shown and the editor sits directly below the queue header

### Requirement: Strip replaces the overall progress bar
While the strip of albums in progress is shown beside the tag editor, the queue header SHALL show its complete count and controls without the overall progress bar, since the strip's own bars show what is downloading. When the strip is not shown, the overall progress bar SHALL be shown as usual.

#### Scenario: One album downloading while editing
- **WHEN** the tag editor is open and one album is downloading
- **THEN** the header shows its "N / M complete" count and Cancel, without an overall progress bar, and that album's line in the strip shows its progress bar

#### Scenario: Editing with nothing in progress
- **WHEN** the tag editor is open and no album is in progress
- **THEN** the header shows the overall progress bar under its count
