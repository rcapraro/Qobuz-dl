# Spec Delta

## ADDED Requirements

### Requirement: Edit an album's tags from the queue
A queue group SHALL offer an Edit tags control when it has at least one done track and none of its tracks is queued, downloading, tagging, or part of a running batch. Activating it SHALL show that album's tag editor in place of the queue list, on the Queue tab.

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
- **THEN** the Queue tab shows that album's tag editor instead of the queue list, until the editor is closed
