# file-organization Specification

## Purpose
TBD - created by archiving change add-qobuz-downloader. Update Purpose after archive.

## Requirements

### Requirement: Configurable path templates
The system SHALL render the destination folder and file name from user-defined
templates supporting placeholders including `{albumartist}`, `{artist}`,
`{title}`, `{album}`, `{year}`, `{tracknumber}` (with zero-padding, e.g.
`{tracknumber:02}`), `{bit_depth}`, `{sampling_rate}`, `{container}`, and
`{explicit}`.

#### Scenario: Template rendered
- **WHEN** the folder template is `{albumartist} - {album} ({year})` and the track template is `{tracknumber:02}. {title}`
- **THEN** a track is written to a matching path such as `Artist - Album (2020)/01. Song.flac`

#### Scenario: Multi-disc handling
- **WHEN** an album has more than one disc
- **THEN** the system organizes tracks into per-disc subfolders

### Requirement: Path sanitization
The system SHALL sanitize each rendered path segment by removing or replacing
characters illegal on target filesystems (`/ \ : * ? " < > |`) and trimming
overly long segments.

#### Scenario: Illegal characters removed
- **WHEN** a track title contains characters like `:` or `?`
- **THEN** the rendered path segment has those characters stripped or replaced so the file writes successfully on macOS, Windows, and Linux

### Requirement: Choose download directory
The system SHALL let the user select the base download directory via a native
directory picker.

#### Scenario: Directory selected
- **WHEN** the user picks a download directory in settings
- **THEN** subsequent downloads are written under that directory using the configured templates

### Requirement: Write audio tags
The system SHALL write metadata tags to downloaded files, including title,
artist, album, album artist, track number, total tracks, disc number, total
discs, release date, genre, label, copyright, ISRC, composer, and explicit
flag, for FLAC, MP3, and M4A containers. A field the service does not provide
SHALL be omitted rather than written empty.

#### Scenario: Tags written
- **WHEN** a track finishes downloading
- **THEN** the file contains the correct title, artist, album, track/disc numbers, and year tags

#### Scenario: Extended fields written
- **WHEN** a track of an album with a label, a copyright notice and a full release date finishes downloading
- **THEN** the file contains the label, the copyright, the full release date, the total number of discs, and the total number of tracks on the track's disc

#### Scenario: Explicit track flagged
- **WHEN** a track marked explicit by the service finishes downloading as FLAC, MP3 or M4A
- **THEN** the file carries an explicit flag that the container's usual taggers and players recognize

#### Scenario: Missing field omitted
- **WHEN** the service provides no copyright for an album
- **THEN** its downloaded files contain no copyright tag

### Requirement: Embed cover art
The system SHALL, unless the Cover art setting is Off, fetch the album cover
image and embed it into each downloaded audio file, sized according to that
setting.

#### Scenario: Cover embedded
- **WHEN** the Cover art setting is not Off and an album has a cover image
- **THEN** each downloaded file includes the embedded cover art

#### Scenario: Embedding disabled
- **WHEN** the Cover art setting is Off
- **THEN** downloaded files contain no embedded cover art and downloading still succeeds

### Requirement: Cover art size limit
The Cover art setting SHALL offer Off, 400 px, 500 px and 600 px, 600 px being the size of Qobuz's own cover. The system SHALL use the service's 600 px cover as the source. For a pixel size, the embedded image's longest side SHALL NOT exceed it, and the system SHALL NOT enlarge an image smaller than the chosen size.

#### Scenario: Original Qobuz cover kept
- **WHEN** the setting is 600 px and the album's cover is 600×600
- **THEN** the embedded cover is byte-identical to the cover the service provides

#### Scenario: Smaller cover for smaller files
- **WHEN** the setting is 400 px and the album has a 600×600 cover
- **THEN** the embedded cover is 400×400

#### Scenario: Aspect ratio kept
- **WHEN** the setting is 500 px and the cover is 600×450
- **THEN** the embedded cover is 500×375

#### Scenario: Small cover not enlarged
- **WHEN** the setting is 500 px and the album's cover is 450×450
- **THEN** the embedded cover is that 450×450 image, byte for byte

### Requirement: High-quality cover downscaling
When a cover must be made smaller, the system SHALL resample it with a high-quality windowed-sinc filter applied in linear light, not on gamma-encoded values, and SHALL encode the result as a JPEG of high quality without chroma subsampling. If decoding or resizing fails, the system SHALL embed the source image unchanged instead of failing the track.

#### Scenario: Resampled in linear light
- **WHEN** a cover made of a one-pixel black-and-white checkerboard is downscaled to half its size
- **THEN** the result is a uniform grey whose sRGB value is about 188, not about 128

#### Scenario: Color detail preserved
- **WHEN** a cover is downscaled and re-encoded
- **THEN** the JPEG stores its color channels at full resolution, with no chroma subsampling

#### Scenario: Unreadable cover still embedded
- **WHEN** the cover cannot be decoded for resizing
- **THEN** the original cover bytes are embedded and the track completes normally

### Requirement: Rename an album folder
The system SHALL rename a downloaded album's folder, the nearest folder holding all of its done files, to a name the user confirms, keeping it in the same parent directory. The name SHALL be sanitized like any rendered path segment, and SHALL NOT contain path separators.

#### Scenario: Folder renamed
- **WHEN** an album's files are in `Qobuz/Miles Davis - Kind of Blue (1959) [FLAC] [24B-96kHz]` and the user confirms the name `Miles Davis - Kind of Blue (1959)`
- **THEN** the folder is now `Qobuz/Miles Davis - Kind of Blue (1959)` with the same contents

#### Scenario: Separators sanitized
- **WHEN** the user confirms the name `AC/DC - Back in Black`
- **THEN** the folder is renamed to one sanitized name in the same parent directory, not moved into a subfolder

#### Scenario: Multi-disc album
- **WHEN** an album's files are in `Album/Disc 1` and `Album/Disc 2` and the user renames it
- **THEN** `Album` is renamed and both disc subfolders move with it

### Requirement: Folder rename safety
The system SHALL refuse a rename, change nothing, and say why when: a different file or folder already exists at the target; the folder is the download directory or lies outside it; the folder holds done files of another album in the queue; or the folder holds a subfolder with none of the album's done files, as an artist folder made by a `{albumartist}` folder template does. Loose files in the folder move with it. Renaming that only changes letter case SHALL be allowed.

#### Scenario: Target exists
- **WHEN** a folder with the confirmed name already exists in the same parent
- **THEN** nothing is renamed and the status line reports that the name is taken

#### Scenario: Album directly in the download directory
- **WHEN** the folder template is empty, so an album's files sit directly in the download directory
- **THEN** renaming is refused and the download directory keeps its name

#### Scenario: Folder shared with another album
- **WHEN** another queued album has done files in the same folder
- **THEN** renaming is refused, since it would move that album's files too

#### Scenario: Folder holds other albums
- **WHEN** the folder template is `{albumartist}`, so the album's files sit in `Qobuz/Miles Davis`, which also holds a `Bitches Brew` folder from an earlier download
- **THEN** renaming is refused, naming `Bitches Brew`, and nothing moves

#### Scenario: Case-only rename
- **WHEN** the user renames `kind of blue` to `Kind of Blue` on a case-insensitive filesystem
- **THEN** the folder's name changes case

### Requirement: Queue follows a renamed folder
After a successful rename, every queue item whose file was inside the renamed folder SHALL refer to its file at the new location, so opening the album's folder and editing its tags keep working.

#### Scenario: Open folder after renaming
- **WHEN** the user renames an album's folder and then activates Open folder on its group
- **THEN** the renamed folder opens

### Requirement: Rename template
The system SHALL provide a rename template, separate from the download folder template, that suggests an album folder's new name. It SHALL use the same placeholders and modifiers as the path templates, with values taken from the album's files: tag values from its first track in disc and track order, and `container`, `bit_depth` and `sampling_rate` from that file's audio format.

#### Scenario: Suggested from edited tags
- **WHEN** the album title tag was edited to "Kind of Blue (Remaster)" and the rename template is `{albumartist} - {album} ({year})`
- **THEN** the suggested name is `Miles Davis - Kind of Blue (Remaster) (1959)`

#### Scenario: Template unchanged by downloads
- **WHEN** the user changes the rename template
- **THEN** download folder and track paths are unaffected
