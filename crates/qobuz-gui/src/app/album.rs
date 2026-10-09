//! An album opened from search: its resolved jobs, the track selection, and the
//! pure helpers the detail view renders from.

use super::AlbumResult;
use qobuz_core::engine::Job;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(super) enum DetailState {
    Loading,
    Failed(String),
    /// One job per track in album order, ready to enqueue as-is.
    Loaded(Vec<Job>),
}

#[derive(Debug, Clone)]
pub(super) struct AlbumDetail {
    /// The search row it was opened from, shown while the album loads.
    pub(super) header: AlbumResult,
    pub(super) state: DetailState,
    /// Checked track ids.
    pub(super) selected: HashSet<i64>,
}

impl AlbumDetail {
    pub(super) fn loading(header: AlbumResult) -> Self {
        Self {
            header,
            state: DetailState::Loading,
            selected: HashSet::new(),
        }
    }

    pub(super) fn id(&self) -> &str {
        &self.header.id
    }

    pub(super) fn jobs(&self) -> &[Job] {
        match &self.state {
            DetailState::Loaded(jobs) => jobs,
            _ => &[],
        }
    }

    /// Every track starts checked: the common case is the whole album, minus
    /// a few.
    pub(super) fn loaded(&mut self, jobs: Vec<Job>) {
        self.selected = jobs.iter().map(|j| j.track.id).collect();
        self.state = DetailState::Loaded(jobs);
    }

    pub(super) fn toggle(&mut self, track_id: i64) {
        if !self.selected.remove(&track_id) {
            self.selected.insert(track_id);
        }
    }

    pub(super) fn select_all(&mut self) {
        self.selected = self.jobs().iter().map(|j| j.track.id).collect();
    }

    pub(super) fn select_none(&mut self) {
        self.selected.clear();
    }
}

/// Jobs grouped by disc, each disc's tracks in album order. One group means a
/// single-disc album, which the view renders without disc headings.
pub(super) fn discs(jobs: &[Job]) -> Vec<(u32, Vec<&Job>)> {
    let mut groups: Vec<(u32, Vec<&Job>)> = Vec::new();
    for job in jobs {
        let disc = job.track.disc_number();
        match groups.iter_mut().find(|(d, _)| *d == disc) {
            Some((_, tracks)) => tracks.push(job),
            None => groups.push((disc, vec![job])),
        }
    }
    groups.sort_by_key(|(disc, _)| *disc);
    groups
}

/// The checked jobs, in album order.
pub(super) fn selected_jobs(jobs: &[Job], selected: &HashSet<i64>) -> Vec<Job> {
    jobs.iter()
        .filter(|j| selected.contains(&j.track.id))
        .cloned()
        .collect()
}

/// `m:ss`, or `h:mm:ss` from an hour up.
pub(super) fn format_duration(secs: u32) -> String {
    let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub(super) fn total_duration(jobs: &[Job]) -> u32 {
    jobs.iter().filter_map(|j| j.track.duration).sum()
}

/// The track's performer when it differs from the album artist. Reads the raw
/// name: `Track::artist_name` falls back to "Unknown Artist", which would
/// otherwise show as a guest on every track without a performer.
pub(super) fn guest_performer(job: &Job) -> Option<&str> {
    let performer = job.track.performer.as_ref()?.name.as_deref()?;
    (performer != job.album.artist_name()).then_some(performer)
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use qobuz_core::models::{Album, ArtistRef, Track};

    pub(in crate::app) fn album(artist: &str) -> Album {
        Album {
            id: "al".into(),
            title: "Album".into(),
            artist: Some(ArtistRef {
                id: None,
                name: Some(artist.into()),
            }),
            image: None,
            release_date_original: None,
            genre: None,
            tracks_count: None,
            media_count: None,
            tracks: None,
            label: None,
            copyright: None,
            upc: None,
            hires: false,
            hires_streamable: false,
        }
    }

    pub(in crate::app) fn job(id: i64, disc: u32, performer: Option<&str>) -> Job {
        Job {
            track: Track {
                id,
                title: format!("t{id}"),
                track_number: Some(id as u32),
                media_number: Some(disc),
                performer: performer.map(|name| ArtistRef {
                    id: None,
                    name: Some(name.into()),
                }),
                composer: None,
                isrc: None,
                parental_warning: None,
                duration: Some(100),
                album: None,
                hires: false,
                hires_streamable: false,
            },
            album: album("Band"),
            multi_disc: disc > 1,
            disc_track_total: None,
        }
    }

    fn ids(jobs: &[&Job]) -> Vec<i64> {
        jobs.iter().map(|j| j.track.id).collect()
    }

    #[test]
    fn single_disc_is_one_group() {
        let jobs = [job(1, 1, None), job(2, 1, None)];
        let groups = discs(&jobs);
        assert_eq!(groups.len(), 1);
        assert_eq!(ids(&groups[0].1), [1, 2]);
    }

    #[test]
    fn multi_disc_groups_in_disc_order() {
        let jobs = [
            job(1, 1, None),
            job(2, 2, None),
            job(3, 1, None),
            job(4, 2, None),
        ];
        let groups = discs(&jobs);
        assert_eq!(groups.iter().map(|(d, _)| *d).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(ids(&groups[0].1), [1, 3]);
        assert_eq!(ids(&groups[1].1), [2, 4]);
    }

    #[test]
    fn selection_keeps_album_order() {
        let jobs = [job(1, 1, None), job(2, 1, None), job(3, 1, None)];
        let selected = HashSet::from([3, 1]);
        let picked: Vec<i64> = selected_jobs(&jobs, &selected)
            .iter()
            .map(|j| j.track.id)
            .collect();
        assert_eq!(picked, [1, 3]);
    }

    #[test]
    fn durations_format_below_and_above_an_hour() {
        assert_eq!(format_duration(65), "1:05");
        assert_eq!(format_duration(599), "9:59");
        assert_eq!(format_duration(3725), "1:02:05");
    }

    #[test]
    fn guest_performer_only_when_different() {
        assert_eq!(guest_performer(&job(1, 1, Some("Band"))), None);
        assert_eq!(guest_performer(&job(1, 1, Some("Guest"))), Some("Guest"));
        assert_eq!(guest_performer(&job(1, 1, None)), None);
    }

    #[test]
    fn loaded_album_selects_every_track() {
        let header = super::super::AlbumResult {
            id: "al".into(),
            title: "Album".into(),
            artist: "Band".into(),
            cover: None,
            hires: false,
        };
        let mut detail = AlbumDetail::loading(header);
        detail.loaded(vec![job(1, 1, None), job(2, 1, None)]);
        assert_eq!(detail.selected, HashSet::from([1, 2]));
        detail.select_none();
        assert!(detail.selected.is_empty());
        detail.toggle(2);
        assert_eq!(detail.selected, HashSet::from([2]));
        detail.select_all();
        assert_eq!(detail.selected.len(), 2);
    }
}
