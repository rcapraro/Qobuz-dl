//! Writing audio tags and embedding cover art via `lofty`.

use crate::error::Result;
use crate::models::{Album, Track};
use lofty::config::WriteOptions;
use lofty::ogg::{OggPictureStorage, VorbisComments};
use lofty::picture::{MimeType, Picture, PictureInformation, PictureType};
use lofty::tag::items::Timestamp;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt, TagType};
use std::path::Path;

/// Metadata to write to a downloaded file.
pub struct TrackTags<'a> {
    pub track: &'a Track,
    pub album: &'a Album,
    /// Number of tracks on the track's disc, when known.
    pub disc_track_total: Option<u32>,
    /// JPEG/PNG cover bytes to embed, if enabled and available.
    pub cover: Option<&'a [u8]>,
}

/// Pick the appropriate tag container for a file extension.
fn tag_type_for(path: &Path) -> TagType {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("flac") | Some("ogg") | Some("opus") => TagType::VorbisComments,
        Some("m4a") | Some("mp4") | Some("aac") | Some("alac") => TagType::Mp4Ilst,
        _ => TagType::Id3v2,
    }
}

/// Write tags (and optionally embed cover art) to `path`, in the tag container
/// `dest`'s extension calls for. They differ while the download is still a
/// `.partN` temp file, whose extension says nothing about its format.
pub fn write_tags(path: &Path, dest: &Path, tags: &TrackTags<'_>) -> Result<()> {
    let tag_type = tag_type_for(dest);
    let mut tag = Tag::new(tag_type);

    tag.set_title(tags.track.title.clone());
    tag.set_artist(tags.track.artist_name().to_string());
    tag.set_album(tags.album.title.clone());

    if let Some(tn) = tags.track.track_number {
        tag.set_track(tn);
    }
    if let Some(total) = tags.disc_track_total {
        tag.set_track_total(total);
    }
    let disc = tags.track.disc_number();
    tag.set_disk(disc);
    if let Some(discs) = tags.album.media_count {
        tag.set_disk_total(discs);
    }

    if let Some(date) = release_date(tags.album) {
        tag.set_date(date);
    }
    if let Some(g) = tags.album.genre.as_ref().and_then(|g| g.name.clone()) {
        tag.set_genre(g);
    }

    tag.insert_text(ItemKey::AlbumArtist, tags.album.artist_name().to_string());
    if let Some(c) = tags.track.composer.as_ref().and_then(|c| c.name.clone()) {
        tag.insert_text(ItemKey::Composer, c);
    }
    if let Some(isrc) = tags.track.isrc.clone() {
        tag.insert_text(ItemKey::Isrc, isrc);
    }
    if let Some(label) = tags.album.label.as_ref().and_then(|l| l.name.clone()) {
        tag.insert_text(ItemKey::Label, label);
    }
    if let Some(copyright) = tags.album.copyright.clone() {
        tag.insert_text(ItemKey::CopyrightMessage, copyright);
    }

    let picture = tags.cover.and_then(build_picture);
    // A clean track gets no flag: Qobuz doesn't tell "clean" from "not explicit".
    let explicit = tags.track.is_explicit();
    if tag_type == TagType::VorbisComments {
        // lofty maps no generic key to Vorbis' advisory field, so it is set on
        // the native tag.
        let mut comments = VorbisComments::from(tag);
        if explicit {
            comments.insert(VORBIS_ADVISORY.to_string(), EXPLICIT.to_string());
        }
        if let Some(picture) = picture {
            // Converting a generic tag drops pictures whose header lofty can't
            // parse; its generic save keeps them, so this does too.
            let information = PictureInformation::from_picture(&picture).unwrap_or_default();
            comments.insert_picture(picture, Some(information))?;
        }
        comments.save_to_path(path, WriteOptions::default())?;
    } else {
        if explicit {
            tag.insert_text(ItemKey::ParentalAdvisory, EXPLICIT.to_string());
        }
        if let Some(picture) = picture {
            tag.push_picture(picture);
        }
        tag.save_to_path(path, WriteOptions::default())?;
    }
    Ok(())
}

/// The iTunes advisory field taggers and players read in Vorbis comments.
const VORBIS_ADVISORY: &str = "ITUNESADVISORY";
/// iTunes advisory value for explicit content, in every container.
const EXPLICIT: &str = "1";

/// The album's `YYYY-MM-DD` release date. The year is read as before, from the
/// first four characters, so an unexpected date shape still yields it; month
/// and day are kept only when valid, since taggers reject a date like `-00-00`.
fn release_date(album: &Album) -> Option<Timestamp> {
    let year = album.year()?.parse().ok()?;
    let mut rest = album.release_date_original.as_deref()?.split('-').skip(1);
    let month = rest
        .next()
        .and_then(|m| m.parse().ok())
        .filter(|m| (1..=12).contains(m));
    let day = month.and(
        rest.next()
            .and_then(|d| d.parse().ok())
            .filter(|d| (1..=31).contains(d)),
    );
    Some(Timestamp {
        year,
        month,
        day,
        ..Timestamp::default()
    })
}

fn build_picture(bytes: &[u8]) -> Option<Picture> {
    let mime = sniff_mime(bytes);
    Some(
        Picture::unchecked(bytes.to_vec())
            .pic_type(PictureType::CoverFront)
            .mime_type(mime)
            .build(),
    )
}

fn sniff_mime(bytes: &[u8]) -> MimeType {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        MimeType::Png
    } else {
        MimeType::Jpeg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn tag_type_by_extension() {
        assert_eq!(tag_type_for(Path::new("a.flac")), TagType::VorbisComments);
        assert_eq!(tag_type_for(Path::new("a.m4a")), TagType::Mp4Ilst);
        assert_eq!(tag_type_for(Path::new("a.mp3")), TagType::Id3v2);
    }

    #[test]
    fn part_file_gets_the_destination_container() {
        use crate::test_support::{write_minimal_flac, Scratch};
        use lofty::file::TaggedFileExt;

        let part = Scratch::new("song.part0");
        write_minimal_flac(part.path());
        let track: Track =
            serde_json::from_value(serde_json::json!({"id": 1, "title": "t"})).unwrap();
        let album: Album =
            serde_json::from_value(serde_json::json!({"id": "a", "title": "a"})).unwrap();
        let tags = TrackTags {
            track: &track,
            album: &album,
            disc_track_total: None,
            cover: None,
        };

        write_tags(part.path(), Path::new("song.flac"), &tags).unwrap();

        let tagged = lofty::probe::Probe::open(part.path())
            .unwrap()
            .guess_file_type()
            .unwrap()
            .read()
            .unwrap();
        assert!(tagged.tag(TagType::VorbisComments).is_some());
        assert!(tagged.tag(TagType::Id3v2).is_none());
    }

    fn explicit_track() -> Track {
        serde_json::from_value(serde_json::json!({
            "id": 1, "title": "t", "track_number": 2, "media_number": 1,
            "parental_warning": true
        }))
        .unwrap()
    }

    fn full_album() -> Album {
        serde_json::from_value(serde_json::json!({
            "id": "a", "title": "a", "media_count": 2,
            "release_date_original": "1959-08-17",
            "label": {"name": "Columbia"},
            "copyright": "(P) 1959 Columbia"
        }))
        .unwrap()
    }

    /// Write `track`/`album` tags to a fresh `name` file made by `write`, and
    /// read them back through lofty's generic view.
    fn tag_and_read(
        name: &str,
        write: fn(&Path),
        track: &Track,
        album: &Album,
    ) -> (crate::test_support::Scratch, Tag) {
        use lofty::file::TaggedFileExt;

        let file = crate::test_support::Scratch::new(name);
        write(file.path());
        let tags = TrackTags {
            track,
            album,
            disc_track_total: Some(9),
            cover: None,
        };
        write_tags(file.path(), file.path(), &tags).unwrap();
        let tag = lofty::read_from_path(file.path())
            .unwrap()
            .primary_tag()
            .unwrap()
            .clone();
        (file, tag)
    }

    fn assert_extended_fields(tag: &Tag) {
        assert_eq!(tag.track(), Some(2));
        assert_eq!(tag.track_total(), Some(9));
        assert_eq!(tag.disk_total(), Some(2));
        // ID3v2 stores the label in TPUB, which lofty reads back as Publisher.
        let label = tag
            .get_string(ItemKey::Label)
            .or(tag.get_string(ItemKey::Publisher));
        assert_eq!(label, Some("Columbia"));
        assert_eq!(
            tag.get_string(ItemKey::CopyrightMessage),
            Some("(P) 1959 Columbia")
        );
        let date = tag.date().unwrap();
        assert_eq!((date.year, date.month, date.day), (1959, Some(8), Some(17)));
    }

    #[test]
    fn flac_gets_extended_fields_and_explicit_flag() {
        use crate::test_support::write_minimal_flac;
        use lofty::file::AudioFile;

        let (file, tag) = tag_and_read(
            "song.flac",
            write_minimal_flac,
            &explicit_track(),
            &full_album(),
        );
        assert_extended_fields(&tag);

        let flac = lofty::flac::FlacFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            lofty::config::ParseOptions::new(),
        )
        .unwrap();
        let comments = flac.vorbis_comments().unwrap();
        assert_eq!(comments.get(VORBIS_ADVISORY), Some(EXPLICIT));
    }

    #[test]
    fn mp3_gets_extended_fields_and_explicit_flag() {
        use crate::test_support::write_minimal_mp3;
        use lofty::file::AudioFile;

        let (file, tag) = tag_and_read(
            "song.mp3",
            write_minimal_mp3,
            &explicit_track(),
            &full_album(),
        );
        assert_extended_fields(&tag);

        let mp3 = lofty::mpeg::MpegFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            lofty::config::ParseOptions::new(),
        )
        .unwrap();
        let id3 = mp3.id3v2().unwrap();
        assert_eq!(id3.get_user_text(VORBIS_ADVISORY), Some(EXPLICIT));
    }

    #[test]
    fn flac_keeps_its_cover() {
        use crate::test_support::{write_minimal_flac, Scratch};
        use lofty::file::TaggedFileExt;

        let file = Scratch::new("song.flac");
        write_minimal_flac(file.path());
        let cover = [0xFF, 0xD8, 0xFF, 0xE0, 0, 0];
        let tags = TrackTags {
            track: &explicit_track(),
            album: &full_album(),
            disc_track_total: None,
            cover: Some(&cover),
        };
        write_tags(file.path(), file.path(), &tags).unwrap();

        let tagged = lofty::read_from_path(file.path()).unwrap();
        let pictures = tagged.primary_tag().unwrap().pictures();
        assert_eq!(pictures.len(), 1);
        assert_eq!(pictures[0].pic_type(), PictureType::CoverFront);
        assert_eq!(pictures[0].data(), cover);
    }

    #[test]
    fn mp4_advisory_converts_to_explicit_rating() {
        use lofty::mp4::{AdvisoryRating, Ilst};

        let mut tag = Tag::new(TagType::Mp4Ilst);
        tag.insert_text(ItemKey::ParentalAdvisory, EXPLICIT.to_string());
        let ilst = Ilst::from(tag);
        assert_eq!(ilst.advisory_rating(), Some(AdvisoryRating::Explicit));
    }

    #[test]
    fn clean_track_and_missing_fields_are_omitted() {
        use crate::test_support::write_minimal_flac;
        use lofty::file::AudioFile;

        let track: Track =
            serde_json::from_value(serde_json::json!({"id": 1, "title": "t"})).unwrap();
        let album: Album =
            serde_json::from_value(serde_json::json!({"id": "a", "title": "a"})).unwrap();
        let (file, tag) = tag_and_read("song.flac", write_minimal_flac, &track, &album);
        assert_eq!(tag.get_string(ItemKey::CopyrightMessage), None);
        assert_eq!(tag.get_string(ItemKey::Label), None);
        assert_eq!(tag.disk_total(), None);

        let flac = lofty::flac::FlacFile::read_from(
            &mut std::fs::File::open(file.path()).unwrap(),
            lofty::config::ParseOptions::new(),
        )
        .unwrap();
        assert_eq!(flac.vorbis_comments().unwrap().get(VORBIS_ADVISORY), None);
    }

    #[test]
    fn release_date_keeps_the_parts_that_parse() {
        let album = |date: &str| -> Album {
            serde_json::from_value(serde_json::json!({
                "id": "a", "title": "a", "release_date_original": date
            }))
            .unwrap()
        };
        let full = release_date(&album("1959-08-17")).unwrap();
        assert_eq!((full.year, full.month, full.day), (1959, Some(8), Some(17)));
        let year_only = release_date(&album("1959")).unwrap();
        assert_eq!((year_only.year, year_only.month), (1959, None));
        let zeroed = release_date(&album("1959-00-00")).unwrap();
        assert_eq!((zeroed.year, zeroed.month, zeroed.day), (1959, None, None));
        let bad_day = release_date(&album("1959-08-32")).unwrap();
        assert_eq!((bad_day.month, bad_day.day), (Some(8), None));
        let compact = release_date(&album("19590817")).unwrap();
        assert_eq!((compact.year, compact.month), (1959, None));
        assert!(release_date(&album("unknown")).is_none());
    }

    #[test]
    fn mime_sniff() {
        assert!(matches!(
            sniff_mime(&[0x89, b'P', b'N', b'G', 0]),
            MimeType::Png
        ));
        assert!(matches!(sniff_mime(&[0xFF, 0xD8, 0xFF]), MimeType::Jpeg));
    }
}
