//! Streaming downloads with progress reporting, bounded concurrency, and retry.

use crate::error::{Error, Result};
use futures_util::StreamExt;
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

/// A progress event for a single track download, sent over an mpsc channel.
#[derive(Debug, Clone)]
pub enum Progress {
    /// Bytes transferred so far, and total if known (from Content-Length).
    Bytes { downloaded: u64, total: Option<u64> },
}

/// Fetch the raw bytes at `url` fully into memory (e.g. a small album cover
/// thumbnail). Unauthenticated; builds its own short-lived client.
pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let http = reqwest::Client::builder()
        .user_agent("qobuz-dl/0.1 (+https://github.com/)")
        .timeout(Duration::from_secs(30))
        .build()?;
    let resp = http.get(url).send().await?.error_for_status()?;
    Ok(resp.bytes().await?.to_vec())
}

/// Stream `url` to `dest`, emitting [`Progress::Bytes`] events. The file is
/// written incrementally — never fully buffered in memory.
pub async fn stream_to_file(
    http: &reqwest::Client,
    url: &str,
    dest: &Path,
    progress: Option<&mpsc::Sender<Progress>>,
) -> Result<()> {
    // Idempotency: if the destination already exists, the track was downloaded
    // on a previous run — don't fetch it again.
    if tokio::fs::try_exists(dest).await.unwrap_or(false) {
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let resp = http.get(url).send().await?;
    let status = resp.status();
    if status.as_u16() == 429 {
        return Err(Error::RateLimited {
            retry_after: retry_after_from(resp.headers()),
        });
    }
    if !status.is_success() {
        return Err(Error::Http {
            status: status.as_u16(),
            message: format!("download failed for {}", dest.display()),
        });
    }

    let total = resp.content_length();
    // Write to a temp file, then rename on success so partial files aren't left
    // looking complete. The guard removes the partial file however the transfer
    // ends — an error, or the future being dropped part-way when the batch is
    // cancelled — so no orphaned `.part` is left behind. The name carries a
    // process-unique sequence number so two jobs that render the same
    // destination can never stream into the same temp file.
    let tmp = PartFile::new(part_path(dest));
    stream_to_tmp(resp, tmp.path(), total, progress).await?;
    tokio::fs::rename(tmp.path(), dest).await?;
    tmp.disarm();
    Ok(())
}

/// A `.partN` temp file that deletes itself unless [`PartFile::disarm`] is
/// called. Cleanup has to happen on drop, not just on the error path: a
/// cancelled batch drops the transfer future mid-stream, which never returns an
/// error for an `Err` branch to clean up after.
struct PartFile(Option<std::path::PathBuf>);

impl PartFile {
    fn new(path: std::path::PathBuf) -> Self {
        Self(Some(path))
    }

    fn path(&self) -> &Path {
        self.0.as_deref().expect("armed until disarmed")
    }

    /// Give up ownership of the file — call once it has been renamed into place.
    fn disarm(mut self) {
        self.0 = None;
    }
}

impl Drop for PartFile {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            // `Drop` can't await, so this is the sync unlink. It only runs on
            // the interrupted path, and a single unlink is cheap.
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Stream a response body into `tmp`, emitting progress. Split out so the caller
/// can clean up the partial file on any failure.
async fn stream_to_tmp(
    resp: reqwest::Response,
    tmp: &Path,
    total: Option<u64>,
    progress: Option<&mpsc::Sender<Progress>>,
) -> Result<()> {
    let mut file = tokio::fs::File::create(tmp).await?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        if let Some(tx) = progress {
            let _ = tx.send(Progress::Bytes { downloaded, total }).await;
        }
    }
    file.flush().await?;
    Ok(())
}

/// A process-unique `.partN` sibling of `dest`.
fn part_path(dest: &Path) -> std::path::PathBuf {
    static PART_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = PART_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    dest.with_extension(format!("part{n}"))
}

/// Parse a `Retry-After` header (delta-seconds form) into a duration.
fn retry_after_from(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

/// Run an async operation with exponential-backoff retry on transient errors
/// (rate limiting and network errors). Non-transient errors return immediately.
pub async fn with_retry<F, Fut, T>(max_attempts: u32, mut op: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if attempt >= max_attempts || !e.is_transient() => return Err(e),
            Err(e) => {
                tokio::time::sleep(backoff_delay(attempt, &e)).await;
            }
        }
    }
}

/// Delay before the next retry: honor a server-supplied `Retry-After` when the
/// API provided one; otherwise use jittered exponential backoff
/// (~0.5s, 1s, 2s, 4s … capped, plus up to 50% random jitter).
fn backoff_delay(attempt: u32, err: &Error) -> Duration {
    if let Error::RateLimited {
        retry_after: Some(d),
    } = err
    {
        return *d;
    }
    let base_ms = 500u64 << (attempt - 1).min(5);
    Duration::from_millis(base_ms + jitter_millis(base_ms / 2))
}

/// A cheap dependency-free pseudo-random value in `0..=max`, seeded from the
/// system clock. Precision here is irrelevant — it only spreads out retries to
/// avoid a thundering herd.
fn jitter_millis(max: u64) -> u64 {
    if max == 0 {
        return 0;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    nanos % (max + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_paths_for_the_same_dest_are_unique() {
        let dest = Path::new("/music/song.flac");
        assert_ne!(part_path(dest), part_path(dest));
    }

    /// A uniquely named scratch path under the OS temp dir.
    fn scratch(tag: &str) -> std::path::PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("qobuz-dl-test-{tag}-{n}"))
    }

    #[test]
    fn part_file_is_removed_when_dropped() {
        // The cancellation path: the transfer future is dropped mid-stream and
        // never returns an error, so only `Drop` can clean up.
        let path = scratch("drop");
        std::fs::write(&path, b"partial").unwrap();
        drop(PartFile::new(path.clone()));
        assert!(!path.exists(), "dropping an armed PartFile must delete it");
    }

    #[test]
    fn disarmed_part_file_is_left_alone() {
        // The success path: the file has already been renamed into place, so
        // the guard must not delete what is now the finished download.
        let path = scratch("disarm");
        std::fs::write(&path, b"complete").unwrap();
        PartFile::new(path.clone()).disarm();
        assert!(
            path.exists(),
            "a disarmed PartFile must not delete anything"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn retry_gives_up_on_permanent_error() {
        let mut calls = 0;
        let r: Result<()> = with_retry(3, || {
            calls += 1;
            async { Err(Error::Auth("nope".into())) }
        })
        .await;
        assert!(r.is_err());
        // Permanent error → only one attempt.
        assert_eq!(calls, 1);
    }

    #[tokio::test]
    async fn retry_succeeds_after_transient() {
        let mut calls = 0;
        let r: Result<u32> = with_retry(5, || {
            calls += 1;
            let n = calls;
            async move {
                if n < 3 {
                    Err(Error::RateLimited { retry_after: None })
                } else {
                    Ok(n)
                }
            }
        })
        .await;
        assert_eq!(r.unwrap(), 3);
    }

    #[tokio::test]
    async fn skips_when_destination_exists() {
        let dir = std::env::temp_dir().join("qobuz-dl-test-skip");
        let _ = tokio::fs::create_dir_all(&dir).await;
        let dest = dir.join("already-there.flac");
        tokio::fs::write(&dest, b"existing").await.unwrap();

        let http = reqwest::Client::new();
        // URL is never contacted because the destination already exists.
        let r = stream_to_file(&http, "http://127.0.0.1:0/nope", &dest, None).await;
        assert!(r.is_ok());
        // The pre-existing file is untouched.
        assert_eq!(tokio::fs::read(&dest).await.unwrap(), b"existing");

        let _ = tokio::fs::remove_file(&dest).await;
    }

    #[test]
    fn honors_retry_after_over_backoff() {
        let err = Error::RateLimited {
            retry_after: Some(Duration::from_secs(7)),
        };
        assert_eq!(backoff_delay(1, &err), Duration::from_secs(7));
    }
}
