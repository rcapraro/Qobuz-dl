## ADDED Requirements

### Requirement: Explain an unavailable save while the album is back in the queue
While the edited album has a track that is queued, downloading, tagging or part of a running batch, Save SHALL be unavailable and the editor SHALL say that the album is back in the queue and can be saved once its download ends. Edits SHALL be kept meanwhile. When no such track remains, the notice SHALL go away and Save SHALL be available again if there are edits.

#### Scenario: Album re-queued while editing
- **WHEN** the editor holds unsaved edits and the user adds a track of the same album to the queue
- **THEN** Save is unavailable, the editor says the album is back in the queue and can be saved once its download ends, and the edits are kept

#### Scenario: Download ends
- **WHEN** the edited album's re-queued tracks are done or failed and the batch has ended
- **THEN** the notice goes away and Save is available again for the kept edits
