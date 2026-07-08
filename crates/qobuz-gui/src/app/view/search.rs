//! The Search / Add screen: search results and add-by-URL.

use super::super::{AlbumResult, App, Message};
use super::{card, gutter_padding, section};
use crate::style::{self, action_button, field_input, secondary_button};
use iced::widget::{column, container, image, row, scrollable, text};
use iced::{Element, Length};
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
        for (id, label) in &app.results.tracks {
            rows = rows.push(result_row(label, Reference::Track(id.clone())));
        }
        results = results.push(card("Tracks", rows, |a| a.green));
    }
    if !app.results.artists.is_empty() {
        let mut rows = column![].spacing(style::SPACE_XS);
        for (id, label) in &app.results.artists {
            rows = rows.push(result_row(label, Reference::Artist(id.clone())));
        }
        results = results.push(card("Artists", rows, |a| a.mauve));
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

/// A result row: optional leading cover, label, and an Add button.
fn add_row<'a>(
    cover: Option<Element<'a, Message>>,
    label: &'a str,
    reference: Reference,
) -> Element<'a, Message> {
    let mut r = row![];
    if let Some(cover) = cover {
        r = r.push(cover);
    }
    r.push(text(label).width(Length::Fill))
        .push(secondary_button("Add", Message::Add(reference)))
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center)
        .into()
}

fn result_row<'a>(label: &'a str, reference: Reference) -> Element<'a, Message> {
    add_row(None, label, reference)
}

/// An album result row with its cover thumbnail (or a placeholder while loading).
fn album_result_row<'a>(
    album: &'a AlbumResult,
    thumb: Option<&image::Handle>,
) -> Element<'a, Message> {
    const SIZE: f32 = 52.0;
    let cover: Element<'a, Message> = match thumb {
        Some(handle) => image(handle.clone())
            .width(Length::Fixed(SIZE))
            .height(Length::Fixed(SIZE))
            .into(),
        None => container(text(""))
            .width(Length::Fixed(SIZE))
            .height(Length::Fixed(SIZE))
            .style(style::thumb_placeholder)
            .into(),
    };
    add_row(
        Some(cover),
        &album.label,
        Reference::Album(album.id.clone()),
    )
}
