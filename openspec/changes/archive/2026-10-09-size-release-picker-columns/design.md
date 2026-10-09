# Design

## Context

`release_picker` builds a header row of `column_label`s and `release_row` builds
one `row!` per candidate. They line up only because both use the same width
constants. See proposal.md - Why for the width problem.

iced 0.14 ships a `table` widget (`iced::widget::table`, source in
`iced_widget-0.14.2/src/table.rs`). Its layout runs in passes:

1. Every cell (headers included) of a `Shrink` or `Fixed` column is laid out,
   and the column takes its widest cell.
2. `Fill`/`FillPortion` columns share what is left, in proportion to their
   factors.
3. Cells are placed on that one grid, header row first.

A `table::column(header, view)` defaults to `Shrink`, left/top aligned. The
table defaults to `padding_x: 10.0` per cell side plus separators and has its
own style `Catalog`.

## Goals / Non-Goals

**Goals:**
- The release picker as one `table`, headers and values on one grid.
- The look stays as it is: the `table_head` band behind the headings, and the
  text inset matching the editor's inputs (`INPUT_PADDING + 1`).

**Non-Goals:**
- Moving the track list to `table`. Its cells are text inputs, which fill
  whatever width they are given, so `Shrink` does nothing for them. Only its
  `Fill` factors change.
- Content-weighted `FillPortion` computed from the candidates' text lengths.

## Decisions

**Use iced's `table` rather than measuring text ourselves.** The alternative
was to keep the rows and compute each short column's width from the candidates
(character counts times an average glyph width). That guesses at font metrics
iced already knows, and keeps the two-grid alignment problem. `table` measures
the real rendered cells, header included, so Country stays as wide as its
heading even with two-letter values.

**Fixed shares for the long columns: Title 3, Label 2, Format 1.** A format is
usually short (`CD`, `Digital Media`); at an equal share with Label it showed
mostly empty, so Format gets the smallest share and a rare long one
(`SACD (2 channels) + SACD (multichannel)`) is cut with the whole value on
hover. Content-weighted factors were considered and left out: they could give a
short title less room than a long format, against the spec's title-first rule,
for little gain with at most five candidates.

**Keep the cell inset in the cells, and the gap in the table.** Each cell keeps
its existing `[0, INPUT_PADDING + 1]` container padding. The table's
`padding_x` goes on both sides of every cell, so `padding_x: SPACE_SM / 2`
gives the current `SPACE_SM` gap between columns, at the cost of an extra
`SPACE_SM / 2` at the table's outer edges. `separator_x`/`separator_y` are `0`,
and `padding_y` gives the row spacing the picker has today. Compare the result
with the current picker.

**Header band.** `table`'s `Style` only colors the separators, so it cannot
draw a band behind the header row. Styling each header cell does not work
either: a cell's node is as wide as its own content, not its column, so a
`Date` heading over `2015-03-09` values would leave a gap. The band is one
`style::table_head` container pushed under the table with `Stack::push_under` (a stack takes its size from its first layer, so the table must be that layer), as tall as
the header row: `HEAD_HEIGHT` is a small-text line at iced's default 1.3 line
height plus the table's `padding_y` (`SPACE_XS`) above and below, computed from
`TEXT_SM` and `SPACE_XS` rather than hard-coded.
Rows are kept a line tall: `one_line` already keeps long values on one line.

**Track list ratio.** `FillPortion(3)` for Title and `FillPortion(2)` for
Artist, on both the `column_label`s in `track_list` and the inputs in
`track_row`, so the headings stay above their inputs.

## Risks / Trade-offs

- [`table`'s defaults differ from the current spacing, so the picker shifts by
  a few pixels] → set padding and separators explicitly, and compare with the
  current picker in Latte and Macchiato.
- [An unusual value in a short column, such as a long date string, widens the
  whole column] → acceptable: these values are short by nature, and Title,
  Label and Format still take what is left.
- [The band and the header row are sized separately, so a heading with another
  size or line height would no longer match the band] → `HEAD_HEIGHT` is derived
  from the headings' own text size, and every heading is one line of `TEXT_SM`.
- [The first row now sits `SPACE_XS` below the band, not `SPACE_SM`] → accepted;
  rows stay `SPACE_SM` apart.
