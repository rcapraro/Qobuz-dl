//! Centralized visual design system: spacing/sizing/typography constants,
//! reusable widget builders, and the light/dark theme palettes.
//!
//! All screens draw their spacing, control sizes, and colors from here so the
//! UI reads as one consistent system.

use iced::widget::{button, container, row, text, text_input, Button, Row, Text, TextInput};
use iced::{Background, Border, Color, Element, Font, Length, Theme};
use iced_aw::style::{badge, card, tab_bar, Status};

// ---- Spacing scale ------------------------------------------------------
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 18.0;
pub const SPACE_XL: f32 = 24.0;

// ---- Control sizing -----------------------------------------------------
/// Shared height for buttons and single-line inputs so they align in a row.
pub const CONTROL_HEIGHT: f32 = 36.0;
/// Minimum width for action buttons so same-variant buttons line up.
pub const BUTTON_MIN_WIDTH: f32 = 130.0;
/// Height of compact row-action buttons, so they sit inside a list row.
pub const COMPACT_HEIGHT: f32 = 28.0;
/// Internal padding for text inputs.
pub const INPUT_PADDING: f32 = 8.0;
/// Fixed width for form labels so they form an aligned column.
pub const LABEL_WIDTH: f32 = 130.0;
/// Shared height for all progress bars.
pub const PROGRESS_HEIGHT: f32 = 8.0;

// ---- Typography ---------------------------------------------------------
pub const TEXT_SM: f32 = 13.0;
pub const TEXT_BODY: f32 = 15.0;
pub const TEXT_SECTION: f32 = 18.0;
/// A page-level heading inside a screen, such as an album title.
pub const TEXT_HEADLINE: f32 = 22.0;
/// App title in the header (the "Qobuz-dl" wordmark).
pub const TEXT_TITLE: f32 = 26.0;

// ---- Widget builders ----------------------------------------------------

/// Primary-button style with a high-contrast label. Uses the accent blue with
/// `on_accent` text (the palette's designated readable-on-accent color), which
/// reads strongly in both light and dark flavors — unlike iced's default.
pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let a = accents(theme);
    let (bg, fg) = match status {
        button::Status::Hovered | button::Status::Pressed => (a.primary_hover, a.on_accent),
        button::Status::Disabled => (a.band, a.subtext),
        button::Status::Active => (a.primary_fill, a.on_accent),
    };
    button_style(bg, fg, Color::TRANSPARENT)
}

/// How much the primary blue tints a [`secondary`] button's panel fill, at
/// rest and hovered. Both keep the label at 4.5:1 or more in each flavor.
const SECONDARY_TINT: f32 = 0.12;
const SECONDARY_TINT_HOVER: f32 = 0.22;

/// An outlined button for secondary actions: the panel color tinted with the
/// primary blue, a blue outline and the flavor's text color. The outline is
/// what sets it apart from the card it sits on (a fill alone stays under the
/// 3:1 a control needs), and the shared blue ties it to the primary button,
/// which stays the only solid one.
pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let a = accents(theme);
    let tint = |share| mix(a.panel, a.primary(), share);
    match status {
        button::Status::Active | button::Status::Pressed => {
            button_style(tint(SECONDARY_TINT), a.text, a.primary())
        }
        button::Status::Hovered => button_style(tint(SECONDARY_TINT_HOVER), a.text, a.primary()),
        button::Status::Disabled => button_style(a.panel, a.subtext, a.surface2),
    }
}

fn button_style(background: Color, text_color: Color, outline: Color) -> button::Style {
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: outline,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

/// `from` moved `t` of the way toward `to`, keeping `from`'s opacity.
fn mix(from: Color, to: Color, t: f32) -> Color {
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    Color {
        r: lerp(from.r, to.r),
        g: lerp(from.g, to.g),
        b: lerp(from.b, to.b),
        a: from.a,
    }
}

/// A consistently sized button with a centered label and no press handler yet.
/// iced has no minimum width, so a label longer than `BUTTON_MIN_WIDTH` needs
/// the caller to override with `.width(Length::Shrink)` to avoid clipping.
pub fn styled_button<'a, M>(label: impl text::IntoFragment<'a>) -> Button<'a, M> {
    sized_button(text(label).center())
}

/// A [`styled_button`] that fills the width it is given, its label centered
/// across it, for a button sized by its container rather than its label. A
/// filling button doesn't stretch a label that only shrinks to fit, which
/// would leave it at the left.
pub fn fill_button<'a, M>(label: impl text::IntoFragment<'a>) -> Button<'a, M> {
    sized_button(text(label).center().width(Length::Fill)).width(Length::Fill)
}

fn sized_button<'a, M>(label: Text<'a>) -> Button<'a, M> {
    button(label)
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
    action_button(label, msg).style(secondary)
}

/// A small secondary button for list-row actions and small header or inline
/// controls, sized to its label. No press handler yet, so callers can disable
/// it with `on_press_maybe`.
pub fn compact_button<'a, M>(label: impl text::IntoFragment<'a>) -> Button<'a, M> {
    button(text(label).size(TEXT_SM).center())
        .padding([SPACE_XS, SPACE_SM])
        .height(Length::Fixed(COMPACT_HEIGHT))
        .style(secondary)
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
                background: Some(Background::Color(if filled { a.surface2 } else { a.panel })),
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

/// The subset of a Catppuccin flavor we paint with, plus the few choices that
/// differ by flavor so every pairing keeps WCAG contrast (4.5:1 for text, 3:1
/// for a control's edge). Latte's accents are mid-tones that white text barely
/// reads on, and its `surface0` is dark enough to crowd its own text.
#[derive(Clone, Copy)]
pub struct Accents {
    pub surface1: Color,
    pub surface2: Color,
    /// A card or panel body. Latte uses `mantle`, Catppuccin's secondary-pane
    /// background, over `surface0`.
    pub panel: Color,
    /// A band over a panel: a card's header, the active tab, a table's head.
    pub band: Color,
    pub text: Color,
    /// Secondary text: metadata, numbers, captions. `subtext0`, but `subtext1`
    /// in Latte, where `subtext0` reads at only 4:1 on its panels.
    pub subtext: Color,
    /// Readable text color to place on top of a bright accent fill.
    pub on_accent: Color,
    /// The primary button's fill and hover: blue, lightening to sky in the
    /// dark flavor, and blue deepened toward the text color in Latte, where
    /// sky under light text reads at 2.5:1.
    pub primary_fill: Color,
    pub primary_hover: Color,
    /// The success, progress and error roles as text: their accent in the
    /// dark flavor, and in Latte that accent deepened toward `text` just
    /// enough to read at 4.5:1 on a panel and on the window.
    pub success_text: Color,
    pub progress_text: Color,
    pub error_text: Color,
    /// Whether badges are a solid accent under dark text (dark flavor) or a
    /// wash of it under the flavor's text (Latte, whose accents are too light
    /// for either text color).
    pub solid_badges: bool,
    pub blue: Color,
    pub sapphire: Color,
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

    /// Help and explanatory panels — used by nothing else. Not sky, which is
    /// the primary button's hover colour.
    pub fn info(&self) -> Color {
        self.sapphire
    }
}

/// How much of the accent a Latte badge's wash holds over the panel; every
/// accent keeps the text at 4.5:1 or more on it.
const BADGE_WASH: f32 = 0.2;

/// A badge in the given role's accent, readable in both flavors: solid with
/// dark text in Macchiato, a wash under the flavor's text in Latte.
pub fn role_badge(theme: &Theme, accent: Color) -> badge::Style {
    let a = accents(theme);
    if a.solid_badges {
        badge(accent, a.on_accent)
    } else {
        badge(mix(a.panel, accent, BADGE_WASH), a.text)
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
    surface1: rgb(0x494d64),
    surface2: rgb(0x5b6078),
    panel: rgb(0x363a4f),
    band: rgb(0x494d64),
    text: rgb(0xcad3f5),
    subtext: rgb(0xa5adcb),
    on_accent: rgb(0x181926),
    primary_fill: rgb(0x8aadf4),
    primary_hover: rgb(0x91d7e3),
    success_text: rgb(0xa6da95),
    progress_text: rgb(0xeed49f),
    error_text: rgb(0xed8796),
    solid_badges: true,
    blue: rgb(0x8aadf4),
    sapphire: rgb(0x7dc4e4),
    lavender: rgb(0xb7bdf8),
    teal: rgb(0x8bd5ca),
    green: rgb(0xa6da95),
    yellow: rgb(0xeed49f),
    red: rgb(0xed8796),
    mauve: rgb(0xc6a0f6),
};

const LATTE: Accents = Accents {
    surface1: rgb(0xbcc0cc),
    surface2: rgb(0xacb0be),
    panel: rgb(0xe6e9ef),
    band: rgb(0xccd0da),
    text: rgb(0x4c4f69),
    subtext: rgb(0x5c5f77),
    on_accent: rgb(0xeff1f5),
    // Deepened toward `text`: blue 8 % and 25 %, green 55 %, yellow 66 %,
    // red 20 %.
    primary_fill: rgb(0x2264ea),
    primary_hover: rgb(0x2a60d2),
    success_text: rgb(0x47734d),
    progress_text: rgb(0x7e644f),
    error_text: rgb(0xb71c43),
    solid_badges: false,
    blue: rgb(0x1e66f5),
    sapphire: rgb(0x209fb5),
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
pub const SCROLLBAR_GUTTER: f32 = SPACE_MD;

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
        background: Some(Background::Color(a.panel)),
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

/// A help panel opened inside a card: a soft wash of the info accent with a
/// matching border, so explanations read apart from the form around them.
pub fn help_panel(theme: &Theme) -> container::Style {
    let a = accents(theme);
    let info = a.info();
    container::Style {
        background: Some(Background::Color(Color { a: 0.12, ..info })),
        text_color: Some(a.text),
        border: Border {
            color: Color { a: 0.6, ..info },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

/// A table's column-header band, solid and stronger than the [`stripe`]d
/// rows, so it reads as the table's head rather than one of its rows.
pub fn table_head(theme: &Theme) -> container::Style {
    let a = accents(theme);
    container::Style {
        background: Some(Background::Color(a.band)),
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
        background: Some(Background::Color(a.panel)),
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
/// raised band-colored label outlined in blue, and inactive tabs have no fill.
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
            base.tab_label_background = Background::Color(a.band);
            base.tab_label_border_color = a.primary();
            base.tab_label_border_width = 1.5;
        }
        Status::Hovered => {
            base.tab_label_background = Background::Color(a.band);
        }
        _ => {
            base.tab_label_background = Background::Color(Color::TRANSPARENT);
        }
    }
    base
}

/// Card style: a band-colored header over a panel-colored body, with a
/// defined border so each section reads as a panel on the window background.
pub fn card(theme: &Theme) -> card::Style {
    let a = accents(theme);
    let surface = Background::Color(a.panel);
    card::Style {
        background: surface,
        border_radius: 10.0,
        border_width: 1.0,
        border_color: a.surface2,
        head_background: Background::Color(a.band),
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
            assert_eq!(a.info(), a.sapphire);
        }
    }

    #[test]
    fn flavor_follows_theme() {
        assert_eq!(accents(&theme(false)).panel, LATTE.panel);
        assert_eq!(accents(&theme(true)).panel, MACCHIATO.panel);
    }

    /// WCAG 2 contrast ratio between two opaque colors.
    fn contrast(a: Color, b: Color) -> f32 {
        let channel = |c: f32| {
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let lum = |c: Color| 0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b);
        let (hi, lo) = (lum(a).max(lum(b)), lum(a).min(lum(b)));
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn text_and_controls_keep_their_contrast_in_both_flavors() {
        const TEXT: f32 = 4.5;
        const CONTROL: f32 = 3.0;
        for a in [LATTE, MACCHIATO] {
            let pairs = [
                ("text on panel", a.text, a.panel, TEXT),
                ("text on band", a.text, a.band, TEXT),
                ("muted text on panel", a.subtext, a.panel, TEXT),
                ("primary label", a.on_accent, a.primary_fill, TEXT),
                ("primary label hovered", a.on_accent, a.primary_hover, TEXT),
                (
                    "secondary label",
                    a.text,
                    mix(a.panel, a.primary(), SECONDARY_TINT),
                    TEXT,
                ),
                (
                    "secondary label hovered",
                    a.text,
                    mix(a.panel, a.primary(), SECONDARY_TINT_HOVER),
                    TEXT,
                ),
                ("secondary outline", a.primary(), a.panel, CONTROL),
                ("success text", a.success_text, a.panel, TEXT),
                ("progress text", a.progress_text, a.panel, TEXT),
                ("error text", a.error_text, a.panel, TEXT),
            ];
            for (what, fg, bg, need) in pairs {
                let ratio = contrast(fg, bg);
                assert!(ratio >= need, "{what}: {ratio:.2} < {need}");
            }
            for accent in [a.success(), a.progress(), a.error(), a.quality()] {
                let (bg, fg) = if a.solid_badges {
                    (accent, a.on_accent)
                } else {
                    (mix(a.panel, accent, BADGE_WASH), a.text)
                };
                assert!(contrast(fg, bg) >= TEXT, "badge {accent:?}");
            }
        }
    }
}
