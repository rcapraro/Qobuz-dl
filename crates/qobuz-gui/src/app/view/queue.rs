//! The Queue screen: per-track rows with status badges and overall progress.

use super::super::{startable, App, ItemStatus, Message, QueueItem};
use super::gutter_padding;
use crate::style::{self, styled_button};
use iced::widget::{button, column, container, progress_bar, row, scrollable, text};
use iced::{Element, Font, Length};
use iced_aw::widget::badge::Badge;

pub(in crate::app) fn queue_view(app: &App) -> Element<'_, Message> {
    // Nothing to count, nothing to start, nothing to clear: a "0/0 complete"
    // label over an empty bar reads as broken rather than as empty.
    if app.queue.is_empty() {
        return empty_state();
    }

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

    // The queue is known non-empty here — the empty case returned above.
    if !app.downloading {
        header = header.push(
            button(text("Clear queue").center())
                .padding([style::SPACE_XS, style::SPACE_MD])
                .height(Length::Fixed(style::CONTROL_HEIGHT))
                .style(button::secondary)
                .on_press(Message::ClearQueue),
        );
    }

    // Only while a batch runs. Disabled once cancellation is under way, so it
    // can't be pressed twice while the batch winds down.
    if app.downloading {
        let cancelling = app.cancelling();
        header = header.push(
            button(
                text(if cancelling {
                    "Cancelling…"
                } else {
                    "Cancel"
                })
                .center(),
            )
            .padding([style::SPACE_XS, style::SPACE_MD])
            .height(Length::Fixed(style::CONTROL_HEIGHT))
            .style(button::secondary)
            .on_press_maybe((!cancelling).then_some(Message::CancelDownloads)),
        );
    }

    // Offered only when pressing it would start something. `app.downloading`
    // is load-bearing, not defensive: rows leave `Queued` as they start, so by
    // the time the last item is downloading `startable` is already false —
    // without it the button (and with it the "Downloading…" indicator) would
    // blink out before the batch ends.
    if app.downloading || startable(&app.queue) {
        header = header.push(
            styled_button(if app.downloading {
                "Downloading…"
            } else {
                "Start downloads"
            })
            .on_press_maybe((!app.downloading).then_some(Message::StartDownloads)),
        );
    }

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

/// What the Queue screen shows before anything has been added to it.
fn empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            text("Nothing queued yet.").size(style::TEXT_BODY),
            text("Search for an album or paste a Qobuz URL to add tracks.").size(style::TEXT_SM),
        ]
        .spacing(style::SPACE_SM)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// How far one queue item has advanced, in `0.0..=1.0`.
///
/// `settled_on_error` is the one place the batch aggregate and the item's own
/// bar disagree. A failed item is finished — it will not move again without an
/// explicit retry — so it counts as advanced for the overall bar; otherwise a
/// single permanent failure would pin the batch below full forever, which
/// reads as "still working" when nothing is running. Its *own* bar stays empty
/// though: a full bar under a red error badge would be actively misleading.
fn item_fraction(it: &QueueItem, settled_on_error: bool) -> f32 {
    match &it.status {
        ItemStatus::Queued => 0.0,
        ItemStatus::Downloading => match it.total {
            // Clamped per item so one item can't borrow headroom from the rest
            // of the batch: `with_retry` reuses a single progress forwarder
            // across attempts, so one attempt's total can pair with another
            // attempt's byte count.
            Some(t) if t > 0 => (it.downloaded as f32 / t as f32).clamp(0.0, 1.0),
            _ => 0.0,
        },
        ItemStatus::Tagging | ItemStatus::Done(_) => 1.0,
        ItemStatus::Error(_) => {
            if settled_on_error {
                1.0
            } else {
                0.0
            }
        }
    }
}

/// Per-item progress as counted toward the batch aggregate.
fn batch_fraction(it: &QueueItem) -> f32 {
    item_fraction(it, true)
}

/// Per-item progress as rendered on that item's own bar.
fn row_fraction(it: &QueueItem) -> f32 {
    item_fraction(it, false)
}

/// Overall batch progress in `0.0..=1.0`: the share of the *whole* queue that
/// has advanced, averaged over every item.
///
/// Every item counts toward the denominator, including ones that have not
/// started yet. A track's total size is only known once its download response
/// arrives, and `download_all` gates jobs behind a concurrency semaphore — so
/// weighting by known bytes would measure the current concurrency window
/// rather than the batch, reading full while tracks were still queued and then
/// snapping backwards as the next wave started. Terminal items count as
/// complete, which also covers tracks finished by the already-on-disk skip
/// path, where no bytes are ever transferred.
fn overall_progress(queue: &[QueueItem]) -> f32 {
    if queue.is_empty() {
        return 0.0;
    }
    queue.iter().map(batch_fraction).sum::<f32>() / queue.len() as f32
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
    // The badge label is derived from the same fraction the bar renders, so the
    // two can't drift apart.
    let fraction = row_fraction(it);
    let status_text: String = match &it.status {
        ItemStatus::Queued => "queued".into(),
        // Pad to a constant width so the badge doesn't shift as digits change.
        ItemStatus::Downloading => format!("downloading {:>3.0}%", fraction * 100.0),
        ItemStatus::Tagging => "tagging".into(),
        ItemStatus::Done(q) => format!("done · {q}"),
        ItemStatus::Error(e) => format!("error: {e}"),
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

    fn done() -> ItemStatus {
        ItemStatus::Done("FLAC".into())
    }

    #[test]
    fn empty_queue_is_zero() {
        assert_eq!(overall_progress(&[]), 0.0);
    }

    #[test]
    fn pending_items_count_toward_denominator() {
        // The regression test for the reported bug: with a concurrency window
        // smaller than the batch, the not-yet-started tracks have no known
        // total. They must still hold the bar back — 4 of 20 done is 20%, not
        // a full bar.
        let mut queue: Vec<QueueItem> = (0..4).map(|_| item(Some(100), 100, done())).collect();
        queue.extend((0..16).map(|_| item(None, 0, ItemStatus::Queued)));
        assert_eq!(overall_progress(&queue), 0.2);
    }

    #[test]
    fn partial_download_is_averaged() {
        let queue = vec![
            item(Some(100), 50, ItemStatus::Downloading),
            item(None, 0, ItemStatus::Queued),
        ];
        assert_eq!(overall_progress(&queue), 0.25);
    }

    #[test]
    fn skipped_items_count_as_complete() {
        // The already-on-disk skip path finishes a track without emitting any
        // progress event, so it ends done with no total and no bytes.
        let queue = vec![item(None, 0, done())];
        assert_eq!(overall_progress(&queue), 1.0);
    }

    #[test]
    fn failed_items_are_settled() {
        let queue = vec![
            item(None, 0, done()),
            item(None, 0, ItemStatus::Error("x".into())),
        ];
        assert_eq!(overall_progress(&queue), 1.0);
    }

    #[test]
    fn unknown_total_while_downloading_contributes_nothing() {
        // 999 bytes toward an unknown total is not measurable progress.
        let queue = vec![
            item(None, 999, ItemStatus::Downloading),
            item(None, 0, done()),
        ];
        assert_eq!(overall_progress(&queue), 0.5);
    }

    #[test]
    fn row_bar_is_empty_for_failed_item() {
        let failed = item(None, 0, ItemStatus::Error("boom".into()));
        assert_eq!(row_fraction(&failed), 0.0);
        assert_eq!(batch_fraction(&failed), 1.0);
    }

    #[test]
    fn startable_with_a_queued_track() {
        assert!(startable(&[item(None, 0, ItemStatus::Queued)]));
    }

    #[test]
    fn not_startable_when_queue_is_empty() {
        assert!(!startable(&[]));
    }

    #[test]
    fn not_startable_when_all_done() {
        assert!(!startable(&[item(None, 0, done()), item(None, 0, done())]));
    }

    #[test]
    fn not_startable_when_only_failures_remain() {
        // Relaunching failures belongs to the Retry controls, so Start has
        // nothing left to do once every track has been attempted.
        let queue = vec![
            item(None, 0, done()),
            item(None, 0, ItemStatus::Error("x".into())),
        ];
        assert!(!startable(&queue));
    }

    #[test]
    fn startable_when_a_queued_track_sits_beside_failures() {
        let queue = vec![
            item(None, 0, ItemStatus::Queued),
            item(None, 0, ItemStatus::Error("x".into())),
        ];
        assert!(startable(&queue));
    }

    #[test]
    fn not_startable_while_a_lone_item_downloads() {
        // The button stays visible mid-batch through the `app.downloading`
        // arm of the header condition, not through this predicate.
        assert!(!startable(&[item(Some(100), 50, ItemStatus::Downloading)]));
    }

    #[test]
    fn overdownload_is_clamped_per_item() {
        // A retry can pair one attempt's total with another's byte count; the
        // excess must not spill into the other items' share of the bar.
        let queue = vec![
            item(Some(100), 150, ItemStatus::Downloading),
            item(None, 0, ItemStatus::Queued),
        ];
        assert_eq!(overall_progress(&queue), 0.5);
    }
}
