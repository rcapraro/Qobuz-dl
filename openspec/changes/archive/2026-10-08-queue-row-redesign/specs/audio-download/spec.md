# Spec Delta

## ADDED Requirements

### Requirement: Delivered quality label
For each finished track the system SHALL report the delivered quality as a short label: for FLAC, the format followed by the delivered bit depth and sampling rate in kHz, such as `FLAC 24/96` or `FLAC 16/44.1`; for MP3, `MP3 320`. When the bit depth or sampling rate is not reported, the label SHALL fall back to the delivered tier's name.

#### Scenario: Hi-res FLAC
- **WHEN** a track is delivered as 24-bit FLAC at 96 kHz
- **THEN** its delivered quality reads `FLAC 24/96`

#### Scenario: CD-quality FLAC
- **WHEN** a track is delivered as 16-bit FLAC at 44.1 kHz
- **THEN** its delivered quality reads `FLAC 16/44.1`

#### Scenario: MP3
- **WHEN** a track is delivered as MP3
- **THEN** its delivered quality reads `MP3 320`

#### Scenario: Missing technical details
- **WHEN** the file URL response omits the bit depth or sampling rate
- **THEN** the delivered quality reads the delivered tier's name, such as `FLAC 24/≤96`
