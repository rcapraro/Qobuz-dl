//! Per-screen view builders plus the widget helpers they share.

pub(super) mod queue;
pub(super) mod search;
pub(super) mod settings;

use super::Message;
use crate::style;
use iced::widget::text;
use iced::{Element, Font};

/// Bold variant of the default UI font (Inter). Basing this on `Font::DEFAULT`
/// would fall back to a system sans-serif, so it must name the Inter family to
/// match the surrounding regular-weight Inter text.
pub(super) fn bold() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::with_name("Inter")
    }
}

/// Horizontal padding matching the scrollbar gutter, so scrollable content
/// doesn't sit under the scrollbar.
pub(super) fn gutter_padding() -> iced::Padding {
    iced::Padding {
        left: style::SCROLLBAR_GUTTER,
        right: style::SCROLLBAR_GUTTER,
        ..iced::Padding::ZERO
    }
}

pub(super) fn section(title: &str) -> Element<'_, Message> {
    text(title).size(style::TEXT_SECTION).into()
}

/// A titled card grouping a section's controls. `head` picks the accent color
/// for the card's header from the active Catppuccin flavor.
pub(super) fn card<'a>(
    title: &'a str,
    body: impl Into<Element<'a, Message>>,
    head: fn(&style::Accents) -> iced::Color,
) -> Element<'a, Message> {
    card_el(text(title).size(style::TEXT_SECTION), body, head)
}

/// Like [`card`] but with an arbitrary header element (e.g. a title plus a help
/// toggle) instead of a plain title.
pub(super) fn card_el<'a>(
    head_content: impl Into<Element<'a, Message>>,
    body: impl Into<Element<'a, Message>>,
    head: fn(&style::Accents) -> iced::Color,
) -> Element<'a, Message> {
    iced_aw::widget::card::Card::new(head_content, body)
        .style(move |theme, _status| {
            let a = style::accents(theme);
            style::card(&a, head(&a))
        })
        .into()
}
