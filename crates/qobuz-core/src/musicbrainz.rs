//! Finding an album's release on MusicBrainz and filling tags from it.
//!
//! The release is found by the album's barcode, or failing that by its tracks'
//! ISRCs. Each candidate is ranked by how well it fits the files on disk, and
//! [`fill`] maps the chosen release onto each file's [`TagFields`].

use crate::download::with_retry;
use crate::error::{Error, Result};
use crate::tag_edit::{Field, TagFields, FLAG_ON};
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

const API_BASE: &str = "https://musicbrainz.org/ws/2/";
const COVER_BASE: &str = "https://coverartarchive.org/release/";
/// MusicBrainz asks every client for at most one request per second.
const REQUEST_INTERVAL: Duration = Duration::from_secs(1);
const MAX_CANDIDATES: usize = 5;
/// The share of title words a release found by title must have in common with
/// the album, as [`title_likeness`] measures it.
const MIN_TITLE_LIKENESS: f64 = 0.5;
const MAX_ATTEMPTS: u32 = 3;
const ISRC_PAGE_SIZE: usize = 100;
/// Bounds an ISRC search at the 1 request/s pace: 500 recordings, 5 seconds.
const MAX_ISRC_PAGES: usize = 5;
const RELEASE_INC: &str = "recordings+isrcs+artist-credits+labels+release-groups+genres+\
                           recording-level-rels+work-rels+work-level-rels+artist-rels";

const ISRC_WEIGHT: f64 = 60.0;
const TRACK_COUNT_WEIGHT: f64 = 20.0;
const TITLE_WEIGHT: f64 = 10.0;
/// Heavy enough that another artist's release with the album's title and track
/// count, and no ISRCs to tell them apart, stays below a strong match.
const ARTIST_WEIGHT: f64 = 15.0;
const LABEL_WEIGHT: f64 = 5.0;
const STATUS_WEIGHT: f64 = 5.0;

/// The album as it is on disk: what a release is found by and measured against.
#[derive(Debug, Clone, Default)]
pub struct DiskAlbum {
    pub barcode: Option<String>,
    pub title: String,
    /// The album artist; empty when unknown.
    pub artist: String,
    pub label: Option<String>,
    /// The album's whole track and disc counts; the edited tracks may be fewer.
    pub track_count: Option<u32>,
    pub disc_count: Option<u32>,
    pub tracks: Vec<DiskTrack>,
}

/// One edited file, as matched against a release's tracks.
#[derive(Debug, Clone, Default)]
pub struct DiskTrack {
    pub isrc: Option<String>,
    pub disc: Option<u32>,
    pub number: Option<u32>,
}

impl DiskAlbum {
    fn isrcs(&self) -> BTreeSet<String> {
        self.tracks
            .iter()
            .filter_map(|t| t.isrc.as_deref().map(normalize_isrc))
            .filter(|isrc| !isrc.is_empty())
            .collect()
    }

    fn track_count(&self) -> u32 {
        self.track_count.unwrap_or(self.tracks.len() as u32)
    }
}

/// A release that may be the album, and how well it fits.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub release: Release,
    /// 0–100.
    pub confidence: u8,
}

/// Where a [`lookup`] is, for showing its progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Barcode,
    Isrcs,
    Title,
    /// Reading candidate `n` of `of`, counting from 1.
    Release {
        n: usize,
        of: usize,
    },
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Release {
    #[serde(deserialize_with = "or_default")]
    pub id: String,
    #[serde(deserialize_with = "or_default")]
    pub title: String,
    pub status: Option<String>,
    pub date: Option<String>,
    pub country: Option<String>,
    #[serde(deserialize_with = "or_default")]
    pub artist_credit: Vec<Credit>,
    #[serde(deserialize_with = "or_default")]
    pub label_info: Vec<LabelInfo>,
    #[serde(deserialize_with = "or_default")]
    pub release_group: ReleaseGroup,
    #[serde(deserialize_with = "or_default")]
    pub genres: Vec<Genre>,
    #[serde(deserialize_with = "or_default")]
    pub media: Vec<Medium>,
    /// The whole release's track count; only search results carry it.
    pub track_count: Option<u32>,
    pub disc_count: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Credit {
    #[serde(deserialize_with = "or_default")]
    pub name: String,
    #[serde(deserialize_with = "or_default")]
    pub joinphrase: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LabelInfo {
    pub label: Option<Named>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Named {
    #[serde(deserialize_with = "or_default")]
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ReleaseGroup {
    pub first_release_date: Option<String>,
    #[serde(deserialize_with = "or_default")]
    pub secondary_types: Vec<String>,
    #[serde(deserialize_with = "or_default")]
    pub genres: Vec<Genre>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Genre {
    #[serde(deserialize_with = "or_default")]
    pub name: String,
    #[serde(deserialize_with = "or_default")]
    pub count: u32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Medium {
    #[serde(deserialize_with = "or_default")]
    pub position: u32,
    pub format: Option<String>,
    #[serde(deserialize_with = "or_default")]
    pub track_count: u32,
    #[serde(deserialize_with = "or_default")]
    pub tracks: Vec<ReleaseTrack>,
}

/// MusicBrainz medium formats that never hold the album's audio. Blu-ray and
/// generic DVD are left out: both are also used for audio-only discs.
const NON_AUDIO_FORMATS: [&str; 9] = [
    "DVD-Video",
    "HD-DVD",
    "VCD",
    "SVCD",
    "UMD",
    "VHS",
    "VHD",
    "Betamax",
    "Data CD",
];

impl Medium {
    fn is_audio(&self) -> bool {
        !self
            .format
            .as_deref()
            .is_some_and(|f| NON_AUDIO_FORMATS.contains(&f))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ReleaseTrack {
    #[serde(deserialize_with = "or_default")]
    pub position: u32,
    #[serde(deserialize_with = "or_default")]
    pub title: String,
    #[serde(deserialize_with = "or_default")]
    pub artist_credit: Vec<Credit>,
    #[serde(deserialize_with = "or_default")]
    pub recording: Recording,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Recording {
    #[serde(deserialize_with = "or_default")]
    pub isrcs: Vec<String>,
    #[serde(deserialize_with = "or_default")]
    pub relations: Vec<Relation>,
    /// Only search results carry the releases a recording is on.
    #[serde(deserialize_with = "or_default")]
    pub releases: Vec<Release>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Relation {
    #[serde(rename = "type", deserialize_with = "or_default")]
    pub kind: String,
    pub work: Option<Work>,
    pub artist: Option<Named>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Work {
    #[serde(deserialize_with = "or_default")]
    pub relations: Vec<Relation>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ReleaseSearch {
    #[serde(deserialize_with = "or_default")]
    releases: Vec<Release>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RecordingSearch {
    #[serde(deserialize_with = "or_default")]
    recordings: Vec<Recording>,
    /// How many recordings match in all, over every page.
    #[serde(deserialize_with = "or_default")]
    count: usize,
}

/// MusicBrainz writes `null` for some empty values, which `#[serde(default)]`
/// alone rejects.
fn or_default<'de, D, T>(d: D) -> std::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}

impl Release {
    /// The track count on the release's audio media. A search result lists
    /// only the media holding the matched recording, so its own release-wide
    /// count is used when it has one.
    pub fn total_tracks(&self) -> u32 {
        self.track_count
            .unwrap_or_else(|| self.audio_media().map(|m| m.track_count).sum())
    }

    /// The number of audio media: a deluxe edition's DVD is not a disc of the
    /// album.
    pub fn disc_count(&self) -> u32 {
        self.audio_media().count() as u32
    }

    /// The track count on the audio media a release search lists. Unlike a
    /// recording search, it lists every medium, so a bonus DVD's tracks, which
    /// the release-wide count includes, can be left out.
    fn listed_audio_tracks(&self) -> u32 {
        if self.media.is_empty() {
            return self.track_count.unwrap_or_default();
        }
        self.audio_media().map(|m| m.track_count).sum()
    }

    fn audio_media(&self) -> impl Iterator<Item = &Medium> {
        self.media.iter().filter(|m| m.is_audio())
    }

    /// The audio media with their disc numbers, counting audio media only, as
    /// [`Release::disc_count`] does.
    fn numbered_audio_media(&self) -> impl Iterator<Item = (u32, &Medium)> {
        (1..).zip(self.audio_media())
    }

    fn artist(&self) -> String {
        credit(&self.artist_credit)
    }

    pub fn label(&self) -> Option<&str> {
        self.labels().next()
    }

    fn labels(&self) -> impl Iterator<Item = &str> {
        self.label_info
            .iter()
            .filter_map(|l| l.label.as_ref())
            .map(|l| l.name.as_str())
            .filter(|n| !n.is_empty())
    }

    /// The media's formats, each named once: "CD", "SACD + DVD-Video".
    pub fn formats(&self) -> String {
        let mut formats: Vec<&str> = Vec::new();
        for format in self.media.iter().filter_map(|m| m.format.as_deref()) {
            if !formats.contains(&format) {
                formats.push(format);
            }
        }
        formats.join(" + ")
    }

    fn is_official(&self) -> bool {
        self.status.as_deref() == Some("Official")
    }

    fn isrcs(&self) -> BTreeSet<String> {
        self.media
            .iter()
            .flat_map(|m| &m.tracks)
            .flat_map(|t| &t.recording.isrcs)
            .map(|isrc| normalize_isrc(isrc))
            .collect()
    }

    fn genre(&self) -> Option<&str> {
        top_genre(&self.genres).or_else(|| top_genre(&self.release_group.genres))
    }

    fn date(&self) -> Option<String> {
        [&self.release_group.first_release_date, &self.date]
            .into_iter()
            .flatten()
            .find_map(|d| Field::Date.normalize(d))
    }
}

/// The genre with the most votes; on a tie, the first alphabetically.
fn top_genre(genres: &[Genre]) -> Option<&str> {
    genres
        .iter()
        .filter(|g| !g.name.is_empty())
        .max_by(|a, b| a.count.cmp(&b.count).then(b.name.cmp(&a.name)))
        .map(|g| g.name.as_str())
}

fn credit(credits: &[Credit]) -> String {
    credits
        .iter()
        .map(|c| format!("{}{}", c.name, c.joinphrase))
        .collect::<String>()
        .trim()
        .to_string()
}

/// The composers of the works `recording` performs, each once, in order.
fn composers(recording: &Recording) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    let works = recording
        .relations
        .iter()
        .filter(|r| r.kind == "performance")
        .filter_map(|r| r.work.as_ref());
    for work in works {
        let composers = work
            .relations
            .iter()
            .filter(|r| r.kind == "composer")
            .filter_map(|r| r.artist.as_ref())
            .map(|a| a.name.as_str());
        for name in composers {
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

fn normalize_isrc(isrc: &str) -> String {
    isrc.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// Lowercase words separated by single spaces, for comparing names.
fn normalize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The share of words two titles have in common, 0–1, ignoring case and
/// punctuation: "Hickox Conducts Vaughan Williams" and "Richard Hickox
/// conducts Vaughan Williams" share 4 of their 5 words.
fn title_likeness(a: &str, b: &str) -> f64 {
    let (a, b) = (normalize_name(a), normalize_name(b));
    let a: BTreeSet<&str> = a.split(' ').filter(|w| !w.is_empty()).collect();
    let b: BTreeSet<&str> = b.split(' ').filter(|w| !w.is_empty()).collect();
    let all = a.union(&b).count();
    if all == 0 {
        return 0.0;
    }
    a.intersection(&b).count() as f64 / all as f64
}

/// Words shared by unrelated artists' names, which say nothing about a match.
const ARTIST_FILLER: [&str; 2] = ["the", "and"];

/// How much two artist credits have in common, 0–1, or `None` when either is
/// unknown: the larger share of either's words found in the other, since
/// MusicBrainz often credits more people than Qobuz's album artist. "Richard
/// Hickox" in "Vaughan Williams; London Symphony Orchestra, Richard Hickox" is
/// 1.0.
fn artist_likeness(a: &str, b: &str) -> Option<f64> {
    let (a, b) = (normalize_name(a), normalize_name(b));
    let words = |name: &str| -> BTreeSet<String> {
        name.split(' ')
            .filter(|w| !w.is_empty() && !ARTIST_FILLER.contains(w))
            .map(str::to_string)
            .collect()
    };
    let (a, b) = (words(&a), words(&b));
    if a.is_empty() || b.is_empty() {
        return None;
    }
    let shared = a.intersection(&b).count() as f64;
    Some((shared / a.len() as f64).max(shared / b.len() as f64))
}

/// How well `release` fits `album`, 0–100. A signal either side gives no value
/// for is left out, and the others are weighted among themselves: a release
/// MusicBrainz lists no ISRCs for is not thereby a poor match.
pub fn confidence(release: &Release, album: &DiskAlbum) -> u8 {
    let mut signals = Vec::with_capacity(5);

    let (ours, theirs) = (album.isrcs(), release.isrcs());
    if !ours.is_empty() && !theirs.is_empty() {
        let shared = ours.intersection(&theirs).count();
        signals.push((ISRC_WEIGHT, shared as f64 / ours.len() as f64));
    }

    let our_count = album.track_count().max(1);
    let gap = release.total_tracks().abs_diff(our_count);
    let fit = (1.0 - f64::from(gap) / f64::from(our_count)).max(0.0);
    signals.push((TRACK_COUNT_WEIGHT, fit));

    signals.push((TITLE_WEIGHT, title_likeness(&release.title, &album.title)));

    if let Some(likeness) = artist_likeness(&release.artist(), &album.artist) {
        signals.push((ARTIST_WEIGHT, likeness));
    }

    let has_labels = release.labels().next().is_some();
    if let Some(label) = album
        .label
        .as_deref()
        .map(normalize_name)
        .filter(|_| has_labels)
    {
        let same = release.labels().any(|l| normalize_name(l) == label);
        signals.push((LABEL_WEIGHT, f64::from(u8::from(same))));
    }

    if release.status.is_some() {
        signals.push((STATUS_WEIGHT, f64::from(u8::from(release.is_official()))));
    }

    let total: f64 = signals.iter().map(|(w, _)| w).sum();
    let score: f64 = signals.iter().map(|(w, v)| w * v).sum();
    (score / total * 100.0).round() as u8
}

/// Each of `album`'s edited tracks' fields from `release`, in track order, or
/// `None` for a track that matches none of the release's tracks.
pub fn fill(release: &Release, album: &DiskAlbum) -> Vec<Option<TagFields>> {
    // Positions only line up on a release laid out like the album; on a box
    // set holding one of its recordings they would pick unrelated tracks.
    let by_position = release.total_tracks() == album.track_count();
    album
        .tracks
        .iter()
        .map(|track| {
            let (disc, medium, matched) = match_track(release, track, by_position)?;
            Some(track_fields(release, disc, medium, matched))
        })
        .collect()
}

/// The release track whose recording has `track`'s ISRC, otherwise, when
/// `by_position`, the one at its disc and track number, with its disc number.
/// Only audio media are searched and numbered, so a DVD listed first takes
/// neither a file's tracks nor its disc number.
fn match_track<'a>(
    release: &'a Release,
    track: &DiskTrack,
    by_position: bool,
) -> Option<(u32, &'a Medium, &'a ReleaseTrack)> {
    let all = || {
        release
            .numbered_audio_media()
            .flat_map(|(n, m)| m.tracks.iter().map(move |t| (n, m, t)))
    };
    let isrc = track
        .isrc
        .as_deref()
        .map(normalize_isrc)
        .filter(|i| !i.is_empty());
    let by_isrc = isrc.and_then(|isrc| {
        all().find(|(_, _, t)| t.recording.isrcs.iter().any(|i| normalize_isrc(i) == isrc))
    });
    by_isrc.or_else(|| {
        if !by_position {
            return None;
        }
        let number = track.number?;
        let disc = track.disc.unwrap_or(1);
        all().find(|(n, _, t)| *n == disc && t.position == number)
    })
}

fn track_fields(release: &Release, disc: u32, medium: &Medium, track: &ReleaseTrack) -> TagFields {
    let text = |s: String| (!s.is_empty()).then_some(s);
    let number = |n: u32| (n > 0).then(|| n.to_string());
    let compilation = release
        .release_group
        .secondary_types
        .iter()
        .any(|t| t == "Compilation");
    let values = [
        (Field::Album, text(release.title.clone())),
        (Field::AlbumArtist, text(release.artist())),
        (Field::Date, release.date()),
        (Field::Genre, release.genre().map(str::to_string)),
        (Field::Label, release.label().map(str::to_string)),
        (Field::DiscTotal, number(release.disc_count())),
        // Off is written as an empty value, which clears the flag.
        (
            Field::Compilation,
            Some(if compilation { FLAG_ON } else { "" }.to_string()),
        ),
        (Field::Title, text(track.title.clone())),
        (Field::Artist, text(credit(&track.artist_credit))),
        (Field::TrackNumber, number(track.position)),
        (Field::TrackTotal, number(medium.track_count)),
        (Field::DiscNumber, number(disc)),
        (
            Field::Composer,
            text(composers(&track.recording).join("; ")),
        ),
    ];
    TagFields {
        values: values
            .into_iter()
            .filter_map(|(field, value)| Some((field, value?)))
            .collect(),
        cover: None,
    }
}

/// The releases that may be `album`, best first. Empty when MusicBrainz knows
/// none by its barcode, its ISRCs or its title.
pub async fn lookup(album: &DiskAlbum, mut progress: impl FnMut(Step)) -> Result<Vec<Candidate>> {
    let mut ids = Vec::new();
    let barcode: String = album
        .barcode
        .as_deref()
        .unwrap_or_default()
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    if !barcode.is_empty() {
        progress(Step::Barcode);
        ids = search_barcode(&barcode).await?;
        tracing::debug!(
            "MusicBrainz: {} release(s) with barcode {barcode}",
            ids.len()
        );
    }
    let isrcs = album.isrcs();
    if ids.is_empty() && !isrcs.is_empty() {
        progress(Step::Isrcs);
        let found = search_isrcs(&isrcs).await?;
        ids = rank_by_isrcs(&found, &isrcs, album.track_count());
        tracing::debug!(
            "MusicBrainz: {} release(s) holding {} ISRC(s)",
            ids.len(),
            isrcs.len()
        );
    }
    if ids.is_empty() && !normalize_name(&album.title).is_empty() {
        progress(Step::Title);
        let found = search_title(album).await?;
        ids = fitting_titles(&found, album);
        tracing::debug!(
            "MusicBrainz: {} release(s) titled like \"{}\"",
            ids.len(),
            album.title
        );
    }
    ids.truncate(MAX_CANDIDATES);

    let mut read = Vec::with_capacity(ids.len());
    for (i, id) in ids.iter().enumerate() {
        progress(Step::Release {
            n: i + 1,
            of: ids.len(),
        });
        read.push(release(id).await);
    }
    let mut candidates: Vec<Candidate> = readable(read)?
        .into_iter()
        .map(|release| Candidate {
            confidence: confidence(&release, album),
            release,
        })
        .collect();
    candidates.sort_by_key(|c| std::cmp::Reverse(c.confidence));
    Ok(candidates)
}

/// The releases that could be read. One that could not, such as a release
/// merged since the search was indexed, is left out; the error is returned
/// only when none could be read, so an offline lookup still reports it.
fn readable(results: Vec<Result<Release>>) -> Result<Vec<Release>> {
    let mut releases = Vec::with_capacity(results.len());
    let mut first_error = None;
    for result in results {
        match result {
            Ok(release) => releases.push(release),
            Err(e) => {
                tracing::debug!("MusicBrainz: a candidate release could not be read: {e}");
                first_error.get_or_insert(e);
            }
        }
    }
    match first_error {
        Some(e) if releases.is_empty() => Err(e),
        _ => Ok(releases),
    }
}

async fn search_barcode(barcode: &str) -> Result<Vec<String>> {
    let query = format!("barcode:{barcode}");
    let found: ReleaseSearch =
        get_json("release", &[("query", query.as_str()), ("limit", "25")]).await?;
    Ok(found.releases.into_iter().map(|r| r.id).collect())
}

async fn search_isrcs(isrcs: &BTreeSet<String>) -> Result<Vec<Recording>> {
    let query = isrcs
        .iter()
        .map(|isrc| format!("isrc:{isrc}"))
        .collect::<Vec<_>>()
        .join(" OR ");
    let limit = ISRC_PAGE_SIZE.to_string();
    let mut recordings = Vec::new();
    let mut offset = Some(0);
    while let Some(at) = offset {
        let at_text = at.to_string();
        let page: RecordingSearch = get_json(
            "recording",
            &[
                ("query", query.as_str()),
                ("limit", limit.as_str()),
                ("offset", at_text.as_str()),
            ],
        )
        .await?;
        offset = next_isrc_page(at, page.recordings.len(), page.count);
        recordings.extend(page.recordings);
    }
    Ok(recordings)
}

/// Where the next page of an ISRC search starts, or `None` once every match is
/// read, a page comes back empty, or [`MAX_ISRC_PAGES`] pages are read.
fn next_isrc_page(offset: usize, got: usize, count: usize) -> Option<usize> {
    let next = offset + got;
    (got > 0 && next < count && next < MAX_ISRC_PAGES * ISRC_PAGE_SIZE).then_some(next)
}

/// Release ids from a recording search that hold at least half of `isrcs`,
/// ranked by how many they hold, then by how close their track count is to
/// `track_count`. Fewer would be another album sharing some recordings, such
/// as a box set or one of a compilation's sources. The count is release-wide,
/// bonus video included, since a recording search lists only the matched
/// medium; it only orders releases, each scored on its audio tracks once read.
fn rank_by_isrcs(
    recordings: &[Recording],
    isrcs: &BTreeSet<String>,
    track_count: u32,
) -> Vec<String> {
    let mut held: BTreeMap<&str, (BTreeSet<String>, u32)> = BTreeMap::new();
    for recording in recordings {
        let ours: BTreeSet<String> = recording
            .isrcs
            .iter()
            .map(|i| normalize_isrc(i))
            .filter(|i| isrcs.contains(i))
            .collect();
        if ours.is_empty() {
            continue;
        }
        for release in &recording.releases {
            let entry = held
                .entry(release.id.as_str())
                .or_insert_with(|| (BTreeSet::new(), release.total_tracks()));
            entry.0.extend(ours.iter().cloned());
        }
    }
    let mut ranked: Vec<_> = held
        .into_iter()
        .filter(|(_, (held, _))| held.len() * 2 >= isrcs.len())
        .collect();
    ranked.sort_by_key(|(_, (held, count))| {
        (std::cmp::Reverse(held.len()), count.abs_diff(track_count))
    });
    ranked.into_iter().map(|(id, _)| id.to_string()).collect()
}

/// Releases whose title has the album's words, on as many discs when known.
/// The track count is checked on the results, since the search's `tracks`
/// field counts one medium's tracks, not the release's.
async fn search_title(album: &DiskAlbum) -> Result<Vec<Release>> {
    let mut query = format!("release:({})", normalize_name(&album.title));
    if let Some(discs) = album.disc_count.filter(|n| *n > 0) {
        query.push_str(&format!(" AND mediums:{discs}"));
    }
    let found: ReleaseSearch =
        get_json("release", &[("query", query.as_str()), ("limit", "25")]).await?;
    Ok(found.releases)
}

/// Release ids from a title search that have the album's track count on their
/// audio media, so positions line up, share at least [`MIN_TITLE_LIKENESS`] of
/// its title's words and, when both are known, a word of its artist, most
/// alike first. A title alone is weak evidence: "Greatest Hits" is many
/// artists' album.
fn fitting_titles(releases: &[Release], album: &DiskAlbum) -> Vec<String> {
    let mut fitting: Vec<(f64, &str)> = releases
        .iter()
        .filter(|r| r.listed_audio_tracks() == album.track_count())
        .filter(|r| artist_likeness(&r.artist(), &album.artist).is_none_or(|l| l > 0.0))
        .map(|r| (title_likeness(&r.title, &album.title), r.id.as_str()))
        .filter(|(likeness, _)| *likeness >= MIN_TITLE_LIKENESS)
        .collect();
    fitting.sort_by(|a, b| b.0.total_cmp(&a.0));
    fitting.into_iter().map(|(_, id)| id.to_string()).collect()
}

async fn release(id: &str) -> Result<Release> {
    get_json(&format!("release/{id}"), &[("inc", RELEASE_INC)]).await
}

/// The release's front cover from the Cover Art Archive, or `None` when it has
/// none. The 1200 px rendition is asked for first: originals can be scans of
/// tens of megabytes, while the Archive may lack that rendition for an image.
pub async fn front_cover(release_id: &str) -> Result<Option<Vec<u8>>> {
    for rendition in ["front-1200", "front"] {
        let url = format!("{COVER_BASE}{release_id}/{rendition}");
        if let Some(bytes) = with_retry(MAX_ATTEMPTS, || fetch_cover(&url)).await? {
            return Ok(Some(bytes));
        }
    }
    Ok(None)
}

async fn fetch_cover(url: &str) -> Result<Option<Vec<u8>>> {
    let resp = http()?.get(url).send().await?;
    if !has_cover(resp.status().as_u16())? {
        return Ok(None);
    }
    Ok(Some(resp.bytes().await?.to_vec()))
}

/// Whether a Cover Art Archive answer holds an image; 404 means there is none.
fn has_cover(status: u16) -> Result<bool> {
    match status {
        200..=299 => Ok(true),
        404 => Ok(false),
        s => Err(http_error(s)),
    }
}

async fn get_json<T: serde::de::DeserializeOwned>(path: &str, query: &[(&str, &str)]) -> Result<T> {
    let url = format!("{API_BASE}{path}");
    with_retry(MAX_ATTEMPTS, || async {
        pacer().wait().await;
        let resp = http()?
            .get(&url)
            .query(query)
            .query(&[("fmt", "json")])
            .send()
            .await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            return Err(http_error(status));
        }
        Ok(serde_json::from_str(&resp.text().await?)?)
    })
    .await
}

/// MusicBrainz answers 503 when a client goes over its rate limit.
fn http_error(status: u16) -> Error {
    match status {
        503 => Error::RateLimited { retry_after: None },
        _ => Error::Http {
            status,
            message: "MusicBrainz request failed".into(),
        },
    }
}

fn http() -> Result<&'static reqwest::Client> {
    static HTTP: OnceLock<reqwest::Client> = OnceLock::new();
    if let Some(http) = HTTP.get() {
        return Ok(http);
    }
    let http = reqwest::Client::builder()
        .user_agent(concat!(
            "qobuz-dl/",
            env!("CARGO_PKG_VERSION"),
            " ( https://github.com/rcapraro/Qobuz-dl )"
        ))
        .timeout(Duration::from_secs(30))
        .build()?;
    Ok(HTTP.get_or_init(|| http))
}

/// Spaces request starts at least [`REQUEST_INTERVAL`] apart.
struct Pacer {
    last: Mutex<Option<Instant>>,
}

impl Pacer {
    const fn new() -> Self {
        Self {
            last: Mutex::const_new(None),
        }
    }

    async fn wait(&self) {
        let mut last = self.last.lock().await;
        if let Some(at) = *last {
            tokio::time::sleep_until(at + REQUEST_INTERVAL).await;
        }
        *last = Some(Instant::now());
    }
}

/// One for the whole process, so two lookups in a row still keep the pace.
fn pacer() -> &'static Pacer {
    static PACER: Pacer = Pacer::new();
    &PACER
}

#[cfg(test)]
mod tests {
    use super::*;

    const KIND_OF_BLUE: &str = include_str!("musicbrainz/testdata/release_kind_of_blue.json");
    const KIND_OF_BLUE_2SACD: &str =
        include_str!("musicbrainz/testdata/release_kind_of_blue_2sacd.json");
    const LONG: &str = include_str!("musicbrainz/testdata/release_42_tracks.json");
    const BARCODE_SEARCH: &str = include_str!("musicbrainz/testdata/barcode_search.json");
    const ISRC_SEARCH: &str = include_str!("musicbrainz/testdata/isrc_search.json");
    const TITLE_SEARCH: &str = include_str!("musicbrainz/testdata/title_search.json");

    const KIND_OF_BLUE_ISRCS: [&str; 6] = [
        "USSM15900113",
        "USSM15900114",
        "USSM15900115",
        "USSM15900116",
        "USSM15900117",
        "USSM15900118",
    ];

    fn release(json: &str) -> Release {
        serde_json::from_str(json).unwrap()
    }

    fn track(isrc: Option<&str>, disc: u32, number: u32) -> DiskTrack {
        DiskTrack {
            isrc: isrc.map(str::to_string),
            disc: Some(disc),
            number: Some(number),
        }
    }

    fn kind_of_blue_on_disk() -> DiskAlbum {
        DiskAlbum {
            barcode: Some("074646493564".into()),
            title: "Kind Of Blue".into(),
            artist: "Miles Davis".into(),
            label: Some("Columbia".into()),
            track_count: Some(6),
            disc_count: Some(1),
            tracks: KIND_OF_BLUE_ISRCS
                .iter()
                .zip(1..)
                .map(|(isrc, n)| track(Some(isrc), 1, n))
                .collect(),
        }
    }

    #[test]
    fn fixtures_parse() {
        let r = release(KIND_OF_BLUE);
        assert_eq!(r.title, "Kind of Blue");
        assert_eq!(r.total_tracks(), 6);
        assert_eq!(release(KIND_OF_BLUE_2SACD).total_tracks(), 12);
        let search: ReleaseSearch = serde_json::from_str(BARCODE_SEARCH).unwrap();
        assert_eq!(search.releases.len(), 2);
        let search: RecordingSearch = serde_json::from_str(ISRC_SEARCH).unwrap();
        assert!(search.recordings.iter().all(|r| !r.releases.is_empty()));
    }

    #[test]
    fn a_long_release_lists_every_track() {
        let r = release(LONG);
        let listed: usize = r.media.iter().map(|m| m.tracks.len()).sum();
        assert_eq!(listed, 42);
        assert_eq!(r.total_tracks(), 42);
    }

    #[test]
    fn nulls_read_as_empty() {
        let r = release(
            r#"{"id":"x","title":null,"media":null,"artist-credit":[{"name":"A","joinphrase":null}]}"#,
        );
        assert_eq!(r.title, "");
        assert!(r.media.is_empty());
        assert_eq!(credit(&r.artist_credit), "A");
    }

    /// `tracks` filled from the release in `json`, as an album with its tracks.
    fn filled(json: &str, tracks: &[DiskTrack]) -> Vec<Option<TagFields>> {
        let release = release(json);
        let album = DiskAlbum {
            track_count: Some(release.total_tracks()),
            tracks: tracks.to_vec(),
            ..DiskAlbum::default()
        };
        fill(&release, &album)
    }

    #[test]
    fn positions_are_not_matched_on_a_release_laid_out_otherwise() {
        // A 147-track box holding one of a 36-track compilation's recordings:
        // its other files must not take the box's tracks 2, 3… as theirs.
        let album = DiskAlbum {
            track_count: Some(36),
            tracks: vec![track(Some("USSM15900113"), 1, 1), track(None, 1, 2)],
            ..DiskAlbum::default()
        };
        let fields = fill(&release(KIND_OF_BLUE), &album);
        assert!(fields[0].is_some(), "matched by ISRC");
        assert!(fields[1].is_none(), "not matched by position");
    }

    #[test]
    fn a_release_without_isrcs_is_ranked_on_the_rest() {
        let mut bare = release(KIND_OF_BLUE);
        for medium in &mut bare.media {
            for t in &mut medium.tracks {
                t.recording.isrcs.clear();
            }
        }
        assert_eq!(confidence(&bare, &kind_of_blue_on_disk()), 100);
    }

    #[test]
    fn fills_album_and_track_fields() {
        let fields = filled(KIND_OF_BLUE, &[track(Some("USSM15900115"), 1, 3)])
            .remove(0)
            .unwrap();
        let get = |f| fields.get(f);
        assert_eq!(get(Field::Album), Some("Kind of Blue"));
        assert_eq!(get(Field::AlbumArtist), Some("Miles Davis"));
        assert_eq!(get(Field::Label), Some("Columbia"));
        assert_eq!(get(Field::DiscTotal), Some("1"));
        assert_eq!(get(Field::Title), Some("Blue in Green"));
        assert_eq!(get(Field::Artist), Some("Miles Davis"));
        assert_eq!(get(Field::TrackNumber), Some("3"));
        assert_eq!(get(Field::TrackTotal), Some("6"));
        assert_eq!(get(Field::DiscNumber), Some("1"));
    }

    #[test]
    fn date_is_the_first_release() {
        let fields = filled(KIND_OF_BLUE, &[track(None, 1, 1)])
            .remove(0)
            .unwrap();
        assert_eq!(fields.get(Field::Date), Some("1959-08-17"));
    }

    #[test]
    fn date_falls_back_to_the_release() {
        let json =
            r#"{"date":"2003","media":[{"position":1,"track-count":1,"tracks":[{"position":1}]}]}"#;
        let fields = filled(json, &[track(None, 1, 1)]).remove(0).unwrap();
        assert_eq!(fields.get(Field::Date), Some("2003"));
    }

    #[test]
    fn composers_come_from_the_works() {
        let fields = filled(
            KIND_OF_BLUE,
            &[
                track(Some("USSM15900113"), 1, 1),
                track(Some("USSM15900115"), 1, 3),
            ],
        );
        let composer = |i: usize| fields[i].as_ref().unwrap().get(Field::Composer);
        assert_eq!(composer(0), Some("Miles Davis"));
        assert_eq!(composer(1), Some("Miles Davis; Bill Evans"));
    }

    #[test]
    fn genre_falls_back_to_the_release_group() {
        // The release has no genre; its group's most voted is "jazz" (28).
        let fields = filled(KIND_OF_BLUE, &[track(None, 1, 1)])
            .remove(0)
            .unwrap();
        assert_eq!(fields.get(Field::Genre), Some("jazz"));
    }

    #[test]
    fn genre_spelled_as_musicbrainz_does() {
        let json = r#"{"genres":[{"name":"jazz","count":2},{"name":"modal jazz","count":5}],
            "release-group":{"genres":[{"name":"rock","count":40}]},
            "media":[{"position":1,"track-count":1,"tracks":[{"position":1}]}]}"#;
        let fields = filled(json, &[track(None, 1, 1)]).remove(0).unwrap();
        assert_eq!(fields.get(Field::Genre), Some("modal jazz"));
    }

    #[test]
    fn genre_tie_goes_to_the_first_alphabetically() {
        let genres = [
            Genre {
                name: "soul".into(),
                count: 3,
            },
            Genre {
                name: "funk".into(),
                count: 3,
            },
        ];
        assert_eq!(top_genre(&genres), Some("funk"));
    }

    #[test]
    fn compilation_follows_the_release_group() {
        let tracks = r#""media":[{"position":1,"track-count":1,"tracks":[{"position":1}]}]"#;
        let on = format!(r#"{{"release-group":{{"secondary-types":["Compilation"]}},{tracks}}}"#);
        let off = format!(r#"{{"release-group":{{"secondary-types":[]}},{tracks}}}"#);
        let flag = |json: &str| {
            filled(json, &[track(None, 1, 1)])
                .remove(0)
                .unwrap()
                .get(Field::Compilation)
                .map(str::to_string)
        };
        assert_eq!(flag(&on).as_deref(), Some(FLAG_ON));
        assert_eq!(flag(&off).as_deref(), Some(""));
    }

    #[test]
    fn leaves_what_musicbrainz_does_not_say() {
        let fields = filled(KIND_OF_BLUE, &[track(Some("USSM15900113"), 1, 1)])
            .remove(0)
            .unwrap();
        for field in [
            Field::Isrc,
            Field::Explicit,
            Field::Copyright,
            Field::Comment,
        ] {
            assert_eq!(fields.get(field), None, "{field:?}");
        }
    }

    #[test]
    fn matched_by_isrc_despite_renumbering() {
        let fields = filled(KIND_OF_BLUE, &[track(Some("USSM15900118"), 1, 7)])
            .remove(0)
            .unwrap();
        assert_eq!(fields.get(Field::TrackNumber), Some("6"));
        assert_eq!(
            fields.get(Field::Title),
            Some("Flamenco Sketches (alternate take)")
        );
    }

    #[test]
    fn matched_by_position_without_an_isrc_on_the_release() {
        let fields = filled(KIND_OF_BLUE_2SACD, &[track(Some("XX0000000000"), 2, 2)])
            .remove(0)
            .unwrap();
        assert_eq!(
            fields.get(Field::Title),
            Some("Freddie Freeloader (5.0 mix)")
        );
        assert_eq!(fields.get(Field::DiscNumber), Some("2"));
        assert_eq!(fields.get(Field::DiscTotal), Some("2"));
    }

    #[test]
    fn unmatched_track_is_none() {
        let fields = filled(
            KIND_OF_BLUE,
            &[
                track(Some("USSM15900113"), 1, 1),
                track(Some("XX0000000000"), 3, 9),
            ],
        );
        assert!(fields[0].is_some());
        assert!(fields[1].is_none());
    }

    #[test]
    fn exact_edition_ranks_above_one_with_bonus_tracks() {
        let album = kind_of_blue_on_disk();
        let exact = confidence(&release(KIND_OF_BLUE), &album);
        let bonus = confidence(&release(KIND_OF_BLUE_2SACD), &album);
        assert!(exact > bonus, "{exact} vs {bonus}");
        assert_eq!(exact, 100);
    }

    #[test]
    fn confidence_stays_in_range() {
        let album = DiskAlbum {
            title: "Something Else".into(),
            label: Some("Blue Note".into()),
            track_count: Some(1),
            tracks: vec![track(Some("XX0000000000"), 1, 1)],
            ..DiskAlbum::default()
        };
        for json in [KIND_OF_BLUE, KIND_OF_BLUE_2SACD, LONG] {
            assert!(confidence(&release(json), &album) <= 100);
        }
        assert_eq!(confidence(&Release::default(), &album), 0);
    }

    #[test]
    fn a_release_without_label_or_status_is_ranked_on_the_rest() {
        let mut r = release(KIND_OF_BLUE);
        r.label_info.clear();
        r.status = None;
        assert_eq!(confidence(&r, &kind_of_blue_on_disk()), 100);
    }

    #[test]
    fn a_video_disc_is_not_a_disc_of_the_album() {
        let json = r#"{"media":[
            {"position":1,"format":"CD","track-count":2,"tracks":[{"position":1},{"position":2}]},
            {"position":2,"format":"DVD-Video","track-count":9}]}"#;
        let r = release(json);
        assert_eq!(r.disc_count(), 1);
        assert_eq!(r.total_tracks(), 2);
        let fields = filled(json, &[track(None, 1, 2)]).remove(0).unwrap();
        assert_eq!(fields.get(Field::DiscTotal), Some("1"));
    }

    #[test]
    fn a_search_result_counts_the_whole_release() {
        // A recording search lists only the medium holding the recording.
        let json = r#"{"track-count":36,"media":[{"position":2,"track-count":18}]}"#;
        assert_eq!(release(json).total_tracks(), 36);
    }

    #[test]
    fn isrc_ranking_ties_on_the_whole_release_track_count() {
        let rec = |release: &str, count: u32| Recording {
            isrcs: vec!["A1".into()],
            releases: vec![serde_json::from_str(&format!(
                r#"{{"id":"{release}","track-count":{count},"media":[{{"track-count":18}}]}}"#
            ))
            .unwrap()],
            ..Recording::default()
        };
        let recordings = [rec("deluxe", 54), rec("edition", 36)];
        let ours: BTreeSet<String> = ["A1"].map(String::from).into();
        assert_eq!(rank_by_isrcs(&recordings, &ours, 36), ["edition", "deluxe"]);
    }

    #[test]
    fn ranks_without_any_isrc() {
        let mut album = kind_of_blue_on_disk();
        album.tracks.iter_mut().for_each(|t| t.isrc = None);
        let exact = confidence(&release(KIND_OF_BLUE), &album);
        let bonus = confidence(&release(KIND_OF_BLUE_2SACD), &album);
        assert_eq!(exact, 100);
        assert!(exact > bonus);
    }

    #[test]
    fn isrc_ranking_drops_releases_holding_under_half() {
        let rec = |isrc: &str, release: &str| Recording {
            isrcs: vec![isrc.into()],
            releases: vec![Release {
                id: release.into(),
                ..Release::default()
            }],
            ..Recording::default()
        };
        // Of a compilation's four ISRCs, a box set holds one, its source holds two.
        let recordings = [rec("A1", "box"), rec("B2", "source"), rec("C3", "source")];
        let ours: BTreeSet<String> = ["A1", "B2", "C3", "D4"].map(String::from).into();
        assert_eq!(rank_by_isrcs(&recordings, &ours, 4), ["source"]);
    }

    #[test]
    fn isrc_ranking_counts_only_our_isrcs() {
        let search: RecordingSearch = serde_json::from_str(ISRC_SEARCH).unwrap();
        let ours: BTreeSet<String> = ["USSM15900113", "USSM15900118"].map(String::from).into();
        let ranked = rank_by_isrcs(&search.recordings, &ours, 6);
        // Each release in the fixture holds one of the two; six-track ones first.
        let six = ["723f4d97", "844642e8", "607ef674", "c5db7ff3"];
        assert!(
            ranked[..4]
                .iter()
                .all(|id| six.iter().any(|p| id.starts_with(p))),
            "{ranked:?}"
        );
        assert_eq!(ranked.len(), 9);
    }

    #[test]
    fn isrc_ranking_prefers_releases_holding_more() {
        let rec = |isrcs: &[&str], releases: &[&str]| Recording {
            isrcs: isrcs.iter().map(|s| s.to_string()).collect(),
            releases: releases
                .iter()
                .map(|id| Release {
                    id: id.to_string(),
                    track_count: Some(6),
                    ..Release::default()
                })
                .collect(),
            ..Recording::default()
        };
        let recordings = [
            rec(&["A1"], &["one", "both"]),
            rec(&["B2", "ZZ9"], &["both"]),
        ];
        let ours: BTreeSet<String> = ["A1", "B2"].map(String::from).into();
        assert_eq!(rank_by_isrcs(&recordings, &ours, 6), ["both", "one"]);
    }

    #[test]
    fn title_likeness_counts_shared_words() {
        let ours = "Richard Hickox conducts Vaughan Williams";
        assert_eq!(
            title_likeness("Hickox Conducts Vaughan Williams", ours),
            0.8
        );
        assert_eq!(title_likeness("Kind of Blue", "Kind Of Blue!"), 1.0);
        assert_eq!(title_likeness("", ""), 0.0);
    }

    #[test]
    fn title_search_keeps_releases_laid_out_like_the_album() {
        // Four 36-track, two-disc releases answer the search; two are other
        // albums that only share "conducts".
        let search: ReleaseSearch = serde_json::from_str(TITLE_SEARCH).unwrap();
        let album = DiskAlbum {
            title: "Richard Hickox conducts Vaughan Williams".into(),
            track_count: Some(36),
            disc_count: Some(2),
            ..DiskAlbum::default()
        };
        assert_eq!(
            fitting_titles(&search.releases, &album),
            [
                "f7600f65-1772-43e6-b889-ecf6287c985b",
                "8480adc6-791e-4f17-9138-fa7c063d1f81"
            ]
        );

        let other_count = DiskAlbum {
            track_count: Some(18),
            ..album
        };
        assert!(fitting_titles(&search.releases, &other_count).is_empty());
    }

    #[test]
    fn title_search_counts_audio_tracks_and_checks_the_artist() {
        let found = |artist: &str, media: &str| -> Release {
            release(&format!(
                r#"{{"id":"{artist}","title":"Greatest Hits","track-count":15,
                "artist-credit":[{{"name":"{artist}"}}],"media":[{media}]}}"#
            ))
        };
        let cd_and_dvd = r#"{"format":"CD","track-count":10},
            {"format":"DVD-Video","track-count":5}"#;
        let releases = [found("Queen", cd_and_dvd), found("ABBA", cd_and_dvd)];
        let album = DiskAlbum {
            title: "Greatest Hits".into(),
            artist: "Queen".into(),
            track_count: Some(10),
            ..DiskAlbum::default()
        };
        assert_eq!(fitting_titles(&releases, &album), ["Queen"]);

        let unknown_artist = DiskAlbum {
            artist: String::new(),
            ..album
        };
        assert_eq!(fitting_titles(&releases, &unknown_artist).len(), 2);
    }

    #[test]
    fn artist_likeness_allows_a_fuller_credit() {
        let credit = "Ralph Vaughan Williams; London Symphony Orchestra, Richard Hickox";
        assert_eq!(artist_likeness(credit, "Richard Hickox"), Some(1.0));
        assert_eq!(artist_likeness("ABBA", "Queen"), Some(0.0));
        assert_eq!(
            artist_likeness("The Rolling Stones", "The Beatles"),
            Some(0.0),
            "a shared \"the\" is no match"
        );
        assert_eq!(artist_likeness("", "Queen"), None);
    }

    #[test]
    fn another_artists_release_is_not_a_strong_match() {
        let mut other = release(KIND_OF_BLUE);
        for medium in &mut other.media {
            for t in &mut medium.tracks {
                t.recording.isrcs.clear();
            }
        }
        other.artist_credit = vec![Credit {
            name: "ABBA".into(),
            joinphrase: String::new(),
        }];
        let score = confidence(&other, &kind_of_blue_on_disk());
        assert!(score < 80, "{score}");
    }

    #[test]
    fn a_video_medium_listed_first_takes_no_file() {
        let json = r#"{"media":[
            {"position":1,"format":"DVD-Video","track-count":3,
             "tracks":[{"position":1,"title":"Video 1"},{"position":2,"title":"Video 2"},
                       {"position":3,"title":"Video 3"}]},
            {"position":2,"format":"CD","track-count":3,
             "tracks":[{"position":1,"title":"One"},{"position":2,"title":"Two"},
                       {"position":3,"title":"Three","recording":{"isrcs":["AA1"]}}]}]}"#;
        let fields = filled(json, &[track(None, 1, 2), track(Some("AA1"), 1, 3)]);
        let get = |i: usize, f| fields[i].as_ref().unwrap().get(f).map(str::to_string);
        assert_eq!(get(0, Field::Title).as_deref(), Some("Two"));
        assert_eq!(get(0, Field::DiscNumber).as_deref(), Some("1"));
        assert_eq!(
            get(1, Field::DiscNumber).as_deref(),
            Some("1"),
            "matched by ISRC"
        );
        assert_eq!(get(1, Field::DiscTotal).as_deref(), Some("1"));
    }

    #[test]
    fn isrc_search_reads_every_page_up_to_a_bound() {
        assert_eq!(next_isrc_page(0, 100, 120), Some(100));
        assert_eq!(next_isrc_page(100, 20, 120), None, "all read");
        assert_eq!(next_isrc_page(0, 0, 120), None, "an empty page ends it");
        assert_eq!(next_isrc_page(400, 100, 900), None, "five pages at most");
    }

    #[test]
    fn an_unreadable_release_is_left_out() {
        let gone = || Error::Http {
            status: 404,
            message: "gone".into(),
        };
        let kept = readable(vec![Ok(release(KIND_OF_BLUE)), Err(gone())]).unwrap();
        assert_eq!(kept.len(), 1);
        assert!(readable(vec![Err(gone()), Err(gone())]).is_err());
        assert!(readable(Vec::new()).unwrap().is_empty());
    }

    #[test]
    fn formats_named_once() {
        assert_eq!(
            release(KIND_OF_BLUE_2SACD).formats(),
            "SACD (2 channels) + SACD (multichannel)"
        );
        let r = release(r#"{"media":[{"format":"CD"},{"format":"CD"}]}"#);
        assert_eq!(r.formats(), "CD");
    }

    #[tokio::test(start_paused = true)]
    async fn requests_start_a_second_apart() {
        let pacer = Pacer::new();
        let start = Instant::now();
        let mut starts = Vec::new();
        for _ in 0..3 {
            pacer.wait().await;
            starts.push(Instant::now() - start);
        }
        assert_eq!(starts[0], Duration::ZERO);
        assert!(starts[1] >= REQUEST_INTERVAL);
        assert!(starts[2] - starts[1] >= REQUEST_INTERVAL);
    }

    #[test]
    fn missing_cover_is_none_not_an_error() {
        assert!(matches!(has_cover(200), Ok(true)));
        assert!(matches!(has_cover(404), Ok(false)));
        assert!(matches!(has_cover(503), Err(Error::RateLimited { .. })));
        assert!(matches!(
            has_cover(500),
            Err(Error::Http { status: 500, .. })
        ));
    }
}
