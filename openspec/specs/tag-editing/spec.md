# tag-editing Specification

## Purpose
View and edit the tags of downloaded files from the queue, both in batch across an album and per track, and save the edits safely so a failed write never leaves a file damaged or half-written.

## Requirements

### Requirement: Editable tag fields
The tag editor SHALL edit this fixed list of fields and no others. Album fields: album, album artist, date, genre, label, copyright, total discs, compilation, cover. Track fields: title, artist, track number, total tracks, disc number, composer, ISRC, explicit, comment.

#### Scenario: Fields offered
- **WHEN** the user opens the tag editor for an album
- **THEN** it shows the album fields once for the album and the track fields for each done track

#### Scenario: Other tags not shown
- **WHEN** a file also carries tags outside the list, such as ReplayGain values
- **THEN** the editor does not show them

### Requirement: Editor shows the files' current tags
The tag editor SHALL show the tags as they are stored in each file when it opens, not the Qobuz metadata, so tags changed in another program are what the user sees and edits. It SHALL edit only the album's done tracks whose files exist.

#### Scenario: Tags edited elsewhere
- **WHEN** a downloaded file's title was changed in another tagger and the user opens the editor
- **THEN** the editor shows the changed title

#### Scenario: File missing
- **WHEN** one of the album's done files was moved or deleted before the editor opens
- **THEN** the editor lists the other tracks and states that the missing track cannot be edited

### Requirement: Album fields edited in batch
Each album field SHALL be shown once for the album. When all edited tracks hold the same value, the field SHALL show it. When they differ, it SHALL show that the value is mixed. Setting an album field SHALL apply that value to every edited track. A mixed field the user does not change SHALL keep each track's own value.

#### Scenario: Shared value shown
- **WHEN** every track has the genre Jazz
- **THEN** the album's genre field shows Jazz

#### Scenario: Mixed value kept
- **WHEN** tracks have different labels and the user saves without touching the label field
- **THEN** each file keeps its own label

#### Scenario: Batch value applied
- **WHEN** the user sets the album title to "Kind of Blue (Remaster)" and saves
- **THEN** every edited file has that album title

### Requirement: Track fields edited per track
Each track field SHALL be editable for each track. The editor SHALL also let the user set one track field to the same value on every track at once.

#### Scenario: One title fixed
- **WHEN** the user changes the title of track 3 and saves
- **THEN** only track 3's file has a new title

#### Scenario: Artist set on all tracks
- **WHEN** the user sets the artist on all tracks at once and saves
- **THEN** every edited file has that artist

### Requirement: Clearing a field
The editor SHALL let the user clear any field, which removes it from the affected files on save. Clearing SHALL be distinct from leaving a field untouched.

#### Scenario: Comment cleared
- **WHEN** the user clears the comment field for the album and saves
- **THEN** no edited file has a comment tag

#### Scenario: Untouched field not cleared
- **WHEN** the user leaves the composer field untouched and saves
- **THEN** every file keeps its composer tag as it was

### Requirement: Field validation
Track number, disc number, total tracks and total discs SHALL accept only positive whole numbers. Date SHALL accept a year (`YYYY`), a year and month (`YYYY-MM`) or a full date (`YYYY-MM-DD`). Explicit and compilation SHALL be yes/no values. Saving SHALL NOT be possible while any field is invalid, and the invalid field SHALL be indicated.

#### Scenario: Invalid track number
- **WHEN** the user types "1a" as a track number
- **THEN** the field is marked invalid and saving is unavailable until it is corrected

#### Scenario: Year accepted
- **WHEN** the user enters "1959" as the date
- **THEN** the date is valid

### Requirement: Reset to Qobuz metadata
The editor SHALL offer a control that fills the fields from the Qobuz metadata the queue holds for each track, the same values a fresh download would write. Fields Qobuz does not provide SHALL be left as they are. Nothing SHALL be written to the files until the user saves.

#### Scenario: Reset fills service values
- **WHEN** the user has edited the album title and activates Reset to Qobuz
- **THEN** the album title field shows Qobuz's title again and the files are unchanged until saved

#### Scenario: Comment left alone
- **WHEN** a file has a comment and the user activates Reset to Qobuz
- **THEN** the comment field is unchanged, since Qobuz provides no comment

### Requirement: Replace or remove the cover
The editor SHALL let the user replace the album's cover with a JPEG or PNG image file chosen from disk, and SHALL let the user remove the cover. Both actions apply to every edited track on save.

#### Scenario: Cover replaced
- **WHEN** the user replaces the cover with a 600×600 image, leaves the resize option at Don't resize, and saves
- **THEN** every edited file embeds that image unchanged as its front cover, and no other front cover

#### Scenario: Cover removed
- **WHEN** the user removes the cover and saves
- **THEN** no edited file has an embedded front cover

#### Scenario: Unreadable image
- **WHEN** the chosen file is not a readable JPEG or PNG image
- **THEN** the editor reports it and the cover field is unchanged

### Requirement: Resize the embedded cover
The editor SHALL offer a resize option for the cover: Don't resize, 400 px, 500 px or 600 px. A size SHALL apply on save to the cover each edited file ends up with, whether kept or replaced, using the same high-quality downscaling as downloaded covers. A cover already within the size SHALL NOT be changed or enlarged. The option SHALL be unavailable while the cover is set to be removed.

#### Scenario: Embedded covers reduced
- **WHEN** the files embed a 600×600 cover and the user picks 400 px and saves
- **THEN** every edited file embeds that cover at 400×400, and its other tags are unchanged

#### Scenario: Each file's own cover resized
- **WHEN** two files embed different covers, one 1200×1200 and one 600×600, and the user picks 500 px and saves
- **THEN** each file keeps its own cover, reduced to 500×500

#### Scenario: Cover already small enough
- **WHEN** a file's cover is 450×450, the user picks 500 px, and makes no other edit to that file
- **THEN** that file is not written

#### Scenario: Replacement resized
- **WHEN** the user replaces the cover with a 3000×3000 image, picks 500 px, and saves
- **THEN** every edited file embeds that image at 500×500

#### Scenario: Replacement defaults to the download size
- **WHEN** the user picks a replacement image while the Cover art setting is 400 px
- **THEN** the resize option is set to 400 px, and the user can change it before saving

### Requirement: Save writes only edited fields
Saving SHALL change only the fields the user set or cleared, in the files those edits apply to. Every other tag in each file, including tags outside the editable list, SHALL be preserved. Files with no edit SHALL NOT be written at all.

#### Scenario: Unlisted tags survive
- **WHEN** a file carries ReplayGain tags and the user changes its genre and saves
- **THEN** the file has the new genre and the same ReplayGain tags

#### Scenario: Unedited file untouched
- **WHEN** the user changes only track 2's title and saves
- **THEN** the other files' contents and modification times are unchanged

### Requirement: Saving is safe for each file
Each file SHALL be saved atomically: at every moment its path holds either the complete original file or the complete edited one. If saving a file fails, it SHALL stay exactly as it was, saving SHALL continue with the other files, and the editor SHALL report how many files were saved and which failed and why. A save failure SHALL NOT change the track's state in the queue.

#### Scenario: One file fails
- **WHEN** one of 12 files is read-only and the user saves an album-wide edit
- **THEN** 11 files are saved, the read-only file is unchanged, the editor reports that 1 file failed and why, and the track stays done in the queue

#### Scenario: Interrupted save
- **WHEN** the app quits while a file is being saved
- **THEN** that file is either the complete original or the complete edited file

### Requirement: Leaving the editor
The editor SHALL offer a control to close it and return to the queue. If there are unsaved edits, closing SHALL discard them, and the control SHALL say so. Edits SHALL never be written without an explicit save.

#### Scenario: Close with unsaved edits
- **WHEN** the user edits a field and closes the editor without saving
- **THEN** the files are unchanged and the queue is shown again

#### Scenario: Reopen after saving
- **WHEN** the user saves, closes, and opens the editor again
- **THEN** it shows the saved values read back from the files

### Requirement: Genre suggestions
The genre field SHALL suggest the standard ID3v1 genres (genres 0–79 as listed in the ID3v2.3 specification, Appendix A, with "Psychadelic" spelled correctly and "AlternRock" written as "Alternative Rock") in alphabetical order, narrowed to those matching what has been typed. It SHALL still accept any genre that is not in the list.

#### Scenario: Suggestions narrow as the user types
- **WHEN** the user types "Cla" in the genre field
- **THEN** the suggestions include Classical and Classic Rock, and picking Classical sets the genre to Classical

#### Scenario: Free genre kept
- **WHEN** the user types "Classique", which is not a standard genre, and saves
- **THEN** every edited file has the genre Classique
