# Proposal

## Why

In the MusicBrainz release picker, the release title — the value the user
chooses by — is cut short while Date and Country sit half empty. Every column
has a fixed width or a fixed share that ignores what it holds: Date is 132 px
because it shares a constant sized for the album Date *input*, though a shown
date needs about 98 px, and a year alone about 55 px. In the tag editor's track
list, Title and Artist split the row evenly although titles usually run longer.

## What Changes

- The release picker sizes its short columns (Match, Date, Country, Tracks and
  the Use button) to their widest value or heading among the releases shown, and
  gives all the remaining width to Title, Label and Format (3 : 2 : 1), Title taking the
  larger share. Long values stay on one line with the whole value on hover, as
  today.
- The release picker's headings and values lie on one grid, so they stay
  aligned whatever the widths.
- The tag editor's track list gives Title a larger share of the row than
  Artist (3 : 2) instead of an even split.
- Not changed: the queue (its widths stay fixed so rows don't shift during a
  download), the album track list on the Search tab, and column resizing,
  which is not offered.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tag-editing`: adds how the release picker sizes its columns and how the
  track list shares its width between Title and Artist.

## Impact

- `crates/qobuz-gui/src/app/view/tag_editor.rs`: `release_picker` and
  `release_row` become one iced 0.14 `table` (from `iced::widget`, already a
  dependency; no new crate). `MATCH_WIDTH`, `COUNTRY_WIDTH` and `USE_WIDTH` go
  once unused; `DATE_WIDTH` and `NUMBER_WIDTH` stay for the album Date input and
  the track number inputs. `track_list` and `track_row` change Title and Artist
  from `Fill` to `FillPortion(3)` and `FillPortion(2)`.
- No core, config or message changes.
