//! The Queue screen: per-track rows with status badges and overall progress.

use super::super::album::guest_performer;
use super::super::tasks::thumbnail;
use super::super::{startable, App, EditorSlot, ItemStatus, Message, QueueItem};
use super::{bold, cover, gutter_padding, hidden_button, quality_badge, slot, tag_editor};
use crate::style::{
    self, compact_button, field_input, fill_button, secondary_button, styled_button,
};
use iced::widget::text::Wrapping;
use iced::widget::{column, container, progress_bar, row, scrollable, text, Space};
use iced::{Color, Element, Font, Length};
use iced_aw::widget::badge::Badge;
use qobuz_core::engine::Job;
use qobuz_core::models::Album;
use qobuz_core::rename;

pub(in crate::app) fn queue_view(app: &App) -> Element<'_, Message> {
    // The editor takes the list's place, but downloads of other albums go on,
    // so their progress and Cancel/Start stay above it.
    let editor = match &app.tag_editor {
        Some(EditorSlot::Loading(_)) => Some(tag_editor::loading_view()),
        Some(EditorSlot::Open(editor)) => Some(tag_editor::tag_editor_view(app, editor)),
        None => None,
    };
    if let Some(editor) = editor {
        return column![queue_header(app, true), editor]
            .spacing(style::SPACE_MD)
            .into();
    }

    // Nothing to count, nothing to start, nothing to clear: a "0 / 0 complete"
    // label over an empty bar reads as broken rather than as empty.
    if app.queue.is_empty() {
        return empty_state();
    }

    let mut list = column![].spacing(style::SPACE_MD);
    for group in groups(&app.queue) {
        list = list.push(group_view(app, group));
    }

    column![
        queue_header(app, false),
        scrollable(list.padding(gutter_padding())).height(Length::Fill),
    ]
    .spacing(style::SPACE_MD)
    .into()
}

/// The queue's count, its actions and the overall progress bar. While an
/// album's tags are edited, Retry failed and Clear queue are left out: they
/// would change the queue under the editor.
fn queue_header(app: &App, editing: bool) -> Element<'_, Message> {
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

    // Button height even with no buttons, as while tags are edited, so the
    // editor opens without the count above it moving.
    let mut header =
        row![text(format!("{done} / {} complete", app.queue.len())).width(Length::Fill),]
            .spacing(style::SPACE_SM)
            .height(Length::Fixed(style::CONTROL_HEIGHT))
            .align_y(iced::Alignment::Center);

    if failed > 0 && !app.downloading && !editing {
        header = header.push(
            secondary_button(format!("Retry failed ({failed})"), Message::RetryFailed)
                .width(Length::Shrink),
        );
    }

    // The queue is known non-empty here: the empty case returned before.
    if !app.downloading && !editing {
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
            .style(style::secondary)
            .on_press_maybe((!cancelling).then_some(Message::CancelDownloads)),
        );
    }

    // Offered only when pressing it would start something. `app.downloading`
    // is load-bearing, not defensive: rows leave `Queued` as they start, so by
    // the time the last item is downloading `startable` is already false —
    // without it the button (and with it the "Downloading…" indicator) would
    // blink out before the batch ends.
    if app.downloading || startable(&app.queue) {
        const START: &str = "Start downloads";
        // Sized by the longer label: the fixed button width clipped both.
        header = header.push(slot(
            hidden_button(START),
            fill_button(if app.downloading {
                "Downloading…"
            } else {
                START
            })
            .on_press_maybe((!app.downloading).then_some(Message::StartDownloads)),
        ));
    }

    column![
        header,
        progress_bar(0.0..=1.0, overall.clamp(0.0, 1.0))
            .girth(Length::Fixed(style::PROGRESS_HEIGHT)),
    ]
    .spacing(style::SPACE_MD)
    .into()
}

const GROUP_COVER_SIZE: f32 = 40.0;
// The widest usual content of each fixed slot. A longer value, such as a tier
// name the API fell back to, takes its own length instead.
const QUALITY_SAMPLE: &str = "FLAC 24/176.4";
const COUNT_SAMPLE: &str = "99 / 99 done · 9 failed";
const STATUS_SAMPLE: &str = "Done · FLAC 24/176.4";

/// One album's panel: a header summarising the group, its own progress bar,
/// and, unless collapsed, the group's track rows.
fn group_view<'a>(app: &'a App, group: Group<'a>) -> Element<'a, Message> {
    let album = group.album;
    let summary = summarize(&group.items);
    let quality = group_quality(&group.items);
    let collapsed = app.collapsed.contains(&album.id);
    let thumb = thumbnail(album.image.as_ref()).and_then(|url| app.thumbnails.get(&url));

    let done = format!("{} / {} done", summary.done, summary.total);
    let failed = (summary.failed > 0).then(|| format!(" · {} failed", summary.failed));
    let count_sample = longer(
        &format!("{done}{}", failed.as_deref().unwrap_or_default()),
        COUNT_SAMPLE,
    );
    let mut count = row![text(done)
        .size(style::TEXT_SM)
        .wrapping(Wrapping::None)
        .style(style::muted_text)];
    if let Some(failed) = failed {
        count = count.push(
            text(failed)
                .size(style::TEXT_SM)
                .wrapping(Wrapping::None)
                .style(|theme| text::Style {
                    color: Some(style::accents(theme).error_text),
                }),
        );
    }

    // Both slots are reserved before the first track is done, so the title
    // keeps its width and the header its height as the batch progresses.
    let quality_slot = slot(
        hidden_badge(longer(quality.unwrap_or_default(), QUALITY_SAMPLE), false),
        match quality {
            Some(quality) => quality_badge(quality),
            None => Space::new().into(),
        },
    );
    let title_line = row![
        text(&album.title).font(bold()).width(Length::Fill),
        quality_slot,
        slot(
            text(count_sample)
                .size(style::TEXT_SM)
                .color(Color::TRANSPARENT),
            count
        ),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);

    // The actions sit on the artist line so they never squeeze the title.
    let artist_line = row![
        text(album.artist_name())
            .size(style::TEXT_SM)
            .style(style::muted_text)
            .width(Length::Fill),
        match app.rename.as_ref().filter(|r| r.album_id == album.id) {
            Some(rename) => rename_field(&rename.name),
            None => group_actions(app, &group),
        },
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);

    let head = row![
        compact_button(if collapsed { "▶" } else { "▼" })
            .on_press(Message::ToggleGroup(album.id.clone())),
        cover(thumb, GROUP_COVER_SIZE),
        column![title_line, artist_line]
            .spacing(style::SPACE_SM)
            .width(Length::Fill),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center);

    let mut body = column![
        head,
        progress_bar(0.0..=1.0, summary.fraction.clamp(0.0, 1.0))
            .girth(Length::Fixed(style::PROGRESS_HEIGHT)),
    ]
    .spacing(style::SPACE_SM);
    if !collapsed {
        let mut rows = column![].spacing(style::SPACE_SM);
        for it in &group.items {
            rows = rows.push(queue_row(it, quality));
        }
        // Indented under the header so rows read as belonging to the album.
        body = body.push(container(rows).padding(iced::Padding {
            left: style::SPACE_XL,
            ..iced::Padding::ZERO
        }));
    }

    container(body)
        .style(style::surface)
        .padding(style::SPACE_MD)
        .width(Length::Fill)
        .into()
}

/// Fits a typical `Artist - Album (Year)` name beside the group's artist.
const RENAME_FIELD_WIDTH: f32 = 320.0;

/// A group header's actions on its settled tracks and their folder.
fn group_actions<'a>(app: &'a App, group: &Group<'a>) -> Element<'a, Message> {
    let id = &group.album.id;
    // The Rename folder field's height even while empty, so the header keeps
    // its height when the first action appears or the field opens.
    let mut actions = row![]
        .spacing(style::SPACE_SM)
        .height(Length::Fixed(style::CONTROL_HEIGHT))
        .align_y(iced::Alignment::Center);
    if app.album_folder(id).is_some() {
        actions = actions
            .push(compact_button("Open folder").on_press(Message::OpenAlbumFolder(id.clone())));
    }
    if app.group_renamable(&group.items) {
        actions = actions
            .push(compact_button("Rename folder").on_press(Message::RenameFolder(id.clone())));
    }
    if app.group_editable(group.items.iter().copied()) {
        actions = actions.push(compact_button("Edit tags").on_press(Message::EditTags(id.clone())));
    }
    if group.items.iter().any(|it| app.can_remove(it)) {
        actions = actions.push(compact_button("Remove").on_press(Message::RemoveGroup(id.clone())));
    }
    actions.into()
}

/// The Rename folder field that replaces a group header's actions while open.
fn rename_field(name: &str) -> Element<'_, Message> {
    let valid = rename::folder_name(name).is_some();
    row![
        field_input("folder name", name)
            .on_input(Message::RenameNameChanged)
            .on_submit(Message::ConfirmRename)
            .width(Length::Fixed(RENAME_FIELD_WIDTH)),
        compact_button("Rename")
            .style(style::primary_button)
            .on_press_maybe(valid.then_some(Message::ConfirmRename)),
        compact_button("Cancel").on_press(Message::CancelRename),
    ]
    .spacing(style::SPACE_SM)
    .height(Length::Fixed(style::CONTROL_HEIGHT))
    .align_y(iced::Alignment::Center)
    .into()
}

/// The longer of `value` and `sample`, by character count, for sizing a
/// [`slot`] that must also fit an unusually long value.
fn longer(value: &str, sample: &'static str) -> String {
    if value.chars().count() > sample.chars().count() {
        value.to_owned()
    } else {
        sample.to_owned()
    }
}

/// A badge drawn fully transparent, to size a [`slot`] like a real one.
fn hidden_badge<'a>(label: String, mono: bool) -> Element<'a, Message> {
    let mut label = text(label).size(style::TEXT_SM).color(Color::TRANSPARENT);
    if mono {
        label = label.font(Font::MONOSPACE);
    }
    Badge::new(label)
        .style(|_theme, _status| style::badge(Color::TRANSPARENT, Color::TRANSPARENT))
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
/// A failed item is finished — it will not move again without an explicit
/// retry — so it counts as advanced; otherwise a single permanent failure would
/// pin the batch below full forever, which reads as "still working" when
/// nothing is running. A failed row draws no bar of its own, so this never
/// shows as a full bar under a red badge.
fn item_fraction(it: &QueueItem) -> f32 {
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
        ItemStatus::Tagging | ItemStatus::Done(_) | ItemStatus::Error(_) => 1.0,
    }
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
        (sum + item_fraction(it), n + 1)
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

/// A queue item's status badge: neutral while queued, else its role's accent.
fn badge_style(status: &ItemStatus) -> fn(&iced::Theme) -> iced_aw::style::badge::Style {
    match status {
        ItemStatus::Queued => |theme| {
            let a = style::accents(theme);
            style::badge(a.band, a.text)
        },
        ItemStatus::Downloading | ItemStatus::Tagging => {
            |theme| style::role_badge(theme, style::accents(theme).progress())
        }
        ItemStatus::Done(_) => |theme| style::role_badge(theme, style::accents(theme).success()),
        ItemStatus::Error(_) => |theme| style::role_badge(theme, style::accents(theme).error()),
    }
}

/// The delivered quality most of a group's done tracks share, shown once on its
/// header. Ties go to the label reached first in queue order, so it is stable.
fn group_quality<'a>(items: &[&'a QueueItem]) -> Option<&'a str> {
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for it in items {
        if let ItemStatus::Done(label) = &it.status {
            match counts.iter_mut().find(|(seen, _)| seen == label) {
                Some((_, n)) => *n += 1,
                None => counts.push((label, 1)),
            }
        }
    }
    // `max_by_key` keeps the last of equal maxima; reversing makes that the first.
    counts
        .into_iter()
        .rev()
        .max_by_key(|&(_, n)| n)
        .map(|(label, _)| label)
}

/// A track row: title, its status (with the delivered quality only when it
/// differs from its album's), and a bar only while in progress.
fn queue_row<'a>(it: &'a QueueItem, group_quality: Option<&str>) -> Element<'a, Message> {
    // The badge label is derived from the same fraction the bar renders, so the
    // two can't drift apart.
    let fraction = item_fraction(it);
    let status_text: String = match &it.status {
        ItemStatus::Queued => "Queued".into(),
        // Pad to a constant width so the badge doesn't shift as digits change.
        ItemStatus::Downloading => format!("Downloading {:>3.0}%", fraction * 100.0),
        ItemStatus::Tagging => "Tagging".into(),
        ItemStatus::Done(delivered) if Some(delivered.as_str()) != group_quality => {
            format!("Done · {delivered}")
        }
        ItemStatus::Done(_) => "Done".into(),
        ItemStatus::Error(_) => "Failed".into(),
    };

    let pick = badge_style(&it.status);
    // Monospace so the padded percentage keeps a constant width (the default
    // font's digits vary in width and shift the badge).
    let sample = hidden_badge(longer(&status_text, STATUS_SAMPLE), true);
    let badge = Badge::new(
        text(status_text)
            .size(style::TEXT_SM)
            .font(Font::MONOSPACE)
            .wrapping(Wrapping::None),
    )
    .style(move |theme, _status| pick(theme));

    let (title, guest) = row_label(&it.job);
    let mut first_line = row![text(title)]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);
    // Beside the title and cut at the row's end, so a failure neither makes
    // its row taller nor squeezes the title.
    if let ItemStatus::Error(reason) = &it.status {
        first_line = first_line.push(
            container(
                text(format!("— {reason}"))
                    .size(style::TEXT_SM)
                    .wrapping(Wrapping::None)
                    .style(|theme| text::Style {
                        color: Some(style::accents(theme).error_text),
                    }),
            )
            .width(Length::Fill)
            .clip(true),
        );
    }
    let mut label = column![first_line].spacing(2);
    if let Some(guest) = guest {
        label = label.push(text(guest).size(style::TEXT_SM).style(style::muted_text));
    }
    // A fixed slot, so the title keeps its width as the status changes.
    let top = row![label.width(Length::Fill), slot(sample, badge)]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);

    let bar_slot = Length::Fixed(style::PROGRESS_HEIGHT);
    // An empty slot of the bar's height keeps every row the same height, so the
    // list doesn't shift as tracks start and finish.
    let bar: Element<'a, Message> =
        if matches!(it.status, ItemStatus::Downloading | ItemStatus::Tagging) {
            progress_bar(0.0..=1.0, fraction.clamp(0.0, 1.0))
                .girth(bar_slot)
                .into()
        } else {
            Space::new().height(bar_slot).into()
        };
    column![top, bar].spacing(style::SPACE_XS).into()
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
            path: None,
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

    fn quality_of(statuses: Vec<ItemStatus>) -> Option<String> {
        let queue: Vec<QueueItem> = statuses
            .into_iter()
            .enumerate()
            .map(|(i, s)| track_in(i as i64, "a", s))
            .collect();
        let items: Vec<&QueueItem> = queue.iter().collect();
        group_quality(&items).map(str::to_owned)
    }

    fn done_as(label: &str) -> ItemStatus {
        ItemStatus::Done(label.into())
    }

    #[test]
    fn group_quality_when_all_match() {
        let q = quality_of(vec![done_as("FLAC 24/96"), done_as("FLAC 24/96")]);
        assert_eq!(q.as_deref(), Some("FLAC 24/96"));
    }

    #[test]
    fn group_quality_ignores_a_downgraded_track() {
        let q = quality_of(vec![
            done_as("FLAC 24/96"),
            done_as("FLAC 16/44.1"),
            done_as("FLAC 24/96"),
        ]);
        assert_eq!(q.as_deref(), Some("FLAC 24/96"));
    }

    #[test]
    fn group_quality_none_until_a_track_is_done() {
        let q = quality_of(vec![ItemStatus::Queued, ItemStatus::Error("x".into())]);
        assert_eq!(q, None);
    }

    #[test]
    fn group_quality_tie_goes_to_the_first_label() {
        let q = quality_of(vec![done_as("FLAC 16/44.1"), done_as("FLAC 24/96")]);
        assert_eq!(q.as_deref(), Some("FLAC 16/44.1"));
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
    fn failed_item_counts_as_settled() {
        let failed = item(None, 0, ItemStatus::Error("boom".into()));
        assert_eq!(item_fraction(&failed), 1.0);
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
