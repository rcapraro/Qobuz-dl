//! The Settings screen: credentials, account, file organization, and options.

use super::super::help::{account_help, credentials_help, options_help, template_help};
use super::super::{App, Message, TokenOrigin};
use super::{card_el, gutter_padding};
use crate::style::{
    self, action_button, field_input, labeled_row, secondary_button, styled_button,
};
use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text};
use iced::{Element, Length};
use iced_aw::widget::number_input::NumberInput;
use qobuz_core::quality::Quality;

pub(in crate::app) fn settings_view(app: &App) -> Element<'_, Message> {
    let creds_fields = row![
        field_input("app_id", &app.config.app_id)
            .on_input(Message::AppIdChanged)
            .width(Length::FillPortion(1)),
        field_input("app_secret", &app.config.app_secret)
            .secure(true)
            .on_input(Message::AppSecretChanged)
            .width(Length::FillPortion(1)),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);
    let mut creds_body = column![
        creds_fields,
        row![
            action_button("Auto-detect", Message::AutoDetectCredentials),
            // Wider label than the fixed button width; size to content so it
            // isn't clipped to "Check".
            action_button("Check signing", Message::CheckSigning).width(Length::Shrink),
            text("Fetch app_id and app_secret from the Qobuz web player.").size(style::TEXT_SM),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(style::SPACE_SM);
    if app.show_credentials_help {
        creds_body = creds_body.push(credentials_help());
    }

    let token_status = match &app.token {
        Some(t) => format!(
            "Token: saved in keyring ({}) — {}.",
            masked_token(&t.value),
            match t.origin {
                TokenOrigin::Restored => "restored at startup",
                TokenOrigin::ValidatedThisSession => "validated this session",
            }
        ),
        None => "Token: none saved — paste a user_auth_token below and press Sign in.".to_string(),
    };
    let can_sign_in = !app.token_input.trim().is_empty();
    let mut auth_body = column![
        text(token_status).size(style::TEXT_SM),
        row![
            field_input("paste your user_auth_token", &app.token_input)
                .secure(true)
                .on_input(Message::TokenChanged)
                .width(Length::Fill),
            styled_button("Sign in").on_press_maybe(can_sign_in.then_some(Message::LoginToken)),
            styled_button("Sign out")
                .style(button::secondary)
                .on_press_maybe(app.signed_in().then_some(Message::SignOut)),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(style::SPACE_SM);
    if app.show_account_help {
        auth_body = auth_body.push(account_help());
    }

    let dir_row = labeled_row(
        "Download to:",
        row![
            text(app.config.download_dir.display().to_string()).width(Length::Fill),
            secondary_button("Choose…", Message::PickDir),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center),
    );

    let options_controls = row![
        text("Quality:"),
        pick_list(
            Quality::ALL.to_vec(),
            Some(app.config.quality),
            Message::QualitySelected,
        ),
        checkbox("Embed cover art", app.config.embed_art).on_toggle(Message::EmbedArtToggled),
        iced::widget::horizontal_space(),
        text("Concurrency:"),
        NumberInput::new(&app.config.concurrency, 1..=16, Message::ConcurrencyChanged)
            .step(1)
            .width(Length::Fixed(120.0)),
    ]
    .spacing(style::SPACE_MD)
    .align_y(iced::Alignment::Center);
    let mut options_body = column![options_controls].spacing(style::SPACE_SM);
    if app.show_options_help {
        options_body = options_body.push(options_help());
    }

    let preview = template_preview(app);
    let mut org_body = column![
        dir_row,
        labeled_row(
            "Folder:",
            field_input("folder format", &app.config.folder_format)
                .on_input(Message::FolderFormatChanged),
        ),
        labeled_row(
            "Track:",
            field_input("track format", &app.config.track_format)
                .on_input(Message::TrackFormatChanged),
        ),
        container(text(preview).size(style::TEXT_SM)).padding([style::SPACE_XS, 0]),
    ]
    .spacing(style::SPACE_SM);
    if app.show_template_help {
        org_body = org_body.push(template_help());
    }

    scrollable(
        column![
            help_card(
                "API credentials",
                creds_body,
                |a| a.mauve,
                app.show_credentials_help,
                Message::ToggleCredentialsHelp
            ),
            help_card(
                "Account",
                auth_body,
                |a| a.green,
                app.show_account_help,
                Message::ToggleAccountHelp
            ),
            help_card(
                "File organization",
                org_body,
                |a| a.teal,
                app.show_template_help,
                Message::ToggleTemplateHelp
            ),
            help_card(
                "Options",
                options_body,
                |a| a.peach,
                app.show_options_help,
                Message::ToggleOptionsHelp
            ),
            action_button("Save settings", Message::SaveSettings),
        ]
        .spacing(style::SPACE_LG)
        .padding(gutter_padding()),
    )
    .into()
}

/// A settings card whose header carries a right-aligned "?" help toggle.
fn help_card<'a>(
    title: &'a str,
    body: impl Into<Element<'a, Message>>,
    head: fn(&style::Accents) -> iced::Color,
    shown: bool,
    toggle: Message,
) -> Element<'a, Message> {
    let header = row![
        text(title).size(style::TEXT_SECTION).width(Length::Fill),
        style::help_button(shown, toggle),
    ]
    .align_y(iced::Alignment::Center);
    card_el(header, body, head)
}

/// Mask a stored token for display: at most the last 4 characters are shown
/// (`••••…k3Zq`), and only when the token is long enough (8+ chars) that the
/// suffix reveals a small fraction. Shorter tokens are fully masked.
fn masked_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() < 8 {
        return "••••••••".to_string();
    }
    let suffix: String = chars[chars.len() - 4..].iter().collect();
    format!("••••…{suffix}")
}

/// A representative rendered path using the current templates.
fn template_preview(app: &App) -> String {
    use qobuz_core::template::{render_path, render_segment, TemplateContext};
    let mut ctx = TemplateContext::new();
    ctx.set("albumartist", "Miles Davis")
        .set("artist", "Miles Davis")
        .set("album", "Kind of Blue")
        .set("title", "So What")
        .set("year", "1959")
        .set("container", app.config.quality.extension().to_uppercase())
        .set("bit_depth", "24")
        .set("sampling_rate", "96")
        .set("explicit", "")
        .with_track_number(1);
    let folder = render_path(&app.config.folder_format, &ctx).join("/");
    let file = render_segment(&app.config.track_format, &ctx);
    format!(
        "Preview: {}/{}.{}",
        folder,
        file,
        app.config.quality.extension()
    )
}

#[cfg(test)]
mod tests {
    use super::masked_token;

    #[test]
    fn long_token_shows_only_last_four() {
        assert_eq!(masked_token("abcdefghijk3Zq"), "••••…k3Zq");
    }

    #[test]
    fn short_token_is_fully_masked() {
        assert_eq!(masked_token("abcdefg"), "••••••••");
        assert_eq!(masked_token("abc"), "••••••••");
    }

    #[test]
    fn empty_token_is_fully_masked() {
        assert_eq!(masked_token(""), "••••••••");
    }
}
