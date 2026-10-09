# Tasks

## 1. Release picker on iced's table

- [x] 1.1 Rebuild `release_picker`/`release_row` in `crates/qobuz-gui/src/app/view/tag_editor.rs` as one `iced::widget::table` over the candidates: Match, Date, Country, Tracks and Use as `Shrink` columns; Title `FillPortion(3)`, Label `FillPortion(2)` and Format `FillPortion(1)` through `one_line`; the section title, explanation and Cancel stay above the table. Verify with `cargo build -p qobuz-gui`.
- [x] 1.2 Match the current look: each cell keeps its `[0, INPUT_PADDING + 1]` padding, the table's `padding_x` is `SPACE_SM / 2` with zero separators, and the headings keep the `style::table_head` band, pushed under the table with `Stack::push_under` since the table's style cannot draw it. Verify by opening the picker in Latte and Macchiato: headings sit above their values, no gaps show in the band, and rows are one line tall.
- [x] 1.3 Remove `MATCH_WIDTH`, `COUNTRY_WIDTH` and `USE_WIDTH` once unused, keeping `DATE_WIDTH` and `NUMBER_WIDTH`. Verify `cargo clippy --workspace --all-targets` reports no warnings.
- [x] 1.4 Check the spec's scenarios on a real lookup: year-only dates give a narrow Date column, a full `YYYY-MM-DD` date shows whole, and a long title is cut on one line with the whole title on hover.

## 2. Track list ratio

- [x] 2.1 In `track_list` and `track_row`, change Title and Artist from `Length::Fill` to `FillPortion(3)` and `FillPortion(2)` on both the headings and the inputs. Verify in the editor that each title field is 1.5 times as wide as its artist field, with the headings above them.

## 3. Integration

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and verify all pass.

## Workflow follow-up

- Archive the change after review, then verify `openspec/specs/tag-editing/spec.md` holds the three added requirements.
