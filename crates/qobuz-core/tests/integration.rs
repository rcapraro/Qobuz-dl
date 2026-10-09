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

/// Looks Kind of Blue up on MusicBrainz, by barcode and by ISRC, and fetches
/// covers from the Cover Art Archive. Run with `MUSICBRAINZ_LIVE=1`.
#[tokio::test]
async fn live_musicbrainz_lookup() {
    use qobuz_core::musicbrainz::{self, DiskAlbum, DiskTrack};
    if env("MUSICBRAINZ_LIVE").is_none() {
        eprintln!("skipping live_musicbrainz_lookup (set MUSICBRAINZ_LIVE=1)");
        return;
    }
    let tracks = (14..=18)
        .map(|n| format!("USSM159001{n}"))
        .chain(["USSM15900113".to_string()])
        .zip([2, 3, 4, 5, 6, 1])
        .map(|(isrc, number)| DiskTrack {
            isrc: Some(isrc),
            disc: Some(1),
            number: Some(number),
        })
        .collect();
    let mut album = DiskAlbum {
        barcode: Some("074646493564".into()),
        title: "Kind of Blue".into(),
        label: Some("Columbia".into()),
        track_count: Some(6),
        disc_count: Some(1),
        tracks,
    };

    let mut steps = Vec::new();
    let found = musicbrainz::lookup(&album, |s| steps.push(s))
        .await
        .unwrap();
    eprintln!("barcode: {steps:?}");
    for c in &found {
        eprintln!(
            "  {}% {} {:?} {}",
            c.confidence,
            c.release.id,
            c.release.date,
            c.release.formats()
        );
    }
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].release.id, "d3f0f7aa-d53c-4b96-9f19-f34ec06fdbc8");
    let filled = musicbrainz::fill(&found[0].release, &album);
    assert!(filled.iter().all(Option::is_some));

    album.barcode = None;
    let found = musicbrainz::lookup(&album, |_| {}).await.unwrap();
    eprintln!(
        "isrc: {:?}",
        found.iter().map(|c| c.confidence).collect::<Vec<_>>()
    );
    assert!(!found.is_empty() && found.len() <= 5);

    // Qobuz sells this compilation under a barcode MusicBrainz lacks, and the
    // only one of its ISRCs MusicBrainz knows is on unrelated box sets.
    let hickox = DiskAlbum {
        barcode: Some("0724357398657".into()),
        title: "Richard Hickox conducts Vaughan Williams".into(),
        label: Some("Warner Classics".into()),
        track_count: Some(36),
        disc_count: Some(2),
        tracks: vec![
            DiskTrack {
                isrc: Some("GBAYD8300252".into()),
                disc: Some(1),
                number: Some(1),
            },
            DiskTrack {
                isrc: Some("GBAYD8400248".into()),
                disc: Some(2),
                number: Some(1),
            },
        ],
    };
    let mut steps = Vec::new();
    let found = musicbrainz::lookup(&hickox, |s| steps.push(s))
        .await
        .unwrap();
    eprintln!("hickox: {steps:?}");
    for c in &found {
        eprintln!(
            "  {}% {} {:?}",
            c.confidence, c.release.id, c.release.country
        );
    }
    assert!(steps.contains(&musicbrainz::Step::Title));
    assert_eq!(found.len(), 2);
    assert!(found
        .iter()
        .all(|c| c.release.title == "Hickox Conducts Vaughan Williams"));
    let filled = musicbrainz::fill(&found[0].release, &hickox);
    assert!(filled.iter().all(Option::is_some));

    let cover = musicbrainz::front_cover("6949dcd6-8630-490d-bdd6-d997b85014ac")
        .await
        .unwrap();
    assert!(cover.is_some_and(|c| qobuz_core::artwork::is_cover_image(&c)));
    let none = musicbrainz::front_cover("d3f0f7aa-d53c-4b96-9f19-f34ec06fdbc8")
        .await
        .unwrap();
    assert!(none.is_none());
}
