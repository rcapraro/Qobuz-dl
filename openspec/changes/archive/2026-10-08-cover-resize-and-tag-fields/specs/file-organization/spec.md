# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
