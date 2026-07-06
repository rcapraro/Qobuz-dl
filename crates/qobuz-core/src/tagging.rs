//! Writing audio tags and embedding cover art via `lofty`.

use crate::error::Result;
use crate::models::{Album, Track};
use lofty::config::WriteOptions;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::tag::{Accessor, ItemKey, Tag, TagExt, TagType};
use std::path::Path;

/// Metadata to write to a downloaded file.
pub struct TrackTags<'a> {
    pub track: &'a Track,
    pub album: &'a Album,
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

/// Write tags (and optionally embed cover art) to `path`.
pub fn write_tags(path: &Path, tags: &TrackTags<'_>) -> Result<()> {
    let mut tag = Tag::new(tag_type_for(path));

    tag.set_title(tags.track.title.clone());
    tag.set_artist(tags.track.artist_name().to_string());
    tag.set_album(tags.album.title.clone());

    if let Some(tn) = tags.track.track_number {
        tag.set_track(tn);
    }
    let disc = tags.track.disc_number();
    tag.set_disk(disc);

    if let Some(year) = tags.album.year().and_then(|y| y.parse::<u32>().ok()) {
        tag.set_year(year);
    }
    if let Some(g) = tags.album.genre.as_ref().and_then(|g| g.name.clone()) {
        tag.set_genre(g);
    }

    // Album artist, composer, ISRC via generic item keys.
    tag.insert_text(ItemKey::AlbumArtist, tags.album.artist_name().to_string());
    if let Some(c) = tags.track.composer.as_ref().and_then(|c| c.name.clone()) {
        tag.insert_text(ItemKey::Composer, c);
    }
    if let Some(isrc) = tags.track.isrc.clone() {
        tag.insert_text(ItemKey::Isrc, isrc);
    }

    if let Some(bytes) = tags.cover {
        if let Some(pic) = build_picture(bytes) {
            tag.push_picture(pic);
        }
    }

    tag.save_to_path(path, WriteOptions::default())?;
    Ok(())
}

fn build_picture(bytes: &[u8]) -> Option<Picture> {
    let mime = sniff_mime(bytes);
    Some(Picture::new_unchecked(
        PictureType::CoverFront,
        Some(mime),
        None,
        bytes.to_vec(),
    ))
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
    fn mime_sniff() {
        assert!(matches!(sniff_mime(&[0x89, b'P', b'N', b'G', 0]), MimeType::Png));
        assert!(matches!(sniff_mime(&[0xFF, 0xD8, 0xFF]), MimeType::Jpeg));
    }
}
