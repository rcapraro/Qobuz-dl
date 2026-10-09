//! Per-screen view builders plus the widget helpers they share.

pub(super) mod album;
pub(super) mod queue;
pub(super) mod search;
pub(super) mod settings;
pub(super) mod tag_editor;

use super::status::{Status, StatusKind};
use super::Message;
use crate::style::{self, compact_button};
use iced::widget::{container, image, row, stack, text, tooltip, Text};
use iced::{Alignment, Color, Element, Font, Length};
use iced_aw::widget::badge::Badge;
use std::time::Duration;

/// Bold variant of the default UI font (Inter). Basing this on `Font::DEFAULT`
/// would fall back to a system sans-serif, so it must name the Inter family to
/// match the surrounding regular-weight Inter text.
pub(super) fn bold() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::with_name("Inter")
    }
}

/// Right padding matching the scrollbar gutter, so scrollable content doesn't
/// sit under the scrollbar while its left edge stays on the fixed controls'.
pub(super) fn gutter_padding() -> iced::Padding {
    iced::Padding {
        right: style::SCROLLBAR_GUTTER,
        ..iced::Padding::ZERO
    }
}

pub(super) fn section(title: &str) -> Element<'_, Message> {
    text(title).size(style::TEXT_SECTION).into()
}

/// Icon glyph and accent role for a status kind. Every glyph is present in the
/// bundled Inter, so none falls back to an OS font.
fn status_look(kind: StatusKind) -> (&'static str, fn(&style::Accents) -> Color) {
    match kind {
        StatusKind::Info => ("•", |a| a.subtext),
        StatusKind::Progress => ("…", |a| a.progress_text),
        StatusKind::Success => ("✓", |a| a.success_text),
        StatusKind::Error => ("✗", |a| a.error_text),
    }
}

/// The status line under the header. Its height is fixed so that dismissing a
/// message, or the dismiss control appearing, never shifts the tabs below.
pub(super) fn status_bar(status: Option<&Status>) -> Element<'_, Message> {
    let kind = status.map_or(StatusKind::Info, |s| s.kind);
    let (glyph, role) = status_look(kind);

    let mut line = row![].spacing(style::SPACE_SM).align_y(Alignment::Center);
    if let Some(s) = status {
        line = line
            .push(
                text(glyph)
                    .size(style::TEXT_BODY)
                    .font(bold())
                    .style(move |theme| text::Style {
                        color: Some(role(&style::accents(theme))),
                    }),
            )
            .push(text(&s.text).size(style::TEXT_SM).width(Length::Fill));
        if s.kind == StatusKind::Error {
            line = line.push(compact_button("×").on_press(Message::DismissStatus));
        }
    }

    container(line)
        .style(move |theme| {
            let outline = (kind == StatusKind::Error).then(|| style::accents(theme).error());
            style::status_surface(theme, outline)
        })
        .padding([0.0, style::SPACE_MD])
        .center_y(Length::Fixed(style::CONTROL_HEIGHT))
        .width(Length::Fill)
        .into()
}

/// A square cover image, or a placeholder while it loads or when there is none.
pub(super) fn cover<'a>(thumb: Option<&image::Handle>, size: f32) -> Element<'a, Message> {
    match thumb {
        Some(handle) => image(handle.clone())
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into(),
        None => container(text(""))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .style(style::thumb_placeholder)
            .into(),
    }
}

/// How long the pointer rests on a cut line before its whole text shows.
const TOOLTIP_DELAY: Duration = Duration::from_millis(500);

/// `line` kept on one line and cut at the width it is given, so a row stays
/// one line tall however long its text; `full`, the whole text, shows on hover.
pub(super) fn one_line<'a>(
    line: Text<'a>,
    full: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    let line = container(line.wrapping(text::Wrapping::None))
        .width(Length::Fill)
        .clip(true);
    let full = container(text(full))
        .padding(style::SPACE_SM)
        .style(style::surface);
    tooltip(line, full, tooltip::Position::Bottom)
        .delay(TOOLTIP_DELAY)
        .into()
}

/// A small audio-quality chip ("Hi-Res", a delivered format), styled like the
/// queue's status badges but in the quality accent.
pub(super) fn quality_badge<'a>(label: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    Badge::new(
        text(label)
            .size(style::TEXT_SM)
            .wrapping(text::Wrapping::None),
    )
    .style(|theme, _status| {
        let a = style::accents(theme);
        style::role_badge(theme, a.quality())
    })
    .into()
}

/// A slot sized by `sample`, which is never seen, with `content` laid over it
/// at its end. The slot keeps its size as `content` changes, so whatever sits
/// beside it doesn't move. `sample` must be at least as large as `content`.
pub(super) fn slot<'a>(
    sample: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    stack![
        sample.into(),
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::End)
            .align_y(Alignment::Center),
    ]
    // Shrink, or the stack takes on the content's Fill and stops sizing by the
    // sample.
    .width(Length::Shrink)
    .height(Length::Shrink)
    .into()
}

/// A [`style::styled_button`] sized to `label` and drawn fully transparent, to
/// size a [`slot`] for a button whose label changes.
pub(super) fn hidden_button(label: &str) -> Element<'_, Message> {
    style::styled_button(label)
        .width(Length::Shrink)
        .style(|_theme, _status| iced::widget::button::Style {
            text_color: Color::TRANSPARENT,
            ..iced::widget::button::Style::default()
        })
        .into()
}

/// A card grouping a section's controls under an arbitrary header element, such
/// as a title plus a help toggle.
pub(super) fn card_el<'a>(
    head_content: impl Into<Element<'a, Message>>,
    body: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    iced_aw::widget::card::Card::new(head_content, body)
        .style(|theme, _status| style::card(theme))
        .into()
}
