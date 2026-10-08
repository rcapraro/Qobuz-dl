//! Opening download folders in the system file manager, through each OS's own
//! command rather than a dependency.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The command that opens `path`, or `NotFound` when it no longer exists, so
/// a moved or deleted download is reported instead of opening nothing.
pub(super) fn command(path: &Path) -> io::Result<Command> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} no longer exists", path.display()),
        ));
    }
    Ok(platform_command(path))
}

#[cfg(target_os = "macos")]
fn platform_command(dir: &Path) -> Command {
    let mut cmd = Command::new("open");
    cmd.arg(dir);
    cmd
}

/// Explorer splits its command line on commas as well as spaces, while Rust
/// quotes an argument only when it contains a space, so a path like `A,B`
/// would be cut in two. The path is quoted by hand and passed raw.
#[cfg(windows)]
fn platform_command(dir: &Path) -> Command {
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("explorer");
    cmd.raw_arg(format!("\"{}\"", dir.display()));
    cmd
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_command(dir: &Path) -> Command {
    let mut cmd = Command::new("xdg-open");
    cmd.arg(dir);
    cmd
}

/// The folder an album's downloaded files share: their common parent folder,
/// or the nearest common ancestor when a template split them. `None` when no
/// file is given, so the caller can hide its control.
pub(super) fn shared_folder<'a>(files: impl IntoIterator<Item = &'a Path>) -> Option<PathBuf> {
    let mut shared: Option<PathBuf> = None;
    for dir in files.into_iter().filter_map(Path::parent) {
        shared = Some(match shared {
            None => dir.to_path_buf(),
            Some(current) => current
                .components()
                .zip(dir.components())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect(),
        });
    }
    shared
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(files: &[&str]) -> Option<PathBuf> {
        shared_folder(files.iter().map(Path::new))
    }

    #[test]
    fn one_folder() {
        assert_eq!(
            folder(&["/music/Album/01.flac", "/music/Album/02.flac"]),
            Some(PathBuf::from("/music/Album"))
        );
    }

    #[test]
    fn split_album_opens_the_nearest_shared_folder() {
        assert_eq!(
            folder(&["/music/A/Album/01.flac", "/music/B/Album/02.flac"]),
            Some(PathBuf::from("/music"))
        );
    }

    #[test]
    fn disc_subfolders_share_the_album_folder() {
        assert_eq!(
            folder(&["/music/Album/CD1/01.flac", "/music/Album/CD2/01.flac"]),
            Some(PathBuf::from("/music/Album"))
        );
    }

    #[test]
    fn no_file_is_none() {
        assert_eq!(folder(&[]), None);
    }

    #[test]
    fn missing_path_is_not_found_without_a_command() {
        let gone = std::env::temp_dir().join("qobuz-dl-open-test-does-not-exist");
        let err = command(&gone).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(err.to_string().contains("no longer exists"));
    }

    #[test]
    fn existing_folder_builds_a_command() {
        assert!(command(&std::env::temp_dir()).is_ok());
    }
}
