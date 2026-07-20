//! The Search / Add screen: search results and add-by-URL.

use super::super::{AlbumResult, App, Message, TrackResult};
use super::{bold, card, gutter_padding, section};
use crate::style::{self, action_button, field_input, secondary_button};
use iced::widget::{column, container, image, row, scrollable, text};
use iced::{Element, Length};
use iced_aw::widget::badge::Badge;
use qobuz_core::catalog::Reference;

pub(in crate::app) fn search_view(app: &App) -> Element<'_, Message> {
    let search_bar = row![
        field_input("search albums, tracks, artists…", &app.search_query)
            .on_input(Message::SearchQueryChanged)
            .on_submit(Message::SearchSubmit)
            .width(Length::Fill),
        action_button("Search", Message::SearchSubmit),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);

    let url_bar = row![
        field_input(
            "paste a Qobuz URL or ID (album / track / playlist)",
            &app.url_input
        )
        .on_input(Message::UrlChanged)
        .on_submit(Message::AddUrl)
        .width(Length::Fill),
        action_button("Add", Message::AddUrl),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);

    let mut results = column![].spacing(style::SPACE_MD);
    if !app.results.albums.is_empty() {
        let mut rows = column![].spacing(style::SPACE_XS);
        for a in &app.results.albums {
            let thumb = a.cover.as_ref().and_then(|u| app.thumbnails.get(u));
            rows = rows.push(album_result_row(a, thumb));
        }
        results = results.push(card("Albums", rows, |a| a.blue));
    }
    if !app.results.tracks.is_empty() {
        let mut rows = column![].spacing(style::SPACE_XS);
        for t in &app.results.tracks {
            let thumb = t.cover.as_ref().and_then(|u| app.thumbnails.get(u));
            rows = rows.push(track_result_row(t, thumb));
        }
        results = results.push(card("Tracks", rows, |a| a.green));
    }

    column![
        section("Add by search"),
        search_bar,
        section("Add by URL / ID"),
        url_bar,
        scrollable(results.padding(gutter_padding())).height(Length::Fill),
    ]
    .spacing(style::SPACE_MD)
    .into()
}

/// A result row: optional leading cover, a bold title with an optional artist
/// subtitle, an optional Hi-Res badge, and an Add button.
fn add_row<'a>(
    cover: Option<Element<'a, Message>>,
    title: &'a str,
    artist: Option<&'a str>,
    hires: bool,
    reference: Reference,
) -> Element<'a, Message> {
    let mut label = column![text(title).font(bold())].spacing(2);
    if let Some(artist) = artist {
        label = label.push(text(artist).size(style::TEXT_SM));
    }

    let mut r = row![];
    if let Some(cover) = cover {
        r = r.push(cover);
    }
    r = r.push(label.width(Length::Fill));
    if hires {
        r = r.push(hires_badge());
    }
    r.push(secondary_button("Add", Message::Add(reference)))
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center)
        .into()
}

/// A small "Hi-Res" quality chip, styled like the queue's status badges.
fn hires_badge<'a>() -> Element<'a, Message> {
    Badge::new(text("Hi-Res").size(style::TEXT_SM))
        .style(|theme, _status| {
            let a = style::accents(theme);
            style::badge(a.teal, a.on_accent)
        })
        .into()
}

/// A 52×52 cover thumbnail, or a placeholder while it loads / when there is no
/// cover. Shared by album and track rows.
fn cover_element<'a>(thumb: Option<&image::Handle>) -> Element<'a, Message> {
    const SIZE: f32 = 52.0;
    match thumb {
        Some(handle) => image(handle.clone())
            .width(Length::Fixed(SIZE))
            .height(Length::Fixed(SIZE))
            .into(),
        None => container(text(""))
            .width(Length::Fixed(SIZE))
            .height(Length::Fixed(SIZE))
            .style(style::thumb_placeholder)
            .into(),
    }
}

/// A track result row with its album cover thumbnail.
fn track_result_row<'a>(
    track: &'a TrackResult,
    thumb: Option<&image::Handle>,
) -> Element<'a, Message> {
    add_row(
        Some(cover_element(thumb)),
        &track.title,
        Some(&track.artist),
        track.hires,
        Reference::Track(track.id.clone()),
    )
}

/// An album result row with its cover thumbnail (or a placeholder while loading).
fn album_result_row<'a>(
    album: &'a AlbumResult,
    thumb: Option<&image::Handle>,
) -> Element<'a, Message> {
    add_row(
        Some(cover_element(thumb)),
        &album.title,
        Some(&album.artist),
        album.hires,
        Reference::Album(album.id.clone()),
    )
}
