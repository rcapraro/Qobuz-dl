//! Async wrappers around `qobuz-core` calls, run via `Task::perform`. Each is a
//! thin `map_err(|e| e.to_string())` boundary — no logic lives here.

use super::open;
use super::paging::{Page, Section, PAGE_SIZE};
use super::{AlbumResult, SearchPayload, TrackResult};
use iced::futures::channel::mpsc::Sender;
use iced::futures::{future, SinkExt, Stream};
use qobuz_core::catalog::Reference;
use qobuz_core::engine::{self, Job};
use qobuz_core::models::{AlbumList, Image, TrackList};
use qobuz_core::musicbrainz::{self, Candidate, DiskAlbum, Step};
use qobuz_core::rename;
use qobuz_core::tag_edit::{self, CoverEdit, CoverPlan, Saved, TagEdits, TagFields};
use qobuz_core::{AppCredentials, QobuzClient, SigningCheck};
use std::path::PathBuf;

pub(super) async fn pick_dir() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|h| h.path().to_path_buf())
}

pub(super) async fn auto_detect_credentials() -> Result<AppCredentials, String> {
    qobuz_core::discover_app_credentials()
        .await
        .map_err(|e| e.to_string())
}

/// Probe whether request signing still works, independent of any real track.
pub(super) async fn check_signing_probe(client: QobuzClient) -> Result<SigningCheck, String> {
    client.check_signing().await.map_err(|e| e.to_string())
}

/// Validate a pasted `user_auth_token` and return it on success.
pub(super) async fn login_token(
    app_id: String,
    app_secret: String,
    token: String,
) -> Result<String, String> {
    let mut c = QobuzClient::new(app_id, app_secret).map_err(|e| e.to_string())?;
    c.login_with_token(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(token)
}

/// The first page of albums and of tracks, fetched concurrently. If one type
/// fails, its section stays empty and the error goes in `failure`, so the
/// other type's results are kept; only both failing is an error.
pub(super) async fn do_search(client: QobuzClient, query: String) -> Result<SearchPayload, String> {
    let (albums, tracks) = future::join(
        client.search_albums(&query, PAGE_SIZE, 0),
        client.search_tracks(&query, PAGE_SIZE, 0),
    )
    .await;
    let failure = match (&albums, &tracks) {
        (Err(e), Err(_)) => return Err(e.to_string()),
        (Err(e), Ok(_)) => Some(format!("album search failed: {e}")),
        (Ok(_), Err(e)) => Some(format!("track search failed: {e}")),
        (Ok(_), Ok(_)) => None,
    };
    Ok(SearchPayload {
        query,
        albums: albums
            .map(|l| Section::first(album_page(l)))
            .unwrap_or_default(),
        tracks: tracks
            .map(|l| Section::first(track_page(l)))
            .unwrap_or_default(),
        failure,
    })
}

pub(super) async fn more_albums(
    client: QobuzClient,
    query: String,
    offset: u32,
) -> Result<Page<AlbumResult>, String> {
    client
        .search_albums(&query, PAGE_SIZE, offset)
        .await
        .map(album_page)
        .map_err(|e| e.to_string())
}

pub(super) async fn more_tracks(
    client: QobuzClient,
    query: String,
    offset: u32,
) -> Result<Page<TrackResult>, String> {
    client
        .search_tracks(&query, PAGE_SIZE, offset)
        .await
        .map(track_page)
        .map_err(|e| e.to_string())
}

/// Prefer a small image for the thumbnail to keep downloads cheap.
pub(super) fn thumbnail(image: Option<&Image>) -> Option<String> {
    image.and_then(|i| {
        i.small
            .clone()
            .or_else(|| i.thumbnail.clone())
            .or_else(|| i.large.clone())
    })
}

fn album_page(list: AlbumList) -> Page<AlbumResult> {
    let items = list
        .items
        .into_iter()
        .map(|a| AlbumResult {
            hires: a.is_hires(),
            artist: a.artist_name().to_string(),
            cover: thumbnail(a.image.as_ref()),
            title: a.title,
            id: a.id,
        })
        .collect();
    Page {
        items,
        total: list.total,
    }
}

fn track_page(list: TrackList) -> Page<TrackResult> {
    let items = list
        .items
        .into_iter()
        .map(|t| TrackResult {
            id: t.id.to_string(),
            hires: t.is_hires(),
            artist: t.artist_name().to_string(),
            // A track's preview is its album's cover.
            cover: thumbnail(t.album.as_ref().and_then(|al| al.image.as_ref())),
            title: t.title,
        })
        .collect();
    Page {
        items,
        total: list.total,
    }
}

/// Open a folder in the system file manager. Waits for the opener so
/// it is reaped, but ignores its exit code: `explorer` exits 1 even when it
/// opened the window. Only a missing path or a command that cannot start is
/// an error.
pub(super) async fn open_path(folder: PathBuf) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        open::command(&folder)?.status()?;
        Ok::<_, std::io::Error>(())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("Could not open: {e}"))
}

/// Post a desktop notification, fire-and-forget. `show()` can wait on the OS
/// notification service (on macOS it may wait for delivery), so it runs on a
/// blocking thread; a failure is only logged, never surfaced in the app.
pub(super) async fn notify(summary: String, body: String) {
    let posted = tokio::task::spawn_blocking(move || {
        set_notification_identity();
        notify_rust::Notification::new()
            .appname("Qobuz-dl")
            .summary(&summary)
            .body(&body)
            .show()
            .map(|_| ())
    })
    .await;
    match posted {
        Ok(Ok(())) => {}
        Ok(Err(e)) => tracing::warn!("could not post notification: {e}"),
        Err(e) => tracing::warn!("notification task failed: {e}"),
    }
}

/// Which app macOS shows as the sender of our notifications. It can be set
/// only once per process and must happen before the first notification. The
/// lookup runs an AppleScript, so it waits until the first notification
/// instead of slowing every launch. It finds the installed Qobuz-dl.app; a dev
/// binary has none and resolves to Finder. Hard-coding our identifier instead
/// would fail on a dev binary and use up the one attempt, leaving no way to
/// fall back.
#[cfg(target_os = "macos")]
fn set_notification_identity() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let bundle = notify_rust::get_bundle_identifier_or_default("Qobuz-dl");
        if let Err(e) = notify_rust::set_application(&bundle) {
            tracing::warn!("could not set the notification sender to {bundle}: {e}");
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn set_notification_identity() {}

/// Download the bytes of an album cover thumbnail via the core client.
pub(super) async fn fetch_thumbnail(url: String) -> Result<Vec<u8>, ()> {
    qobuz_core::fetch_bytes(&url).await.map_err(|_| ())
}

pub(super) async fn resolve(client: QobuzClient, reference: Reference) -> Result<Vec<Job>, String> {
    engine::resolve(&client, &reference)
        .await
        .map_err(|e| e.to_string())
}

/// What reading one track's file gave: its tags, or why it can't be edited.
pub(super) type ReadTags = (Job, PathBuf, Result<TagFields, String>);

/// Read the tags of an album's done tracks on a blocking thread.
pub(super) async fn read_tags(files: Vec<(Job, PathBuf)>) -> Result<Vec<ReadTags>, String> {
    tokio::task::spawn_blocking(move || {
        files
            .into_iter()
            .map(|(job, path)| {
                let tags = tag_edit::read_fields(&path).map_err(|e| e.to_string());
                (job, path, tags)
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

/// Pick a JPEG or PNG file and read it. `Ok(None)` when the dialog is
/// cancelled; an error when the file can't be read or isn't such an image.
pub(super) async fn pick_cover() -> Result<Option<Vec<u8>>, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .add_filter("Image", &["jpg", "jpeg", "png"])
        .pick_file()
        .await
    else {
        return Ok(None);
    };
    // Not `file.read()`: rfd unwraps the read result, so an unreadable file
    // would panic instead of being reported.
    let unreadable = || format!("{} is not a readable JPEG or PNG image.", file.file_name());
    let bytes = tokio::fs::read(file.path())
        .await
        .map_err(|e| format!("{}: {e}", unreadable()))?;
    readable_cover(bytes, unreadable).await.map(Some)
}

/// `bytes` if they are a JPEG or PNG image, checked off the UI thread;
/// otherwise the `unreadable` message.
async fn readable_cover(
    bytes: Vec<u8>,
    unreadable: impl FnOnce() -> String,
) -> Result<Vec<u8>, String> {
    let (bytes, readable) = tokio::task::spawn_blocking(move || {
        let readable = qobuz_core::artwork::is_cover_image(&bytes);
        (bytes, readable)
    })
    .await
    .map_err(|e| e.to_string())?;
    readable.then_some(bytes).ok_or_else(unreadable)
}

/// Resize a replacement cover once for the whole album, off the UI thread.
pub(super) async fn plan_cover(edit: CoverEdit) -> Result<CoverPlan, String> {
    tokio::task::spawn_blocking(move || CoverPlan::new(edit))
        .await
        .map_err(|e| e.to_string())
}

/// Save one file's edits on a blocking thread.
pub(super) async fn save_tags(
    path: PathBuf,
    edits: TagEdits,
    cover: CoverPlan,
) -> Result<Saved, String> {
    tokio::task::spawn_blocking(move || tag_edit::save_file(&path, &edits, &cover))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// The rename template's folder name for the album holding `file`, read from
/// its tags on a blocking thread; `None` when the template renders nothing.
pub(super) async fn suggest_folder_name(
    template: String,
    file: PathBuf,
) -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(move || rename::suggest(&template, &file))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Rename an album folder on a blocking thread; the result is its new path.
pub(super) async fn rename_folder(
    download_dir: PathBuf,
    folder: PathBuf,
    album_files: Vec<PathBuf>,
    name: String,
) -> Result<PathBuf, String> {
    tokio::task::spawn_blocking(move || {
        rename::rename_folder(&download_dir, &folder, &album_files, &name)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

/// What a MusicBrainz lookup reports: each step, then the releases found.
#[derive(Debug, Clone)]
pub(super) enum LookupEvent {
    Step(Step),
    Found(Result<Vec<Candidate>, String>),
}

/// Look `album` up on MusicBrainz, streaming its steps as it goes.
pub(super) fn musicbrainz_lookup(album: DiskAlbum) -> impl Stream<Item = LookupEvent> {
    iced::stream::channel(8, move |mut output: Sender<LookupEvent>| async move {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let lookup = async move {
            let found = musicbrainz::lookup(&album, |step| {
                let _ = tx.send(step);
            })
            .await;
            // Closing the channel is what ends the drain below.
            drop(tx);
            found
        };
        let mut steps = output.clone();
        let drain = async move {
            while let Some(step) = rx.recv().await {
                let _ = steps.send(LookupEvent::Step(step)).await;
            }
        };
        let (found, ()) = future::join(lookup, drain).await;
        let _ = output
            .send(LookupEvent::Found(found.map_err(|e| e.to_string())))
            .await;
    })
}

/// The release's front cover from the Cover Art Archive, `None` when it has
/// none; an error when it can't be fetched or isn't a JPEG or PNG image.
pub(super) async fn musicbrainz_cover(release_id: String) -> Result<Option<Vec<u8>>, String> {
    let Some(bytes) = musicbrainz::front_cover(&release_id)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Ok(None);
    };
    readable_cover(bytes, || {
        "its cover is not a readable JPEG or PNG image".to_string()
    })
    .await
    .map(Some)
}
