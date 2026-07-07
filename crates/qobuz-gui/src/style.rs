//! Centralized visual design system: spacing/sizing/typography constants,
//! reusable widget builders, and the light/dark theme palettes.
//!
//! All screens draw their spacing, control sizes, and colors from here so the
//! UI reads as one consistent system.

use iced::widget::{button, container, row, text, text_input, Button, Row, Text, TextInput};
use iced::{Background, Border, Color, Element, Font, Length, Theme};
use iced_aw::style::{badge, card, tab_bar, Status};

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
/// App title in the header (the "Qobuz-dl" wordmark).
pub const TEXT_TITLE: u16 = 26;

// ---- Widget builders ----------------------------------------------------

/// Primary-button style with a high-contrast label. Uses the accent blue with
/// `on_accent` text (the palette's designated readable-on-accent color), which
/// reads strongly in both light and dark flavors — unlike iced's default.
pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let a = accents(theme);
    let (bg, fg) = match status {
        button::Status::Hovered | button::Status::Pressed => (a.sky, a.on_accent),
        button::Status::Disabled => (a.surface1, a.text),
        button::Status::Active => (a.blue, a.on_accent),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: fg,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

/// A consistently sized button with a centered label and no press handler yet.
pub fn styled_button<'a, M>(label: &'a str) -> Button<'a, M> {
    button(text(label).center())
        .padding([SPACE_XS, SPACE_MD])
        .width(Length::Fixed(BUTTON_MIN_WIDTH))
        .height(Length::Fixed(CONTROL_HEIGHT))
        .style(primary_button)
}

/// Primary action button with a consistent size.
pub fn action_button<'a, M: Clone + 'a>(label: &'a str, msg: M) -> Button<'a, M> {
    styled_button(label).on_press(msg)
}

/// Secondary (muted) action button with a consistent size.
pub fn secondary_button<'a, M: Clone + 'a>(label: &'a str, msg: M) -> Button<'a, M> {
    action_button(label, msg).style(button::secondary)
}

/// A compact round "?" help toggle sized to sit at the right of a card header.
/// Shows "✕" while its help panel is open. Styled to read on the accent header:
/// an `on_accent` outline that fills on hover.
pub fn help_button<'a, M: Clone + 'a>(shown: bool, msg: M) -> Button<'a, M> {
    button(text(if shown { "✕" } else { "?" }).center().size(TEXT_BODY))
        .width(Length::Fixed(26.0))
        .height(Length::Fixed(26.0))
        .padding(0)
        .on_press(msg)
        .style(|theme, status| {
            let a = accents(theme);
            let filled = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                // A muted Catppuccin surface (lighter than `on_accent`) so the
                // icon reads softly on the accent header rather than near-black.
                background: Some(Background::Color(if filled {
                    a.surface1
                } else {
                    a.surface0
                })),
                text_color: a.text,
                border: Border {
                    color: a.on_accent,
                    width: 1.5,
                    radius: 13.0.into(),
                },
                ..button::Style::default()
            }
        })
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
        text(label)
            .size(TEXT_BODY)
            .width(Length::Fixed(LABEL_WIDTH)),
        control.into(),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center)
}

// ---- Theme --------------------------------------------------------------

/// The active theme: Catppuccin Latte in light mode, Macchiato in dark mode.
/// Standard widgets (inputs, buttons, pick lists, progress bars) pick up the
/// flavor automatically; the iced_aw styles below layer on the accent colors.
pub fn theme(dark: bool) -> Theme {
    if dark {
        Theme::CatppuccinMacchiato
    } else {
        Theme::CatppuccinLatte
    }
}

// ---- Catppuccin accent palette -----------------------------------------
// The iced extended palette only exposes background/primary/success/danger, so
// we carry the full set of Catppuccin accents to color sections individually.

/// The subset of a Catppuccin flavor we paint with.
#[derive(Clone, Copy)]
pub struct Accents {
    pub base: Color,
    pub mantle: Color,
    pub surface0: Color,
    pub surface1: Color,
    pub surface2: Color,
    pub text: Color,
    /// Readable text color to place on top of a bright accent fill.
    pub on_accent: Color,
    pub blue: Color,
    pub sky: Color,
    pub teal: Color,
    pub green: Color,
    pub yellow: Color,
    pub peach: Color,
    pub red: Color,
    pub mauve: Color,
}

/// Resolve the accent palette for the active flavor (defaults to Macchiato).
pub fn accents(theme: &Theme) -> Accents {
    match theme {
        Theme::CatppuccinLatte => LATTE,
        _ => MACCHIATO,
    }
}

const fn rgb(hex: u32) -> Color {
    Color::from_rgb(
        ((hex >> 16) & 0xff) as f32 / 255.0,
        ((hex >> 8) & 0xff) as f32 / 255.0,
        (hex & 0xff) as f32 / 255.0,
    )
}

const MACCHIATO: Accents = Accents {
    base: rgb(0x24273a),
    mantle: rgb(0x1e2030),
    surface0: rgb(0x363a4f),
    surface1: rgb(0x494d64),
    surface2: rgb(0x5b6078),
    text: rgb(0xcad3f5),
    on_accent: rgb(0x181926),
    blue: rgb(0x8aadf4),
    sky: rgb(0x91d7e3),
    teal: rgb(0x8bd5ca),
    green: rgb(0xa6da95),
    yellow: rgb(0xeed49f),
    peach: rgb(0xf5a97f),
    red: rgb(0xed8796),
    mauve: rgb(0xc6a0f6),
};

const LATTE: Accents = Accents {
    base: rgb(0xeff1f5),
    mantle: rgb(0xe6e9ef),
    surface0: rgb(0xccd0da),
    surface1: rgb(0xbcc0cc),
    surface2: rgb(0xacb0be),
    text: rgb(0x4c4f69),
    on_accent: rgb(0xeff1f5),
    blue: rgb(0x1e66f5),
    sky: rgb(0x04a5e5),
    teal: rgb(0x179299),
    green: rgb(0x40a02b),
    yellow: rgb(0xdf8e1d),
    peach: rgb(0xfe640b),
    red: rgb(0xd20f39),
    mauve: rgb(0x8839ef),
};

// ---- iced_aw + container styles ----------------------------------------

/// Symmetric horizontal gutter inside scrollables: keeps content centered
/// whether or not a scrollbar is shown, and reserves room on the right so the
/// scrollbar never clips a card's edge or border.
pub const SCROLLBAR_GUTTER: f32 = SPACE_MD as f32;

/// A neutral rounded box shown in place of an album cover while its thumbnail
/// loads (or when none is available).
pub fn thumb_placeholder(theme: &Theme) -> container::Style {
    let a = accents(theme);
    container::Style {
        background: Some(Background::Color(a.surface1)),
        border: Border {
            color: a.surface2,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

/// A subtle raised surface for the status line, set off from the app background.
pub fn status_surface(theme: &Theme) -> container::Style {
    let a = accents(theme);
    container::Style {
        background: Some(Background::Color(a.surface0)),
        text_color: Some(a.text),
        border: Border {
            color: a.surface2,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

/// Tab bar style. Tabs use their own surface (distinct from the cards' surface):
/// the active tab is filled with the blue accent, inactive tabs sit on
/// `surface1`, and hover lifts to `surface2`.
pub fn tab_bar(theme: &Theme, status: Status) -> tab_bar::Style {
    let a = accents(theme);
    let mut base = tab_bar::Style {
        background: Some(Background::Color(a.mantle)),
        border_color: Some(a.surface2),
        border_width: 1.0,
        tab_label_border_width: 0.0,
        tab_label_border_color: Color::TRANSPARENT,
        icon_color: a.text,
        text_color: a.text,
        ..tab_bar::Style::default()
    };
    // The TabBar widget passes Status::Active for the *selected* tab,
    // Status::Hovered on hover, and Status::Disabled for inactive tabs.
    match status {
        Status::Active => {
            base.tab_label_background = Background::Color(a.blue);
            base.text_color = a.on_accent;
        }
        Status::Hovered => {
            base.tab_label_background = Background::Color(a.surface2);
            base.text_color = a.text;
        }
        _ => {
            base.tab_label_background = Background::Color(a.surface1);
            base.text_color = a.text;
        }
    }
    base
}

/// A bordered pane enclosing the active tab's content so the tab area is clearly
/// delimited beneath the tab bar.
pub fn panel(theme: &Theme) -> container::Style {
    let a = accents(theme);
    container::Style {
        background: Some(Background::Color(a.base)),
        text_color: Some(a.text),
        border: Border {
            color: a.surface2,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    }
}

/// Card style: an accent-colored header over a `surface0` body (a shade below
/// the tabs' surface), with a defined border so each section reads as a panel.
pub fn card(a: &Accents, head: Color) -> card::Style {
    let surface = Background::Color(a.surface0);
    card::Style {
        background: surface,
        border_radius: 10.0,
        border_width: 1.0,
        border_color: a.surface2,
        head_background: Background::Color(head),
        head_text_color: a.on_accent,
        body_background: surface,
        body_text_color: a.text,
        foot_background: surface,
        foot_text_color: a.text,
        close_color: a.on_accent,
    }
}

/// Badge style from an explicit background/foreground pair.
pub fn badge(background: Color, text_color: Color) -> badge::Style {
    badge::Style {
        background: Background::Color(background),
        border_radius: Some(6.0),
        border_width: 0.0,
        border_color: None,
        text_color,
    }
}
