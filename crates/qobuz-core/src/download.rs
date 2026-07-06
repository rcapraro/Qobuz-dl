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
    /// Switched to writing tags / embedding art.
    Tagging,
}

/// Stream `url` to `dest`, emitting [`Progress::Bytes`] events. The file is
/// written incrementally — never fully buffered in memory.
pub async fn stream_to_file(
    http: &reqwest::Client,
    url: &str,
    dest: &Path,
    progress: Option<&mpsc::Sender<Progress>>,
) -> Result<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let resp = http.get(url).send().await?;
    let status = resp.status();
    if status.as_u16() == 429 {
        return Err(Error::RateLimited);
    }
    if !status.is_success() {
        return Err(Error::Http {
            status: status.as_u16(),
            message: format!("download failed for {}", dest.display()),
        });
    }

    let total = resp.content_length();
    // Write to a temp file, then rename on success so partial files aren't left
    // looking complete.
    let tmp = dest.with_extension("part");
    let mut file = tokio::fs::File::create(&tmp).await?;
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
    drop(file);
    tokio::fs::rename(&tmp, dest).await?;
    Ok(())
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
            Err(e) if attempt >= max_attempts || !is_transient(&e) => return Err(e),
            Err(_) => {
                // 0.5s, 1s, 2s, 4s ...
                let backoff = Duration::from_millis(500u64 << (attempt - 1).min(5));
                tokio::time::sleep(backoff).await;
            }
        }
    }
}

fn is_transient(e: &Error) -> bool {
    matches!(e, Error::RateLimited | Error::Network(_))
        || matches!(e, Error::Http { status, .. } if *status >= 500)
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    Err(Error::RateLimited)
                } else {
                    Ok(n)
                }
            }
        })
        .await;
        assert_eq!(r.unwrap(), 3);
    }
}
