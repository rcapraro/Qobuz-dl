//! Centralized visual design system: spacing/sizing/typography constants,
//! reusable widget builders, and the light/dark theme palettes.
//!
//! All screens draw their spacing, control sizes, and colors from here so the
//! UI reads as one consistent system.

use iced::theme::Palette;
use iced::widget::{button, row, text, text_input, Button, Row, Text, TextInput};
use iced::{Color, Element, Font, Length, Theme};

// ---- Spacing scale ------------------------------------------------------
pub const SPACE_XS: u16 = 4;
pub const SPACE_SM: u16 = 8;
pub const SPACE_MD: u16 = 12;
pub const SPACE_LG: u16 = 18;
pub const SPACE_XL: u16 = 24;

// ---- Control sizing -----------------------------------------------------
/// Shared height for buttons and single-line inputs so they align in a row.
pub const CONTROL_HEIGHT: f32 = 36.0;
/// Minimum width for action buttons so same-variant buttons line up.
pub const BUTTON_MIN_WIDTH: f32 = 130.0;
/// Internal padding for text inputs.
pub const INPUT_PADDING: u16 = 8;
/// Fixed width for form labels so they form an aligned column.
pub const LABEL_WIDTH: f32 = 130.0;
/// Shared height for all progress bars.
pub const PROGRESS_HEIGHT: f32 = 8.0;

// ---- Typography ---------------------------------------------------------
pub const TEXT_SM: u16 = 13;
pub const TEXT_BODY: u16 = 15;
pub const TEXT_SECTION: u16 = 18;

// ---- Widget builders ----------------------------------------------------

/// A consistently sized button with a centered label and no press handler yet.
pub fn styled_button<'a, M>(label: &'a str) -> Button<'a, M> {
    button(text(label).center())
        .padding([SPACE_XS, SPACE_MD])
        .width(Length::Fixed(BUTTON_MIN_WIDTH))
        .height(Length::Fixed(CONTROL_HEIGHT))
}

/// Primary action button with a consistent size.
pub fn action_button<'a, M: Clone + 'a>(label: &'a str, msg: M) -> Button<'a, M> {
    styled_button(label).on_press(msg)
}

/// Secondary (muted) action button with a consistent size.
pub fn secondary_button<'a, M: Clone + 'a>(label: &'a str, msg: M) -> Button<'a, M> {
    action_button(label, msg).style(button::secondary)
}

/// A text input with consistent padding and body text size. Callers add
/// `.on_input`, `.width`, `.secure`, etc.
pub fn field_input<'a, M: Clone + 'a>(placeholder: &'a str, value: &'a str) -> TextInput<'a, M> {
    text_input(placeholder, value)
        .padding(INPUT_PADDING)
        .size(TEXT_BODY)
}

/// Monospace text at the small size, for template/token strings.
pub fn mono(content: &str) -> Text<'_> {
    text(content).font(Font::MONOSPACE).size(TEXT_SM)
}

/// A label + control row: fixed-width label so labels form an aligned column,
/// with uniform spacing and vertical centering.
pub fn labeled_row<'a, M: 'a>(label: &'a str, control: impl Into<Element<'a, M>>) -> Row<'a, M> {
    row![
        text(label).size(TEXT_BODY).width(Length::Fixed(LABEL_WIDTH)),
        control.into(),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center)
}

// ---- Theme --------------------------------------------------------------

/// The active theme for the given mode, built from a hand-tuned palette.
pub fn theme(dark: bool) -> Theme {
    if dark {
        Theme::custom("Qobuz Dark".to_string(), dark_palette())
    } else {
        Theme::custom("Qobuz Light".to_string(), light_palette())
    }
}

fn dark_palette() -> Palette {
    Palette {
        background: Color::from_rgb8(0x15, 0x17, 0x1c),
        text: Color::from_rgb8(0xe6, 0xe8, 0xec),
        primary: Color::from_rgb8(0x2d, 0x9c, 0xdb),
        success: Color::from_rgb8(0x2e, 0xcc, 0x71),
        danger: Color::from_rgb8(0xe7, 0x4c, 0x3c),
    }
}

fn light_palette() -> Palette {
    Palette {
        background: Color::from_rgb8(0xf7, 0xf8, 0xfa),
        text: Color::from_rgb8(0x1a, 0x1c, 0x22),
        primary: Color::from_rgb8(0x1f, 0x7a, 0xbf),
        success: Color::from_rgb8(0x1e, 0x9e, 0x57),
        danger: Color::from_rgb8(0xc0, 0x39, 0x2b),
    }
}
