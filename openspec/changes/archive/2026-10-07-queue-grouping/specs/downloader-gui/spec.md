# Spec Delta

## ADDED Requirements

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

### Requirement: Remove a group's queued tracks
While no batch is running, a group header SHALL offer a control that removes
that group's tracks that are still queued, leaving its downloading, done, and
failed tracks in place. The control SHALL NOT be offered while a batch is
running or when the group has no queued track.

#### Scenario: Remove queued tracks of one album
- **WHEN** a group holds queued and done tracks and the user activates its remove control
- **THEN** only that group's queued tracks are removed, its done tracks stay, and other groups are untouched

#### Scenario: Group disappears when emptied
- **WHEN** every track of a group was queued and the user removes them
- **THEN** the group no longer appears

#### Scenario: Not offered during a batch
- **WHEN** a download batch is running
- **THEN** no group offers the remove control
