//! The tag editor for one album in the queue: the tags read from its files,
//! what the user typed over them, and the edits that come out of that. Pure,
//! so every rule here is unit-tested; files are read and written in `tasks`.

use iced::widget::combo_box;
use iced::widget::image::Handle;
use qobuz_core::engine::Job;
use qobuz_core::models::ArtistRef;
use qobuz_core::musicbrainz::{self, Candidate, DiskAlbum, DiskTrack, Release, Step};
use qobuz_core::tag_edit::{
    CoverAction, CoverEdit, CoverPlan, Field, FieldEdit, FieldKind, Saved, TagEdits, TagFields,
    FLAG_ON, STANDARD_GENRES,
};
use qobuz_core::tagging::fields_from;
use qobuz_core::CoverSize;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::path::PathBuf;

/// From here a release is shown as a strong match, and a lone one is filled
/// without asking.
pub(super) const STRONG_MATCH: u8 = 80;

/// An album field's value across the album's files before any edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Shared {
    /// Every file holds this, `None` meaning none has the field.
    Same(Option<String>),
    /// The files hold different values.
    Mixed,
}

/// One album field: what the files hold and what the user typed.
#[derive(Debug, Clone)]
pub(super) struct AlbumField {
    field: Field,
    pub(super) original: Shared,
    pub(super) text: String,
    /// Set by the field's explicit Clear, the only way to clear a mixed field:
    /// an empty mixed field otherwise means "keep each file's value".
    cleared: bool,
}

impl AlbumField {
    fn new<'a>(field: Field, mut values: impl Iterator<Item = Option<&'a str>>) -> Self {
        let first = values.next().flatten();
        let original = if values.all(|v| comparable(field, v) == comparable(field, first)) {
            Shared::Same(first.map(str::to_string))
        } else {
            Shared::Mixed
        };
        let text = match &original {
            Shared::Same(value) => value.clone().unwrap_or_default(),
            Shared::Mixed => String::new(),
        };
        Self {
            field,
            original,
            text,
            cleared: false,
        }
    }

    pub(super) fn is_mixed(&self) -> bool {
        self.original == Shared::Mixed
    }

    /// Whether the field still shows the files' differing values untouched.
    pub(super) fn shows_mixed(&self) -> bool {
        self.is_mixed() && self.text.trim().is_empty() && !self.cleared
    }

    fn edit(&self) -> Option<FieldEdit> {
        match &self.original {
            Shared::Same(original) => text_edit(self.field, original.as_deref(), &self.text),
            Shared::Mixed if !self.text.trim().is_empty() => {
                Some(FieldEdit::Set(self.text.trim().to_string()))
            }
            Shared::Mixed => self.cleared.then_some(FieldEdit::Clear),
        }
    }

    fn set(&mut self, text: String) {
        self.text = text;
        self.cleared = false;
    }

    fn clear(&mut self) {
        self.text.clear();
        self.cleared = true;
    }
}

/// The edit typing `text` over `original` asks for: none while they match,
/// a clear for an emptied field, otherwise the new value.
fn text_edit(field: Field, original: Option<&str>, text: &str) -> Option<FieldEdit> {
    let text = text.trim();
    if comparable(field, Some(text)) == comparable(field, original) {
        None
    } else if text.is_empty() {
        Some(FieldEdit::Clear)
    } else {
        Some(FieldEdit::Set(text.to_string()))
    }
}

/// `value` as the file would store it, so values that only differ in form
/// (" So What ", "01" for 1) compare equal; an empty value counts as none.
fn comparable(field: Field, value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    let stored = field.normalize(value).unwrap_or_else(|| value.to_string());
    (!stored.is_empty()).then_some(stored)
}

/// Whether `text` can't be saved in `field`. An empty field is never invalid:
/// it means clear or keep.
pub(super) fn invalid(field: Field, text: &str) -> bool {
    let text = text.trim();
    !text.is_empty() && !field.accepts(text)
}

/// One done track whose file was read.
#[derive(Debug, Clone)]
pub(super) struct EditorTrack {
    pub(super) track_id: i64,
    pub(super) path: PathBuf,
    /// Its queue job: the Qobuz metadata Reset to Qobuz restores.
    job: Job,
    original: TagFields,
    /// The text of each track field.
    text: BTreeMap<Field, String>,
}

impl EditorTrack {
    pub(super) fn text(&self, field: Field) -> &str {
        self.text.get(&field).map_or("", String::as_str)
    }

    fn edit(&self, field: Field) -> Option<FieldEdit> {
        text_edit(field, self.original.get(field), self.text(field))
    }

    /// The values Qobuz actually holds for this track. A download writes
    /// "Unknown Artist" when Qobuz names no artist, which is not Qobuz's
    /// value, so those are left out; a clean track's explicit flag is off.
    fn qobuz(&self) -> TagFields {
        let (track, album) = (&self.job.track, &self.job.album);
        let mut fields = fields_from(track, album, self.job.disc_track_total);
        let named = |artist: Option<&ArtistRef>| artist.and_then(|a| a.name.as_deref()).is_some();
        if !named(track.performer.as_ref()) {
            fields.values.remove(&Field::Artist);
        }
        if !named(album.artist.as_ref()) {
            fields.values.remove(&Field::AlbumArtist);
        }
        if track.parental_warning == Some(false) {
            fields.values.insert(Field::Explicit, String::new());
        }
        fields
    }
}

/// A done track whose file could not be read, and why.
#[derive(Debug, Clone)]
pub(super) struct Unreadable {
    pub(super) title: String,
    pub(super) reason: String,
}

/// The editor's open album.
#[derive(Debug, Clone)]
pub(super) struct TagEditor {
    pub(super) album_id: String,
    pub(super) title: String,
    pub(super) artist: String,
    pub(super) tracks: Vec<EditorTrack>,
    pub(super) unreadable: Vec<Unreadable>,
    pub(super) album: BTreeMap<Field, AlbumField>,
    pub(super) cover: CoverAction,
    pub(super) resize: Option<CoverSize>,
    /// Tracks whose less-used fields are shown.
    pub(super) expanded: HashSet<i64>,
    pub(super) saving: Option<Saving>,
    /// The cover shown in the editor, as an image handle.
    pub(super) preview: Option<Handle>,
    /// The Genre box, suggesting the standard genres.
    pub(super) genres: combo_box::State<String>,
    /// Whether Fill from MusicBrainz also replaces the cover.
    pub(super) include_cover: bool,
    pub(super) lookup: Option<Lookup>,
}

/// A Fill from MusicBrainz under way.
#[derive(Debug, Clone)]
pub(super) enum Lookup {
    /// Looking releases up, at this step once one is reached.
    Searching(Option<Step>),
    /// Several releases may be the album; the user picks one.
    Choosing(Vec<Candidate>),
    /// A release was chosen and its cover is being fetched. Nothing is filled
    /// until it arrives, so a failed fetch leaves every field as it was.
    Cover(Box<Candidate>),
}

/// What a lookup's result led to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Found {
    Nothing,
    Choosing,
    Chosen(Chosen),
}

/// What became of a chosen release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Chosen {
    Filled(Filled),
    /// The cover of the release with this id is needed first.
    NeedsCover(String),
}

/// How a fill from MusicBrainz went, for the status line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Filled {
    pub(super) release: String,
    pub(super) confidence: u8,
    pub(super) unmatched: usize,
    pub(super) no_cover: bool,
    /// Too few tracks matched, so the album fields and cover were left alone.
    pub(super) album_kept: bool,
}

impl Filled {
    pub(super) fn status(&self) -> String {
        let mut text = format!(
            "Filled from MusicBrainz: {} · {} % match.",
            self.release, self.confidence
        );
        if self.unmatched > 0 {
            let s = if self.unmatched == 1 { "" } else { "s" };
            text.push_str(&format!(" {} track{s} not matched.", self.unmatched));
        }
        if self.album_kept {
            text.push_str(
                " Album fields and cover left as they were: fewer than half the tracks matched.",
            );
        } else if self.no_cover {
            text.push_str(" MusicBrainz has no cover for this release.");
        }
        text
    }
}

/// A release as the user tells it apart: "Kind of Blue (2003, US)".
pub(super) fn release_name(release: &Release) -> String {
    let details: Vec<&str> = [release.date.as_deref(), release.country.as_deref()]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect();
    if details.is_empty() {
        release.title.clone()
    } else {
        format!("{} ({})", release.title, details.join(", "))
    }
}

/// Whether a fill leaves the album fields and cover alone: a release matching
/// fewer than half the files is likely another edition, whose album-wide
/// values would be written to every file.
fn keeps_album(values: &[Option<TagFields>]) -> bool {
    let unmatched = values.iter().filter(|v| v.is_none()).count();
    unmatched * 2 > values.len()
}

/// A save in progress, one file at a time, so a large album never needs more
/// than one file's worth of extra disk space and the editor can show progress.
#[derive(Debug, Clone)]
pub(super) struct Saving {
    pub(super) done: usize,
    pub(super) total: usize,
    pending: VecDeque<(i64, PathBuf, TagEdits)>,
    /// Set once the cover change is ready; files wait for it.
    plan: Option<CoverPlan>,
    written: usize,
    failures: Vec<String>,
}

/// The next file to save: its track, path, edits and the album's cover plan.
pub(super) type NextSave = (i64, PathBuf, TagEdits, CoverPlan);

/// How a finished save went, for the status line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SaveReport {
    pub(super) written: usize,
    pub(super) failures: Vec<String>,
}

impl TagEditor {
    /// The editor for an album whose done tracks were read: each track's queue
    /// job, its file, and what reading it gave.
    pub(super) fn new(
        album_id: String,
        read: Vec<(Job, PathBuf, Result<TagFields, String>)>,
    ) -> Self {
        let (title, artist) = read
            .first()
            .map(|(job, _, _)| (job.album.title.clone(), job.album.artist_name().to_string()))
            .unwrap_or_default();
        let mut read = read;
        read.sort_by_key(|(job, _, _)| (job.track.disc_number(), job.track.track_number));

        let mut tracks = Vec::new();
        let mut unreadable = Vec::new();
        for (job, path, result) in read {
            match result {
                Ok(original) => tracks.push(EditorTrack {
                    track_id: job.track.id,
                    path,
                    text: Field::TRACK
                        .into_iter()
                        .map(|f| (f, original.get(f).unwrap_or_default().to_string()))
                        .collect(),
                    job,
                    original,
                }),
                Err(reason) => unreadable.push(Unreadable {
                    title: job.track.title,
                    reason,
                }),
            }
        }
        // Only the first cover is shown; saving reads each file's own cover
        // again, so holding every track's copy would only cost memory.
        let mut covers = tracks.iter_mut().filter(|t| t.original.cover.is_some());
        covers.next();
        covers.for_each(|t| t.original.cover = None);
        let album = Field::ALBUM
            .into_iter()
            .map(|f| {
                (
                    f,
                    AlbumField::new(f, tracks.iter().map(|t| t.original.get(f))),
                )
            })
            .collect();
        Self {
            album_id,
            title,
            artist,
            tracks,
            unreadable,
            album,
            cover: CoverAction::Keep,
            resize: None,
            expanded: HashSet::new(),
            saving: None,
            preview: None,
            genres: combo_box::State::default(),
            include_cover: false,
            lookup: None,
        }
        .with_derived()
    }

    fn with_derived(mut self) -> Self {
        self.refresh_preview();
        self.refresh_genres();
        self
    }

    /// Rebuild the genre box with the field's current text. The box keeps its
    /// own copy of what was typed, so a genre set any other way (Reset to
    /// Qobuz, ×) would otherwise reappear as the old text on the next focus.
    fn refresh_genres(&mut self) {
        let mut options: Vec<String> = STANDARD_GENRES.iter().map(|g| g.to_string()).collect();
        options.sort_by_key(|g| g.to_lowercase());
        let text = self.album_field(Field::Genre).text.clone();
        let selection = (!text.is_empty()).then_some(&text);
        self.genres = combo_box::State::with_selection(options, selection);
    }

    /// Rebuild the cover preview's image handle. Kept rather than built per
    /// frame: a new handle is a new image to iced, decoded again each redraw.
    fn refresh_preview(&mut self) {
        self.preview = self
            .cover_preview()
            .map(|bytes| Handle::from_bytes(bytes.to_vec()));
    }

    pub(super) fn album_field(&self, field: Field) -> &AlbumField {
        &self.album[&field]
    }

    pub(super) fn set_album_text(&mut self, field: Field, text: String) {
        if let Some(f) = self.album.get_mut(&field) {
            f.set(text);
        }
    }

    pub(super) fn clear_album(&mut self, field: Field) {
        if let Some(f) = self.album.get_mut(&field) {
            f.clear();
        }
        if field == Field::Genre {
            self.refresh_genres();
        }
    }

    /// Turn an album flag on or off; off clears it even where values differ.
    pub(super) fn set_album_flag(&mut self, field: Field, on: bool) {
        if on {
            self.set_album_text(field, FLAG_ON.to_string());
        } else {
            self.clear_album(field);
        }
    }

    /// An album flag's state, `None` while it still shows differing values.
    pub(super) fn album_flag(&self, field: Field) -> Option<bool> {
        let f = self.album_field(field);
        (!f.shows_mixed()).then(|| f.text.trim() == FLAG_ON)
    }

    pub(super) fn set_track_text(&mut self, track_id: i64, field: Field, text: String) {
        if let Some(t) = self.track_mut(track_id) {
            t.text.insert(field, text);
        }
    }

    pub(super) fn set_track_flag(&mut self, track_id: i64, field: Field, on: bool) {
        let text = if on { FLAG_ON } else { "" };
        self.set_track_text(track_id, field, text.to_string());
    }

    /// Give every track the value `field` has on track `track_id`.
    pub(super) fn apply_to_all(&mut self, track_id: i64, field: Field) {
        let Some(text) = self.track(track_id).map(|t| t.text(field).to_string()) else {
            return;
        };
        for t in &mut self.tracks {
            t.text.insert(field, text.clone());
        }
    }

    pub(super) fn toggle_expanded(&mut self, track_id: i64) {
        if !self.expanded.remove(&track_id) {
            self.expanded.insert(track_id);
        }
    }

    /// Fill the fields from the queue's Qobuz metadata. A field Qobuz has no
    /// value for keeps what it shows; an album field whose Qobuz values differ
    /// between tracks does too.
    pub(super) fn reset_to_qobuz(&mut self) {
        let qobuz = self.tracks.iter().map(|t| Some(t.qobuz())).collect();
        self.fill(qobuz, true);
    }

    /// Fill the fields from a source's values for each track, in track order,
    /// `None` for a track the source has nothing for. With `album_fields`, an
    /// album field is set when every track the source has agrees on it; an
    /// empty value turns it off, even where the files differ.
    fn fill(&mut self, values: Vec<Option<TagFields>>, album_fields: bool) {
        let known: Vec<&TagFields> = values.iter().flatten().collect();
        let album = self.album.iter_mut().filter(|_| album_fields);
        for (field, album_field) in album {
            let mut values = known.iter().map(|q| q.get(*field));
            let Some(Some(first)) = values.next() else {
                continue;
            };
            if !values.all(|v| v == Some(first)) {
                continue;
            }
            if first.is_empty() {
                album_field.clear();
            } else {
                album_field.set(first.to_string());
            }
        }
        for (track, q) in self.tracks.iter_mut().zip(&values) {
            let Some(q) = q else {
                continue;
            };
            for field in Field::TRACK {
                if let Some(value) = q.get(field) {
                    track.text.insert(field, value.to_string());
                }
            }
        }
        self.refresh_genres();
    }

    /// Whether a save or a MusicBrainz lookup is under way.
    pub(super) fn busy(&self) -> bool {
        self.saving.is_some() || self.lookup.is_some()
    }

    /// Start a Fill from MusicBrainz: the album to look up, or `None` while
    /// the editor is busy.
    pub(super) fn start_lookup(&mut self) -> Option<DiskAlbum> {
        if self.busy() {
            return None;
        }
        self.lookup = Some(Lookup::Searching(None));
        Some(self.disk_album())
    }

    /// The album as the lookup knows it: the files' ISRCs and numbering, and
    /// the barcode, title, artist, label and track count of its Qobuz release.
    fn disk_album(&self) -> DiskAlbum {
        let qobuz = self.tracks.first().map(|t| &t.job.album);
        DiskAlbum {
            barcode: qobuz.and_then(|a| a.upc.clone()),
            title: qobuz.map(|a| a.title.clone()).unwrap_or_default(),
            artist: qobuz
                .map(|a| a.artist_name().to_string())
                .unwrap_or_default(),
            label: qobuz.and_then(|a| a.label.as_ref()?.name.clone()),
            track_count: qobuz.and_then(|a| a.tracks_count),
            disc_count: qobuz.and_then(|a| a.media_count),
            tracks: self.disk_tracks(),
        }
    }

    fn disk_tracks(&self) -> Vec<DiskTrack> {
        let number = |t: &EditorTrack, f| t.original.get(f).and_then(|n| n.parse().ok());
        self.tracks
            .iter()
            .map(|t| DiskTrack {
                isrc: t.original.get(Field::Isrc).map(str::to_string),
                disc: number(t, Field::DiscNumber),
                number: number(t, Field::TrackNumber),
            })
            .collect()
    }

    pub(super) fn lookup_step(&mut self, step: Step) {
        if let Some(Lookup::Searching(at)) = &mut self.lookup {
            *at = Some(step);
        }
    }

    pub(super) fn searching(&self) -> bool {
        matches!(self.lookup, Some(Lookup::Searching(_)))
    }

    /// The lookup found these releases, best first; `None` when no lookup is
    /// searching, so a late result is dropped. A lone release is filled only
    /// when it is a strong match; a weaker one is listed for the user to judge.
    pub(super) fn found(&mut self, mut candidates: Vec<Candidate>) -> Option<Found> {
        if !self.searching() {
            return None;
        }
        Some(match candidates.len() {
            0 => {
                self.lookup = None;
                Found::Nothing
            }
            1 if candidates[0].confidence >= STRONG_MATCH => {
                Found::Chosen(self.take(candidates.remove(0)))
            }
            _ => {
                self.lookup = Some(Lookup::Choosing(candidates));
                Found::Choosing
            }
        })
    }

    /// The user picked the release at `index` in the list.
    pub(super) fn choose(&mut self, index: usize) -> Option<Chosen> {
        let Some(Lookup::Choosing(candidates)) = &mut self.lookup else {
            return None;
        };
        if index >= candidates.len() {
            return None;
        }
        let candidate = candidates.swap_remove(index);
        Some(self.take(candidate))
    }

    /// Fill from `candidate`, or first fetch its cover when Include cover is on
    /// and the cover would be used: with fewer than half the files matched it
    /// would be left out, so the matched tracks are filled at once.
    fn take(&mut self, candidate: Candidate) -> Chosen {
        let values = musicbrainz::fill(&candidate.release, &self.disk_album());
        if self.include_cover && !keeps_album(&values) {
            let id = candidate.release.id.clone();
            self.lookup = Some(Lookup::Cover(Box::new(candidate)));
            return Chosen::NeedsCover(id);
        }
        self.lookup = None;
        Chosen::Filled(self.fill_with(&candidate, values, false))
    }

    /// The chosen release's cover arrived, `None` when it has none: fill the
    /// fields and take the cover.
    pub(super) fn cover_fetched(
        &mut self,
        cover: Option<Vec<u8>>,
        download_size: CoverSize,
    ) -> Option<Filled> {
        let Some(Lookup::Cover(candidate)) = self.lookup.take() else {
            return None;
        };
        let filled = self.fill_from(&candidate, cover.is_none());
        if let Some(image) = cover.filter(|_| !filled.album_kept) {
            self.replace_cover(image, download_size);
        }
        Some(filled)
    }

    /// End the lookup with nothing changed: cancelled, or it failed.
    pub(super) fn end_lookup(&mut self) {
        self.lookup = None;
    }

    fn fill_from(&mut self, candidate: &Candidate, no_cover: bool) -> Filled {
        let values = musicbrainz::fill(&candidate.release, &self.disk_album());
        self.fill_with(candidate, values, no_cover)
    }

    fn fill_with(
        &mut self,
        candidate: &Candidate,
        values: Vec<Option<TagFields>>,
        no_cover: bool,
    ) -> Filled {
        let unmatched = values.iter().filter(|v| v.is_none()).count();
        let album_kept = keeps_album(&values);
        self.fill(values, !album_kept);
        Filled {
            release: release_name(&candidate.release),
            confidence: candidate.confidence,
            unmatched,
            no_cover,
            album_kept,
        }
    }

    /// Embed `image` in every file. The resize option takes the size a
    /// download would use, so a large picked image isn't embedded as is.
    pub(super) fn replace_cover(&mut self, image: Vec<u8>, download_size: CoverSize) {
        self.cover = CoverAction::Replace(image);
        self.resize = Some(download_size);
        self.refresh_preview();
    }

    pub(super) fn remove_cover(&mut self) {
        self.cover = CoverAction::Remove;
        self.resize = None;
        self.refresh_preview();
    }

    pub(super) fn keep_cover(&mut self) {
        self.cover = CoverAction::Keep;
        self.resize = None;
        self.refresh_preview();
    }

    pub(super) fn set_resize(&mut self, size: Option<CoverSize>) {
        if self.cover != CoverAction::Remove {
            self.resize = size;
        }
    }

    /// The cover shown in the editor: a replacement, else the first file's.
    pub(super) fn cover_preview(&self) -> Option<&[u8]> {
        match &self.cover {
            CoverAction::Replace(image) => Some(image),
            CoverAction::Remove => None,
            CoverAction::Keep => self.tracks.iter().find_map(|t| t.original.cover.as_deref()),
        }
    }

    /// Whether any typed value is invalid, which blocks saving.
    pub(super) fn has_invalid(&self) -> bool {
        let album = self.album.iter().any(|(f, a)| invalid(*f, &a.text));
        let tracks = self
            .tracks
            .iter()
            .any(|t| t.text.iter().any(|(f, text)| invalid(*f, text)));
        album || tracks
    }

    /// Whether anything would change on save.
    pub(super) fn has_edits(&self) -> bool {
        self.cover != CoverAction::Keep
            || self.resize.is_some()
            || self.album.values().any(|f| f.edit().is_some())
            || self
                .tracks
                .iter()
                .any(|t| Field::TRACK.into_iter().any(|f| t.edit(f).is_some()))
    }

    /// Each file's field edits: the album fields applied to every track, then
    /// the track's own. A file whose fields are untouched is still listed
    /// while the cover changes, since that applies to every file.
    pub(super) fn edits(&self) -> Vec<(i64, PathBuf, TagEdits)> {
        let album: TagEdits = self
            .album
            .iter()
            .filter_map(|(f, a)| Some((*f, a.edit()?)))
            .collect();
        let cover_changes = self.cover != CoverAction::Keep || self.resize.is_some();
        self.tracks
            .iter()
            .filter_map(|t| {
                let mut edits = album.clone();
                edits.extend(
                    Field::TRACK
                        .into_iter()
                        .filter_map(|f| Some((f, t.edit(f)?))),
                );
                (cover_changes || !edits.is_empty()).then(|| (t.track_id, t.path.clone(), edits))
            })
            .collect()
    }

    pub(super) fn cover_edit(&self) -> CoverEdit {
        CoverEdit {
            action: self.cover.clone(),
            resize: self.resize,
        }
    }

    /// Start saving every file with edits, returning the cover change to plan
    /// before the first file is written.
    pub(super) fn start_saving(&mut self) -> CoverEdit {
        let files = self.edits();
        self.saving = Some(Saving {
            done: 0,
            total: files.len(),
            pending: files.into(),
            plan: None,
            written: 0,
            failures: Vec::new(),
        });
        self.cover_edit()
    }

    /// The cover plan is ready: the first file to save, if any.
    pub(super) fn cover_planned(&mut self, plan: CoverPlan) -> Option<NextSave> {
        self.saving.as_mut()?.plan = Some(plan);
        self.next_save()
    }

    /// One file finished: the next to save, or `None` once all are done.
    pub(super) fn file_saved(
        &mut self,
        track_id: i64,
        result: Result<Saved, String>,
    ) -> Option<NextSave> {
        let title = self
            .track(track_id)
            .map(|t| t.job.track.title.clone())
            .unwrap_or_default();
        let saving = self.saving.as_mut()?;
        saving.done += 1;
        match result {
            Ok(Saved::Written) => saving.written += 1,
            Ok(Saved::Unchanged) => {}
            Err(reason) => saving.failures.push(format!("{title}: {reason}")),
        }
        self.next_save()
    }

    /// End the save, whether finished or abandoned, and report it.
    pub(super) fn finish_saving(&mut self) -> Option<SaveReport> {
        let saving = self.saving.take()?;
        Some(SaveReport {
            written: saving.written,
            failures: saving.failures,
        })
    }

    fn next_save(&mut self) -> Option<NextSave> {
        let saving = self.saving.as_mut()?;
        let plan = saving.plan.clone()?;
        let (track_id, path, edits) = saving.pending.pop_front()?;
        Some((track_id, path, edits, plan))
    }

    fn track(&self, track_id: i64) -> Option<&EditorTrack> {
        self.tracks.iter().find(|t| t.track_id == track_id)
    }

    fn track_mut(&mut self, track_id: i64) -> Option<&mut EditorTrack> {
        self.tracks.iter_mut().find(|t| t.track_id == track_id)
    }
}

/// An input in the editor. None of them touches the files.
#[derive(Debug, Clone)]
pub(super) enum Edit {
    AlbumText(Field, String),
    ClearAlbum(Field),
    AlbumFlag(Field, bool),
    TrackText(i64, Field, String),
    TrackFlag(i64, Field, bool),
    ApplyToAll(i64, Field),
    ToggleExpanded(i64),
    ResetToQobuz,
    IncludeCover(bool),
    RemoveCover,
    KeepCover,
    Resize(Resize),
}

/// The cover resize option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Resize {
    DontResize,
    To(CoverSize),
}

impl Resize {
    pub(super) fn all() -> Vec<Resize> {
        std::iter::once(Resize::DontResize)
            .chain(CoverSize::ALL.map(Resize::To))
            .collect()
    }

    pub(super) fn of(size: Option<CoverSize>) -> Self {
        size.map_or(Resize::DontResize, Resize::To)
    }
}

impl std::fmt::Display for Resize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Resize::DontResize => f.write_str("Don't resize"),
            Resize::To(size) => size.fmt(f),
        }
    }
}

impl TagEditor {
    pub(super) fn apply(&mut self, edit: Edit) {
        match edit {
            Edit::AlbumText(field, text) => self.set_album_text(field, text),
            Edit::ClearAlbum(field) => self.clear_album(field),
            Edit::AlbumFlag(field, on) => self.set_album_flag(field, on),
            Edit::TrackText(id, field, text) => self.set_track_text(id, field, text),
            Edit::TrackFlag(id, field, on) => self.set_track_flag(id, field, on),
            Edit::ApplyToAll(id, field) => self.apply_to_all(id, field),
            Edit::ToggleExpanded(id) => self.toggle_expanded(id),
            Edit::ResetToQobuz if self.lookup.is_none() => self.reset_to_qobuz(),
            Edit::ResetToQobuz => {}
            Edit::IncludeCover(on) => self.include_cover = on,
            Edit::RemoveCover => self.remove_cover(),
            Edit::KeepCover => self.keep_cover(),
            Edit::Resize(choice) => self.set_resize(match choice {
                Resize::DontResize => None,
                Resize::To(size) => Some(size),
            }),
        }
    }
}

/// The label shown beside a field.
pub(super) fn label(field: Field) -> &'static str {
    match field {
        Field::Album => "Album:",
        Field::AlbumArtist => "Album artist:",
        Field::Date => "Date:",
        Field::Genre => "Genre:",
        Field::Label => "Label:",
        Field::Copyright => "Copyright:",
        Field::DiscTotal => "Total discs:",
        Field::TrackTotal => "Total tracks:",
        Field::Compilation => "Compilation:",
        Field::Title => "Title:",
        Field::Artist => "Artist:",
        Field::TrackNumber => "Track:",
        Field::DiscNumber => "Disc:",
        Field::Composer => "Composer:",
        Field::Isrc => "ISRC:",
        Field::Explicit => "Explicit:",
        Field::Comment => "Comment:",
    }
}

/// The placeholder an empty input shows: its expected format, if any.
pub(super) fn hint(field: Field) -> &'static str {
    match field.kind() {
        FieldKind::Date => "YYYY-MM-DD",
        FieldKind::Number => "#",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::super::album::tests::job;
    use super::*;

    fn fields(pairs: &[(Field, &str)]) -> TagFields {
        TagFields {
            values: pairs.iter().map(|(f, v)| (*f, v.to_string())).collect(),
            cover: None,
        }
    }

    /// An editor over tracks 1.. whose files hold `files`, in order.
    fn editor(files: &[&[(Field, &str)]]) -> TagEditor {
        let read = files
            .iter()
            .enumerate()
            .map(|(i, pairs)| {
                let id = i as i64 + 1;
                (
                    job(id, 1, Some("Band")),
                    PathBuf::from(format!("/m/{id}.flac")),
                    Ok(fields(pairs)),
                )
            })
            .collect();
        TagEditor::new("al".into(), read)
    }

    fn edits_of(editor: &TagEditor) -> Vec<TagEdits> {
        editor.edits().into_iter().map(|(_, _, e)| e).collect()
    }

    #[test]
    fn shared_album_value_is_shown() {
        let e = editor(&[&[(Field::Genre, "Jazz")], &[(Field::Genre, "Jazz")]]);
        let genre = e.album_field(Field::Genre);
        assert!(!genre.is_mixed());
        assert_eq!(genre.text, "Jazz");
        assert!(!e.has_edits());
    }

    #[test]
    fn mixed_album_value_is_kept_unless_cleared() {
        let mut e = editor(&[&[(Field::Label, "A")], &[(Field::Label, "B")]]);
        assert!(e.album_field(Field::Label).shows_mixed());
        assert!(e.edits().is_empty());

        e.clear_album(Field::Label);
        assert_eq!(edits_of(&e)[0].get(&Field::Label), Some(&FieldEdit::Clear));
        assert!(!e.album_field(Field::Label).shows_mixed());
    }

    #[test]
    fn album_value_applies_to_every_track() {
        let mut e = editor(&[&[(Field::Album, "Old")], &[(Field::Album, "Old")]]);
        e.set_album_text(Field::Album, "Kind of Blue (Remaster)".into());
        let set = FieldEdit::Set("Kind of Blue (Remaster)".into());
        for edits in edits_of(&e) {
            assert_eq!(edits.get(&Field::Album), Some(&set));
        }
    }

    #[test]
    fn emptied_field_is_cleared_and_restored_text_is_no_edit() {
        let mut e = editor(&[&[(Field::Comment, "note")]]);
        e.set_album_text(Field::Comment, String::new());
        assert!(edits_of(&e).is_empty(), "comment is a track field");

        e.set_track_text(1, Field::Comment, String::new());
        assert_eq!(
            edits_of(&e)[0].get(&Field::Comment),
            Some(&FieldEdit::Clear)
        );
        e.set_track_text(1, Field::Comment, "note".into());
        assert!(e.edits().is_empty());
    }

    #[test]
    fn only_the_edited_track_gets_a_title_edit() {
        let mut e = editor(&[
            &[(Field::Title, "a")],
            &[(Field::Title, "b")],
            &[(Field::Title, "c")],
        ]);
        e.set_track_text(3, Field::Title, "C".into());
        let edits = e.edits();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].0, 3);
    }

    #[test]
    fn apply_to_all_copies_a_track_value() {
        let mut e = editor(&[&[(Field::Artist, "x")], &[(Field::Artist, "y")]]);
        e.set_track_text(1, Field::Artist, "Miles Davis".into());
        e.apply_to_all(1, Field::Artist);
        assert!(e
            .tracks
            .iter()
            .all(|t| t.text(Field::Artist) == "Miles Davis"));
        assert_eq!(e.edits().len(), 2);
    }

    #[test]
    fn invalid_values_block_saving() {
        let mut e = editor(&[&[(Field::TrackNumber, "1")]]);
        e.set_track_text(1, Field::TrackNumber, "1a".into());
        assert!(e.has_invalid());
        e.set_track_text(1, Field::TrackNumber, "2".into());
        assert!(!e.has_invalid());
        e.set_album_text(Field::Date, "1959".into());
        assert!(!e.has_invalid());
        e.set_album_text(Field::Date, "59".into());
        assert!(e.has_invalid());
    }

    #[test]
    fn album_flag_is_mixed_until_set() {
        let mut e = editor(&[&[(Field::Compilation, FLAG_ON)], &[]]);
        assert_eq!(e.album_flag(Field::Compilation), None);
        e.set_album_flag(Field::Compilation, false);
        assert_eq!(e.album_flag(Field::Compilation), Some(false));
        assert_eq!(
            edits_of(&e)[0].get(&Field::Compilation),
            Some(&FieldEdit::Clear)
        );
    }

    #[test]
    fn reset_restores_qobuz_values_and_leaves_the_rest() {
        let mut e = editor(&[&[(Field::Title, "edited"), (Field::Comment, "mine")]]);
        e.set_album_text(Field::Album, "edited".into());
        e.reset_to_qobuz();
        // `job` builds track "t1" on album "Album".
        assert_eq!(e.tracks[0].text(Field::Title), "t1");
        assert_eq!(e.album_field(Field::Album).text, "Album");
        assert_eq!(e.tracks[0].text(Field::Comment), "mine");
    }

    #[test]
    fn genre_box_suggests_the_standard_genres_alphabetically() {
        let e = editor(&[&[(Field::Genre, "Classique")]]);
        let options = e.genres.options();
        assert_eq!(options.len(), STANDARD_GENRES.len());
        assert!(options.iter().any(|g| g == "Classical"));
        assert!(options
            .windows(2)
            .all(|w| w[0].to_lowercase() <= w[1].to_lowercase()));
    }

    #[test]
    fn cover_resize_defaults() {
        let mut e = editor(&[&[]]);
        assert_eq!(e.resize, None);
        e.replace_cover(vec![1, 2], CoverSize::Px400);
        assert_eq!(e.resize, Some(CoverSize::Px400));
        e.remove_cover();
        assert_eq!(e.resize, None);
        e.set_resize(Some(CoverSize::Px500));
        assert_eq!(e.resize, None, "no resizing a removed cover");
        e.keep_cover();
        e.set_resize(Some(CoverSize::Px500));
        assert_eq!(e.cover_edit().resize, Some(CoverSize::Px500));
        assert_eq!(e.edits().len(), 1, "a cover change lists every file");
    }

    #[test]
    fn values_that_only_differ_in_form_are_not_edits() {
        let mut e = editor(&[&[(Field::Title, "So What "), (Field::TrackNumber, "1")]]);
        assert!(!e.has_edits(), "a trailing space on disk isn't an edit");
        e.set_track_text(1, Field::TrackNumber, "01".into());
        e.set_track_text(1, Field::Title, " So What".into());
        assert!(!e.has_edits());
    }

    #[test]
    fn total_tracks_is_edited_per_track() {
        let mut e = editor(&[&[(Field::TrackTotal, "9")], &[(Field::TrackTotal, "6")]]);
        assert!(!e.album.contains_key(&Field::TrackTotal));
        assert_eq!(e.tracks[1].text(Field::TrackTotal), "6");
        e.set_track_text(2, Field::TrackTotal, "7".into());
        let edits = e.edits();
        assert_eq!(edits.len(), 1);
        assert_eq!(
            edits[0].2.get(&Field::TrackTotal),
            Some(&FieldEdit::Set("7".into()))
        );
    }

    #[test]
    fn reset_leaves_what_qobuz_does_not_say() {
        // `job` with no performer and no parental warning.
        let read = vec![(
            job(1, 1, None),
            PathBuf::from("/m/1.flac"),
            Ok(fields(&[
                (Field::Artist, "Fixed by hand"),
                (Field::Explicit, FLAG_ON),
            ])),
        )];
        let mut e = TagEditor::new("al".into(), read);
        e.reset_to_qobuz();
        assert_eq!(e.tracks[0].text(Field::Artist), "Fixed by hand");
        assert_eq!(e.tracks[0].text(Field::Explicit), FLAG_ON);
    }

    #[test]
    fn only_the_first_cover_is_kept() {
        let with_cover = |id: i64, cover: u8| {
            let mut f = fields(&[]);
            f.cover = Some(vec![cover]);
            (
                job(id, 1, None),
                PathBuf::from(format!("/m/{id}.flac")),
                Ok(f),
            )
        };
        let e = TagEditor::new("al".into(), vec![with_cover(1, 1), with_cover(2, 2)]);
        assert_eq!(e.cover_preview(), Some(&[1u8][..]));
        assert!(e.tracks[1].original.cover.is_none());
    }

    #[test]
    fn saving_goes_file_by_file_and_reports() {
        let mut e = editor(&[&[(Field::Title, "a")], &[(Field::Title, "b")]]);
        e.set_album_text(Field::Genre, "Jazz".into());
        e.start_saving();
        assert_eq!(e.saving.as_ref().map(|s| s.total), Some(2));

        let plan = CoverPlan::new(CoverEdit::default());
        let (first, ..) = e.cover_planned(plan).unwrap();
        let (second, ..) = e.file_saved(first, Ok(Saved::Written)).unwrap();
        assert_eq!(e.saving.as_ref().map(|s| s.done), Some(1));
        assert!(e.file_saved(second, Err("read-only".into())).is_none());

        let report = e.finish_saving().unwrap();
        assert_eq!(report.written, 1);
        assert_eq!(report.failures, ["t2: read-only"]);
        assert!(e.saving.is_none());
    }

    #[test]
    fn unreadable_tracks_are_listed_apart() {
        let read = vec![
            (job(1, 1, None), PathBuf::from("/m/1.flac"), Ok(fields(&[]))),
            (
                job(2, 1, None),
                PathBuf::from("/m/2.flac"),
                Err("gone".to_string()),
            ),
        ];
        let e = TagEditor::new("al".into(), read);
        assert_eq!(e.tracks.len(), 1);
        assert_eq!(e.unreadable[0].title, "t2");
        assert_eq!(e.unreadable[0].reason, "gone");
    }

    #[test]
    fn fill_keeps_tracks_the_source_has_nothing_for() {
        let mut e = editor(&[
            &[(Field::Title, "a"), (Field::Album, "Old")],
            &[(Field::Title, "b"), (Field::Album, "Old")],
        ]);
        let source = fields(&[(Field::Title, "A"), (Field::Album, "New")]);
        e.fill(vec![Some(source), None], true);
        assert_eq!(e.tracks[0].text(Field::Title), "A");
        assert_eq!(e.tracks[1].text(Field::Title), "b");
        assert_eq!(e.album_field(Field::Album).text, "New");
    }

    #[test]
    fn fill_turns_a_mixed_flag_off() {
        let mut e = editor(&[&[(Field::Compilation, FLAG_ON)], &[]]);
        let off = || Some(fields(&[(Field::Compilation, "")]));
        e.fill(vec![off(), off()], true);
        assert_eq!(
            edits_of(&e)[0].get(&Field::Compilation),
            Some(&FieldEdit::Clear)
        );
    }

    mod musicbrainz_fill {
        use super::*;
        use qobuz_core::musicbrainz::{Medium, Recording, ReleaseTrack};

        /// A one-track release whose track has `isrc`.
        fn candidate(id: &str, title: &str, isrc: &str, confidence: u8) -> Candidate {
            let track = ReleaseTrack {
                position: 1,
                title: title.into(),
                recording: Recording {
                    isrcs: vec![isrc.into()],
                    ..Recording::default()
                },
                ..ReleaseTrack::default()
            };
            let release = Release {
                id: id.into(),
                title: "Kind of Blue".into(),
                date: Some("2003".into()),
                country: Some("US".into()),
                media: vec![Medium {
                    position: 1,
                    track_count: 1,
                    tracks: vec![track],
                    ..Medium::default()
                }],
                ..Release::default()
            };
            Candidate {
                release,
                confidence,
            }
        }

        fn searching() -> TagEditor {
            let mut e = editor(&[&[
                (Field::Title, "old"),
                (Field::Isrc, "USSM15900113"),
                (Field::TrackNumber, "1"),
            ]]);
            e.tracks[0].job.album.upc = Some("074646493564".into());
            assert!(e.start_lookup().is_some());
            e
        }

        #[test]
        fn lookup_reads_the_album_on_disk() {
            let mut e = editor(&[&[(Field::Isrc, "USSM15900113"), (Field::TrackNumber, "4")]]);
            e.tracks[0].job.album.upc = Some("074646493564".into());
            let album = e.start_lookup().unwrap();
            assert_eq!(album.barcode.as_deref(), Some("074646493564"));
            assert_eq!(album.tracks[0].isrc.as_deref(), Some("USSM15900113"));
            assert_eq!(album.tracks[0].number, Some(4));
            assert!(e.busy());
            assert!(e.start_lookup().is_none(), "one lookup at a time");
        }

        #[test]
        fn nothing_found_changes_nothing() {
            let mut e = searching();
            assert_eq!(e.found(Vec::new()), Some(Found::Nothing));
            assert!(e.lookup.is_none());
            assert!(!e.has_edits());
        }

        #[test]
        fn a_single_release_fills_directly() {
            let mut e = searching();
            let found = e.found(vec![candidate("r1", "So What", "USSM15900113", 96)]);
            let Some(Found::Chosen(Chosen::Filled(filled))) = found else {
                panic!("{found:?}");
            };
            assert!(e.lookup.is_none());
            assert_eq!(e.tracks[0].text(Field::Title), "So What");
            assert!(e.has_edits());
            assert_eq!(
                filled.status(),
                "Filled from MusicBrainz: Kind of Blue (2003, US) · 96 % match."
            );
        }

        #[test]
        fn several_releases_are_listed_until_one_is_chosen() {
            let mut e = searching();
            let found = e.found(vec![
                candidate("r1", "So What", "USSM15900113", 96),
                candidate("r2", "So What (mono)", "USSM15900113", 71),
            ]);
            assert_eq!(found, Some(Found::Choosing));
            assert!(matches!(e.lookup, Some(Lookup::Choosing(ref c)) if c.len() == 2));
            assert!(!e.has_edits());

            assert!(matches!(e.choose(1), Some(Chosen::Filled(_))));
            assert_eq!(e.tracks[0].text(Field::Title), "So What (mono)");
            assert!(e.lookup.is_none());
        }

        #[test]
        fn cancelling_the_list_leaves_every_field() {
            let mut e = searching();
            e.found(vec![
                candidate("r1", "So What", "USSM15900113", 96),
                candidate("r2", "Other", "USSM15900113", 71),
            ]);
            e.end_lookup();
            assert!(e.lookup.is_none());
            assert_eq!(e.tracks[0].text(Field::Title), "old");
            assert!(!e.has_edits());
        }

        #[test]
        fn unmatched_tracks_are_counted() {
            let mut e = editor(&[
                &[(Field::Isrc, "USSM15900113"), (Field::TrackNumber, "1")],
                &[(Field::Isrc, "XX0000000000"), (Field::TrackNumber, "9")],
            ]);
            e.start_lookup();
            let found = e.found(vec![candidate("r1", "So What", "USSM15900113", 90)]);
            let Some(Found::Chosen(Chosen::Filled(filled))) = found else {
                panic!("{found:?}");
            };
            assert_eq!(filled.unmatched, 1);
            assert!(filled.status().ends_with("1 track not matched."));
        }

        #[test]
        fn reset_waits_for_the_lookup() {
            let mut e = searching();
            e.apply(Edit::ResetToQobuz);
            assert_eq!(e.tracks[0].text(Field::Title), "old");
        }

        #[test]
        fn with_cover_nothing_is_filled_before_the_cover() {
            let mut e = searching();
            e.apply(Edit::IncludeCover(true));
            let found = e.found(vec![candidate("r1", "So What", "USSM15900113", 96)]);
            assert_eq!(found, Some(Found::Chosen(Chosen::NeedsCover("r1".into()))));
            assert!(e.busy());
            assert_eq!(e.tracks[0].text(Field::Title), "old");

            let filled = e
                .cover_fetched(Some(vec![1, 2, 3]), CoverSize::Px400)
                .unwrap();
            assert!(!filled.no_cover);
            assert_eq!(e.cover, CoverAction::Replace(vec![1, 2, 3]));
            assert_eq!(e.resize, Some(CoverSize::Px400));
            assert_eq!(e.tracks[0].text(Field::Title), "So What");
            assert!(!e.busy());
        }

        #[test]
        fn a_release_without_cover_still_fills() {
            let mut e = searching();
            e.include_cover = true;
            e.found(vec![candidate("r1", "So What", "USSM15900113", 96)]);
            let filled = e.cover_fetched(None, CoverSize::Px600).unwrap();
            assert!(filled.no_cover);
            assert!(filled
                .status()
                .ends_with("MusicBrainz has no cover for this release."));
            assert_eq!(e.cover, CoverAction::Keep);
            assert_eq!(e.tracks[0].text(Field::Title), "So What");
        }

        #[test]
        fn a_failed_cover_fills_nothing() {
            let mut e = searching();
            e.include_cover = true;
            e.found(vec![candidate("r1", "So What", "USSM15900113", 96)]);
            e.end_lookup();
            assert_eq!(e.tracks[0].text(Field::Title), "old");
            assert_eq!(e.cover, CoverAction::Keep);
            assert!(!e.has_edits());
        }

        #[test]
        fn few_matches_leave_the_album_fields_and_cover() {
            let file = |isrc: &'static str| [(Field::Isrc, isrc), (Field::Album, "Old")];
            let (a, b, c) = (
                file("USSM15900113"),
                file("XX0000000001"),
                file("XX0000000002"),
            );
            let mut e = editor(&[&a, &b, &c]);
            e.include_cover = true;
            e.start_lookup();
            let found = e.found(vec![candidate("r1", "So What", "USSM15900113", 40)]);
            assert_eq!(found, Some(Found::Choosing), "a weak match is listed");
            let Some(Chosen::Filled(filled)) = e.choose(0) else {
                panic!("filled at once: the cover would be left out");
            };

            assert!(filled.album_kept);
            assert!(e.lookup.is_none());
            assert_eq!(e.album_field(Field::Album).text, "Old");
            assert_eq!(e.cover, CoverAction::Keep);
            assert_eq!(
                e.tracks[0].text(Field::Title),
                "So What",
                "the match is still filled"
            );
            assert!(filled
                .status()
                .contains("Album fields and cover left as they were"));
        }

        #[test]
        fn release_named_by_what_tells_it_apart() {
            let mut r = candidate("r", "t", "i", 0).release;
            assert_eq!(release_name(&r), "Kind of Blue (2003, US)");
            r.country = None;
            r.date = None;
            assert_eq!(release_name(&r), "Kind of Blue");
        }
    }
}
