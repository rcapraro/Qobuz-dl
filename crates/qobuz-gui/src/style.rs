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
/// Height of compact row-action buttons, so they sit inside a list row.
pub const COMPACT_HEIGHT: f32 = 28.0;
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
/// A page-level heading inside a screen, such as an album title.
pub const TEXT_HEADLINE: u16 = 22;
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
        button::Status::Active => (a.primary(), a.on_accent),
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
/// iced has no minimum width, so a label longer than `BUTTON_MIN_WIDTH` needs
/// the caller to override with `.width(Length::Shrink)` to avoid clipping.
pub fn styled_button<'a, M>(label: impl text::IntoFragment<'a>) -> Button<'a, M> {
    button(text(label).center())
        .padding([SPACE_XS, SPACE_MD])
        .width(Length::Fixed(BUTTON_MIN_WIDTH))
        .height(Length::Fixed(CONTROL_HEIGHT))
        .style(primary_button)
}

/// Primary action button with a consistent size.
pub fn action_button<'a, M: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    msg: M,
) -> Button<'a, M> {
    styled_button(label).on_press(msg)
}

/// Secondary (muted) action button with a consistent size.
pub fn secondary_button<'a, M: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    msg: M,
) -> Button<'a, M> {
    action_button(label, msg).style(button::secondary)
}

/// A small secondary button for list-row actions and small header or inline
/// controls, sized to its label. No press handler yet, so callers can disable
/// it with `on_press_maybe`.
pub fn compact_button<'a, M>(label: impl text::IntoFragment<'a>) -> Button<'a, M> {
    button(text(label).size(TEXT_SM).center())
        .padding([SPACE_XS, SPACE_SM])
        .height(Length::Fixed(COMPACT_HEIGHT))
        .style(button::secondary)
}

/// Makes existing content, such as a cover and title, clickable as it is:
/// no padding and no button chrome. No press handler yet.
pub fn content_button<'a, M>(content: impl Into<Element<'a, M>>) -> Button<'a, M> {
    button(content).padding(0).style(button::text)
}

/// A compact round "?" help toggle sized to sit at the right of a card header.
/// Shows "×" while its help panel is open. Outlined on the card header and
/// filled on hover.
pub fn help_button<'a, M: Clone + 'a>(shown: bool, msg: M) -> Button<'a, M> {
    button(text(if shown { "×" } else { "?" }).center().size(TEXT_BODY))
        .width(Length::Fixed(26.0))
        .height(Length::Fixed(26.0))
        .padding(0)
        .on_press(msg)
        .style(|theme, status| {
            let a = accents(theme);
            let filled = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(Background::Color(if filled {
                    a.surface2
                } else {
                    a.surface0
                })),
                text_color: a.text,
                border: Border {
                    color: a.surface2,
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
    pub surface0: Color,
    pub surface1: Color,
    pub surface2: Color,
    pub text: Color,
    /// Secondary text (Catppuccin `subtext0`): metadata, numbers, captions.
    pub subtext: Color,
    /// Readable text color to place on top of a bright accent fill.
    pub on_accent: Color,
    pub blue: Color,
    pub sky: Color,
    pub lavender: Color,
    pub teal: Color,
    pub green: Color,
    pub yellow: Color,
    pub red: Color,
    pub mauve: Color,
}

/// Each accent carries exactly one meaning across the app. Views ask for a
/// role, never a hue, so that meaning is decided here and nowhere else.
impl Accents {
    pub fn brand(&self) -> Color {
        self.mauve
    }

    /// The primary action and the active selection.
    pub fn primary(&self) -> Color {
        self.blue
    }

    pub fn success(&self) -> Color {
        self.green
    }

    /// Work under way: downloading, tagging, a pending request.
    pub fn progress(&self) -> Color {
        self.yellow
    }

    pub fn error(&self) -> Color {
        self.red
    }

    /// A page's identity panel, such as an opened album's header. Never used
    /// for status, actions, or quality.
    pub fn highlight(&self) -> Color {
        self.lavender
    }

    /// Audio quality (Hi-Res, delivered format) — used by nothing else.
    pub fn quality(&self) -> Color {
        self.teal
    }
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
    surface0: rgb(0x363a4f),
    surface1: rgb(0x494d64),
    surface2: rgb(0x5b6078),
    text: rgb(0xcad3f5),
    subtext: rgb(0xa5adcb),
    on_accent: rgb(0x181926),
    blue: rgb(0x8aadf4),
    sky: rgb(0x91d7e3),
    lavender: rgb(0xb7bdf8),
    teal: rgb(0x8bd5ca),
    green: rgb(0xa6da95),
    yellow: rgb(0xeed49f),
    red: rgb(0xed8796),
    mauve: rgb(0xc6a0f6),
};

const LATTE: Accents = Accents {
    surface0: rgb(0xccd0da),
    surface1: rgb(0xbcc0cc),
    surface2: rgb(0xacb0be),
    text: rgb(0x4c4f69),
    subtext: rgb(0x6c6f85),
    on_accent: rgb(0xeff1f5),
    blue: rgb(0x1e66f5),
    sky: rgb(0x04a5e5),
    lavender: rgb(0x7287fd),
    teal: rgb(0x179299),
    green: rgb(0x40a02b),
    yellow: rgb(0xdf8e1d),
    red: rgb(0xd20f39),
    mauve: rgb(0x8839ef),
};

// ---- iced_aw + container styles ----------------------------------------

/// Right-hand gutter inside scrollables: reserves room so the scrollbar never
/// clips a card's edge or border. There is no left gutter, so scrolled content
/// starts on the same edge as the fixed controls above it.
pub const SCROLLBAR_GUTTER: f32 = SPACE_MD as f32;

/// Secondary text, a step quieter than body text, for metadata, numbers, and
/// captions, so titles carry the visual weight.
pub fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(accents(theme).subtext),
    }
}

/// A raised, bordered region inside a tab, grouping related content the way a
/// card does but without a header.
pub fn surface(theme: &Theme) -> container::Style {
    let a = accents(theme);
    container::Style {
        background: Some(Background::Color(a.surface0)),
        text_color: Some(a.text),
        border: Border {
            color: a.surface2,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    }
}

/// A page's identity block, such as an opened album's header: a soft wash of
/// the highlight accent with a matching border, set apart from neutral
/// [`surface`]s without resembling any status, action, or quality colour.
pub fn hero(theme: &Theme) -> container::Style {
    let a = accents(theme);
    let highlight = a.highlight();
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.12,
            ..highlight
        })),
        text_color: Some(a.text),
        border: Border {
            color: Color {
                a: 0.5,
                ..highlight
            },
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    }
}

/// A table's column-header band: a solid, stronger grey than the [`stripe`]d
/// rows, so it reads as the table's head rather than one of its rows.
pub fn table_head(theme: &Theme) -> container::Style {
    let a = accents(theme);
    container::Style {
        background: Some(Background::Color(a.surface2)),
        text_color: Some(a.text),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Shading for every other row of a table on a [`surface`], so long lists stay
/// easy to follow across their columns.
pub fn stripe(theme: &Theme) -> container::Style {
    let a = accents(theme);
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.45,
            ..a.surface1
        })),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

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
/// Without an accent outline it keeps a neutral border of the same width, so the
/// bar's edge stays visible in the light theme and switching kinds never shifts
/// its content.
pub fn status_surface(theme: &Theme, outline: Option<Color>) -> container::Style {
    let a = accents(theme);
    container::Style {
        background: Some(Background::Color(a.surface0)),
        text_color: Some(a.text),
        border: Border {
            color: outline.unwrap_or(a.surface2),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

/// Tab bar style. The bar sits on the window background: the active tab is a
/// raised `surface1` label outlined in blue, and inactive tabs have no fill.
/// Hover shares the active surface because iced_aw reports `Hovered` instead of
/// `Active` for the selected tab under the pointer, so it must still look raised.
pub fn tab_bar(theme: &Theme, status: Status) -> tab_bar::Style {
    let a = accents(theme);
    let mut base = tab_bar::Style {
        background: None,
        border_color: None,
        border_width: 0.0,
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
            base.tab_label_background = Background::Color(a.surface1);
            base.tab_label_border_color = a.primary();
            base.tab_label_border_width = 1.5;
        }
        Status::Hovered => {
            base.tab_label_background = Background::Color(a.surface1);
        }
        _ => {
            base.tab_label_background = Background::Color(Color::TRANSPARENT);
        }
    }
    base
}

/// Card style: a neutral `surface1` header over a `surface0` body, with a
/// defined border so each section reads as a panel on the window background.
pub fn card(theme: &Theme) -> card::Style {
    let a = accents(theme);
    let surface = Background::Color(a.surface0);
    card::Style {
        background: surface,
        border_radius: 10.0,
        border_width: 1.0,
        border_color: a.surface2,
        head_background: Background::Color(a.surface1),
        head_text_color: a.text,
        body_background: surface,
        body_text_color: a.text,
        foot_background: surface,
        foot_text_color: a.text,
        close_color: a.text,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_map_to_their_hue_in_both_flavors() {
        for a in [LATTE, MACCHIATO] {
            assert_eq!(a.brand(), a.mauve);
            assert_eq!(a.primary(), a.blue);
            assert_eq!(a.success(), a.green);
            assert_eq!(a.progress(), a.yellow);
            assert_eq!(a.error(), a.red);
            assert_eq!(a.quality(), a.teal);
            assert_eq!(a.highlight(), a.lavender);
        }
    }

    #[test]
    fn flavor_follows_theme() {
        assert_eq!(accents(&theme(false)).surface0, LATTE.surface0);
        assert_eq!(accents(&theme(true)).surface0, MACCHIATO.surface0);
    }
}
