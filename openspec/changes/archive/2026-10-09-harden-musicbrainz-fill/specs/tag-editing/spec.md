## MODIFIED Requirements

### Requirement: Finding the album's release
The lookup SHALL first search for releases whose barcode is the album's Qobuz barcode. Failing that, it SHALL look for releases holding at least half of the edited tracks' ISRCs, and when none does, for releases with the album's track count on their audio media, its disc count when known, at least half of its title's words, and, when both are known, an artist credit sharing a word with the album artist. If none is found, the editor SHALL report an error and change no field.

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

#### Scenario: Same title by another artist
- **WHEN** the album is Queen's 17-track "Greatest Hits", found by neither barcode nor ISRC, and MusicBrainz has a 17-track "Greatest Hits" credited to ABBA
- **THEN** ABBA's release is not offered

#### Scenario: Video disc not counted when searching by title
- **WHEN** the album has 10 tracks and the release with its title is a 10-track CD with a 5-track DVD-Video
- **THEN** that release is offered

#### Scenario: Large album found by ISRC
- **WHEN** a 40-track album's ISRCs match more than 100 recordings on MusicBrainz
- **THEN** every matching recording is taken into account when ranking releases by the ISRCs they hold

#### Scenario: Nothing found
- **WHEN** neither the barcode, the ISRCs nor the title leads to a release
- **THEN** the status line shows an error saying no MusicBrainz release matches the album, and no field changes

### Requirement: Match confidence
Each candidate release SHALL get a match confidence from 0 to 100 % that measures how well it fits the album on disk. The confidence SHALL rise with the share of edited tracks whose ISRC is on the release and with agreement between the release's track count and the album's, and SHALL also take into account title, artist, label and whether the release is official. A release MusicBrainz lists no ISRCs for SHALL be measured on the other signals alone.

#### Scenario: Exact edition ranks first
- **WHEN** one candidate holds every edited track's ISRC and has the album's track count, and another holds half of them and has two bonus tracks
- **THEN** the first candidate's confidence is higher than the second's

#### Scenario: Release without ISRCs
- **WHEN** a candidate has the album's title, artist, label and track count, is official, and MusicBrainz lists no ISRCs for it
- **THEN** its confidence is 100 %

#### Scenario: Another artist's release
- **WHEN** a candidate has the album's title and track count, MusicBrainz lists no ISRCs for it, and its artist shares no word with the album artist
- **THEN** its confidence is below 80 %

#### Scenario: Confidence shown
- **WHEN** the editor reports or lists a candidate release
- **THEN** it shows that release's confidence as a percentage

### Requirement: Choosing among releases
When the lookup finds more than one candidate, or one below 80 % confidence, the editor SHALL replace its track list with the candidates, best first by confidence, each showing its title, date, country, label, format, track count and confidence. Choosing one SHALL fill the fields from it. Cancelling SHALL return to the track list with no field changed. A single candidate of at least 80 % SHALL be used without the list, and the editor SHALL report it and its confidence.

#### Scenario: Several releases share the barcode
- **WHEN** the barcode matches a 2015 US CD and a 2015 EU CD
- **THEN** the track list is replaced by both releases, the one with the higher confidence first, and choosing one fills the fields from it

#### Scenario: Picker cancelled
- **WHEN** the user cancels the list of releases
- **THEN** the track list is shown again and every field is as it was before the lookup

#### Scenario: Single release used directly
- **WHEN** the lookup finds exactly one candidate, at 90 %
- **THEN** the fields are filled from it without showing the list, and the editor reports the release used and its confidence

#### Scenario: Weak single release listed
- **WHEN** the lookup finds exactly one candidate, at 35 %
- **THEN** no field changes, and the track list is replaced by that one release for the user to choose or cancel

### Requirement: Matching tracks to the release
Each edited track SHALL be matched to the release track whose recording has the file's ISRC, or, when no recording has it and the release has the album's track count, to the release track at the file's disc and track number. Disc numbers SHALL count audio media only, when matching and when filled. A track left unmatched SHALL keep its track fields, and the editor SHALL report how many were not. When fewer than half of the edited tracks match, the album fields and cover SHALL be left alone.

#### Scenario: Matched by ISRC despite renumbering
- **WHEN** a file is numbered track 7 but its ISRC is on the release's track 6
- **THEN** that file's track fields are filled from track 6

#### Scenario: Matched by position on a release without ISRCs
- **WHEN** MusicBrainz lists no ISRCs for a release with the album's 36 tracks
- **THEN** each file is filled from the release track at its disc and track number

#### Scenario: Video medium listed first
- **WHEN** a release lists a DVD-Video as its first medium and the album's CD as its second, and a file at disc 1, track 3 has no ISRC on the release
- **THEN** that file is filled from the CD's track 3, and its disc number is 1

#### Scenario: No position match on a release laid out otherwise
- **WHEN** a release has 147 tracks, the album has 36, and a file's ISRC is not on the release
- **THEN** that file is not matched, rather than taking the release's track at its position

#### Scenario: Unmatched track kept
- **WHEN** one of 12 files matches no release track by ISRC or position
- **THEN** that track's fields are unchanged, the other 11 are filled, and the editor reports that 1 track was not matched

#### Scenario: Most tracks unmatched
- **WHEN** a chosen release matches 2 of 36 files
- **THEN** those 2 tracks' fields are filled, the album fields and cover are left as they are, and the editor says so

#### Scenario: Most tracks unmatched with Include cover
- **WHEN** Include cover is on and a chosen release matches 2 of 36 files
- **THEN** the release's cover is not fetched, those 2 tracks' fields are filled at once, and the cover is left as it is

### Requirement: Lookup failures
When MusicBrainz or the Cover Art Archive cannot be reached or returns an error, the editor SHALL report the failure and leave every field and the cover unchanged. A candidate release that cannot be read SHALL be left out of the candidates; the lookup SHALL fail only when none of them can be read.

#### Scenario: Offline
- **WHEN** the user fills from MusicBrainz without a network connection
- **THEN** the editor reports that MusicBrainz could not be reached and no field changes

#### Scenario: One release unreadable
- **WHEN** the barcode search finds 5 releases and reading one of them fails
- **THEN** the other 4 are offered as candidates

#### Scenario: Result of a cancelled lookup
- **WHEN** the user cancels a lookup and starts another before the first one's result arrives
- **THEN** the first result is dropped and the second lookup goes on
