//! Static help panels for the settings cards. Pure content — nothing here
//! reads `App` state.

use super::view::{card, section};
use super::Message;
use crate::style::{self, secondary_button};
use iced::widget::{column, row, text, Column};
use iced::{Element, Length};

/// One `mono token — description` row for a help panel.
fn help_term(token: &'static str, desc: &'static str) -> Element<'static, Message> {
    row![
        style::mono(token).width(Length::Fixed(style::LABEL_WIDTH)),
        text(desc).size(style::TEXT_SM),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

/// A column of small help lines.
fn help_lines(lines: impl IntoIterator<Item = String>) -> Column<'static, Message> {
    lines
        .into_iter()
        .fold(column![].spacing(style::SPACE_XS), |col, line| {
            col.push(text(line).size(style::TEXT_SM))
        })
}

/// DevTools shortcut hint, formatted for the host OS. The `⌥⌘` glyphs are only
/// emitted on macOS (where the system font renders them); other platforms get
/// pure-ASCII `Ctrl+Shift+I`.
fn devtools_shortcut() -> &'static str {
    if cfg!(target_os = "macos") {
        "F12 / ⌥⌘I"
    } else {
        "F12 / Ctrl+Shift+I"
    }
}

/// Help for the API credentials card: what the fields are and how to obtain them.
pub(super) fn credentials_help() -> Element<'static, Message> {
    column![
        help_term("app_id", "Public client id, sent as the x-app-id request header."),
        help_term(
            "app_secret",
            "Secret used to sign track file-URL requests; never sent as-is.",
        ),
        help_lines(["Normally you don't need to do anything here — press \"Auto-detect\" and the app extracts both values from the Qobuz web player for you.".into()]),
        text("Manual fallback (if auto-detect fails)").size(style::TEXT_BODY),
        help_lines([
            "1. Open play.qobuz.com in a browser and sign in.".into(),
            format!("2. Open the browser developer tools ({}) → Network tab.", devtools_shortcut()),
            "3. Reload; in any request to the Qobuz API, read the request header \"x-app-id\" — that is your app_id.".into(),
            "4. In the Sources/Debugger tab, open the player's main JavaScript bundle and search for the app secret (a long hex string used for signing); copy it as app_secret.".into(),
            "If signing later fails, the web player may have rotated these — press Auto-detect again or re-extract them.".into(),
        ]),
    ]
    .spacing(style::SPACE_XS)
    .into()
}

/// Help for the Account card: how to obtain the user_auth_token and sign in.
pub(super) fn account_help() -> Element<'static, Message> {
    column![
        text("Signing in with a token").size(style::TEXT_BODY),
        help_lines([
            "Qobuz sign-in uses your account's user_auth_token (email/password login is not supported — it does not work for partner/bundled accounts such as Qobuz via a telco).".into(),
            "How to get your token from the Qobuz web player:".into(),
            "1. Open play.qobuz.com in a browser and sign in normally.".into(),
            format!("2. Open the browser developer tools ({}) → Network tab.", devtools_shortcut()),
            "3. Reload the page, then click any request to the Qobuz API and read the request header \"x-user-auth-token\".".into(),
            "4. Copy that value, paste it above, and press Sign in.".into(),
            "• The token is stored in your operating system keyring, not in the config file.".into(),
            "• The status line above shows whether a token is saved (masked) and how the session was established.".into(),
            "• The header shows \"● signed in\" once a session is active; Sign out removes the token from the keyring.".into(),
        ]),
    ]
    .spacing(style::SPACE_XS)
    .into()
}

/// Help for the Options card: quality tiers, concurrency, and cover-art embedding.
pub(super) fn options_help() -> Element<'static, Message> {
    column![
        text("Options").size(style::TEXT_BODY),
        help_lines([
            "• Quality: MP3 320 · FLAC 16/44.1 (CD) · FLAC 24/≤96 · FLAC 24/≤192 (Hi-Res). The service may deliver a lower tier than requested; the actual quality is read from the response.".into(),
            "• Concurrency: how many tracks download at once (1–16).".into(),
            "• Embed cover art: writes the album cover into each downloaded file's tags.".into(),
        ]),
    ]
    .spacing(style::SPACE_XS)
    .into()
}

/// Example folder templates offered in the help panel (using real placeholders).
const FOLDER_EXAMPLES: &[&str] = &[
    "{albumartist}/{album} ({year})",
    "{albumartist} - {album} [{container}]",
    "{albumartist}/{album} ({year}) [{bit_depth}B-{sampling_rate}kHz]",
];

/// Example track templates offered in the help panel (using real placeholders).
const TRACK_EXAMPLES: &[&str] = &[
    "{tracknumber:02} - {title}",
    "{tracknumber:02}. {artist} - {title}",
    "{artist} - {title}{explicit}",
];

/// One example row: the template string plus Copy and Apply actions.
fn example_row<'a>(template: &'a str, apply: Message) -> Element<'a, Message> {
    row![
        style::mono(template).width(Length::Fill),
        secondary_button("Copy", Message::CopyTemplate(template.to_string())),
        secondary_button("Apply", apply),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

/// The template-syntax help panel: placeholders, rules, and copyable examples.
pub(super) fn template_help() -> Element<'static, Message> {
    let placeholders = [
        ("{albumartist}", "Album's primary artist"),
        ("{artist}", "Track artist"),
        ("{album}", "Album title"),
        ("{title}", "Track title"),
        ("{year}", "Release year"),
        ("{container}", "Format/extension, e.g. FLAC"),
        ("{bit_depth}", "Bit depth, e.g. 24"),
        ("{sampling_rate}", "Sample rate in kHz, e.g. 96"),
        ("{explicit}", "\" [E]\" for explicit tracks, else empty"),
        ("{composer}", "Composer, when available"),
        ("{tracknumber}", "Track number; pad with {tracknumber:02}"),
    ];
    let mut list = column![].spacing(style::SPACE_XS);
    for (token, desc) in placeholders {
        list = list.push(
            row![
                style::mono(token).width(Length::Fixed(style::LABEL_WIDTH + 40.0)),
                text(desc).size(style::TEXT_SM),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
        );
    }

    let rules = column![
        text("Syntax").size(style::TEXT_BODY),
        help_lines([
            "• Use {placeholder} tokens; unknown tokens render as empty text.".into(),
            "• Zero-pad numbers with {name:0N}, e.g. {tracknumber:02} → 01.".into(),
            "• In the folder format, \"/\" creates nested subfolders.".into(),
            "• Illegal filename characters are replaced automatically.".into(),
        ]),
    ]
    .spacing(style::SPACE_XS);

    let mut folder_ex = column![section("Folder examples")].spacing(style::SPACE_XS);
    for &t in FOLDER_EXAMPLES {
        folder_ex = folder_ex.push(example_row(t, Message::FolderFormatChanged(t.to_string())));
    }
    let mut track_ex = column![section("Track examples")].spacing(style::SPACE_XS);
    for &t in TRACK_EXAMPLES {
        track_ex = track_ex.push(example_row(t, Message::TrackFormatChanged(t.to_string())));
    }

    let body = column![section("Placeholders"), list, rules, folder_ex, track_ex,]
        .spacing(style::SPACE_SM);
    card("Template help", body, |a| a.sky)
}
