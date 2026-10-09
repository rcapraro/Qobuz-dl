# Spec Delta

## ADDED Requirements

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
