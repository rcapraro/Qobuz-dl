//! Renaming a downloaded album's folder in place.

use crate::error::{Error, Result};
use crate::tag_edit::{self, Field, TagFields, FLAG_ON};
use crate::template::{render_segment, sanitize_segment, TemplateContext};
use lofty::file::AudioFile;
use lofty::properties::FileProperties;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What [`sanitize_segment`] returns when nothing of its input survives.
const NOTHING_LEFT: &str = "_";

/// The folder name `input` sanitizes to, or `None` when nothing of it is left.
pub fn folder_name(input: &str) -> Option<String> {
    let name = sanitize_segment(input);
    (name != NOTHING_LEFT || input.trim() == NOTHING_LEFT).then_some(name)
}

/// The folder name `template` suggests for the album holding `file`, from the
/// file's current tags and audio format, or `None` when it renders to nothing.
/// Blocking.
pub fn suggest(template: &str, file: &Path) -> Result<Option<String>> {
    let fields = tag_edit::read_fields(file)?;
    let tagged = lofty::read_from_path(file)?;
    let ctx = context(&fields, tagged.properties(), file);
    let name = render_segment(template, &ctx);
    Ok((name != NOTHING_LEFT).then_some(name))
}

/// The same keys the download engine sets, read back from a file.
fn context(fields: &TagFields, properties: &FileProperties, file: &Path) -> TemplateContext {
    let tag = |field| fields.get(field).unwrap_or_default().to_string();
    let mut ctx = TemplateContext::new();
    ctx.set("albumartist", tag(Field::AlbumArtist))
        .set("artist", tag(Field::Artist))
        .set("album", tag(Field::Album))
        .set("title", tag(Field::Title))
        .set(
            "year",
            tag(Field::Date).split('-').next().unwrap_or_default(),
        )
        .set(
            "container",
            file.extension()
                .map(|ext| ext.to_string_lossy().to_uppercase())
                .unwrap_or_default(),
        )
        .set(
            "bit_depth",
            properties
                .bit_depth()
                .map(|b| b.to_string())
                .unwrap_or_default(),
        )
        .set(
            "sampling_rate",
            properties
                .sample_rate()
                .filter(|&hz| hz > 0)
                .map(|hz| format!("{}", f64::from(hz) / 1000.0))
                .unwrap_or_default(),
        )
        .set(
            "explicit",
            if fields.get(Field::Explicit) == Some(FLAG_ON) {
                " [E]"
            } else {
                ""
            },
        );
    if let Some(composer) = fields.get(Field::Composer) {
        ctx.set("composer", composer);
    }
    if let Some(n) = fields.get(Field::TrackNumber).and_then(|n| n.parse().ok()) {
        ctx.with_track_number(n);
    }
    ctx
}

/// Rename `folder`, which must lie strictly inside `download_dir`, to
/// `new_name` sanitized, keeping its parent directory. Refuses, changing
/// nothing, when another entry already has that name, or when `folder` holds
/// a subfolder none of `album_files` is in: with a folder format such as
/// `{albumartist}`, the album's folder is its artist's, holding other albums.
/// A rename that only changes letter case is allowed. Returns the folder's
/// new path, built on `folder` as given rather than canonicalized.
pub fn rename_folder(
    download_dir: &Path,
    folder: &Path,
    album_files: &[PathBuf],
    new_name: &str,
) -> Result<PathBuf> {
    let name = folder_name(new_name).ok_or_else(|| refused("the name is empty"))?;
    let root = download_dir.canonicalize()?;
    let current = folder.canonicalize()?;
    if current == root {
        return Err(refused("the files are directly in the download folder"));
    }
    if !current.starts_with(&root) {
        return Err(refused("the folder is outside the download folder"));
    }
    if folder.file_name().is_none() {
        return Err(refused("the folder has no name"));
    }
    if let Some(other) = foreign_subfolder(folder, album_files)? {
        return Err(refused(&format!(
            "it also holds \"{}\", which isn't part of this album",
            other.to_string_lossy()
        )));
    }
    let target = folder.with_file_name(&name);
    if target.symlink_metadata().is_ok() {
        if !same_entry(&target, folder) {
            return Err(refused(&format!("\"{name}\" already exists")));
        }
        if folder.file_name() == Some(name.as_ref()) {
            return Ok(folder.to_path_buf());
        }
    }
    std::fs::rename(folder, &target)?;
    Ok(target)
}

/// The name of a subfolder of `folder` that holds none of `album_files`.
/// Loose files, such as scans or logs, belong to the folder and move with it.
fn foreign_subfolder(folder: &Path, album_files: &[PathBuf]) -> Result<Option<OsString>> {
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() && !album_files.iter().any(|f| f.starts_with(&path)) {
            return Ok(Some(entry.file_name()));
        }
    }
    Ok(None)
}

fn refused(reason: &str) -> Error {
    Error::Rename(reason.to_string())
}

/// Whether two paths name the same directory entry, as a case-only rename's
/// target does on a case-insensitive filesystem. Compared by inode because
/// macOS canonicalizes a path in the case it was given.
#[cfg(unix)]
fn same_entry(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (a.metadata(), b.metadata()) {
        (Ok(a), Ok(b)) => (a.dev(), a.ino()) == (b.dev(), b.ino()),
        _ => false,
    }
}

/// Windows canonicalizes to the name stored on disk, so equal canonical paths
/// mean the same entry.
#[cfg(not(unix))]
fn same_entry(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag_edit::{save_file, CoverEdit, CoverPlan, FieldEdit, TagEdits};
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A uniquely named directory under the OS temp dir, removed when dropped.
    struct ScratchDir(PathBuf);

    impl ScratchDir {
        fn new() -> Self {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let n = SEQ.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("qobuz-dl-rename-{}-{n}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        /// A folder `rel` under this one, holding a file `track.flac`.
        fn album(&self, rel: &str) -> PathBuf {
            let folder = self.0.join(rel);
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(folder.join("track.flac"), b"audio").unwrap();
            folder
        }

        fn names(&self) -> Vec<String> {
            let mut names: Vec<String> = std::fs::read_dir(&self.0)
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }
    }

    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A FLAC stream whose STREAMINFO says 24-bit stereo at 96 kHz.
    fn write_hires_flac(path: &Path) {
        let mut info = [0u8; 0x22];
        info[0..2].copy_from_slice(&4096u16.to_be_bytes());
        info[2..4].copy_from_slice(&4096u16.to_be_bytes());
        let packed: u64 = (96_000 << 44) | (1 << 41) | (23 << 36);
        info[10..18].copy_from_slice(&packed.to_be_bytes());
        let mut bytes = b"fLaC".to_vec();
        bytes.extend_from_slice(&[0x80, 0x00, 0x00, 0x22]);
        bytes.extend_from_slice(&info);
        std::fs::write(path, bytes).unwrap();
    }

    fn tagged_flac(dir: &ScratchDir, album: &str) -> PathBuf {
        let file = dir.path().join("01.flac");
        write_hires_flac(&file);
        let edits: TagEdits = [
            (Field::AlbumArtist, "Miles Davis"),
            (Field::Album, album),
            (Field::Date, "1959-08-17"),
            (Field::TrackNumber, "1"),
        ]
        .into_iter()
        .map(|(f, v)| (f, FieldEdit::Set(v.to_string())))
        .collect();
        save_file(&file, &edits, &CoverPlan::new(CoverEdit::default())).unwrap();
        file
    }

    #[test]
    fn suggests_from_the_file_tags() {
        let dir = ScratchDir::new();
        let file = tagged_flac(&dir, "Kind of Blue (Remaster)");
        assert_eq!(
            suggest("{albumartist} - {album} ({year})", &file)
                .unwrap()
                .as_deref(),
            Some("Miles Davis - Kind of Blue (Remaster) (1959)")
        );
    }

    #[test]
    fn template_rendering_nothing_suggests_nothing() {
        let dir = ScratchDir::new();
        let file = tagged_flac(&dir, "Kind of Blue");
        assert_eq!(suggest("", &file).unwrap(), None);
        assert_eq!(suggest("{composer}", &file).unwrap(), None);
    }

    #[test]
    fn suggests_audio_format_from_the_file() {
        let dir = ScratchDir::new();
        let file = tagged_flac(&dir, "Kind of Blue");
        assert_eq!(
            suggest(
                "{album} [{container}] [{bit_depth}B-{sampling_rate}kHz] {tracknumber:02}",
                &file
            )
            .unwrap()
            .as_deref(),
            Some("Kind of Blue [FLAC] [24B-96kHz] 01")
        );
    }

    #[test]
    fn slash_in_the_template_stays_one_segment() {
        let dir = ScratchDir::new();
        let file = tagged_flac(&dir, "Kind of Blue");
        assert_eq!(
            suggest("{albumartist}/{album}", &file).unwrap().as_deref(),
            Some("Miles Davis Kind of Blue")
        );
    }

    #[test]
    fn rename_keeps_the_contents() {
        let dir = ScratchDir::new();
        let old = dir.album("Kind of Blue [FLAC]");
        let new = rename_folder(dir.path(), &old, &[], "Kind of Blue (1959)").unwrap();
        assert_eq!(new, dir.path().join("Kind of Blue (1959)"));
        assert!(!old.exists());
        assert!(new.join("track.flac").exists());
    }

    #[test]
    fn separators_in_the_name_are_sanitized() {
        let dir = ScratchDir::new();
        let old = dir.album("Back in Black");
        let new = rename_folder(dir.path(), &old, &[], "AC/DC - Back in Black").unwrap();
        assert_eq!(new, dir.path().join("AC DC - Back in Black"));
        assert_eq!(dir.names(), vec!["AC DC - Back in Black"]);
    }

    #[test]
    fn multi_disc_folder_moves_its_discs() {
        let dir = ScratchDir::new();
        let files = ["Album/Disc 1", "Album/Disc 2"].map(|d| dir.album(d).join("track.flac"));
        let new = rename_folder(dir.path(), &dir.path().join("Album"), &files, "Renamed").unwrap();
        assert!(new.join("Disc 1/track.flac").exists());
        assert!(new.join("Disc 2/track.flac").exists());
    }

    #[test]
    fn folder_holding_another_album_is_refused() {
        let dir = ScratchDir::new();
        let artist = dir.album("Miles Davis");
        let files = [artist.join("track.flac")];
        dir.album("Miles Davis/Bitches Brew");
        let err = rename_folder(dir.path(), &artist, &files, "Kind of Blue").unwrap_err();
        assert!(err.to_string().contains("Bitches Brew"), "{err}");
        assert_eq!(dir.names(), vec!["Miles Davis"]);
    }

    #[test]
    fn loose_files_move_with_the_folder() {
        let dir = ScratchDir::new();
        let old = dir.album("A");
        std::fs::write(old.join("scan.jpg"), b"jpg").unwrap();
        let new = rename_folder(dir.path(), &old, &[], "B").unwrap();
        assert!(new.join("scan.jpg").exists());
    }

    #[test]
    fn existing_target_is_refused() {
        let dir = ScratchDir::new();
        let old = dir.album("A");
        dir.album("B");
        let err = rename_folder(dir.path(), &old, &[], "B").unwrap_err();
        assert!(matches!(err, Error::Rename(_)), "{err}");
        assert_eq!(dir.names(), vec!["A", "B"]);
        assert!(old.join("track.flac").exists());
    }

    #[test]
    fn download_dir_itself_is_refused() {
        let dir = ScratchDir::new();
        dir.album("");
        let err = rename_folder(dir.path(), dir.path(), &[], "Renamed").unwrap_err();
        assert!(matches!(err, Error::Rename(_)), "{err}");
        assert!(dir.path().join("track.flac").exists());
    }

    #[test]
    fn folder_outside_the_download_dir_is_refused() {
        let root = ScratchDir::new();
        let downloads = root.album("downloads");
        let elsewhere = root.album("elsewhere");
        let err = rename_folder(&downloads, &elsewhere, &[], "Renamed").unwrap_err();
        assert!(matches!(err, Error::Rename(_)), "{err}");
        assert!(elsewhere.exists());
    }

    #[test]
    fn name_that_sanitizes_to_nothing_is_refused() {
        let dir = ScratchDir::new();
        let old = dir.album("A");
        assert_eq!(folder_name(" /?: "), None);
        let err = rename_folder(dir.path(), &old, &[], " /?: ").unwrap_err();
        assert!(matches!(err, Error::Rename(_)), "{err}");
        assert!(old.exists());
    }

    #[test]
    fn same_name_changes_nothing() {
        let dir = ScratchDir::new();
        let old = dir.album("A");
        assert_eq!(rename_folder(dir.path(), &old, &[], "A").unwrap(), old);
        assert_eq!(dir.names(), vec!["A"]);
    }

    #[test]
    fn case_only_rename_succeeds() {
        let dir = ScratchDir::new();
        let old = dir.album("kind of blue");
        let new = rename_folder(dir.path(), &old, &[], "Kind of Blue").unwrap();
        assert_eq!(new, dir.path().join("Kind of Blue"));
        assert_eq!(dir.names(), vec!["Kind of Blue"]);
    }
}
