//! Async wrappers around `qobuz-core` calls, run via `Task::perform`. Each is a
//! thin `map_err(|e| e.to_string())` boundary — no logic lives here.

use super::{AlbumResult, SearchPayload};
use qobuz_core::catalog::Reference;
use qobuz_core::engine::{self, Job};
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

pub(super) async fn do_search(client: QobuzClient, query: String) -> Result<SearchPayload, String> {
    let r = client.search(&query, 25).await.map_err(|e| e.to_string())?;
    let mut payload = SearchPayload::default();
    if let Some(list) = r.albums {
        for a in list.items {
            let label = format!("{} — {}", a.artist_name(), a.title);
            // Prefer a small image for the thumbnail to keep downloads cheap.
            let cover = a.image.as_ref().and_then(|i| {
                i.small
                    .clone()
                    .or_else(|| i.thumbnail.clone())
                    .or_else(|| i.large.clone())
            });
            payload.albums.push(AlbumResult {
                id: a.id,
                label,
                cover,
            });
        }
    }
    if let Some(list) = r.tracks {
        for t in list.items {
            let label = format!("{} — {}", t.artist_name(), t.title);
            payload.tracks.push((t.id.to_string(), label));
        }
    }
    if let Some(list) = r.artists {
        for a in list.items {
            payload.artists.push((a.id.to_string(), a.name));
        }
    }
    Ok(payload)
}

/// Download the bytes of an album cover thumbnail via the core client.
pub(super) async fn fetch_thumbnail(url: String) -> Result<Vec<u8>, ()> {
    qobuz_core::fetch_bytes(&url).await.map_err(|_| ())
}

pub(super) async fn resolve(client: QobuzClient, reference: Reference) -> Result<Vec<Job>, String> {
    engine::resolve(&client, &reference)
        .await
        .map_err(|e| e.to_string())
}
