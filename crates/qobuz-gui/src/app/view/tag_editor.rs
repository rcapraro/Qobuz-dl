//! An album's tag editor, shown on the Queue tab in place of the list: the
//! album fields once, each track's own fields, the cover, and Save.

use super::super::tag_editor::{
    hint, invalid, label, Edit, EditorTrack, Lookup, Resize, TagEditor,
};
use super::super::{App, Message};
use super::{bold, cover, hidden_button, one_line, section, slot};
use crate::style::{self, compact_button, field_input, fill_button, labeled_row, styled_button};
use iced::widget::{
    checkbox, column, combo_box, container, pick_list, row, scrollable, space, stack, table, text,
    text_input, Container, TextInput,
};
use iced::{Element, Length};
use iced_aw::widget::badge::Badge;
use qobuz_core::musicbrainz::{Candidate, Step};
use qobuz_core::tag_edit::{CoverAction, Field, FieldKind};

const COVER_SIZE: f32 = 112.0;
const COVER_THUMB_SIZE: f32 = 64.0;
/// Fits a three-digit track, disc or total.
const NUMBER_WIDTH: f32 = 64.0;
/// Fits `YYYY-MM-DD`.
const DATE_WIDTH: f32 = 132.0;
/// Fits the "(mixed)" placeholder of a Yes/No pick list.
const FLAG_WIDTH: f32 = 110.0;
/// Fits "More ▼" and "Less ▲", so the column doesn't shift as rows toggle.
const TOGGLE_WIDTH: f32 = 80.0;
/// So every row's Apply to all lines up.
const APPLY_WIDTH: f32 = 104.0;
/// Space the form keeps clear of the scrollbar, beyond its gutter, so the
/// row-end buttons don't sit against it.
const RIGHT_MARGIN: f32 = style::SCROLLBAR_GUTTER + style::SPACE_LG;
/// The release list's header band, drawn under the table since the table has
/// no style for it: a heading's line, at iced's default 1.3 line height, with
/// the table's `SPACE_XS` row padding above and below.
const HEAD_HEIGHT: f32 = style::TEXT_SM * 1.3 + 2.0 * style::SPACE_XS;
/// From here a release is shown as a strong match.
const STRONG_MATCH: u8 = 80;
/// Below this a release is shown as a poor match.
const WEAK_MATCH: u8 = 50;
/// The track fields shown when a track row is expanded, each with Apply to all.
const MORE_FIELDS: [Field; 6] = [
    Field::Artist,
    Field::TrackTotal,
    Field::Composer,
    Field::Isrc,
    Field::Explicit,
    Field::Comment,
];

/// Shown while the album's files are being read.
pub(in crate::app) fn loading_view<'a>() -> Element<'a, Message> {
    container(text("Reading tags…").style(style::muted_text))
        .style(style::surface)
        .padding(style::SPACE_XL)
        .center_x(Length::Fill)
        .into()
}

pub(in crate::app) fn tag_editor_view<'a>(
    app: &'a App,
    editor: &'a TagEditor,
) -> Element<'a, Message> {
    let tracks = match &editor.lookup {
        Some(Lookup::Choosing(candidates)) => release_picker(candidates),
        _ => track_list(editor),
    };
    let mut body = column![album_form(editor), tracks].spacing(style::SPACE_LG);
    if !editor.unreadable.is_empty() {
        body = body.push(unreadable(editor));
    }
    let body = body.padding(iced::Padding {
        right: RIGHT_MARGIN,
        ..iced::Padding::ZERO
    });
    column![header(app, editor), scrollable(body).height(Length::Fill)]
        .spacing(style::SPACE_MD)
        .into()
}

/// The album's identity and the editor's actions: the sources that fill the
/// fields (Reset to Qobuz, Fill from MusicBrainz and its cover option), then
/// Close, which says it discards unsaved edits, and Save, the primary action.
fn header<'a>(app: &'a App, editor: &'a TagEditor) -> Element<'a, Message> {
    let saving = editor.saving.is_some();
    let busy = editor.busy();
    let has_edits = editor.has_edits();
    const DISCARD: &str = "Discard changes";
    let close = if has_edits { DISCARD } else { "Close" };
    // Sized by the longer label, so Save stays put when Close becomes Discard
    // changes.
    let close_slot = slot(
        hidden_button(DISCARD),
        fill_button(close)
            .style(style::secondary)
            .on_press_maybe((!saving).then_some(Message::CloseTagEditor)),
    );
    // A running lookup is stopped from the button that started it, sized by
    // the longer label so the checkbox beside it stays put.
    const FILL: &str = "Fill from MusicBrainz";
    let musicbrainz = if editor.lookup.is_some() {
        fill_button("Cancel lookup").on_press(Message::CancelLookup)
    } else {
        fill_button(FILL).on_press_maybe((!busy).then_some(Message::FillFromMusicBrainz))
    };
    let musicbrainz_slot = slot(hidden_button(FILL), musicbrainz.style(style::secondary));
    let actions = row![
        styled_button("Reset to Qobuz")
            .style(style::secondary)
            .width(Length::Shrink)
            .on_press_maybe((!busy).then_some(Message::Editor(Edit::ResetToQobuz))),
        musicbrainz_slot,
        checkbox(editor.include_cover)
            .label("Include cover")
            .text_size(style::TEXT_BODY)
            .on_toggle_maybe((!busy).then_some(|on| Message::Editor(Edit::IncludeCover(on)))),
        space::horizontal(),
        close_slot,
        styled_button("Save")
            .on_press_maybe((has_edits && app.save_ready(editor)).then_some(Message::SaveTags)),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);

    let info = column![
        text(&editor.title).size(style::TEXT_HEADLINE).font(bold()),
        text(&editor.artist).size(style::TEXT_SECTION),
        text(progress(
            editor,
            has_edits,
            !app.tags_editable(&editor.album_id)
        ))
        .size(style::TEXT_SM)
        .style(style::muted_text),
        actions,
    ]
    .spacing(style::SPACE_XS)
    .width(Length::Fill);

    container(
        row![cover(editor.preview.as_ref(), COVER_SIZE), info]
            .spacing(style::SPACE_LG)
            .align_y(iced::Alignment::Center),
    )
    .style(style::hero)
    .padding(style::SPACE_LG)
    .width(Length::Fill)
    .into()
}

/// What the editor is doing, why it can't save while its album is back in the
/// queue, or whether it holds unsaved edits.
fn progress(editor: &TagEditor, has_edits: bool, requeued: bool) -> String {
    if let Some(s) = &editor.saving {
        return format!("Saving {} of {}…", (s.done + 1).min(s.total), s.total);
    }
    match &editor.lookup {
        Some(Lookup::Searching(None)) => "Looking up MusicBrainz…".to_owned(),
        Some(Lookup::Searching(Some(Step::Barcode))) => {
            "Searching MusicBrainz by barcode…".to_owned()
        }
        Some(Lookup::Searching(Some(Step::Isrcs))) => "Searching MusicBrainz by ISRC…".to_owned(),
        Some(Lookup::Searching(Some(Step::Title))) => "Searching MusicBrainz by title…".to_owned(),
        Some(Lookup::Searching(Some(Step::Release { n, of }))) => {
            format!("Reading MusicBrainz release {n} of {of}…")
        }
        Some(Lookup::Choosing(_)) => "Choose the release that matches your files".to_owned(),
        Some(Lookup::Cover(_)) => "Fetching the cover from the Cover Art Archive…".to_owned(),
        None if requeued => "Back in the queue: save once its download ends".to_owned(),
        None if has_edits => "Unsaved changes".to_owned(),
        None => String::new(),
    }
}

/// The album fields on the label column, short ones sharing a row with inline
/// labels, as in Settings: names and text fill the row, numbers and the date
/// take only the width they need.
fn album_form(editor: &TagEditor) -> Element<'_, Message> {
    let fill = Length::Fill;
    column![
        section("Album"),
        labeled_row(label(Field::Album), album_input(editor, Field::Album, fill)),
        labeled_row(
            label(Field::AlbumArtist),
            album_input(editor, Field::AlbumArtist, fill)
        ),
        labeled_row(
            label(Field::Genre),
            inline_row([
                genre_input(editor),
                inline_label(Field::Label),
                album_input(editor, Field::Label, fill),
            ]),
        ),
        labeled_row(
            label(Field::Copyright),
            album_input(editor, Field::Copyright, fill)
        ),
        labeled_row(
            label(Field::Date),
            inline_row([
                album_input(editor, Field::Date, Length::Fixed(DATE_WIDTH)),
                inline_label(Field::DiscTotal),
                album_input(editor, Field::DiscTotal, Length::Fixed(NUMBER_WIDTH)),
                inline_label(Field::Compilation),
                flag_pick(editor.album_flag(Field::Compilation), |on| {
                    Message::Editor(Edit::AlbumFlag(Field::Compilation, on))
                }),
            ]),
        ),
        labeled_row("Cover:", cover_controls(editor)),
    ]
    .spacing(style::SPACE_SM)
    .into()
}

fn inline_row<'a>(
    controls: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    iced::widget::Row::with_children(controls)
        .spacing(style::SPACE_MD)
        .align_y(iced::Alignment::Center)
        .into()
}

/// The genre: any text, with the standard genres suggested as it is typed.
fn genre_input(editor: &TagEditor) -> Element<'_, Message> {
    let f = editor.album_field(Field::Genre);
    let placeholder = if f.shows_mixed() { "(mixed)" } else { "" };
    let set = |genre| Message::Editor(Edit::AlbumText(Field::Genre, genre));
    let input = combo_box(
        &editor.genres,
        placeholder,
        (!f.text.is_empty()).then_some(&f.text),
        set,
    )
    .on_input(set)
    .padding(style::INPUT_PADDING)
    .size(style::TEXT_BODY)
    .width(Length::Fill);
    let mut control = row![input]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);
    if f.is_mixed() {
        control = control
            .push(compact_button("×").on_press(Message::Editor(Edit::ClearAlbum(Field::Genre))));
    }
    control.into()
}

/// The label of a control that shares its row, beside it.
fn inline_label<'a>(field: Field) -> Element<'a, Message> {
    text(label(field)).size(style::TEXT_BODY).into()
}

/// An album field's input, `width` wide. A field whose values differ shows
/// "(mixed)" and a × to clear it everywhere, since leaving it empty keeps
/// each file's value.
fn album_input(editor: &TagEditor, field: Field, width: Length) -> Element<'_, Message> {
    let f = editor.album_field(field);
    let placeholder = if f.shows_mixed() {
        "(mixed)"
    } else {
        hint(field)
    };
    let input = validated(
        field_input(placeholder, &f.text)
            .on_input(move |text| Message::Editor(Edit::AlbumText(field, text)))
            .width(width),
        invalid(field, &f.text),
    );
    let mut control = row![input]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);
    if f.is_mixed() {
        control =
            control.push(compact_button("×").on_press(Message::Editor(Edit::ClearAlbum(field))));
    }
    control.into()
}

/// Yes/No for a flag; `None` shows "(mixed)" until one is picked.
fn flag_pick<'a>(
    selected: Option<bool>,
    on_pick: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    pick_list(
        [YesNo::Yes, YesNo::No],
        selected.map(YesNo::from),
        move |choice| on_pick(choice == YesNo::Yes),
    )
    .placeholder("(mixed)")
    .padding(style::INPUT_PADDING)
    .text_size(style::TEXT_BODY)
    .width(Length::Fixed(FLAG_WIDTH))
    .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YesNo {
    Yes,
    No,
}

impl From<bool> for YesNo {
    fn from(on: bool) -> Self {
        if on {
            YesNo::Yes
        } else {
            YesNo::No
        }
    }
}

impl std::fmt::Display for YesNo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            YesNo::Yes => "Yes",
            YesNo::No => "No",
        })
    }
}

/// The cover thumbnail, Replace…, Remove (or Keep once removal is chosen),
/// and the resize option, which has nothing to resize once the cover goes.
fn cover_controls(editor: &TagEditor) -> Element<'_, Message> {
    let idle = |edit| (!editor.busy()).then_some(Message::Editor(edit));
    let mut controls = row![
        cover(editor.preview.as_ref(), COVER_THUMB_SIZE),
        compact_button("Replace…").on_press_maybe((!editor.busy()).then_some(Message::PickCover)),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);
    if editor.cover == CoverAction::Remove {
        controls = controls
            .push(
                text("Removed on save")
                    .size(style::TEXT_SM)
                    .style(style::muted_text),
            )
            .push(compact_button("Keep").on_press_maybe(idle(Edit::KeepCover)));
    } else {
        controls = controls
            .push(compact_button("Remove").on_press_maybe(idle(Edit::RemoveCover)))
            .push(text("Resize:").size(style::TEXT_SM))
            .push(pick_list(
                Resize::all(),
                Some(Resize::of(editor.resize)),
                |choice| Message::Editor(Edit::Resize(choice)),
            ));
    }
    controls.into()
}

/// One row per track (disc, number, title, artist), with its less-used
/// fields under it when expanded.
fn track_list(editor: &TagEditor) -> Element<'_, Message> {
    let head = row![
        column_label("Disc", Length::Fixed(NUMBER_WIDTH)),
        column_label("#", Length::Fixed(NUMBER_WIDTH)),
        column_label("Title", Length::FillPortion(3)),
        column_label("Artist", Length::FillPortion(2)),
        column_label("", Length::Fixed(TOGGLE_WIDTH)),
    ]
    .spacing(style::SPACE_SM);
    let mut list = column![
        section("Tracks"),
        container(head)
            .style(style::table_head)
            .padding([style::SPACE_XS, 0.0])
    ]
    .spacing(style::SPACE_SM);
    for track in &editor.tracks {
        list = list.push(track_row(editor, track));
        if editor.expanded.contains(&track.track_id) {
            list = list.push(more_fields(track));
        }
    }
    list.into()
}

/// The releases a MusicBrainz lookup found, best match first, in place of the
/// track list until one is chosen or the list is cancelled.
fn release_picker(candidates: &[Candidate]) -> Element<'_, Message> {
    let heading = |label| inset(text(label).size(style::TEXT_SM));
    let columns = [
        table::column(heading("Match"), |(_, c): (usize, &Candidate)| {
            inset(confidence_badge(c.confidence))
        }),
        table::column(heading("Title"), |(_, c): (usize, &Candidate)| {
            long_cell(c.release.title.as_str())
        })
        .width(Length::FillPortion(3)),
        table::column(heading("Date"), |(_, c): (usize, &Candidate)| {
            inset(text(or_dash(c.release.date.as_deref())))
        }),
        table::column(heading("Country"), |(_, c): (usize, &Candidate)| {
            inset(text(or_dash(c.release.country.as_deref())))
        }),
        table::column(heading("Label"), |(_, c): (usize, &Candidate)| {
            long_cell(or_dash(c.release.label()))
        })
        .width(Length::FillPortion(2)),
        table::column(heading("Format"), |(_, c): (usize, &Candidate)| {
            long_cell(or_dash(Some(&c.release.formats())).to_owned())
        })
        .width(Length::FillPortion(1)),
        table::column(heading("Tracks"), |(_, c): (usize, &Candidate)| {
            inset(text(c.release.total_tracks().to_string()))
        }),
        table::column(heading(""), |(index, _): (usize, &Candidate)| {
            compact_button("Use").on_press(Message::ChooseRelease(index))
        }),
    ]
    .map(|column| column.align_y(iced::Alignment::Center));
    let releases = table(columns, candidates.iter().enumerate())
        .padding_x(style::SPACE_SM / 2.0)
        .padding_y(style::SPACE_XS)
        .separator(0.0);
    let band = container(space::horizontal())
        .width(Length::Fill)
        .height(Length::Fixed(HEAD_HEIGHT))
        .style(style::table_head);

    column![
        row![
            section("MusicBrainz releases"),
            space::horizontal(),
            compact_button("Cancel").on_press(Message::CancelLookup),
        ]
        .align_y(iced::Alignment::Center),
        text("Ranked by how well each release matches your files.")
            .size(style::TEXT_SM)
            .style(style::muted_text),
        stack![releases].push_under(band),
    ]
    .spacing(style::SPACE_SM)
    .into()
}

/// A cell or column heading, its text inset like an input's, so headings sit
/// right above their values.
fn inset<'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content).padding([0.0, style::INPUT_PADDING + 1.0])
}

/// A release value that can run long (title, label, format), kept on one line
/// so every row is a line tall, with the whole value on hover.
fn long_cell<'a>(value: impl text::IntoFragment<'a> + Clone) -> Container<'a, Message> {
    inset(one_line(text(value.clone()), value))
}

fn or_dash(value: Option<&str>) -> &str {
    value.filter(|v| !v.is_empty()).unwrap_or("–")
}

/// A release's match confidence, colored by how far it can be trusted.
fn confidence_badge<'a>(confidence: u8) -> Element<'a, Message> {
    Badge::new(
        text(format!("{confidence} %"))
            .size(style::TEXT_SM)
            .wrapping(text::Wrapping::None),
    )
    .style(move |theme, _status| {
        let a = style::accents(theme);
        let role = match confidence {
            STRONG_MATCH.. => a.success(),
            WEAK_MATCH.. => a.progress(),
            _ => a.error(),
        };
        style::role_badge(theme, role)
    })
    .into()
}

/// A column heading `width` wide, its text inset like an input's so it sits
/// right above the values.
fn column_label(label: &str, width: Length) -> Element<'_, Message> {
    inset(text(label).size(style::TEXT_SM)).width(width).into()
}

fn track_row<'a>(editor: &'a TagEditor, track: &'a EditorTrack) -> Element<'a, Message> {
    let id = track.track_id;
    let expanded = editor.expanded.contains(&id);
    row![
        track_input(track, Field::DiscNumber).width(Length::Fixed(NUMBER_WIDTH)),
        track_input(track, Field::TrackNumber).width(Length::Fixed(NUMBER_WIDTH)),
        track_input(track, Field::Title).width(Length::FillPortion(3)),
        track_input(track, Field::Artist).width(Length::FillPortion(2)),
        compact_button(if expanded { "Less ▲" } else { "More ▼" })
            .width(Length::Fixed(TOGGLE_WIDTH))
            .on_press(Message::Editor(Edit::ToggleExpanded(id))),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

fn track_input<'a>(track: &'a EditorTrack, field: Field) -> TextInput<'a, Message> {
    let id = track.track_id;
    let value = track.text(field);
    validated(
        field_input(hint(field), value)
            .on_input(move |text| Message::Editor(Edit::TrackText(id, field, text))),
        invalid(field, value),
    )
}

/// A track's other fields in a panel under its row, starting at the Title
/// column, each with Apply to all at the row's end.
fn more_fields(track: &EditorTrack) -> Element<'_, Message> {
    let id = track.track_id;
    let mut fields = column![].spacing(style::SPACE_SM);
    for field in MORE_FIELDS {
        let control: Element<'_, Message> = if field.kind() == FieldKind::Flag {
            checkbox(!track.text(field).is_empty())
                .on_toggle(move |on| Message::Editor(Edit::TrackFlag(id, field, on)))
                .into()
        } else {
            let width = match field.kind() {
                FieldKind::Number => Length::Fixed(NUMBER_WIDTH),
                _ => Length::Fill,
            };
            track_input(track, field).width(width).into()
        };
        fields = fields.push(labeled_row(
            label(field),
            row![
                container(control).width(Length::Fill),
                compact_button("Apply to all")
                    .width(Length::Fixed(APPLY_WIDTH))
                    .on_press(Message::Editor(Edit::ApplyToAll(id, field))),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
        ));
    }
    let title_column = 2.0 * (NUMBER_WIDTH + style::SPACE_SM);
    container(
        container(fields)
            .style(style::surface)
            .padding(style::SPACE_MD)
            .width(Length::Fill),
    )
    .padding(iced::Padding {
        left: title_column,
        bottom: style::SPACE_XS,
        ..iced::Padding::ZERO
    })
    .into()
}

/// Done tracks whose files couldn't be read, so they aren't edited.
fn unreadable(editor: &TagEditor) -> Element<'_, Message> {
    let mut list = column![section("Not editable")].spacing(style::SPACE_XS);
    for track in &editor.unreadable {
        list = list.push(
            text(format!("{} — {}", track.title, track.reason))
                .size(style::TEXT_SM)
                .style(|theme| text::Style {
                    color: Some(style::accents(theme).error_text),
                }),
        );
    }
    list.into()
}

/// Outline an input in the error accent when its value can't be saved.
fn validated(input: TextInput<'_, Message>, bad: bool) -> TextInput<'_, Message> {
    if !bad {
        return input;
    }
    input.style(|theme, status| {
        let mut s = text_input::default(theme, status);
        s.border.color = style::accents(theme).error();
        s.border.width = 1.5;
        s
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::album::tests::job;
    use super::*;
    use qobuz_core::tag_edit::TagFields;
    use std::path::PathBuf;

    fn editor() -> TagEditor {
        let read = vec![(
            job(1, 1, Some("Band")),
            PathBuf::from("/m/1.flac"),
            Ok(TagFields::default()),
        )];
        TagEditor::new("al".into(), read)
    }

    #[test]
    fn status_line_says_why_a_requeued_album_cannot_save() {
        let back = "Back in the queue: save once its download ends";
        let mut e = editor();
        assert_eq!(progress(&e, false, false), "");
        assert_eq!(progress(&e, false, true), back);

        e.set_album_text(Field::Album, "New".into());
        assert_eq!(progress(&e, true, false), "Unsaved changes");
        assert_eq!(
            progress(&e, true, true),
            back,
            "kept edits wait for the download"
        );

        let mut looking = editor();
        looking.start_lookup();
        assert_eq!(progress(&looking, false, true), "Looking up MusicBrainz…");

        e.start_saving();
        assert_eq!(progress(&e, true, true), "Saving 1 of 1…");
    }
}
