//! End-to-end album download flow.
//!
//! This test is gated behind environment variables so CI (and offline builds)
//! skip it. To run it against a real Qobuz account:
//!
//! ```bash
//! QOBUZ_APP_ID=... QOBUZ_APP_SECRET=... QOBUZ_TOKEN=... \
//! QOBUZ_ALBUM_ID=... cargo test -p qobuz-core --test integration -- --nocapture
//! ```

use qobuz_core::{catalog::Reference, config::Config, QobuzClient};
use std::path::PathBuf;
use tokio::sync::mpsc;

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

#[tokio::test]
async fn live_album_download_flow() {
    let (Some(app_id), Some(app_secret), Some(token), Some(album_id)) = (
        env("QOBUZ_APP_ID"),
        env("QOBUZ_APP_SECRET"),
        env("QOBUZ_TOKEN"),
        env("QOBUZ_ALBUM_ID"),
    ) else {
        eprintln!("skipping live_album_download_flow (set QOBUZ_APP_ID/SECRET/TOKEN/ALBUM_ID)");
        return;
    };

    let client = QobuzClient::new(app_id, app_secret)
        .expect("client")
        .with_token(token);

    let jobs = qobuz_core::resolve(&client, &Reference::Album(album_id))
        .await
        .expect("resolve album");
    assert!(!jobs.is_empty(), "album should resolve to tracks");

    let tmp = std::env::temp_dir().join("qobuz-dl-itest");
    let config = Config {
        download_dir: tmp.clone(),
        quality: qobuz_core::Quality::FlacCd,
        concurrency: 2,
        ..Config::default()
    };

    let (tx, mut rx) = mpsc::channel(256);
    // Never cancelled here — this exercises the normal end-to-end path.
    let handle = tokio::spawn(qobuz_core::download_all(
        client,
        config,
        jobs.into_iter().take(1).collect(),
        tx,
        qobuz_core::CancellationToken::new(),
    ));

    let mut done_paths: Vec<PathBuf> = Vec::new();
    while let Some(ev) = rx.recv().await {
        match ev {
            qobuz_core::JobEvent::Done { path, .. } => done_paths.push(path),
            qobuz_core::JobEvent::Failed { error, .. } => panic!("download failed: {error}"),
            qobuz_core::JobEvent::Cancelled { track_id } => {
                panic!("unexpected cancellation of track {track_id}")
            }
            _ => {}
        }
    }
    handle.await.unwrap();

    assert!(!done_paths.is_empty(), "at least one file downloaded");
    for p in &done_paths {
        assert!(p.exists(), "downloaded file exists: {}", p.display());
    }
}
