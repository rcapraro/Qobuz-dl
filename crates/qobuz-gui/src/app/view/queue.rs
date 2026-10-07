//! The Queue screen: per-track rows with status badges and overall progress.

use super::super::album::guest_performer;
use super::super::tasks::thumbnail;
use super::super::{startable, App, ItemStatus, Message, QueueItem};
use super::{bold, cover, gutter_padding, quality_badge};
use crate::style::{self, compact_button, secondary_button, styled_button};
use iced::widget::{button, column, container, progress_bar, row, scrollable, text};
use iced::{Element, Font, Length};
use iced_aw::widget::badge::Badge;
use qobuz_core::engine::Job;
use qobuz_core::models::Album;

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
        header = header.push(
            secondary_button(format!("Retry failed ({failed})"), Message::RetryFailed)
                .width(Length::Shrink),
        );
    }

    // The queue is known non-empty here — the empty case returned above.
    if !app.downloading {
        header = header.push(secondary_button("Clear queue", Message::ClearQueue));
    }

    // Only while a batch runs. Disabled once cancellation is under way, so it
    // can't be pressed twice while the batch winds down.
    if app.downloading {
        let cancelling = app.cancelling();
        header = header.push(
            styled_button(if cancelling {
                "Cancelling…"
            } else {
                "Cancel"
            })
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

    let mut list = column![].spacing(style::SPACE_MD);
    for group in groups(&app.queue) {
        list = list.push(group_view(app, group));
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

const GROUP_COVER_SIZE: f32 = 40.0;

/// One album's panel: a header summarising the group, its own progress bar,
/// and, unless collapsed, the group's track rows.
fn group_view<'a>(app: &'a App, group: Group<'a>) -> Element<'a, Message> {
    let album = group.album;
    let summary = summarize(&group.items);
    let collapsed = app.collapsed.contains(&album.id);
    let thumb = thumbnail(album.image.as_ref()).and_then(|url| app.thumbnails.get(&url));

    let mut count = row![text(format!("{} / {} done", summary.done, summary.total))
        .size(style::TEXT_SM)
        .style(style::muted_text)];
    if summary.failed > 0 {
        count = count.push(
            text(format!(" · {} failed", summary.failed))
                .size(style::TEXT_SM)
                .style(|theme| text::Style {
                    color: Some(style::accents(theme).error()),
                }),
        );
    }

    let mut head = row![
        compact_button(if collapsed { "▶" } else { "▼" })
            .on_press(Message::ToggleGroup(album.id.clone())),
        cover(thumb, GROUP_COVER_SIZE),
        column![
            text(&album.title).font(bold()),
            text(album.artist_name())
                .size(style::TEXT_SM)
                .style(style::muted_text),
        ]
        .spacing(2)
        .width(Length::Fill),
        count,
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);

    let has_queued = group
        .items
        .iter()
        .any(|it| matches!(it.status, ItemStatus::Queued));
    if has_queued && !app.downloading {
        head = head.push(compact_button("Remove").on_press(Message::RemoveGroup(album.id.clone())));
    }

    let mut body = column![
        head,
        progress_bar(0.0..=1.0, summary.fraction.clamp(0.0, 1.0))
            .height(Length::Fixed(style::PROGRESS_HEIGHT)),
    ]
    .spacing(style::SPACE_SM);
    if !collapsed {
        let mut rows = column![].spacing(style::SPACE_SM);
        for it in &group.items {
            rows = rows.push(queue_row(it, app.downloading));
        }
        // Indented under the header so rows read as belonging to the album.
        body = body.push(container(rows).padding(iced::Padding {
            left: style::SPACE_XL as f32,
            ..iced::Padding::ZERO
        }));
    }

    container(body)
        .style(style::surface)
        .padding(style::SPACE_MD)
        .width(Length::Fill)
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
    progress(queue)
}

/// [`overall_progress`]'s rule over any set of items, so a group's bar follows
/// exactly the same semantics as the whole queue's.
fn progress<'a>(items: impl IntoIterator<Item = &'a QueueItem>) -> f32 {
    let (sum, count) = items.into_iter().fold((0.0, 0usize), |(sum, n), it| {
        (sum + batch_fraction(it), n + 1)
    });
    if count == 0 {
        0.0
    } else {
        sum / count as f32
    }
}

/// One album's tracks in the queue, in queue order.
struct Group<'a> {
    album: &'a Album,
    items: Vec<&'a QueueItem>,
}

/// The queue split by album, groups ordered by their first track. Grouping
/// follows the album id, so a track queued later joins its album's group
/// rather than starting a new one.
fn groups(queue: &[QueueItem]) -> Vec<Group<'_>> {
    let mut groups: Vec<Group<'_>> = Vec::new();
    for it in queue {
        let album = &it.job.album;
        match groups.iter_mut().find(|g| g.album.id == album.id) {
            Some(group) => group.items.push(it),
            None => groups.push(Group {
                album,
                items: vec![it],
            }),
        }
    }
    groups
}

#[derive(Debug, PartialEq)]
struct GroupSummary {
    done: usize,
    failed: usize,
    total: usize,
    fraction: f32,
}

fn summarize(items: &[&QueueItem]) -> GroupSummary {
    let count = |pred: fn(&ItemStatus) -> bool| items.iter().filter(|it| pred(&it.status)).count();
    GroupSummary {
        done: count(|s| matches!(s, ItemStatus::Done(_))),
        failed: count(|s| matches!(s, ItemStatus::Error(_))),
        total: items.len(),
        fraction: progress(items.iter().copied()),
    }
}

/// A row's title within its album group: `"3. Blue in Green"`, plus the
/// performer when it differs from the album artist the group header shows.
fn row_label(job: &Job) -> (String, Option<&str>) {
    let title = match job.track.track_number {
        Some(n) => format!("{n}. {}", job.track.title),
        None => job.track.title.clone(),
    };
    (title, guest_performer(job))
}

/// Background/foreground accent selector for a queue item's status badge.
fn badge_palette(status: &ItemStatus) -> fn(&style::Accents) -> (iced::Color, iced::Color) {
    match status {
        ItemStatus::Queued => |a| (a.surface2, a.text),
        ItemStatus::Downloading | ItemStatus::Tagging => |a| (a.progress(), a.on_accent),
        ItemStatus::Done(_) => |a| (a.success(), a.on_accent),
        ItemStatus::Error(_) => |a| (a.error(), a.on_accent),
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
        ItemStatus::Done(_) => "done".into(),
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

    let (title, guest) = row_label(&it.job);
    let mut label = column![text(title)].spacing(2);
    if let Some(guest) = guest {
        label = label.push(text(guest).size(style::TEXT_SM).style(style::muted_text));
    }
    let mut top = row![label.width(Length::Fill), badge]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);

    if let ItemStatus::Done(delivered) = &it.status {
        top = top.push(quality_badge(delivered.as_str()));
    }

    // A failed track can be relaunched (disabled while a batch is running).
    if matches!(it.status, ItemStatus::Error(_)) {
        let retry = compact_button("Retry")
            .on_press_maybe((!downloading).then_some(Message::RetryTrack(it.track_id)));
        top = top.push(retry);
    }

    // A still-queued track can be removed from the queue (disabled while a
    // batch is running).
    if matches!(it.status, ItemStatus::Queued) {
        let remove = compact_button("Remove")
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
    use super::super::super::album::tests::job;
    use super::super::super::{queue_tab_label, remaining, QueueItem};
    use super::*;

    /// Track `track_id` of album `album_id`, by the album's own artist.
    fn track_in(track_id: i64, album_id: &str, status: ItemStatus) -> QueueItem {
        let mut job = job(track_id, 1, Some("Band"));
        job.album.id = album_id.into();
        QueueItem {
            track_id,
            job,
            status,
            downloaded: 0,
            total: None,
        }
    }

    fn item(total: Option<u64>, downloaded: u64, status: ItemStatus) -> QueueItem {
        QueueItem {
            downloaded,
            total,
            ..track_in(1, "a", status)
        }
    }

    fn group_ids(queue: &[QueueItem]) -> Vec<(String, Vec<i64>)> {
        groups(queue)
            .iter()
            .map(|g| {
                let ids = g.items.iter().map(|it| it.track_id).collect();
                (g.album.id.clone(), ids)
            })
            .collect()
    }

    #[test]
    fn one_album_is_one_group() {
        let queue = [
            track_in(1, "a", ItemStatus::Queued),
            track_in(2, "a", ItemStatus::Queued),
        ];
        assert_eq!(group_ids(&queue), [("a".into(), vec![1, 2])]);
    }

    #[test]
    fn later_track_joins_its_album_group() {
        let queue = [
            track_in(1, "a", ItemStatus::Queued),
            track_in(2, "b", ItemStatus::Queued),
            track_in(3, "a", ItemStatus::Queued),
        ];
        assert_eq!(
            group_ids(&queue),
            [("a".into(), vec![1, 3]), ("b".into(), vec![2])]
        );
    }

    #[test]
    fn playlist_spreads_across_albums() {
        let queue = [
            track_in(1, "x", ItemStatus::Queued),
            track_in(2, "y", ItemStatus::Queued),
            track_in(3, "z", ItemStatus::Queued),
        ];
        assert_eq!(groups(&queue).len(), 3);
    }

    #[test]
    fn group_with_a_failure_settles_to_complete() {
        let queue = [
            track_in(1, "a", done()),
            track_in(2, "a", ItemStatus::Error("x".into())),
        ];
        let group = &groups(&queue)[0];
        assert_eq!(
            summarize(&group.items),
            GroupSummary {
                done: 1,
                failed: 1,
                total: 2,
                fraction: 1.0,
            }
        );
    }

    #[test]
    fn group_progress_counts_only_its_tracks() {
        let queue = [
            track_in(1, "a", done()),
            track_in(2, "b", ItemStatus::Queued),
        ];
        let gs = groups(&queue);
        assert_eq!(summarize(&gs[0].items).fraction, 1.0);
        assert_eq!(summarize(&gs[1].items).fraction, 0.0);
    }

    #[test]
    fn label_has_number_and_no_album_artist() {
        let it = track_in(3, "a", ItemStatus::Queued);
        assert_eq!(row_label(&it.job), ("3. t3".into(), None));
    }

    #[test]
    fn label_without_number_is_the_title() {
        let mut it = track_in(3, "a", ItemStatus::Queued);
        it.job.track.track_number = None;
        assert_eq!(row_label(&it.job).0, "t3");
    }

    #[test]
    fn label_shows_a_guest_performer() {
        let mut it = track_in(3, "a", ItemStatus::Queued);
        it.job.track.performer.as_mut().unwrap().name = Some("Guest".into());
        assert_eq!(row_label(&it.job).1, Some("Guest"));
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

    #[test]
    fn queue_tab_has_no_count_when_empty() {
        assert_eq!(queue_tab_label(&[]), "Queue");
    }

    #[test]
    fn queue_tab_has_no_count_when_all_settled() {
        let queue = vec![
            item(None, 0, done()),
            item(None, 0, ItemStatus::Error("x".into())),
        ];
        assert_eq!(remaining(&queue), 0);
        assert_eq!(queue_tab_label(&queue), "Queue");
    }

    #[test]
    fn queue_tab_counts_queued_downloading_and_tagging() {
        let queue = vec![
            item(None, 0, ItemStatus::Queued),
            item(Some(100), 50, ItemStatus::Downloading),
            item(None, 0, ItemStatus::Tagging),
            item(None, 0, done()),
            item(None, 0, ItemStatus::Error("x".into())),
        ];
        assert_eq!(remaining(&queue), 3);
        assert_eq!(queue_tab_label(&queue), "Queue (3)");
    }
}
