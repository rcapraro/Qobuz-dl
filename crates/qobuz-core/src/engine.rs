//! High-level download orchestration: resolve a reference into track jobs, then
//! download, tag, and organize each with bounded concurrency and per-item
//! failure isolation.

use crate::catalog::Reference;
use crate::client::QobuzClient;
use crate::config::Config;
use crate::download::{self, Progress};
use crate::error::{Error, Result};
use crate::models::{Album, Track};
use crate::quality::Quality;
use crate::tagging::{self, TrackTags};
use crate::template::{self, TemplateContext};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Semaphore};

const MAX_ATTEMPTS: u32 = 4;

/// One downloadable unit: a track plus the album it belongs to.
#[derive(Debug, Clone)]
pub struct Job {
    pub track: Track,
    pub album: Album,
    pub multi_disc: bool,
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
                album,
                multi_disc: false,
            }])
        }
        Reference::Playlist(id) => {
            let playlist = client.playlist(id).await?;
            let tracks = playlist.tracks.map(|t| t.items).unwrap_or_default();
            let mut jobs = Vec::new();
            for track in tracks {
                if let Some(album) = track.album.clone() {
                    jobs.push(Job {
                        track,
                        album,
                        multi_disc: false,
                    });
                }
            }
            Ok(jobs)
        }
        Reference::Artist(_) => Err(Error::Config(
            "artist links aren't directly downloadable — pick one of the artist's albums".into(),
        )),
    }
}

fn jobs_from_album(mut album: Album) -> Vec<Job> {
    let multi_disc = album.media_count.unwrap_or(1) > 1;
    let tracks = album.tracks.take().map(|t| t.items).unwrap_or_default();
    tracks
        .into_iter()
        .map(|track| Job {
            track,
            album: album.clone(),
            multi_disc,
        })
        .collect()
}

/// Download every job with bounded concurrency, emitting [`JobEvent`]s. A single
/// job failure is isolated (reported via `JobEvent::Failed`) and does not abort
/// the batch.
pub async fn download_all(
    client: QobuzClient,
    config: Config,
    jobs: Vec<Job>,
    events: mpsc::Sender<JobEvent>,
) {
    let semaphore = Arc::new(Semaphore::new(config.concurrency.max(1)));
    let client = Arc::new(client);
    let config = Arc::new(config);
    // Cache cover art per album across the batch.
    let cover_cache: Arc<Mutex<HashMap<String, Option<Vec<u8>>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut handles = Vec::new();
    for job in jobs {
        let permit_sem = semaphore.clone();
        let client = client.clone();
        let config = config.clone();
        let events = events.clone();
        let cover_cache = cover_cache.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit_sem.acquire().await;
            let track_id = job.track.id;
            let _ = events
                .send(JobEvent::Started {
                    track_id,
                    title: job.track.title.clone(),
                })
                .await;

            match download_one(&client, &config, &job, &events, &cover_cache).await {
                Ok((path, delivered)) => {
                    let _ = events
                        .send(JobEvent::Done {
                            track_id,
                            path,
                            delivered,
                        })
                        .await;
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
        }));
    }

    for h in handles {
        let _ = h.await;
    }
}

async fn download_one(
    client: &QobuzClient,
    config: &Config,
    job: &Job,
    events: &mpsc::Sender<JobEvent>,
    cover_cache: &Mutex<HashMap<String, Option<Vec<u8>>>>,
) -> Result<(PathBuf, String)> {
    let track_id = job.track.id;
    let track_id_str = track_id.to_string();

    // 1. Request a fresh signed URL just-in-time (retry transient failures).
    let file = download::with_retry(MAX_ATTEMPTS, || {
        client.file_url(&track_id_str, config.quality)
    })
    .await?;
    let url = file.url.clone().ok_or(Error::NoFileUrl)?;

    // Determine actually delivered quality.
    let delivered_quality = file
        .format_id
        .and_then(Quality::from_format_id)
        .unwrap_or(config.quality);
    let ext = delivered_quality.extension();

    // 2. Build destination path from templates.
    let dest = build_path(config, job, &file, ext);

    // 3. Stream to disk with progress, retrying transient failures.
    let (tx, mut rx) = mpsc::channel::<Progress>(32);
    let ev = events.clone();
    let forward = tokio::spawn(async move {
        while let Some(p) = rx.recv().await {
            if let Progress::Bytes { downloaded, total } = p {
                let _ = ev
                    .send(JobEvent::Progress {
                        track_id,
                        downloaded,
                        total,
                    })
                    .await;
            }
        }
    });

    let http = client.http().clone();
    let url_for_dl = url.clone();
    let dest_for_dl = dest.clone();
    download::with_retry(MAX_ATTEMPTS, || {
        let http = http.clone();
        let url = url_for_dl.clone();
        let dest = dest_for_dl.clone();
        let tx = tx.clone();
        async move { download::stream_to_file(&http, &url, &dest, Some(&tx)).await }
    })
    .await?;
    drop(tx);
    let _ = forward.await;

    // 4. Fetch cover art (cached per album) if embedding is enabled.
    let _ = events.send(JobEvent::Tagging { track_id }).await;
    let cover = if config.embed_art {
        fetch_cover(client, &job.album, cover_cache).await
    } else {
        None
    };

    // 5. Write tags + embed art.
    let tags = TrackTags {
        track: &job.track,
        album: &job.album,
        cover: cover.as_deref(),
    };
    tagging::write_tags(&dest, &tags)?;

    let delivered = describe_delivered(&file, delivered_quality);
    Ok((dest, delivered))
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

async fn fetch_cover(
    client: &QobuzClient,
    album: &Album,
    cache: &Mutex<HashMap<String, Option<Vec<u8>>>>,
) -> Option<Vec<u8>> {
    {
        let guard = cache.lock().await;
        if let Some(hit) = guard.get(&album.id) {
            return hit.clone();
        }
    }
    let url = album.image.as_ref().and_then(|i| i.best())?;
    let bytes = match client.http().get(url).send().await {
        Ok(r) => r.bytes().await.ok().map(|b| b.to_vec()),
        Err(_) => None,
    };
    cache.lock().await.insert(album.id.clone(), bytes.clone());
    bytes
}

fn describe_delivered(file: &crate::models::FileUrl, quality: Quality) -> String {
    match (file.bit_depth, file.sampling_rate) {
        (Some(b), Some(s)) => format!("{} {}bit/{}kHz", quality.label(), b, s),
        _ => quality.label().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileUrl, Image};

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
        };
        Job {
            track,
            album,
            multi_disc: false,
        }
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
}
