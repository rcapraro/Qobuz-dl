//! Persistent, non-secret application settings.
//!
//! The `user_auth_token` and password are intentionally NOT part of this struct
//! and are never serialized here — the token lives in the OS keyring (see
//! [`crate::auth`]).

use crate::error::{Error, Result};
use crate::quality::Quality;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_FOLDER_FORMAT: &str =
    "{albumartist} - {album} ({year}) [{container}] [{bit_depth}B-{sampling_rate}kHz]";
pub const DEFAULT_TRACK_FORMAT: &str = "{tracknumber:02}. {artist} - {title}";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Base directory downloads are written under.
    pub download_dir: PathBuf,
    /// Desired download quality tier.
    pub quality: Quality,
    /// Template for the album/folder path (may contain `/`).
    pub folder_format: String,
    /// Template for the track file name (extension appended automatically).
    pub track_format: String,
    /// Whether to embed cover art into downloaded files.
    pub embed_art: bool,
    /// Max simultaneous track downloads.
    pub concurrency: usize,
    /// Qobuz web-player API app id (required).
    pub app_id: String,
    /// Qobuz web-player API app secret (required for signed calls).
    pub app_secret: String,
    /// Cached user id for the raw-token auth path.
    pub user_id: String,
    /// Whether the GUI uses the dark theme (true) or the light theme (false).
    pub dark_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            download_dir: default_download_dir(),
            quality: Quality::default(),
            folder_format: DEFAULT_FOLDER_FORMAT.to_string(),
            track_format: DEFAULT_TRACK_FORMAT.to_string(),
            embed_art: true,
            concurrency: 3,
            app_id: String::new(),
            app_secret: String::new(),
            user_id: String::new(),
            dark_mode: true,
        }
    }
}

impl Config {
    /// Path of the JSON config file in the platform config dir.
    pub fn config_path() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "qobuzdl", "qobuz-dl")
            .ok_or_else(|| Error::Config("could not resolve a config directory".into()))?;
        Ok(dirs.config_dir().join("config.json"))
    }

    /// Load config from disk, falling back to defaults if none exists.
    pub fn load() -> Result<Config> {
        let path = Self::config_path()?;
        match std::fs::read_to_string(&path) {
            Ok(s) => Ok(serde_json::from_str(&s)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(e) => Err(Error::Io(e)),
        }
    }

    /// Persist config to disk (creating the directory as needed).
    ///
    /// This serializes only non-secret settings plus the app credentials; the
    /// `user_auth_token`/password are never written here.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn has_app_credentials(&self) -> bool {
        !self.app_id.trim().is_empty() && !self.app_secret.trim().is_empty()
    }
}

fn default_download_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|d| d.audio_dir().map(|p| p.to_path_buf()))
        .or_else(|| directories::UserDirs::new().map(|d| d.home_dir().join("Music")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Qobuz")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = Config::default();
        assert!(c.embed_art);
        assert!(c.dark_mode);
        assert_eq!(c.concurrency, 3);
        assert_eq!(c.quality, Quality::Flac24);
        assert!(c.folder_format.contains("{albumartist}"));
        assert!(!c.has_app_credentials());
    }

    #[test]
    fn serialized_config_has_no_token_field() {
        let c = Config::default();
        let json = serde_json::to_string(&c).unwrap();
        assert!(!json.contains("user_auth_token"));
        assert!(!json.to_lowercase().contains("password"));
    }

    #[test]
    fn roundtrips_through_json() {
        let c = Config {
            app_id: "123".into(),
            ..Config::default()
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.app_id, "123");
    }

    #[test]
    fn loads_config_without_dark_mode_field() {
        // Older config files predate `dark_mode`; #[serde(default)] must fill it.
        let json = r#"{"app_id": "abc"}"#;
        let c: Config = serde_json::from_str(json).unwrap();
        assert_eq!(c.app_id, "abc");
        assert!(c.dark_mode);
    }
}
