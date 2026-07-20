//! The Queue screen: per-track rows with status badges and overall progress.

use super::super::{App, ItemStatus, Message, QueueItem};
use super::gutter_padding;
use crate::style::{self, styled_button};
use iced::widget::{button, column, progress_bar, row, scrollable, text};
use iced::{Element, Font, Length};
use iced_aw::widget::badge::Badge;

pub(in crate::app) fn queue_view(app: &App) -> Element<'_, Message> {
    let done = app
        .queue
        .iter()
        .filter(|it| matches!(it.status, ItemStatus::Done(_)))
        .count();
    let overall = overall_progress(&app.queue);

    let failed = app
        .queue
        .iter()
        .filter(|it| matches!(it.status, ItemStatus::Error(_)))
        .count();

    let mut header =
        row![text(format!("{done}/{} complete", app.queue.len())).width(Length::Fill),]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center);

    if failed > 0 && !app.downloading {
        // Built manually (not `secondary_button`) so the label can be an
        // owned String rather than a borrowed `&str`.
        header = header.push(
            button(text(format!("Retry failed ({failed})")).center())
                .padding([style::SPACE_XS, style::SPACE_MD])
                .height(Length::Fixed(style::CONTROL_HEIGHT))
                .style(button::secondary)
                .on_press(Message::RetryFailed),
        );
    }

    if !app.queue.is_empty() && !app.downloading {
        header = header.push(
            button(text("Clear queue").center())
                .padding([style::SPACE_XS, style::SPACE_MD])
                .height(Length::Fixed(style::CONTROL_HEIGHT))
                .style(button::secondary)
                .on_press(Message::ClearQueue),
        );
    }

    header = header.push(
        styled_button(if app.downloading {
            "Downloading…"
        } else {
            "Start downloads"
        })
        .on_press_maybe((!app.downloading).then_some(Message::StartDownloads)),
    );

    let mut list = column![].spacing(style::SPACE_SM);
    for it in &app.queue {
        list = list.push(queue_row(it, app.downloading));
    }

    column![
        header,
        progress_bar(0.0..=1.0, overall.clamp(0.0, 1.0))
            .height(Length::Fixed(style::PROGRESS_HEIGHT)),
        scrollable(list.padding(gutter_padding())).height(Length::Fill),
    ]
    .spacing(style::SPACE_MD)
    .into()
}

/// Overall batch progress in `0.0..=1.0`. Byte counts are only meaningful for
/// items whose total size is known — counting bytes of unknown-total items
/// against a denominator that excludes them would overstate progress — so
/// byte-based progress uses known-total items only, falling back to the
/// done-item fraction when no totals are known yet.
fn overall_progress(queue: &[QueueItem]) -> f32 {
    let (total_bytes, got_bytes) = queue
        .iter()
        .fold((0u64, 0u64), |(tb, gb), it| match it.total {
            Some(t) => (tb + t, gb + it.downloaded),
            None => (tb, gb),
        });
    if total_bytes > 0 {
        got_bytes as f32 / total_bytes as f32
    } else if queue.is_empty() {
        0.0
    } else {
        let done = queue
            .iter()
            .filter(|it| matches!(it.status, ItemStatus::Done(_)))
            .count();
        done as f32 / queue.len() as f32
    }
}

/// Background/foreground accent selector for a queue item's status badge.
fn badge_palette(status: &ItemStatus) -> fn(&style::Accents) -> (iced::Color, iced::Color) {
    match status {
        ItemStatus::Queued => |a| (a.surface2, a.text),
        ItemStatus::Downloading => |a| (a.blue, a.on_accent),
        ItemStatus::Tagging => |a| (a.yellow, a.on_accent),
        ItemStatus::Done(_) => |a| (a.green, a.on_accent),
        ItemStatus::Error(_) => |a| (a.red, a.on_accent),
    }
}

fn queue_row(it: &QueueItem, downloading: bool) -> Element<'_, Message> {
    let (status_text, fraction): (String, f32) = match &it.status {
        ItemStatus::Queued => ("queued".into(), 0.0),
        ItemStatus::Downloading => {
            let f = match it.total {
                Some(t) if t > 0 => it.downloaded as f32 / t as f32,
                _ => 0.0,
            };
            // Pad to a constant width so the badge doesn't shift as digits change.
            (format!("downloading {:>3.0}%", f * 100.0), f)
        }
        ItemStatus::Tagging => ("tagging".into(), 1.0),
        ItemStatus::Done(q) => (format!("done · {q}"), 1.0),
        ItemStatus::Error(e) => (format!("error: {e}"), 0.0),
    };

    let pick = badge_palette(&it.status);
    // Monospace so the padded percentage keeps a constant width (the default
    // font's digits vary in width and shift the badge).
    let badge = Badge::new(text(status_text).size(style::TEXT_SM).font(Font::MONOSPACE)).style(
        move |theme, _status| {
            let a = style::accents(theme);
            let (bg, fg) = pick(&a);
            style::badge(bg, fg)
        },
    );

    let mut top = row![text(&it.title).width(Length::Fill), badge]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);

    // A failed track can be relaunched (disabled while a batch is running).
    if matches!(it.status, ItemStatus::Error(_)) {
        let retry = button(text("Retry").size(style::TEXT_SM))
            .padding([style::SPACE_XS, style::SPACE_SM])
            .style(button::secondary)
            .on_press_maybe((!downloading).then_some(Message::RetryTrack(it.track_id)));
        top = top.push(retry);
    }

    // A still-queued track can be removed from the queue (disabled while a
    // batch is running).
    if matches!(it.status, ItemStatus::Queued) {
        let remove = button(text("Remove").size(style::TEXT_SM))
            .padding([style::SPACE_XS, style::SPACE_SM])
            .style(button::secondary)
            .on_press_maybe((!downloading).then_some(Message::DequeueTrack(it.track_id)));
        top = top.push(remove);
    }

    column![
        top,
        progress_bar(0.0..=1.0, fraction.clamp(0.0, 1.0))
            .height(Length::Fixed(style::PROGRESS_HEIGHT)),
    ]
    .spacing(style::SPACE_XS)
    .into()
}

#[cfg(test)]
mod tests {
    use super::super::super::QueueItem;
    use super::*;
    use qobuz_core::engine::Job;

    fn item(total: Option<u64>, downloaded: u64, status: ItemStatus) -> QueueItem {
        let album = qobuz_core::models::Album {
            id: "a".into(),
            title: "album".into(),
            artist: None,
            image: None,
            release_date_original: None,
            genre: None,
            tracks_count: None,
            media_count: None,
            tracks: None,
            label: None,
            hires: false,
            hires_streamable: false,
        };
        let track = qobuz_core::models::Track {
            id: 1,
            title: "t".into(),
            track_number: None,
            media_number: None,
            performer: None,
            composer: None,
            isrc: None,
            parental_warning: None,
            duration: None,
            album: Some(album.clone()),
            hires: false,
            hires_streamable: false,
        };
        QueueItem {
            track_id: 1,
            job: Job {
                track,
                album,
                multi_disc: false,
            },
            title: "t".into(),
            status,
            downloaded,
            total,
        }
    }

    #[test]
    fn unknown_totals_do_not_inflate_progress() {
        // 50 of 100 known bytes plus 999 bytes toward an unknown total must
        // read as 50%, not (50+999)/100.
        let queue = vec![
            item(Some(100), 50, ItemStatus::Downloading),
            item(None, 999, ItemStatus::Downloading),
        ];
        assert_eq!(overall_progress(&queue), 0.5);
    }

    #[test]
    fn empty_queue_is_zero() {
        assert_eq!(overall_progress(&[]), 0.0);
    }

    #[test]
    fn falls_back_to_done_fraction_without_totals() {
        let queue = vec![
            item(None, 0, ItemStatus::Done("FLAC".into())),
            item(None, 0, ItemStatus::Queued),
        ];
        assert_eq!(overall_progress(&queue), 0.5);
    }
}
