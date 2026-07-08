//! Serde models for the subset of the Qobuz JSON API we consume.
//!
//! The API returns many more fields than modeled here; unknown fields are
//! ignored. Only what we need for downloading, naming, and tagging is captured.

use serde::Deserialize;

/// An artist reference as embedded in albums/tracks.
#[derive(Debug, Clone, Deserialize)]
pub struct ArtistRef {
    pub id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
}

/// Album cover art URLs.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Image {
    #[serde(default)]
    pub large: Option<String>,
    #[serde(default)]
    pub small: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
}

impl Image {
    /// Best available cover URL.
    pub fn best(&self) -> Option<&str> {
        self.large
            .as_deref()
            .or(self.small.as_deref())
            .or(self.thumbnail.as_deref())
    }
}

/// An album, as returned by `album/get` (and embedded in track/search results).
#[derive(Debug, Clone, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artist: Option<ArtistRef>,
    #[serde(default)]
    pub image: Option<Image>,
    /// Release date "YYYY-MM-DD".
    #[serde(default)]
    pub release_date_original: Option<String>,
    #[serde(default)]
    pub genre: Option<Genre>,
    #[serde(default)]
    pub tracks_count: Option<u32>,
    #[serde(default)]
    pub media_count: Option<u32>,
    /// Present when fetched with track expansion.
    #[serde(default)]
    pub tracks: Option<TrackList>,
    #[serde(default)]
    pub label: Option<Label>,
}

impl Album {
    pub fn year(&self) -> Option<&str> {
        self.release_date_original
            .as_deref()
            .and_then(|d| d.get(0..4))
    }

    pub fn artist_name(&self) -> &str {
        self.artist
            .as_ref()
            .and_then(|a| a.name.as_deref())
            .unwrap_or("Unknown Artist")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Genre {
    #[serde(default)]
    pub name: Option<String>,
}

/// A track, as returned by `track/get` or embedded in an album/playlist.
#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub track_number: Option<u32>,
    #[serde(default)]
    pub media_number: Option<u32>,
    #[serde(default)]
    pub performer: Option<ArtistRef>,
    #[serde(default)]
    pub composer: Option<ArtistRef>,
    #[serde(default)]
    pub isrc: Option<String>,
    #[serde(default)]
    pub parental_warning: Option<bool>,
    #[serde(default)]
    pub duration: Option<u32>,
    /// Present when the track is fetched standalone (`track/get`).
    #[serde(default)]
    pub album: Option<Album>,
}

impl Track {
    pub fn artist_name(&self) -> &str {
        self.performer
            .as_ref()
            .and_then(|a| a.name.as_deref())
            .unwrap_or("Unknown Artist")
    }

    pub fn disc_number(&self) -> u32 {
        self.media_number.unwrap_or(1)
    }

    pub fn is_explicit(&self) -> bool {
        self.parental_warning.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TrackList {
    #[serde(default)]
    pub items: Vec<Track>,
    #[serde(default)]
    pub total: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub limit: Option<u32>,
}

/// A playlist, as returned by `playlist/get?extra=tracks`.
#[derive(Debug, Clone, Deserialize)]
pub struct Playlist {
    pub id: serde_json::Value,
    pub name: String,
    #[serde(default)]
    pub tracks: Option<TrackList>,
    #[serde(default)]
    pub tracks_count: Option<u32>,
}

/// An artist, as returned by `artist/get?extra=albums`.
#[derive(Debug, Clone, Deserialize)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub albums: Option<AlbumList>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AlbumList {
    #[serde(default)]
    pub items: Vec<Album>,
    #[serde(default)]
    pub total: Option<u32>,
}

/// Response from `track/getFileUrl` — a temporary, expiring CDN URL plus the
/// actually delivered quality (which may be lower than requested).
#[derive(Debug, Clone, Deserialize)]
pub struct FileUrl {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub format_id: Option<u32>,
    #[serde(default)]
    pub bit_depth: Option<u32>,
    #[serde(default)]
    pub sampling_rate: Option<f64>,
    #[serde(default)]
    pub mime_type: Option<String>,
    /// Some restricted responses use this flag.
    #[serde(default)]
    pub restrictions: Option<serde_json::Value>,
}

/// Aggregated `catalog/search` (or per-type search) result.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchResults {
    #[serde(default)]
    pub albums: Option<AlbumList>,
    #[serde(default)]
    pub tracks: Option<TrackList>,
    #[serde(default)]
    pub artists: Option<ArtistSearchList>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArtistSearchList {
    #[serde(default)]
    pub items: Vec<Artist>,
    #[serde(default)]
    pub total: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn album_year_extracted() {
        let json = r#"{"id":"1","title":"X","release_date_original":"2019-05-03"}"#;
        let a: Album = serde_json::from_str(json).unwrap();
        assert_eq!(a.year(), Some("2019"));
    }

    #[test]
    fn unknown_fields_ignored() {
        let json = r#"{"id":42,"title":"Song","some_future_field":true}"#;
        let t: Track = serde_json::from_str(json).unwrap();
        assert_eq!(t.id, 42);
        assert_eq!(t.disc_number(), 1);
        assert!(!t.is_explicit());
    }

    #[test]
    fn image_best_prefers_large() {
        let img = Image {
            large: Some("L".into()),
            small: Some("S".into()),
            thumbnail: None,
        };
        assert_eq!(img.best(), Some("L"));
    }
}
