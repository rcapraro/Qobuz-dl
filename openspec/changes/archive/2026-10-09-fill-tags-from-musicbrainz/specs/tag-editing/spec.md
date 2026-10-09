# Spec Delta

## ADDED Requirements

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
