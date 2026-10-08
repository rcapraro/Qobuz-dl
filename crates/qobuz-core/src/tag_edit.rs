//! Reading and editing the tags of downloaded files.
//!
//! Edits go through each container's native tag via lofty's split/merge, so
//! everything outside the edited fields is kept, and every file is saved by
//! editing a copy and renaming it over the original.

use crate::artwork::{self, CoverSize};
use crate::download::PartFile;
use crate::error::{Error, Result};
use crate::tagging::{self, VORBIS_ADVISORY};
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::AudioFile;
use lofty::flac::FlacFile;
use lofty::id3::v2::Id3v2Tag;
use lofty::mp4::{Ilst, Mp4File};
use lofty::mpeg::MpegFile;
use lofty::ogg::{OggPictureStorage, VorbisComments};
use lofty::picture::{Picture, PictureInformation, PictureType};
use lofty::tag::items::Timestamp;
use lofty::tag::{Accessor, ItemKey, MergeTag, SplitTag, Tag, TagType};
use std::collections::BTreeMap;
use std::path::Path;

/// A tag the editor can change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field {
    Album,
    AlbumArtist,
    Date,
    Genre,
    Label,
    Copyright,
    DiscTotal,
    TrackTotal,
    Compilation,
    Title,
    Artist,
    TrackNumber,
    DiscNumber,
    Composer,
    Isrc,
    Explicit,
    Comment,
}

/// How a field's value is written and validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    /// A positive whole number.
    Number,
    /// `YYYY`, `YYYY-MM` or `YYYY-MM-DD`.
    Date,
    /// On (the value [`FLAG_ON`]) or off (absent).
    Flag,
}

/// The value of a flag field that is on; a flag that is off is absent.
pub const FLAG_ON: &str = "1";

/// The ID3v1 genre list (genres 0–79), as given in Appendix A of the ID3v2.3
/// specification, except that its misspelled "Psychadelic" is spelled
/// correctly and its truncated "AlternRock" is written in full. The editor
/// offers these as suggestions; any other genre can still be typed.
pub const STANDARD_GENRES: [&str; 80] = [
    "Blues",
    "Classic Rock",
    "Country",
    "Dance",
    "Disco",
    "Funk",
    "Grunge",
    "Hip-Hop",
    "Jazz",
    "Metal",
    "New Age",
    "Oldies",
    "Other",
    "Pop",
    "R&B",
    "Rap",
    "Reggae",
    "Rock",
    "Techno",
    "Industrial",
    "Alternative",
    "Ska",
    "Death Metal",
    "Pranks",
    "Soundtrack",
    "Euro-Techno",
    "Ambient",
    "Trip-Hop",
    "Vocal",
    "Jazz+Funk",
    "Fusion",
    "Trance",
    "Classical",
    "Instrumental",
    "Acid",
    "House",
    "Game",
    "Sound Clip",
    "Gospel",
    "Noise",
    "Alternative Rock",
    "Bass",
    "Soul",
    "Punk",
    "Space",
    "Meditative",
    "Instrumental Pop",
    "Instrumental Rock",
    "Ethnic",
    "Gothic",
    "Darkwave",
    "Techno-Industrial",
    "Electronic",
    "Pop-Folk",
    "Eurodance",
    "Dream",
    "Southern Rock",
    "Comedy",
    "Cult",
    "Gangsta",
    "Top 40",
    "Christian Rap",
    "Pop/Funk",
    "Jungle",
    "Native American",
    "Cabaret",
    "New Wave",
    "Psychedelic",
    "Rave",
    "Showtunes",
    "Trailer",
    "Lo-Fi",
    "Tribal",
    "Acid Punk",
    "Acid Jazz",
    "Polka",
    "Retro",
    "Musical",
    "Rock & Roll",
    "Hard Rock",
];

impl Field {
    /// Fields shared by an album, edited once for all its tracks.
    pub const ALBUM: [Field; 8] = [
        Field::Album,
        Field::AlbumArtist,
        Field::Date,
        Field::Genre,
        Field::Label,
        Field::Copyright,
        Field::DiscTotal,
        Field::Compilation,
    ];

    /// Fields edited per track. Total tracks is one of them: downloads write
    /// each disc's own count, so on a multi-disc album it differs by disc.
    pub const TRACK: [Field; 9] = [
        Field::Title,
        Field::Artist,
        Field::TrackNumber,
        Field::TrackTotal,
        Field::DiscNumber,
        Field::Composer,
        Field::Isrc,
        Field::Explicit,
        Field::Comment,
    ];

    pub fn kind(self) -> FieldKind {
        match self {
            Field::DiscTotal | Field::TrackTotal | Field::TrackNumber | Field::DiscNumber => {
                FieldKind::Number
            }
            Field::Date => FieldKind::Date,
            Field::Compilation | Field::Explicit => FieldKind::Flag,
            _ => FieldKind::Text,
        }
    }

    /// Whether `value` can be written to this field.
    pub fn accepts(self, value: &str) -> bool {
        self.normalize(value).is_some()
    }

    /// `value` in the form it is stored and compared in, or `None` if invalid.
    /// Surrounding whitespace never counts, so `" 1"` and `"01"` are `"1"`.
    pub fn normalize(self, value: &str) -> Option<String> {
        let value = value.trim();
        match self.kind() {
            FieldKind::Text => Some(value.to_string()),
            FieldKind::Number => parse_number(value).map(|n| n.to_string()),
            FieldKind::Date => parse_date(value).map(format_date),
            FieldKind::Flag => (value == FLAG_ON).then(|| FLAG_ON.to_string()),
        }
    }
}

fn parse_number(value: &str) -> Option<u32> {
    let all_digits = !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit());
    all_digits
        .then(|| value.parse().ok())
        .flatten()
        .filter(|n| *n > 0)
}

/// Parse `YYYY`, `YYYY-MM` or `YYYY-MM-DD`.
pub fn parse_date(value: &str) -> Option<Timestamp> {
    let mut parts = value.split('-');
    let year = digits(parts.next()?, 4)?.parse().ok()?;
    let mut date = Timestamp {
        year,
        ..Timestamp::default()
    };
    if let Some(month) = parts.next() {
        date.month = Some(
            digits(month, 2)?
                .parse()
                .ok()
                .filter(|m| (1..=12).contains(m))?,
        );
        if let Some(day) = parts.next() {
            date.day = Some(
                digits(day, 2)?
                    .parse()
                    .ok()
                    .filter(|d| (1..=31).contains(d))?,
            );
        }
    }
    parts.next().is_none().then_some(date)
}

/// `part` if it is exactly `len` ASCII digits.
fn digits(part: &str, len: usize) -> Option<&str> {
    (part.len() == len && part.bytes().all(|b| b.is_ascii_digit())).then_some(part)
}

/// A date as `YYYY`, `YYYY-MM` or `YYYY-MM-DD`, ignoring any time of day.
pub fn format_date(date: Timestamp) -> String {
    match (date.month, date.day) {
        (Some(m), Some(d)) => format!("{:04}-{m:02}-{d:02}", date.year),
        (Some(m), None) => format!("{:04}-{m:02}", date.year),
        _ => format!("{:04}", date.year),
    }
}

/// The editable tags of one file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagFields {
    /// Each present field's normalized value.
    pub values: BTreeMap<Field, String>,
    /// The front cover's bytes.
    pub cover: Option<Vec<u8>>,
}

impl TagFields {
    pub fn get(&self, field: Field) -> Option<&str> {
        self.values.get(&field).map(String::as_str)
    }
}

/// A change to one field. A field without an edit is kept as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldEdit {
    Set(String),
    Clear,
}

/// The field changes for one file.
pub type TagEdits = BTreeMap<Field, FieldEdit>;

/// What becomes of an album's front cover on save.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CoverAction {
    #[default]
    Keep,
    /// A JPEG or PNG image, embedded in every file.
    Replace(Vec<u8>),
    Remove,
}

/// The cover change applied to every file of an album.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoverEdit {
    pub action: CoverAction,
    /// Fit the cover each file ends up with within this size; ignored on Remove.
    pub resize: Option<CoverSize>,
}

/// A [`CoverEdit`] ready to apply to each file: a replacement is resized once
/// here rather than once per file. Blocking and CPU-bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverPlan(Plan);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Plan {
    Keep,
    Resize(CoverSize),
    Embed(Vec<u8>),
    Remove,
}

impl CoverPlan {
    pub fn new(edit: CoverEdit) -> Self {
        Self(match (edit.action, edit.resize) {
            (CoverAction::Keep, None) => Plan::Keep,
            (CoverAction::Keep, Some(size)) => Plan::Resize(size),
            (CoverAction::Replace(bytes), None) => Plan::Embed(bytes),
            (CoverAction::Replace(bytes), Some(size)) => {
                Plan::Embed(artwork::prepare_cover(bytes, size))
            }
            (CoverAction::Remove, _) => Plan::Remove,
        })
    }

    /// The cover a file whose cover is `current` should hold, or `None` when
    /// it keeps the one it has.
    fn target(&self, current: Option<&[u8]>) -> Option<Option<Vec<u8>>> {
        let wanted = match &self.0 {
            Plan::Keep => return None,
            Plan::Resize(size) => Some(artwork::prepare_cover(current?.to_vec(), *size)),
            Plan::Embed(bytes) => Some(bytes.clone()),
            Plan::Remove => None,
        };
        (wanted.as_deref() != current).then_some(wanted)
    }
}

/// Whether [`save_file`] wrote the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saved {
    Written,
    /// Every edit already matched the file, so it was left untouched.
    Unchanged,
}

/// The containers the editor handles, chosen by extension like the tags
/// written on download.
enum Container {
    Flac,
    Mp3,
    Mp4,
}

fn container_of(path: &Path) -> Result<Container> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("flac") => Ok(Container::Flac),
        Some("mp3") => Ok(Container::Mp3),
        Some("m4a") | Some("mp4") => Ok(Container::Mp4),
        _ => Err(Error::Tagging(format!(
            "can't edit the tags of {}",
            path.display()
        ))),
    }
}

/// The editable tags stored in the file at `path`.
pub fn read_fields(path: &Path) -> Result<TagFields> {
    let mut file = std::fs::File::open(path)?;
    let options = ParseOptions::new().read_properties(false);
    Ok(match container_of(path)? {
        Container::Flac => {
            let flac = FlacFile::read_from(&mut file, options)?;
            let comments = flac.vorbis_comments().cloned().unwrap_or_default();
            let explicit = vorbis_explicit(&comments);
            let (_, tag) = comments.split_tag();
            let mut fields = fields_of(&tag);
            if explicit {
                fields.values.insert(Field::Explicit, FLAG_ON.to_string());
            }
            fields.cover = front_cover(flac.pictures().iter().map(|(p, _)| p), false);
            fields
        }
        Container::Mp3 => {
            let mp3 = MpegFile::read_from(&mut file, options)?;
            let (_, tag) = mp3.id3v2().cloned().unwrap_or_default().split_tag();
            fields_of(&tag)
        }
        Container::Mp4 => {
            let mp4 = Mp4File::read_from(&mut file, options)?;
            let (_, tag) = mp4.ilst().cloned().unwrap_or_default().split_tag();
            fields_of(&tag)
        }
    })
}

fn fields_of(tag: &Tag) -> TagFields {
    let mut fields = TagFields::default();
    for field in Field::ALBUM.into_iter().chain(Field::TRACK) {
        if let Some(value) = get_field(tag, field) {
            fields.values.insert(field, value);
        }
    }
    fields.cover = front_cover(tag.pictures().iter(), tag.tag_type() == TagType::Mp4Ilst);
    fields
}

/// The front cover among `pictures`. MP4 cover atoms carry no picture type,
/// so there any picture is the cover.
fn front_cover<'a>(
    mut pictures: impl Iterator<Item = &'a Picture>,
    untyped: bool,
) -> Option<Vec<u8>> {
    pictures
        .find(|p| untyped || p.pic_type() == PictureType::CoverFront)
        .map(|p| p.data().to_vec())
}

fn vorbis_explicit(comments: &VorbisComments) -> bool {
    comments
        .get(VORBIS_ADVISORY)
        .is_some_and(advisory_is_explicit)
}

/// iTunes advisory values: 1 (or the older 4) is explicit, 2 is clean.
fn advisory_is_explicit(value: &str) -> bool {
    matches!(value.trim(), "1" | "4")
}

/// Whether `field` is stored outside lofty's generic tag in this container.
pub(crate) fn is_native(tag_type: TagType, field: Field) -> bool {
    tag_type == TagType::VorbisComments && field == Field::Explicit
}

/// `field`'s normalized value in a generic tag.
fn get_field(tag: &Tag, field: Field) -> Option<String> {
    let text = |key| tag.get_string(key).map(str::to_string);
    let value = match field {
        Field::Album => tag.album().map(|v| v.into_owned()),
        Field::AlbumArtist => text(ItemKey::AlbumArtist),
        Field::Date => tag.date().map(format_date),
        Field::Genre => tag.genre().map(|v| v.into_owned()),
        Field::Label => text(ItemKey::Label).or_else(|| {
            // ID3's TPUB is both publisher and label, and reads as Publisher.
            (tag.tag_type() == TagType::Id3v2)
                .then(|| text(ItemKey::Publisher))
                .flatten()
        }),
        Field::Copyright => text(ItemKey::CopyrightMessage),
        Field::DiscTotal => tag.disk_total().map(|n| n.to_string()),
        Field::TrackTotal => tag.track_total().map(|n| n.to_string()),
        Field::Compilation => tag
            .get_string(ItemKey::FlagCompilation)
            .filter(|v| matches!(v.trim(), "1" | "true"))
            .map(|_| FLAG_ON.to_string()),
        Field::Title => tag.title().map(|v| v.into_owned()),
        Field::Artist => tag.artist().map(|v| v.into_owned()),
        Field::TrackNumber => tag.track().map(|n| n.to_string()),
        Field::DiscNumber => tag.disk().map(|n| n.to_string()),
        Field::Composer => text(ItemKey::Composer),
        Field::Isrc => text(ItemKey::Isrc),
        Field::Explicit => tag
            .get_string(ItemKey::ParentalAdvisory)
            .filter(|v| advisory_is_explicit(v))
            .map(|_| FLAG_ON.to_string()),
        Field::Comment => tag.comment().map(|v| v.into_owned()),
    };
    value.filter(|v| !v.is_empty())
}

/// Write `value` to `field` in a generic tag, or remove the field for `None`.
/// The value must be normalized (see [`Field::accepts`]).
pub(crate) fn set_field(tag: &mut Tag, field: Field, value: Option<&str>) -> Result<()> {
    let number = |v: &str| {
        parse_number(v).ok_or_else(|| Error::Tagging(format!("{field:?}: not a number: {v}")))
    };
    let text = |tag: &mut Tag, key| match value {
        Some(v) => {
            tag.insert_text(key, v.to_string());
        }
        None => tag.remove_key(key),
    };
    match field {
        Field::Album => match value {
            Some(v) => tag.set_album(v.to_string()),
            None => tag.remove_album(),
        },
        Field::AlbumArtist => text(tag, ItemKey::AlbumArtist),
        Field::Date => match value {
            Some(v) => tag
                .set_date(parse_date(v).ok_or_else(|| Error::Tagging(format!("not a date: {v}")))?),
            None => tag.remove_date(),
        },
        Field::Genre => match value {
            Some(v) => tag.set_genre(v.to_string()),
            None => tag.remove_genre(),
        },
        Field::Label => {
            if tag.tag_type() == TagType::Id3v2 {
                // Both keys map to TPUB; keeping either would write it twice.
                tag.remove_key(ItemKey::Publisher);
            }
            text(tag, ItemKey::Label);
        }
        Field::Copyright => text(tag, ItemKey::CopyrightMessage),
        Field::DiscTotal => match value {
            Some(v) => tag.set_disk_total(number(v)?),
            None => tag.remove_disk_total(),
        },
        Field::TrackTotal => match value {
            Some(v) => tag.set_track_total(number(v)?),
            None => tag.remove_track_total(),
        },
        Field::Compilation => text(tag, ItemKey::FlagCompilation),
        Field::Title => match value {
            Some(v) => tag.set_title(v.to_string()),
            None => tag.remove_title(),
        },
        Field::Artist => match value {
            Some(v) => tag.set_artist(v.to_string()),
            None => tag.remove_artist(),
        },
        Field::TrackNumber => match value {
            Some(v) => tag.set_track(number(v)?),
            None => tag.remove_track(),
        },
        Field::DiscNumber => match value {
            Some(v) => tag.set_disk(number(v)?),
            None => tag.remove_disk(),
        },
        Field::Composer => text(tag, ItemKey::Composer),
        Field::Isrc => text(tag, ItemKey::Isrc),
        Field::Explicit => text(tag, ItemKey::ParentalAdvisory),
        Field::Comment => match value {
            Some(v) => tag.set_comment(v.to_string()),
            None => tag.remove_comment(),
        },
    }
    Ok(())
}

/// Apply `edits` to a generic tag, skipping fields stored natively and edits
/// that match what the tag already holds. Returns whether anything changed.
fn edit_tag(tag: &mut Tag, edits: &TagEdits) -> Result<bool> {
    let mut changed = false;
    for (&field, edit) in edits {
        if is_native(tag.tag_type(), field) {
            continue;
        }
        let wanted = wanted_value(field, edit)?;
        if get_field(tag, field) != wanted {
            set_field(tag, field, wanted.as_deref())?;
            changed = true;
        }
    }
    Ok(changed)
}

/// The normalized value an edit asks for, `None` meaning absent.
fn wanted_value(field: Field, edit: &FieldEdit) -> Result<Option<String>> {
    match edit {
        FieldEdit::Clear => Ok(None),
        FieldEdit::Set(value) if value.trim().is_empty() => Ok(None),
        FieldEdit::Set(value) => field
            .normalize(value)
            .map(Some)
            .ok_or_else(|| Error::Tagging(format!("invalid value for {field:?}: {value}"))),
    }
}

/// Replace the generic tag's front cover with `cover`, or drop it for `None`.
fn set_tag_cover(tag: &mut Tag, cover: Option<Vec<u8>>) {
    if tag.tag_type() == TagType::Mp4Ilst {
        while !tag.pictures().is_empty() {
            tag.remove_picture(0);
        }
    } else {
        tag.remove_picture_type(PictureType::CoverFront);
    }
    if let Some(picture) = cover.as_deref().and_then(tagging::build_picture) {
        tag.push_picture(picture);
    }
}

/// Apply `edits` and the album's `cover` change to the file at `path`. The
/// file is written only if something changes, and then through a copy renamed
/// over it, so its path always holds a complete file. Blocking.
pub fn save_file(path: &Path, edits: &TagEdits, cover: &CoverPlan) -> Result<Saved> {
    let mut file = std::fs::File::open(path)?;
    let options = ParseOptions::new().read_properties(false);
    match container_of(path)? {
        Container::Flac => {
            let mut flac = FlacFile::read_from(&mut file, options)?;
            let comments = flac.remove_vorbis_comments().unwrap_or_default();
            let explicit_before = vorbis_explicit(&comments);
            let (rest, mut tag) = comments.split_tag();
            let mut changed = edit_tag(&mut tag, edits)?;
            let mut comments = rest.merge_tag(tag);
            if let Some(edit) = edits.get(&Field::Explicit) {
                let explicit = wanted_value(Field::Explicit, edit)?.is_some();
                if explicit != explicit_before {
                    if explicit {
                        comments.insert(VORBIS_ADVISORY.to_string(), FLAG_ON.to_string());
                    } else {
                        comments.remove(VORBIS_ADVISORY).for_each(drop);
                    }
                    changed = true;
                }
            }
            flac.set_vorbis_comments(comments);

            let current = front_cover(flac.pictures().iter().map(|(p, _)| p), false);
            if let Some(target) = cover.target(current.as_deref()) {
                flac.remove_picture_type(PictureType::CoverFront);
                if let Some(picture) = target.as_deref().and_then(tagging::build_picture) {
                    let information =
                        PictureInformation::from_picture(&picture).unwrap_or_default();
                    flac.insert_picture(picture, Some(information))?;
                }
                changed = true;
            }
            write_if(changed, path, |part| {
                flac.save_to_path(part, WriteOptions::default())
            })
        }
        Container::Mp3 => {
            let mut mp3 = MpegFile::read_from(&mut file, options)?;
            let id3: Id3v2Tag = mp3.remove_id3v2().unwrap_or_default();
            let (rest, mut tag) = id3.split_tag();
            let changed = edit_generic_and_cover(&mut tag, edits, cover)?;
            mp3.set_id3v2(rest.merge_tag(tag));
            write_if(changed, path, |part| {
                mp3.save_to_path(part, WriteOptions::default())
            })
        }
        Container::Mp4 => {
            let mut mp4 = Mp4File::read_from(&mut file, options)?;
            let ilst: Ilst = mp4.remove_ilst().unwrap_or_default();
            let (rest, mut tag) = ilst.split_tag();
            let changed = edit_generic_and_cover(&mut tag, edits, cover)?;
            mp4.set_ilst(rest.merge_tag(tag));
            write_if(changed, path, |part| {
                mp4.save_to_path(part, WriteOptions::default())
            })
        }
    }
}

/// [`edit_tag`] plus the cover, for containers whose pictures are in the tag.
fn edit_generic_and_cover(tag: &mut Tag, edits: &TagEdits, cover: &CoverPlan) -> Result<bool> {
    let mut changed = edit_tag(tag, edits)?;
    let untyped = tag.tag_type() == TagType::Mp4Ilst;
    let current = front_cover(tag.pictures().iter(), untyped);
    if let Some(target) = cover.target(current.as_deref()) {
        set_tag_cover(tag, target);
        changed = true;
    }
    Ok(changed)
}

/// Save through a copy of `path` with `save`, then rename the copy over it.
fn write_if(
    changed: bool,
    path: &Path,
    save: impl FnOnce(&Path) -> lofty::error::Result<()>,
) -> Result<Saved> {
    if !changed {
        return Ok(Saved::Unchanged);
    }
    let part = PartFile::copy_of(path)?;
    save(part.path())?;
    part.publish_sync(path)?;
    Ok(Saved::Written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Album, Track};
    use crate::tagging::{fields_from, write_tags, TrackTags};
    use crate::test_support::{write_minimal_flac, write_minimal_m4a, write_minimal_mp3, Scratch};
    use image::{ImageOutputFormat, Rgb, RgbImage};
    use std::io::Cursor;

    fn png(side: u32, shade: u8) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        RgbImage::from_pixel(side, side, Rgb([shade, 40, 90]))
            .write_to(&mut out, ImageOutputFormat::Png)
            .unwrap();
        out.into_inner()
    }

    fn side_of(bytes: &[u8]) -> (u32, u32) {
        image::load_from_memory(bytes)
            .unwrap()
            .into_rgb8()
            .dimensions()
    }

    fn track() -> Track {
        serde_json::from_value(serde_json::json!({
            "id": 1, "title": "So What", "track_number": 1, "media_number": 1,
            "performer": {"name": "Miles Davis"}, "composer": {"name": "Miles Davis"},
            "isrc": "USSM15900113", "parental_warning": true
        }))
        .unwrap()
    }

    fn album() -> Album {
        serde_json::from_value(serde_json::json!({
            "id": "a", "title": "Kind of Blue", "artist": {"name": "Miles Davis"},
            "release_date_original": "1959-08-17", "genre": {"name": "Jazz"},
            "media_count": 1, "label": {"name": "Columbia"},
            "copyright": "(P) 1959 Columbia"
        }))
        .unwrap()
    }

    /// A file named `name`, made by `write` and tagged as a download would be.
    fn downloaded(name: &str, write: fn(&Path), cover: Option<&[u8]>) -> Scratch {
        let file = Scratch::new(name);
        write(file.path());
        let tags = TrackTags {
            track: &track(),
            album: &album(),
            disc_track_total: Some(5),
            cover,
        };
        write_tags(file.path(), file.path(), &tags).unwrap();
        file
    }

    fn flac(cover: Option<&[u8]>) -> Scratch {
        downloaded("song.flac", write_minimal_flac, cover)
    }

    fn mp3(cover: Option<&[u8]>) -> Scratch {
        downloaded("song.mp3", write_minimal_mp3, cover)
    }

    fn m4a(cover: Option<&[u8]>) -> Scratch {
        downloaded("song.m4a", write_minimal_m4a, cover)
    }

    fn set(pairs: &[(Field, &str)]) -> TagEdits {
        pairs
            .iter()
            .map(|(f, v)| (*f, FieldEdit::Set(v.to_string())))
            .collect()
    }

    fn keep() -> CoverPlan {
        CoverPlan::new(CoverEdit::default())
    }

    fn cover_plan(action: CoverAction, resize: Option<CoverSize>) -> CoverPlan {
        CoverPlan::new(CoverEdit { action, resize })
    }

    fn modified(path: &Path) -> std::time::SystemTime {
        std::fs::metadata(path).unwrap().modified().unwrap()
    }

    #[test]
    fn dates_parse_in_three_shapes_only() {
        assert_eq!(parse_date("1959").map(format_date).as_deref(), Some("1959"));
        assert_eq!(
            parse_date("1959-08").map(format_date).as_deref(),
            Some("1959-08")
        );
        assert_eq!(
            parse_date("1959-08-17").map(format_date).as_deref(),
            Some("1959-08-17")
        );
        for bad in [
            "59",
            "1959-8",
            "1959-13",
            "1959-08-32",
            "1959-08-17-1",
            "195a",
            "",
        ] {
            assert!(parse_date(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn standard_genres_keep_their_id3v1_numbers() {
        assert_eq!(STANDARD_GENRES[0], "Blues");
        assert_eq!(STANDARD_GENRES[8], "Jazz");
        assert_eq!(STANDARD_GENRES[32], "Classical");
        assert_eq!(STANDARD_GENRES[79], "Hard Rock");
    }

    #[test]
    fn fields_validate_by_kind() {
        assert!(Field::TrackNumber.accepts("3"));
        assert!(!Field::TrackNumber.accepts("1a"));
        assert!(!Field::TrackNumber.accepts("0"));
        assert!(Field::Date.accepts("1959"));
        assert!(Field::Title.accepts("anything"));
        assert!(Field::Explicit.accepts(FLAG_ON));
        assert!(!Field::Explicit.accepts("yes"));
    }

    #[test]
    fn reads_back_what_a_download_wrote() {
        let cover = png(600, 10);
        let mut expected = fields_from(&track(), &album(), Some(5));
        expected.cover = Some(cover.clone());
        for file in [flac(Some(&cover)), mp3(Some(&cover)), m4a(Some(&cover))] {
            assert_eq!(
                read_fields(file.path()).unwrap(),
                expected,
                "{:?}",
                file.path()
            );
        }
    }

    #[test]
    fn every_field_can_be_set_and_cleared() {
        let values = [
            (Field::Album, "Kind of Blue (Remaster)"),
            (Field::AlbumArtist, "Davis"),
            (Field::Date, "1997-03"),
            (Field::Genre, "Modal Jazz"),
            (Field::Label, "Legacy"),
            (Field::Copyright, "(P) 1997 Sony"),
            (Field::DiscTotal, "2"),
            (Field::TrackTotal, "9"),
            (Field::Compilation, FLAG_ON),
            (Field::Title, "So What (Take 1)"),
            (Field::Artist, "Miles Davis Sextet"),
            (Field::TrackNumber, "3"),
            (Field::DiscNumber, "2"),
            (Field::Composer, "Davis / Evans"),
            (Field::Isrc, "USSM19999999"),
            (Field::Explicit, FLAG_ON),
            (Field::Comment, "Remastered"),
        ];
        for file in [flac(None), mp3(None), m4a(None)] {
            let path = file.path();
            assert_eq!(
                save_file(path, &set(&values), &keep()).unwrap(),
                Saved::Written
            );
            let read = read_fields(path).unwrap();
            for (field, value) in values {
                assert_eq!(read.get(field), Some(value), "{field:?} in {path:?}");
            }

            let clear: TagEdits = values.iter().map(|(f, _)| (*f, FieldEdit::Clear)).collect();
            save_file(path, &clear, &keep()).unwrap();
            let read = read_fields(path).unwrap();
            assert!(read.values.is_empty(), "{path:?}: {:?}", read.values);
        }
    }

    #[test]
    fn turning_explicit_off_removes_the_flag() {
        let file = flac(None);
        let edits = TagEdits::from([(Field::Explicit, FieldEdit::Clear)]);
        save_file(file.path(), &edits, &keep()).unwrap();
        let flac = FlacFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            ParseOptions::new(),
        )
        .unwrap();
        assert_eq!(flac.vorbis_comments().unwrap().get(VORBIS_ADVISORY), None);
    }

    #[test]
    fn unlisted_tags_survive_an_edit() {
        let file = flac(None);
        let mut flac = FlacFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            ParseOptions::new(),
        )
        .unwrap();
        flac.vorbis_comments_mut()
            .unwrap()
            .insert("REPLAYGAIN_TRACK_GAIN".into(), "-7.20 dB".into());
        flac.save_to_path(file.path(), WriteOptions::default())
            .unwrap();

        save_file(file.path(), &set(&[(Field::Genre, "Bebop")]), &keep()).unwrap();

        let flac = FlacFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            ParseOptions::new(),
        )
        .unwrap();
        let comments = flac.vorbis_comments().unwrap();
        assert_eq!(comments.get("REPLAYGAIN_TRACK_GAIN"), Some("-7.20 dB"));
        assert_eq!(comments.get("GENRE"), Some("Bebop"));
    }

    #[test]
    fn mp3_label_is_written_once() {
        let file = mp3(None);
        save_file(file.path(), &set(&[(Field::Label, "Legacy")]), &keep()).unwrap();
        let mp3 = MpegFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            ParseOptions::new(),
        )
        .unwrap();
        let publishers = mp3
            .id3v2()
            .unwrap()
            .into_iter()
            .filter(|frame| frame.id_str() == "TPUB")
            .count();
        assert_eq!(publishers, 1);
        assert_eq!(
            read_fields(file.path()).unwrap().get(Field::Label),
            Some("Legacy")
        );
    }

    #[test]
    fn replacing_the_cover_leaves_one_front_cover() {
        let replacement = png(300, 200);
        for file in [flac(Some(&png(600, 10))), mp3(Some(&png(600, 10)))] {
            let plan = cover_plan(CoverAction::Replace(replacement.clone()), None);
            save_file(file.path(), &TagEdits::new(), &plan).unwrap();
            let tagged = lofty::read_from_path(file.path()).unwrap();
            let pictures = lofty::file::TaggedFileExt::primary_tag(&tagged)
                .unwrap()
                .pictures();
            let fronts: Vec<_> = pictures
                .iter()
                .filter(|p| p.pic_type() == PictureType::CoverFront)
                .collect();
            assert_eq!(fronts.len(), 1, "{:?}", file.path());
            assert_eq!(fronts[0].data(), replacement);
        }
    }

    #[test]
    fn removing_the_cover_keeps_other_pictures() {
        let file = flac(Some(&png(600, 10)));
        let mut flac = FlacFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            ParseOptions::new(),
        )
        .unwrap();
        let back = Picture::unchecked(png(100, 0))
            .pic_type(PictureType::CoverBack)
            .mime_type(lofty::picture::MimeType::Png)
            .build();
        flac.insert_picture(back, None).unwrap();
        flac.save_to_path(file.path(), WriteOptions::default())
            .unwrap();

        save_file(
            file.path(),
            &TagEdits::new(),
            &cover_plan(CoverAction::Remove, None),
        )
        .unwrap();

        let flac = FlacFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            ParseOptions::new(),
        )
        .unwrap();
        let types: Vec<_> = flac.pictures().iter().map(|(p, _)| p.pic_type()).collect();
        assert_eq!(types, [PictureType::CoverBack]);
    }

    #[test]
    fn embedded_cover_is_resized_and_other_tags_kept() {
        for file in [flac(Some(&png(600, 10))), mp3(Some(&png(600, 10)))] {
            let before = read_fields(file.path()).unwrap();
            let plan = cover_plan(CoverAction::Keep, Some(CoverSize::Px400));
            assert_eq!(
                save_file(file.path(), &TagEdits::new(), &plan).unwrap(),
                Saved::Written
            );
            let after = read_fields(file.path()).unwrap();
            assert_eq!(side_of(after.cover.as_deref().unwrap()), (400, 400));
            assert_eq!(after.values, before.values);
        }
    }

    #[test]
    fn each_file_keeps_its_own_cover_when_resized() {
        let large = flac(Some(&png(1200, 10)));
        let small = flac(Some(&png(600, 250)));
        let plan = cover_plan(CoverAction::Keep, Some(CoverSize::Px500));
        for file in [&large, &small] {
            save_file(file.path(), &TagEdits::new(), &plan).unwrap();
        }
        let first = read_fields(large.path()).unwrap().cover.unwrap();
        let second = read_fields(small.path()).unwrap().cover.unwrap();
        assert_eq!(side_of(&first), (500, 500));
        assert_eq!(side_of(&second), (500, 500));
        assert_ne!(first, second);
    }

    #[test]
    fn cover_already_small_enough_leaves_the_file_unwritten() {
        let file = flac(Some(&png(450, 10)));
        let before = std::fs::read(file.path()).unwrap();
        let stamp = modified(file.path());
        let plan = cover_plan(CoverAction::Keep, Some(CoverSize::Px500));
        assert_eq!(
            save_file(file.path(), &TagEdits::new(), &plan).unwrap(),
            Saved::Unchanged
        );
        assert_eq!(std::fs::read(file.path()).unwrap(), before);
        assert_eq!(modified(file.path()), stamp);
    }

    #[test]
    fn replacement_is_resized_once_for_every_file() {
        let plan = cover_plan(CoverAction::Replace(png(3000, 99)), Some(CoverSize::Px500));
        let files = [flac(None), flac(Some(&png(600, 10)))];
        for file in &files {
            save_file(file.path(), &TagEdits::new(), &plan).unwrap();
        }
        let covers: Vec<_> = files
            .iter()
            .map(|f| read_fields(f.path()).unwrap().cover.unwrap())
            .collect();
        assert_eq!(side_of(&covers[0]), (500, 500));
        assert_eq!(covers[0], covers[1]);
    }

    #[test]
    fn edits_matching_the_file_leave_it_unwritten() {
        let file = flac(None);
        let stamp = modified(file.path());
        let edits = set(&[(Field::Album, "Kind of Blue"), (Field::TrackNumber, "01")]);
        assert_eq!(
            save_file(file.path(), &edits, &keep()).unwrap(),
            Saved::Unchanged
        );
        assert_eq!(modified(file.path()), stamp);
    }

    #[test]
    fn saving_replaces_the_file_at_its_path() {
        let file = flac(None);
        let before = std::fs::read(file.path()).unwrap();
        save_file(
            file.path(),
            &set(&[(Field::Title, "Blue in Green")]),
            &keep(),
        )
        .unwrap();
        assert_ne!(std::fs::read(file.path()).unwrap(), before);
        assert_eq!(
            read_fields(file.path()).unwrap().get(Field::Title),
            Some("Blue in Green")
        );
        assert!(no_part_files_beside(file.path()));
    }

    #[test]
    fn read_only_file_is_reported_and_left_unchanged() {
        let file = flac(None);
        let before = std::fs::read(file.path()).unwrap();
        let mut permissions = std::fs::metadata(file.path()).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(file.path(), permissions.clone()).unwrap();

        let result = save_file(file.path(), &set(&[(Field::Title, "x")]), &keep());

        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(file.path(), permissions).unwrap();
        assert!(result.is_err());
        assert_eq!(std::fs::read(file.path()).unwrap(), before);
        assert!(no_part_files_beside(file.path()));
    }

    #[test]
    fn unsupported_file_is_an_error() {
        assert!(read_fields(Path::new("/tmp/song.wav")).is_err());
    }

    fn no_part_files_beside(path: &Path) -> bool {
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .all(|entry| {
                let name = entry.unwrap().file_name().to_string_lossy().to_string();
                !(name.starts_with(&stem) && name.contains(".part"))
            })
    }
}
