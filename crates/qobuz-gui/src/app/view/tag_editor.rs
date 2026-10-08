//! An album's tag editor, shown on the Queue tab in place of the list: the
//! album fields once, each track's own fields, the cover, and Save.

use super::super::tag_editor::{hint, invalid, label, Edit, EditorTrack, Resize, TagEditor};
use super::super::{App, Message};
use super::{bold, cover, section};
use crate::style::{self, compact_button, field_input, labeled_row, styled_button};
use iced::widget::{
    button, checkbox, column, combo_box, container, pick_list, row, scrollable, text, text_input,
    TextInput,
};
use iced::{Element, Length};
use qobuz_core::tag_edit::{CoverAction, Field, FieldKind};

const COVER_SIZE: f32 = 96.0;
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
const RIGHT_MARGIN: f32 = style::SCROLLBAR_GUTTER + style::SPACE_LG as f32;
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
    let mut body = column![album_form(editor), track_list(editor)].spacing(style::SPACE_LG);
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

/// The album's identity and the editor's actions: Reset to Qobuz, Save (the
/// primary action) and Close, which says it discards unsaved edits.
fn header<'a>(app: &'a App, editor: &'a TagEditor) -> Element<'a, Message> {
    let saving = editor.saving.is_some();
    let has_edits = editor.has_edits();
    let progress = match &editor.saving {
        Some(s) => format!("Saving {} of {}…", (s.done + 1).min(s.total), s.total),
        None if has_edits => "Unsaved changes".to_owned(),
        None => String::new(),
    };
    let info = column![
        text(&editor.title).size(style::TEXT_HEADLINE).font(bold()),
        text(&editor.artist).size(style::TEXT_SECTION),
        text(progress).size(style::TEXT_SM).style(style::muted_text),
    ]
    .spacing(style::SPACE_XS)
    .width(Length::Fill);

    let close = if has_edits {
        "Discard changes"
    } else {
        "Close"
    };
    let actions = row![
        styled_button("Reset to Qobuz")
            .style(button::secondary)
            .width(Length::Shrink)
            .on_press_maybe((!saving).then_some(Message::Editor(Edit::ResetToQobuz))),
        styled_button(close)
            .style(button::secondary)
            .width(Length::Shrink)
            .on_press_maybe((!saving).then_some(Message::CloseTagEditor)),
        styled_button("Save")
            .on_press_maybe((has_edits && app.save_ready(editor)).then_some(Message::SaveTags)),
    ]
    .spacing(style::SPACE_SM);

    container(
        row![cover(editor.preview.as_ref(), COVER_SIZE), info, actions]
            .spacing(style::SPACE_LG)
            .align_y(iced::Alignment::Center),
    )
    .style(style::hero)
    .padding(style::SPACE_LG)
    .width(Length::Fill)
    .into()
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
    .size(style::TEXT_BODY.into())
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
    let mut controls = row![
        cover(editor.preview.as_ref(), COVER_THUMB_SIZE),
        compact_button("Replace…")
            .on_press_maybe(editor.saving.is_none().then_some(Message::PickCover)),
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
            .push(compact_button("Keep").on_press(Message::Editor(Edit::KeepCover)));
    } else {
        controls = controls
            .push(compact_button("Remove").on_press(Message::Editor(Edit::RemoveCover)))
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
        column_label("Title", Length::Fill),
        column_label("Artist", Length::Fill),
        column_label("", Length::Fixed(TOGGLE_WIDTH)),
    ]
    .spacing(style::SPACE_SM);
    let mut list = column![
        section("Tracks"),
        container(head)
            .style(style::table_head)
            .padding([style::SPACE_XS, 0])
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

/// A column heading `width` wide, its text inset like an input's so it sits
/// right above the values.
fn column_label(label: &str, width: Length) -> Element<'_, Message> {
    container(text(label).size(style::TEXT_SM).style(style::muted_text))
        .width(width)
        .padding([0, style::INPUT_PADDING + 1])
        .into()
}

fn track_row<'a>(editor: &'a TagEditor, track: &'a EditorTrack) -> Element<'a, Message> {
    let id = track.track_id;
    let expanded = editor.expanded.contains(&id);
    row![
        track_input(track, Field::DiscNumber).width(Length::Fixed(NUMBER_WIDTH)),
        track_input(track, Field::TrackNumber).width(Length::Fixed(NUMBER_WIDTH)),
        track_input(track, Field::Title).width(Length::Fill),
        track_input(track, Field::Artist).width(Length::Fill),
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
            checkbox("", !track.text(field).is_empty())
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
    let title_column = 2.0 * (NUMBER_WIDTH + style::SPACE_SM as f32);
    container(
        container(fields)
            .style(style::surface)
            .padding(style::SPACE_MD)
            .width(Length::Fill),
    )
    .padding(iced::Padding {
        left: title_column,
        bottom: style::SPACE_XS as f32,
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
                    color: Some(style::accents(theme).error()),
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
