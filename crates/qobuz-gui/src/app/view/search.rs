//! The Search / Add screen: one field that searches or adds a pasted link.

use super::super::paging::{Kind, Section};
use super::super::{AlbumResult, App, Message, Screen, TrackResult};
use super::album::album_view;
use super::{bold, card_el, cover, gutter_padding, quality_badge};
use crate::style::{self, action_button, compact_button, field_input, styled_button};
use iced::widget::{button, column, container, image, row, scrollable, text, Column};
use iced::{Element, Length};
use qobuz_core::catalog::Reference;
use qobuz_core::config::Config;

/// What still blocks searching. Credentials come first: signing in needs them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetupGap {
    Credentials,
    SignIn,
}

fn setup_gap(config: &Config, signed_in: bool) -> Option<SetupGap> {
    if !config.has_app_credentials() {
        Some(SetupGap::Credentials)
    } else if !signed_in {
        Some(SetupGap::SignIn)
    } else {
        None
    }
}

pub(in crate::app) fn search_view(app: &App) -> Element<'_, Message> {
    let field = row![
        field_input(
            "Search albums and tracks, or paste a Qobuz URL",
            &app.search_query
        )
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::SearchSubmit)
        .width(Length::Fill),
        action_button("Search", Message::SearchSubmit),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);

    let body = match (setup_gap(&app.config, app.signed_in()), &app.album) {
        (Some(gap), _) => setup_prompt(gap),
        (None, Some(album)) => album_view(album, &app.thumbnails),
        (None, None) => results(app),
    };

    column![field, body].spacing(style::SPACE_MD).into()
}

/// The results list's scrollable, addressed to restore its offset when an
/// album's detail closes.
pub(in crate::app) fn results_id() -> scrollable::Id {
    scrollable::Id::new("search-results")
}

/// Shown in place of results until searching can work, so the first thing a
/// new user meets is what to do next rather than a failed request.
fn setup_prompt<'a>(gap: SetupGap) -> Element<'a, Message> {
    let (title, detail) = match gap {
        SetupGap::Credentials => (
            "App credentials needed",
            "Enter or auto-detect your Qobuz app_id and app_secret in Settings.",
        ),
        SetupGap::SignIn => (
            "Sign-in needed",
            "Paste your user_auth_token in Settings to sign in.",
        ),
    };
    container(
        column![
            text(title).size(style::TEXT_SECTION),
            text(detail).size(style::TEXT_SM),
            action_button("Open Settings", Message::Navigate(Screen::Settings)),
        ]
        .spacing(style::SPACE_MD)
        .align_x(iced::Alignment::Center),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// The explicit "add by ID" action for a query that is also a valid bare ID.
fn offered_id_row(reference: &Reference) -> Element<'_, Message> {
    row![
        text(format!(
            "“{}” is also a valid Qobuz {} ID.",
            reference.id(),
            reference.kind()
        ))
        .size(style::TEXT_SM)
        .width(Length::Fill),
        compact_button(format!("Add {} by ID", reference.kind()))
            .on_press(Message::Add(reference.clone())),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

fn results(app: &App) -> Element<'_, Message> {
    let mut results = column![].spacing(style::SPACE_MD);
    if let Some(reference) = &app.offered_id {
        results = results.push(offered_id_row(reference));
    }
    let albums = &app.results.albums;
    if !albums.items.is_empty() {
        let mut rows = column![].spacing(style::SPACE_XS);
        for a in &albums.items {
            let thumb = a.cover.as_ref().and_then(|u| app.thumbnails.get(u));
            rows = rows.push(album_result_row(a, thumb));
        }
        results = results.push(results_card("Albums", albums, Kind::Albums, rows));
    }
    let tracks = &app.results.tracks;
    if !tracks.items.is_empty() {
        let mut rows = column![].spacing(style::SPACE_XS);
        for t in &tracks.items {
            let thumb = t.cover.as_ref().and_then(|u| app.thumbnails.get(u));
            rows = rows.push(track_result_row(t, thumb));
        }
        results = results.push(results_card("Tracks", tracks, Kind::Tracks, rows));
    }

    scrollable(results.padding(gutter_padding()))
        .id(results_id())
        .on_scroll(Message::ResultsScrolled)
        .height(Length::Fill)
        .into()
}

/// `Albums · 25 of 140`, or `Albums · 25` while the total is unknown.
fn section_title<T>(name: &str, section: &Section<T>) -> String {
    let shown = section.items.len();
    match section.total {
        Some(total) => format!("{name} · {shown} of {total}"),
        None => format!("{name} · {shown}"),
    }
}

/// A results card with its count in the header and, while more remain, a
/// "Show more" control at the foot of its rows.
fn results_card<'a, T>(
    name: &str,
    section: &Section<T>,
    kind: Kind,
    rows: Column<'a, Message>,
) -> Element<'a, Message> {
    let mut body = rows;
    if section.has_more() {
        body = body.push(
            styled_button(if section.loading {
                "Loading…"
            } else {
                "Show more"
            })
            .style(button::secondary)
            .on_press_maybe((!section.loading).then_some(Message::ShowMore(kind))),
        );
    }
    card_el(
        text(section_title(name, section)).size(style::TEXT_SECTION),
        body,
    )
}

const THUMB_SIZE: f32 = 52.0;

/// A result row: cover, a bold title over the artist, an optional Hi-Res
/// badge, and an Add button. With `open`, the cover and title become one
/// control, kept separate from Add so opening never enqueues.
fn add_row<'a>(
    cover: Element<'a, Message>,
    title: &'a str,
    artist: &'a str,
    hires: bool,
    reference: Reference,
    open: Option<Message>,
) -> Element<'a, Message> {
    let label = column![text(title).font(bold()), text(artist).size(style::TEXT_SM)].spacing(2);
    let lead = row![cover, label]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);
    let lead: Element<'a, Message> = match open {
        Some(msg) => button(lead)
            .padding(0)
            .style(button::text)
            .on_press(msg)
            .width(Length::Fill)
            .into(),
        None => lead.width(Length::Fill).into(),
    };

    let mut r = row![lead];
    if hires {
        r = r.push(quality_badge("Hi-Res"));
    }
    r.push(compact_button("Add").on_press(Message::Add(reference)))
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center)
        .into()
}

/// A track result row with its album cover thumbnail.
fn track_result_row<'a>(
    track: &'a TrackResult,
    thumb: Option<&image::Handle>,
) -> Element<'a, Message> {
    add_row(
        cover(thumb, THUMB_SIZE),
        &track.title,
        &track.artist,
        track.hires,
        Reference::Track(track.id.clone()),
        None,
    )
}

/// An album result row; its cover and title open the album's detail.
fn album_result_row<'a>(
    album: &'a AlbumResult,
    thumb: Option<&image::Handle>,
) -> Element<'a, Message> {
    add_row(
        cover(thumb, THUMB_SIZE),
        &album.title,
        &album.artist,
        album.hires,
        Reference::Album(album.id.clone()),
        Some(Message::OpenAlbum(album.clone())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(app_id: &str, app_secret: &str) -> Config {
        Config {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
            ..Config::default()
        }
    }

    #[test]
    fn missing_credentials_come_first() {
        assert_eq!(
            setup_gap(&config("", ""), false),
            Some(SetupGap::Credentials)
        );
        assert_eq!(
            setup_gap(&config("123", ""), true),
            Some(SetupGap::Credentials)
        );
    }

    #[test]
    fn signed_out_with_credentials_needs_sign_in() {
        assert_eq!(
            setup_gap(&config("123", "s3cr3t"), false),
            Some(SetupGap::SignIn)
        );
    }

    #[test]
    fn complete_setup_has_no_gap() {
        assert_eq!(setup_gap(&config("123", "s3cr3t"), true), None);
    }

    fn section(n: usize, total: Option<u32>) -> Section<()> {
        Section::first(super::super::super::paging::Page {
            items: vec![(); n],
            total,
        })
    }

    #[test]
    fn title_shows_count_of_total() {
        assert_eq!(
            section_title("Albums", &section(25, Some(140))),
            "Albums · 25 of 140"
        );
    }

    #[test]
    fn title_without_total_shows_count() {
        assert_eq!(section_title("Tracks", &section(3, None)), "Tracks · 3");
    }
}
