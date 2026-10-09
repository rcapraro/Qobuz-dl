//! High-level download orchestration: resolve a reference into track jobs, then
//! download, tag, and organize each with bounded concurrency and per-item
//! failure isolation.

use crate::artwork::{self, CoverSize};
use crate::catalog::Reference;
use crate::client::QobuzClient;
use crate::config::Config;
use crate::download::{self, PartFile, Progress, Transfer};
use crate::error::{Error, Result};
use crate::models::{Album, Playlist, Track};
use crate::quality::Quality;
use crate::tagging::{self, TrackTags};
use crate::template::{self, TemplateContext};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, OnceCell, Semaphore};
use tokio_util::sync::CancellationToken;

const MAX_ATTEMPTS: u32 = 4;

/// One downloadable unit: a track plus the album it belongs to.
#[derive(Debug, Clone)]
pub struct Job {
    pub track: Track,
    pub album: Album,
    pub multi_disc: bool,
    /// Number of tracks on this track's disc, when known.
    pub disc_track_total: Option<u32>,
}

/// Progress event for a single job, keyed by track id.
#[derive(Debug, Clone)]
pub enum JobEvent {
    Started {
        track_id: i64,
        title: String,
    },
    Progress {
        track_id: i64,
        downloaded: u64,
        total: Option<u64>,
    },
    Tagging {
        track_id: i64,
    },
    Done {
        track_id: i64,
        path: PathBuf,
        delivered: String,
    },
    Failed {
        track_id: i64,
        error: String,
    },
    /// The batch was cancelled before this track finished. Distinct from
    /// [`JobEvent::Failed`] so a cancelled track isn't reported as an error.
    Cancelled {
        track_id: i64,
    },
}

/// Resolve a [`Reference`] into concrete download jobs.
pub async fn resolve(client: &QobuzClient, reference: &Reference) -> Result<Vec<Job>> {
    match reference {
        Reference::Album(id) => {
            let album = client.album(id).await?;
            Ok(jobs_from_album(album))
        }
        Reference::Track(id) => {
            let track = client.track(id).await?;
            let album = track
                .album
                .clone()
                .ok_or_else(|| Error::Config("track is missing album metadata".into()))?;
            Ok(vec![Job {
                track,
                disc_track_total: track_total_without_list(&album),
                album,
                multi_disc: false,
            }])
        }
        Reference::Playlist(id) => {
            let playlist = client.playlist(id).await?;
            Ok(jobs_from_playlist(playlist))
        }
        Reference::Artist(_) => Err(Error::Config(
            "artist links aren't directly downloadable — pick one of the artist's albums".into(),
        )),
    }
}

fn jobs_from_playlist(playlist: Playlist) -> Vec<Job> {
    let tracks = playlist.tracks.map(|t| t.items).unwrap_or_default();
    // A playlist can list the same track twice; duplicate jobs would race on
    // the same destination file. First occurrence wins.
    let mut seen = HashSet::new();
    let mut jobs = Vec::new();
    for track in tracks {
        if !seen.insert(track.id) {
            continue;
        }
        if let Some(album) = track.album.clone() {
            jobs.push(Job {
                track,
                disc_track_total: track_total_without_list(&album),
                album,
                multi_disc: false,
            });
        }
    }
    jobs
}

fn jobs_from_album(mut album: Album) -> Vec<Job> {
    let multi_disc = album.media_count.unwrap_or(1) > 1;
    let tracks = album.tracks.take().map(|t| t.items).unwrap_or_default();
    let mut per_disc: HashMap<u32, u32> = HashMap::new();
    for track in &tracks {
        *per_disc.entry(track.disc_number()).or_default() += 1;
    }
    tracks
        .into_iter()
        .map(|track| Job {
            disc_track_total: per_disc.get(&track.disc_number()).copied(),
            track,
            album: album.clone(),
            multi_disc,
        })
        .collect()
}

/// A track's disc total when the album's track list isn't at hand. The album's
/// count is only that disc's total when the album says it has a single disc;
/// an album that omits its disc count may have several.
fn track_total_without_list(album: &Album) -> Option<u32> {
    match album.media_count {
        Some(1) => album.tracks_count,
        _ => None,
    }
}

/// Download every job with bounded concurrency, emitting [`JobEvent`]s. A single
/// job failure is isolated (reported via `JobEvent::Failed`) and does not abort
/// the batch.
///
/// Cancelling `cancel` stops the batch cooperatively: tracks still waiting for a
/// concurrency slot never start, in-flight transfers are abandoned mid-stream,
/// and a pending retry backoff is interrupted rather than waited out. Each
/// affected track reports [`JobEvent::Cancelled`]. This function still drains
/// every task before returning, so once it completes no download work remains
/// running and the event channel closes exactly as it does for a normal batch.
pub async fn download_all(
    client: QobuzClient,
    config: Config,
    jobs: Vec<Job>,
    events: mpsc::Sender<JobEvent>,
    cancel: CancellationToken,
) {
    let semaphore = Arc::new(Semaphore::new(config.concurrency.max(1)));
    let client = Arc::new(client);
    let config = Arc::new(config);
    // Cache cover art per album across the batch.
    let cover_cache: Arc<CoverCache> = Arc::new(Mutex::new(HashMap::new()));

    // A `JoinSet` rather than detached handles: dropping it aborts its tasks,
    // so a batch can never outlive this future as background work still writing
    // to disk. Every task is drained below, so the last `events` clone is gone
    // by the time this returns and the receiver sees the channel close.
    let mut tasks = tokio::task::JoinSet::new();
    for job in jobs {
        let permit_sem = semaphore.clone();
        let client = client.clone();
        let config = config.clone();
        let events = events.clone();
        let cover_cache = cover_cache.clone();
        let cancel = cancel.clone();

        tasks.spawn(async move {
            let _permit = permit_sem
                .acquire()
                .await
                .expect("semaphore is never closed");
            // Most of a large batch is parked on the line above, so this is
            // where cancelling is nearly free — bail before any network I/O.
            if cancel.is_cancelled() {
                let _ = events
                    .send(JobEvent::Cancelled {
                        track_id: job.track.id,
                    })
                    .await;
                return;
            }
            run_job(&client, &config, &job, &events, &cover_cache, &cancel).await;
        });
    }

    while let Some(res) = tasks.join_next().await {
        // A panicking job task emits no `Failed` event — at least leave a trace.
        if let Err(e) = res {
            tracing::error!("download job task failed: {e}");
        }
    }
}

/// Per-job body: emit `Started`, run the download, report `Done`/`Failed`.
async fn run_job(
    client: &QobuzClient,
    config: &Config,
    job: &Job,
    events: &mpsc::Sender<JobEvent>,
    cover_cache: &CoverCache,
    cancel: &CancellationToken,
) {
    let track_id = job.track.id;
    let _ = events
        .send(JobEvent::Started {
            track_id,
            title: job.track.title.clone(),
        })
        .await;

    match download_one(client, config, job, events, cover_cache, cancel).await {
        Ok((path, delivered)) => {
            let _ = events
                .send(JobEvent::Done {
                    track_id,
                    path,
                    delivered,
                })
                .await;
        }
        // Cancellation isn't a failure — reporting it as one would paint the
        // whole queue red for a batch the user chose to stop.
        Err(Error::Cancelled) => {
            let _ = events.send(JobEvent::Cancelled { track_id }).await;
        }
        Err(e) => {
            let _ = events
                .send(JobEvent::Failed {
                    track_id,
                    error: e.to_string(),
                })
                .await;
        }
    }
}

async fn download_one(
    client: &QobuzClient,
    config: &Config,
    job: &Job,
    events: &mpsc::Sender<JobEvent>,
    cover_cache: &CoverCache,
    cancel: &CancellationToken,
) -> Result<(PathBuf, String)> {
    let (file, dest, delivered_quality, transfer) =
        download_with_progress(client, config, job, events, cancel).await?;
    // An already-present file is left untouched, so re-queuing a downloaded
    // album neither bumps its modification times nor overwrites tags the user
    // edited.
    if let Transfer::Downloaded(part) = transfer {
        finish_download(part, &dest, config, job, events, cover_cache, cancel).await?;
    }
    Ok((dest, describe_delivered(&file, delivered_quality)))
}

/// Tag the downloaded temp file, then move it to `dest`. Tagging before the
/// move means a file at `dest` is always a finished one; on any error or
/// cancel, dropping `part` deletes it and `dest` is never created, so a retry
/// downloads the track again.
async fn finish_download(
    part: PartFile,
    dest: &Path,
    config: &Config,
    job: &Job,
    events: &mpsc::Sender<JobEvent>,
    cover_cache: &CoverCache,
    cancel: &CancellationToken,
) -> Result<()> {
    let _ = events
        .send(JobEvent::Tagging {
            track_id: job.track.id,
        })
        .await;
    // Raced against cancel so a slow cover host can't hold a cancel for the
    // full 30s `fetch_bytes` timeout, times `concurrency` tracks.
    let cover = if config.embed_art {
        tokio::select! {
            biased;
            c = fetch_cover(&job.album, config.cover_size, cover_cache) => c,
            _ = cancel.cancelled() => return Err(Error::Cancelled),
        }
    } else {
        None
    };

    let tags = TrackTags {
        track: &job.track,
        album: &job.album,
        disc_track_total: job.disc_track_total,
        cover: cover.as_deref(),
    };
    tagging::write_tags(part.path(), dest, &tags)?;
    part.publish(dest).await
}

/// Sign, stream, and retry: fetch a fresh signed URL and stream it to disk as
/// one retryable unit (a retry always re-signs rather than reusing a
/// possibly-expired URL), forwarding byte progress as [`JobEvent::Progress`].
/// The destination is derived from the *delivered* quality of each response.
async fn download_with_progress(
    client: &QobuzClient,
    config: &Config,
    job: &Job,
    events: &mpsc::Sender<JobEvent>,
    cancel: &CancellationToken,
) -> Result<(crate::models::FileUrl, PathBuf, Quality, Transfer)> {
    let track_id = job.track.id;
    let track_id_str = track_id.to_string();
    let track_id_str = &track_id_str;

    // Progress forwarder: translate byte progress into JobEvents.
    let (tx, mut rx) = mpsc::channel::<Progress>(32);
    let ev = events.clone();
    let forward = tokio::spawn(async move {
        while let Some(Progress::Bytes { downloaded, total }) = rx.recv().await {
            let _ = ev
                .send(JobEvent::Progress {
                    track_id,
                    downloaded,
                    total,
                })
                .await;
        }
    });

    // One `select!` covers both the in-flight transfer and the backoff sleep
    // between attempts, because both live inside the retry future — so cancel
    // never has to wait out a retry delay (or a long server `Retry-After`).
    // Losing the race just drops that future; the temp file's drop guard in
    // `download` cleans up the partial `.part`.
    let result = tokio::select! {
        r = download::with_retry(MAX_ATTEMPTS, || {
        let tx = tx.clone();
        async move {
            let file = client.file_url(track_id_str, config.quality).await?;
            let url = file.url.clone().ok_or(Error::NoFileUrl)?;
            let delivered_quality = file
                .format_id
                .and_then(Quality::from_format_id)
                .unwrap_or(config.quality);
            let ext = delivered_quality.extension();
            let dest = build_path(config, job, &file, ext);
            let transfer =
                download::stream_to_part(client.http(), &url, &dest, Some(&tx)).await?;
            Ok::<_, Error>((file, dest, delivered_quality, transfer))
        }
        }) => r,
        _ = cancel.cancelled() => Err(Error::Cancelled),
    };
    // Runs on the cancelled path too, so the forwarder task is joined rather
    // than left behind.
    drop(tx);
    let _ = forward.await;
    result
}

/// Render the full destination path: `download_dir / <folder segments> /
/// [Disc N /] <track filename>.<ext>`.
fn build_path(config: &Config, job: &Job, file: &crate::models::FileUrl, ext: &str) -> PathBuf {
    let ctx = build_context(job, file, ext);

    let mut path = config.download_dir.clone();
    for seg in template::render_path(&config.folder_format, &ctx) {
        path.push(seg);
    }
    if job.multi_disc {
        path.push(format!("Disc {}", job.track.disc_number()));
    }
    let filename = template::render_segment(&config.track_format, &ctx);
    path.push(format!("{filename}.{ext}"));
    path
}

fn build_context(job: &Job, file: &crate::models::FileUrl, ext: &str) -> TemplateContext {
    let mut ctx = TemplateContext::new();
    ctx.set("albumartist", job.album.artist_name().to_string())
        .set("artist", job.track.artist_name().to_string())
        .set("album", job.album.title.clone())
        .set("title", job.track.title.clone())
        .set("year", job.album.year().unwrap_or("").to_string())
        .set("container", ext.to_uppercase())
        .set(
            "bit_depth",
            file.bit_depth.map(|b| b.to_string()).unwrap_or_default(),
        )
        .set(
            "sampling_rate",
            file.sampling_rate
                .map(|s| format!("{s}"))
                .unwrap_or_default(),
        )
        .set(
            "explicit",
            if job.track.is_explicit() {
                " [E]".to_string()
            } else {
                String::new()
            },
        );
    if let Some(c) = job.track.composer.as_ref().and_then(|c| c.name.clone()) {
        ctx.set("composer", c);
    }
    ctx.with_track_number(job.track.track_number.unwrap_or(0));
    ctx
}

/// Each album's cover, fetched and sized once per batch. A cell per album
/// rather than a plain value, so the album's tracks that start together wait
/// for one fetch and resize instead of each running their own.
type CoverCache = Mutex<HashMap<String, Arc<OnceCell<Option<Vec<u8>>>>>>;

async fn fetch_cover(album: &Album, size: CoverSize, cache: &CoverCache) -> Option<Vec<u8>> {
    let cell = cache
        .lock()
        .await
        .entry(album.id.clone())
        .or_default()
        .clone();
    cell.get_or_init(|| load_cover(album, size)).await.clone()
}

async fn load_cover(album: &Album, size: CoverSize) -> Option<Vec<u8>> {
    let url = album.image.as_ref().and_then(|i| i.best())?;
    // `fetch_bytes` rejects non-2xx responses — without that check an error
    // page's HTML body would get embedded as "cover art".
    let bytes = crate::download::fetch_bytes(url).await.ok()?;
    let source = bytes.clone();
    match tokio::task::spawn_blocking(move || artwork::prepare_cover(source, size)).await {
        Ok(cover) => Some(cover),
        Err(e) => {
            tracing::warn!("cover embedded at its original size: {e}");
            Some(bytes)
        }
    }
}

/// Short delivered-quality label: `FLAC 24/96`, `FLAC 16/44.1`, or `MP3 320`.
/// Falls back to the tier's name when the response omits the technical details.
fn describe_delivered(file: &crate::models::FileUrl, quality: Quality) -> String {
    if quality == Quality::Mp3 {
        return quality.label().to_string();
    }
    match (file.bit_depth, file.sampling_rate) {
        (Some(b), Some(s)) => format!("FLAC {b}/{s}"),
        _ => quality.label().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileUrl, Image};
    use crate::test_support::{write_minimal_flac, Scratch};

    fn sample_job() -> Job {
        let album = Album {
            id: "a1".into(),
            title: "Kind of Blue".into(),
            artist: Some(crate::models::ArtistRef {
                id: Some(1),
                name: Some("Miles Davis".into()),
            }),
            image: Some(Image {
                large: Some("http://x/cover.jpg".into()),
                ..Default::default()
            }),
            release_date_original: Some("1959-08-17".into()),
            genre: None,
            tracks_count: Some(5),
            media_count: Some(1),
            tracks: None,
            label: None,
            copyright: None,
            upc: None,
            hires: false,
            hires_streamable: false,
        };
        let track = Track {
            id: 42,
            title: "So What".into(),
            track_number: Some(1),
            media_number: Some(1),
            performer: Some(crate::models::ArtistRef {
                id: Some(1),
                name: Some("Miles Davis".into()),
            }),
            composer: None,
            isrc: None,
            parental_warning: Some(false),
            duration: Some(545),
            album: Some(album.clone()),
            hires: false,
            hires_streamable: false,
        };
        Job {
            track,
            album,
            multi_disc: false,
            disc_track_total: Some(5),
        }
    }

    #[tokio::test]
    async fn pre_cancelled_batch_starts_no_downloads() {
        // An already-cancelled token means every job bails right after taking
        // its semaphore permit, before touching the network. The client below
        // has bogus credentials and would fail loudly if any request were made.
        let client = QobuzClient::new(String::from("app"), String::from("secret")).expect("client");
        let config = Config {
            download_dir: std::env::temp_dir().join("qobuz-dl-cancel-test"),
            concurrency: 2,
            ..Config::default()
        };
        let mut a = sample_job();
        a.track.id = 1;
        let mut b = sample_job();
        b.track.id = 2;

        let cancel = CancellationToken::new();
        cancel.cancel();

        let (tx, mut rx) = mpsc::channel(16);
        download_all(client, config, vec![a, b], tx, cancel).await;

        let mut cancelled = Vec::new();
        // `download_all` drains every task before returning, so the channel is
        // already closed here — this loop must terminate on its own.
        while let Some(ev) = rx.recv().await {
            match ev {
                JobEvent::Cancelled { track_id } => cancelled.push(track_id),
                other => panic!("expected only Cancelled events, got {other:?}"),
            }
        }
        cancelled.sort_unstable();
        assert_eq!(cancelled, vec![1, 2]);
    }

    /// Run `finish_download` on `part` for `sample_job()`, with no cover cached.
    async fn finish_part(
        part: &Scratch,
        dest: &Scratch,
        embed_art: bool,
        cancel: &CancellationToken,
    ) -> (Result<()>, mpsc::Receiver<JobEvent>) {
        let config = Config {
            embed_art,
            ..Config::default()
        };
        let (tx, rx) = mpsc::channel(4);
        let r = finish_download(
            PartFile::new(part.path().to_path_buf()),
            dest.path(),
            &config,
            &sample_job(),
            &tx,
            &Mutex::new(HashMap::new()),
            cancel,
        )
        .await;
        (r, rx)
    }

    #[tokio::test]
    async fn finished_download_is_tagged_then_published() {
        use lofty::file::TaggedFileExt;
        use lofty::tag::Accessor;

        let part = Scratch::new("so-what.part0");
        let dest = Scratch::new("so-what.flac");
        write_minimal_flac(part.path());

        let (r, mut rx) = finish_part(&part, &dest, false, &CancellationToken::new()).await;

        r.unwrap();
        assert!(matches!(
            rx.try_recv(),
            Ok(JobEvent::Tagging { track_id: 42 })
        ));
        assert!(!part.path().exists());
        let tagged = lofty::read_from_path(dest.path()).unwrap();
        let title = tagged
            .primary_tag()
            .and_then(|t| t.title().map(|s| s.into_owned()));
        assert_eq!(title.as_deref(), Some("So What"));
    }

    #[tokio::test]
    async fn tagging_failure_leaves_no_file() {
        let part = Scratch::new("garbage.part0");
        let dest = Scratch::new("garbage.flac");
        std::fs::write(part.path(), b"not audio").unwrap();

        let (r, _rx) = finish_part(&part, &dest, false, &CancellationToken::new()).await;

        assert!(r.is_err());
        assert!(!part.path().exists());
        assert!(!dest.path().exists());
    }

    #[tokio::test]
    async fn cancel_during_cover_fetch_leaves_no_file() {
        let part = Scratch::new("cancelled.part0");
        let dest = Scratch::new("cancelled.flac");
        write_minimal_flac(part.path());
        let cancel = CancellationToken::new();
        cancel.cancel();

        let (r, _rx) = finish_part(&part, &dest, true, &cancel).await;

        assert!(matches!(r, Err(Error::Cancelled)));
        assert!(!part.path().exists());
        assert!(!dest.path().exists());
    }

    #[test]
    fn builds_expected_path() {
        let config = Config {
            download_dir: PathBuf::from("/music"),
            folder_format: "{albumartist}/{album} ({year})".into(),
            track_format: "{tracknumber:02} - {title}".into(),
            ..Config::default()
        };
        let file = FileUrl {
            url: Some("u".into()),
            format_id: Some(7),
            bit_depth: Some(24),
            sampling_rate: Some(96.0),
            mime_type: None,
            restrictions: None,
        };
        let path = build_path(&config, &sample_job(), &file, "flac");
        assert_eq!(
            path,
            PathBuf::from("/music/Miles Davis/Kind of Blue (1959)/01 - So What.flac")
        );
    }

    fn file_with(bit_depth: Option<u32>, sampling_rate: Option<f64>) -> FileUrl {
        FileUrl {
            url: Some("u".into()),
            format_id: None,
            bit_depth,
            sampling_rate,
            mime_type: None,
            restrictions: None,
        }
    }

    #[test]
    fn delivered_label_is_short() {
        let hires = file_with(Some(24), Some(96.0));
        assert_eq!(describe_delivered(&hires, Quality::Flac24), "FLAC 24/96");
        let cd = file_with(Some(16), Some(44.1));
        assert_eq!(describe_delivered(&cd, Quality::FlacCd), "FLAC 16/44.1");
        assert_eq!(describe_delivered(&cd, Quality::Mp3), "MP3 320");
        let bare = file_with(None, None);
        assert_eq!(describe_delivered(&bare, Quality::Flac24), "FLAC 24/≤96");
    }

    #[test]
    fn multi_disc_adds_subfolder() {
        let config = Config {
            download_dir: PathBuf::from("/m"),
            folder_format: "{album}".into(),
            track_format: "{title}".into(),
            ..Config::default()
        };
        let mut job = sample_job();
        job.multi_disc = true;
        job.track.media_number = Some(2);
        let file = FileUrl {
            url: Some("u".into()),
            format_id: Some(6),
            bit_depth: Some(16),
            sampling_rate: Some(44.1),
            mime_type: None,
            restrictions: None,
        };
        let path = build_path(&config, &job, &file, "flac");
        assert_eq!(path, PathBuf::from("/m/Kind of Blue/Disc 2/So What.flac"));
    }

    fn track_on_disc(id: i64, disc: u32) -> Track {
        Track {
            id,
            media_number: Some(disc),
            ..sample_job().track
        }
    }

    #[test]
    fn track_total_counts_each_disc() {
        let album = Album {
            media_count: Some(2),
            tracks_count: Some(5),
            tracks: Some(crate::models::TrackList {
                items: vec![
                    track_on_disc(1, 1),
                    track_on_disc(2, 1),
                    track_on_disc(3, 1),
                    track_on_disc(4, 2),
                    track_on_disc(5, 2),
                ],
                total: Some(5),
                offset: None,
                limit: None,
            }),
            ..sample_job().album
        };
        let totals: Vec<_> = jobs_from_album(album)
            .iter()
            .map(|j| j.disc_track_total)
            .collect();
        assert_eq!(totals, [Some(3), Some(3), Some(3), Some(2), Some(2)]);
    }

    #[test]
    fn track_total_without_list_needs_a_single_disc() {
        let single = sample_job().album;
        assert_eq!(track_total_without_list(&single), Some(5));
        let multi = Album {
            media_count: Some(2),
            ..single.clone()
        };
        assert_eq!(track_total_without_list(&multi), None);
        let unknown = Album {
            media_count: None,
            ..single
        };
        assert_eq!(track_total_without_list(&unknown), None);
    }

    #[test]
    fn playlist_duplicates_dedup_to_one_job() {
        let track = sample_job().track;
        let playlist = Playlist {
            id: serde_json::json!(1),
            name: "mix".into(),
            tracks: Some(crate::models::TrackList {
                items: vec![track.clone(), track],
                total: Some(2),
                offset: None,
                limit: None,
            }),
            tracks_count: Some(2),
        };
        let jobs = jobs_from_playlist(playlist);
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].track.id, 42);
    }
}
