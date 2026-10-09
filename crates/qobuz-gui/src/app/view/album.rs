//! An album opened from search: header, selection toolbar, and track list.

use super::super::album::{discs, format_duration, guest_performer, total_duration};
use super::super::album::{AlbumDetail, DetailState};
use super::super::Message;
use super::{bold, cover, gutter_padding, quality_badge};
use crate::style::{self, compact_button, secondary_button, styled_button};
use iced::widget::{checkbox, column, container, image, row, scrollable, space, text};
use iced::{Element, Length};
use qobuz_core::engine::Job;
use qobuz_core::models::Album;
use std::collections::HashMap;

const COVER_SIZE: f32 = 160.0;
// Track-row column widths: a three-digit number, the "Hi-Res" badge, and an
// `h:mm:ss` duration.
const NUMBER_WIDTH: f32 = 32.0;
/// Set explicitly so the column header can reserve the same width.
const CHECKBOX_SIZE: f32 = 16.0;
const BADGE_WIDTH: f32 = 64.0;
const DURATION_WIDTH: f32 = 64.0;

pub(in crate::app) fn album_view<'a>(
    detail: &'a AlbumDetail,
    thumbnails: &HashMap<String, image::Handle>,
) -> Element<'a, Message> {
    let back = row![compact_button("← Back to results").on_press(Message::CloseAlbum)];
    let mut page = column![back, header(detail, thumbnails)].spacing(style::SPACE_MD);

    page = match &detail.state {
        DetailState::Loading => page.push(notice(
            text("Loading tracks…").style(style::muted_text).into(),
        )),
        DetailState::Failed(e) => page.push(notice(
            column![
                text(format!("Could not load this album: {e}")).style(|theme| text::Style {
                    color: Some(style::accents(theme).error_text),
                }),
                secondary_button("Retry", Message::RetryAlbum),
            ]
            .spacing(style::SPACE_SM)
            .align_x(iced::Alignment::Center)
            .into(),
        )),
        DetailState::Loaded(jobs) => page
            .push(selection_bar(detail, jobs.len()))
            .push(track_table(jobs, detail)),
    };
    page.into()
}

/// The album's identity in the brand-tinted hero: cover beside title, artist,
/// details, and quality.
fn header<'a>(
    detail: &'a AlbumDetail,
    thumbnails: &HashMap<String, image::Handle>,
) -> Element<'a, Message> {
    let thumb = detail.header.cover.as_ref().and_then(|u| thumbnails.get(u));
    let jobs = detail.jobs();

    let mut info = column![
        text(&detail.header.title)
            .size(style::TEXT_HEADLINE)
            .font(bold()),
        text(&detail.header.artist).size(style::TEXT_SECTION),
    ]
    .spacing(style::SPACE_XS);
    if let Some(first) = jobs.first() {
        info = info.push(
            text(meta_line(&first.album, jobs))
                .size(style::TEXT_SM)
                .style(style::muted_text),
        );
    }
    if detail.header.hires {
        info = info.push(quality_badge("Hi-Res"));
    }
    container(
        row![cover(thumb, COVER_SIZE), info.width(Length::Fill)]
            .spacing(style::SPACE_XL)
            .align_y(iced::Alignment::Center),
    )
    .style(style::hero)
    .padding(style::SPACE_LG)
    .width(Length::Fill)
    .into()
}

/// A centered message on a surface, in place of the track table.
fn notice(content: Element<'_, Message>) -> Element<'_, Message> {
    container(content)
        .style(style::surface)
        .padding(style::SPACE_XL)
        .center_x(Length::Fill)
        .into()
}

/// `2019 · Label · Genre · 12 tracks · 48:31`, skipping whatever is unknown.
fn meta_line(album: &Album, jobs: &[Job]) -> String {
    let count = album.tracks_count.map_or(jobs.len(), |n| n as usize);
    let tracks = match count {
        1 => "1 track".to_owned(),
        n => format!("{n} tracks"),
    };
    let parts = [
        album.year().map(str::to_owned),
        album.label.as_ref().and_then(|l| l.name.clone()),
        album.genre.as_ref().and_then(|g| g.name.clone()),
        Some(tracks),
        Some(format_duration(total_duration(jobs))),
    ];
    parts.into_iter().flatten().collect::<Vec<_>>().join(" · ")
}

/// What is selected and what to do with it, directly above the table it acts on.
fn selection_bar(detail: &AlbumDetail, total: usize) -> Element<'_, Message> {
    let n = detail.selected.len();
    let label = match n {
        1 => "Add 1 track".to_owned(),
        n => format!("Add {n} tracks"),
    };
    row![
        text(format!("{n} of {total} selected")).style(style::muted_text),
        compact_button("Select all").on_press(Message::SelectAllTracks),
        compact_button("Select none").on_press(Message::SelectNoTracks),
        space::horizontal(),
        styled_button(label)
            .width(Length::Shrink)
            .on_press_maybe((n > 0).then_some(Message::AddSelected)),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

/// The tracks as a table on a surface: a column header, then striped rows,
/// grouped under a heading per disc on multi-disc albums.
fn track_table<'a>(jobs: &'a [Job], detail: &AlbumDetail) -> Element<'a, Message> {
    let groups = discs(jobs);
    let headed = groups.len() > 1;
    let mut rows = column![].spacing(2);
    for (disc, tracks) in groups {
        if headed {
            rows = rows.push(
                container(
                    text(format!("Disc {disc}"))
                        .size(style::TEXT_SM)
                        .font(bold())
                        .style(style::muted_text),
                )
                .padding([style::SPACE_SM, style::SPACE_SM]),
            );
        }
        for (i, job) in tracks.into_iter().enumerate() {
            let row = track_row(job, detail.selected.contains(&job.track.id));
            let mut cell = container(row)
                .padding([style::SPACE_XS, style::SPACE_SM])
                .width(Length::Fill);
            if i % 2 == 1 {
                cell = cell.style(style::stripe);
            }
            rows = rows.push(cell);
        }
    }

    container(
        column![
            // Same gutter and cell padding as the rows, so the band and its
            // labels line up with the striped rows beneath.
            container(
                container(column_header())
                    .style(style::table_head)
                    .padding([style::SPACE_XS, style::SPACE_SM])
                    .width(Length::Fill),
            )
            .padding(gutter_padding()),
            scrollable(rows.padding(gutter_padding())).height(Length::Fill),
        ]
        .spacing(style::SPACE_SM),
    )
    .style(style::surface)
    .padding(style::SPACE_MD)
    .height(Length::Fill)
    .into()
}

/// Column labels laid out on the same grid as [`track_row`].
fn column_header<'a>() -> Element<'a, Message> {
    let label = |s: &'a str| text(s).size(style::TEXT_SM).font(bold());
    row![
        space::horizontal().width(Length::Fixed(CHECKBOX_SIZE)),
        label("#")
            .width(Length::Fixed(NUMBER_WIDTH))
            .align_x(iced::Alignment::End),
        label("Title").width(Length::Fill),
        label("Time")
            .width(Length::Fixed(DURATION_WIDTH))
            .align_x(iced::Alignment::End),
        space::horizontal().width(Length::Fixed(BADGE_WIDTH)),
    ]
    .spacing(style::SPACE_SM)
    .into()
}

fn track_row(job: &Job, checked: bool) -> Element<'_, Message> {
    let track = &job.track;
    let id = track.id;
    // An unchecked track dims, so what will be added reads at a glance.
    let mut title_text = text(&track.title);
    if !checked {
        title_text = title_text.style(style::muted_text);
    }
    let mut title = column![title_text].spacing(2);
    if let Some(guest) = guest_performer(job) {
        title = title.push(text(guest).size(style::TEXT_SM).style(style::muted_text));
    }
    let number = track
        .track_number
        .map(|n| n.to_string())
        .unwrap_or_default();

    // The title takes all free width; duration and badge are fixed columns at
    // the right edge. The badge column is reserved even when empty, so
    // durations stay aligned on rows without one, and long titles wrap
    // instead of pushing either.
    let badge: Element<'_, Message> = if track.is_hires() {
        quality_badge("Hi-Res")
    } else {
        space::horizontal().into()
    };
    row![
        checkbox(checked)
            .size(CHECKBOX_SIZE)
            .spacing(0)
            .on_toggle(move |_| Message::ToggleTrack(id)),
        number_column(number, NUMBER_WIDTH),
        title.width(Length::Fill),
        number_column(
            track.duration.map(format_duration).unwrap_or_default(),
            DURATION_WIDTH
        ),
        container(badge).width(Length::Fixed(BADGE_WIDTH)),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

/// A right-aligned figure in a fixed-width column, in the same Inter as the
/// title: iced can't enable Inter's tabular digits, but right alignment keeps
/// the column's edge straight.
fn number_column<'a>(value: String, width: f32) -> Element<'a, Message> {
    text(value)
        .style(style::muted_text)
        .width(Length::Fixed(width))
        .align_x(iced::Alignment::End)
        .into()
}

#[cfg(test)]
mod tests {
    use super::super::super::album::tests::{album, job};
    use super::*;
    use qobuz_core::models::{Genre, Label};

    #[test]
    fn meta_line_lists_known_parts() {
        let mut a = album("Band");
        a.release_date_original = Some("2019-05-03".into());
        a.label = Some(Label {
            name: Some("Label".into()),
        });
        a.genre = Some(Genre {
            name: Some("Jazz".into()),
        });
        a.tracks_count = Some(12);
        let jobs = [job(1, 1, None), job(2, 1, None)];
        assert_eq!(
            meta_line(&a, &jobs),
            "2019 · Label · Jazz · 12 tracks · 3:20"
        );
    }

    #[test]
    fn meta_line_skips_unknown_parts() {
        let jobs = [job(1, 1, None)];
        assert_eq!(meta_line(&album("Band"), &jobs), "1 track · 1:40");
    }
}
