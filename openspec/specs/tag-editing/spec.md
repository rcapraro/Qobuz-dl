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

### Requirement: Fill from MusicBrainz
The editor SHALL offer a Fill from MusicBrainz control beside Reset to Qobuz. It SHALL look up the album's release on MusicBrainz and fill the fields from it. Nothing SHALL be written to the files until the user saves.

#### Scenario: Fill changes only the form
- **WHEN** the user activates Fill from MusicBrainz and the album's release is found
- **THEN** the fields show the release's values, the editor reports unsaved changes, and the files are unchanged until saved

### Requirement: Busy during a lookup
While a lookup runs, a release is being chosen or its cover is fetched, the editor SHALL show that it is busy, and Save, Reset to Qobuz, Fill from MusicBrainz and the field and cover controls SHALL be unavailable. The user SHALL be able to cancel at any of these steps, which leaves every field and the cover as they were.

#### Scenario: Busy while looking up
- **WHEN** a MusicBrainz lookup is running
- **THEN** the editor shows the lookup's progress, and Save and the fields and cover controls are unavailable

#### Scenario: Cancelling a cover download
- **WHEN** the user has unsaved edits, Include cover is on, and the user cancels while the release's cover is being fetched
- **THEN** the lookup stops, no field or cover changes, and the unsaved edits are kept

#### Scenario: Closing during a lookup
- **WHEN** the user closes the editor while a lookup is running
- **THEN** the lookup's result is discarded and the queue is shown again

### Requirement: Finding the album's release
The lookup SHALL first search for releases whose barcode is the album's Qobuz barcode. When none is found or the album has no barcode, it SHALL look for releases holding at least half of the edited tracks' ISRCs, and when none does, for releases with the album's track count, its disc count when known, and at least half of its title's words. When none is found, the editor SHALL report it as an error and leave every field unchanged.

#### Scenario: Found by barcode
- **WHEN** the album's barcode belongs to one MusicBrainz release
- **THEN** that release is used

#### Scenario: Found by ISRC
- **WHEN** no MusicBrainz release has the album's barcode but the edited tracks' ISRCs belong to recordings on a release
- **THEN** that release is offered

#### Scenario: Box set sharing a recording
- **WHEN** no release has the barcode of a 36-track compilation, and the only release holding any of its ISRCs is a box set holding one of them
- **THEN** the box set is not offered

#### Scenario: Found by title
- **WHEN** Qobuz's barcode for "Richard Hickox conducts Vaughan Williams" (36 tracks, 2 discs) is unknown to MusicBrainz and none of its ISRCs leads to a release, but MusicBrainz has "Hickox Conducts Vaughan Williams" with 36 tracks on 2 discs
- **THEN** that release is offered, and a 36-track release titled "Rattle Conducts Britten" is not

#### Scenario: Nothing found
- **WHEN** neither the barcode, the ISRCs nor the title leads to a release
- **THEN** the status line shows an error saying no MusicBrainz release matches the album, and no field changes

### Requirement: Match confidence
Each candidate release SHALL get a match confidence from 0 to 100 % that measures how well it fits the album on disk. The confidence SHALL rise with the share of edited tracks whose ISRC is on the release and with agreement between the release's track count and the album's, and SHALL also take into account title, label and whether the release is official. A release MusicBrainz lists no ISRCs for SHALL be measured on the other signals alone.

#### Scenario: Exact edition ranks first
- **WHEN** one candidate holds every edited track's ISRC and has the album's track count, and another holds half of them and has two bonus tracks
- **THEN** the first candidate's confidence is higher than the second's

#### Scenario: Release without ISRCs
- **WHEN** a candidate has the album's title, label and track count, is official, and MusicBrainz lists no ISRCs for it
- **THEN** its confidence is 100 %

#### Scenario: Confidence shown
- **WHEN** the editor reports or lists a candidate release
- **THEN** it shows that release's confidence as a percentage

### Requirement: Choosing among releases
When the lookup finds more than one candidate, the editor SHALL replace its track list with a list of the candidates sorted by confidence, highest first, each showing its title, date, country, label, format, track count and confidence. Choosing one SHALL fill the fields from it. Cancelling SHALL return to the track list with no field changed. A single candidate SHALL be used without the list, and the editor SHALL report which release it used and its confidence.

#### Scenario: Several releases share the barcode
- **WHEN** the barcode matches a 2015 US CD and a 2015 EU CD
- **THEN** the track list is replaced by both releases, the one with the higher confidence first, and choosing one fills the fields from it

#### Scenario: Picker cancelled
- **WHEN** the user cancels the list of releases
- **THEN** the track list is shown again and every field is as it was before the lookup

#### Scenario: Single release used directly
- **WHEN** the lookup finds exactly one candidate
- **THEN** the fields are filled from it without showing the list, and the editor reports the release used and its confidence

### Requirement: Fields filled from a release
Album, album artist, label, disc total (counting only audio discs), compilation, and each track's title, artist, composer, track number, total tracks and disc number SHALL be filled from the release. Date SHALL be the release group's first release date. Genre SHALL be the release's most-voted genre, as MusicBrainz spells it, falling back to its release group's. A field with no MusicBrainz value SHALL be left as it is, and ISRC, explicit, copyright and comment SHALL never change.

#### Scenario: Original date for a remaster
- **WHEN** the files hold the date 2015-03-13 of a remaster whose release group was first released on 1959-08-17
- **THEN** the date field shows 1959-08-17

#### Scenario: Composer from the works
- **WHEN** a release track's recording is a performance of a work composed by Miles Davis and the file has no composer
- **THEN** that track's composer field shows Miles Davis

#### Scenario: Genre spelled as MusicBrainz does
- **WHEN** the release's most-voted genre is "modal jazz"
- **THEN** the genre field shows "modal jazz"

#### Scenario: Explicit left alone
- **WHEN** a track is marked explicit and the user fills from MusicBrainz
- **THEN** the track's explicit field is unchanged

#### Scenario: Not a compilation
- **WHEN** the files are marked as a compilation and the release group is not a compilation
- **THEN** the compilation field is set to no

#### Scenario: Video disc not counted
- **WHEN** the chosen release is a CD with a bonus DVD-Video
- **THEN** the disc total is 1

### Requirement: Matching tracks to the release
Each edited track SHALL be matched to the release track whose recording has the file's ISRC, or, when no recording has it and the release has the album's track count, to the release track at the file's disc and track number. A track left unmatched SHALL keep its track fields, and the editor SHALL report how many tracks were not matched. When fewer than half of the edited tracks match, the album fields and cover SHALL be left as they are.

#### Scenario: Matched by ISRC despite renumbering
- **WHEN** a file is numbered track 7 but its ISRC is on the release's track 6
- **THEN** that file's track fields are filled from track 6

#### Scenario: Matched by position on a release without ISRCs
- **WHEN** MusicBrainz lists no ISRCs for a release with the album's 36 tracks
- **THEN** each file is filled from the release track at its disc and track number

#### Scenario: No position match on a release laid out otherwise
- **WHEN** a release has 147 tracks, the album has 36, and a file's ISRC is not on the release
- **THEN** that file is not matched, rather than taking the release's track at its position

#### Scenario: Unmatched track kept
- **WHEN** one of 12 files matches no release track by ISRC or position
- **THEN** that track's fields are unchanged, the other 11 are filled, and the editor reports that 1 track was not matched

#### Scenario: Most tracks unmatched
- **WHEN** a chosen release matches 2 of 36 files
- **THEN** those 2 tracks' fields are filled, the album fields and cover are left as they are, and the editor says so

### Requirement: Cover from MusicBrainz
The editor SHALL offer an Include cover option for the fill, off by default. When it is on and the chosen release has a front cover in the Cover Art Archive, the cover SHALL be replaced by that image, as when the user picks a replacement image. When the release has no front cover, the editor SHALL report it and leave the cover as it is.

#### Scenario: Fill without cover
- **WHEN** the user fills from MusicBrainz with Include cover off
- **THEN** the cover is unchanged

#### Scenario: Fill with cover
- **WHEN** the user fills from MusicBrainz with Include cover on and the release has a front cover
- **THEN** the cover is set to be replaced by that image, and the resize option is set to the Cover art setting

#### Scenario: No cover available
- **WHEN** Include cover is on and the release has no front cover
- **THEN** the other fields are filled, the cover is unchanged, and the editor reports that MusicBrainz has no cover for the release

### Requirement: Lookup failures
When MusicBrainz or the Cover Art Archive cannot be reached or returns an error, the editor SHALL report the failure and leave every field and the cover unchanged.

#### Scenario: Offline
- **WHEN** the user fills from MusicBrainz without a network connection
- **THEN** the editor reports that MusicBrainz could not be reached and no field changes

### Requirement: Considerate use of MusicBrainz
The app SHALL contact MusicBrainz and the Cover Art Archive only when the user asks for a fill, SHALL send at most one request per second to MusicBrainz, and SHALL identify itself with a User-Agent naming the app, its version and the project's URL. It SHALL send only the album's barcode, ISRCs, title and disc count, and release identifiers.

#### Scenario: Request pace
- **WHEN** a lookup needs several MusicBrainz requests
- **THEN** no two of them start less than one second apart

#### Scenario: No background traffic
- **WHEN** the user opens the tag editor and does not activate Fill from MusicBrainz
- **THEN** the app sends no request to MusicBrainz or the Cover Art Archive

### Requirement: Release list columns sized by content
In the list of MusicBrainz releases, the match, date, country and track count columns and the column of choose buttons SHALL each be as wide as their widest value or heading among the releases shown, and no wider. Title, label and format SHALL share all the remaining width, title taking the largest share. A value too long for its column SHALL stay on one line, cut short, with the whole value shown on hover.

#### Scenario: Short dates leave room for the title
- **WHEN** every release listed has a year-only date such as `2015`
- **THEN** the date column is only as wide as its heading or `2015`, whichever is wider, and the freed width goes to title, label and format

#### Scenario: Full dates fit
- **WHEN** a release listed has the date `2015-03-09`
- **THEN** the date is shown whole, on one line

#### Scenario: Long title
- **WHEN** a release title is longer than the title column
- **THEN** it is shown on one line, cut short, and hovering it shows the whole title

### Requirement: Release list headings aligned with values
In the list of MusicBrainz releases, each heading SHALL start at the same horizontal position as the values in its column, whatever widths the columns take.

#### Scenario: Widths change with the results
- **WHEN** a lookup lists releases whose country column is wider than a previous lookup's
- **THEN** every heading still sits right above its column's values

### Requirement: Track list favors the title
In the tag editor's track list, the title column SHALL be wider than the artist column, sharing the width they have between them in the ratio 3 to 2, on both the headings and every track's row.

#### Scenario: Title and artist widths
- **WHEN** the track list is shown
- **THEN** each track's title field is one and a half times as wide as its artist field, and the Title and Artist headings sit above them

### Requirement: Explain an unavailable save while the album is back in the queue
While the edited album has a track that is queued, downloading, tagging or part of a running batch, Save SHALL be unavailable and the editor SHALL say that the album is back in the queue and can be saved once its download ends. Edits SHALL be kept meanwhile. When no such track remains, the notice SHALL go away and Save SHALL be available again if there are edits.

#### Scenario: Album re-queued while editing
- **WHEN** the editor holds unsaved edits and the user adds a track of the same album to the queue
- **THEN** Save is unavailable, the editor says the album is back in the queue and can be saved once its download ends, and the edits are kept

#### Scenario: Download ends
- **WHEN** the edited album's re-queued tracks are done or failed and the batch has ended
- **THEN** the notice goes away and Save is available again for the kept edits
