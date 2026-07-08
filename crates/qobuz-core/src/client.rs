//! Async Qobuz JSON API client.

use crate::error::{Error, Result};
use crate::models::*;
use crate::quality::Quality;
use crate::signature;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const API_BASE: &str = "https://www.qobuz.com/api.json/0.2/";
const PAGE_SIZE: u32 = 500;

/// Outcome of [`QobuzClient::check_signing`]: which secret actually signed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SigningCheck {
    /// The primary (entered) secret signs successfully.
    Primary,
    /// The primary secret was rejected, but a saved candidate signs. Carries the
    /// working candidate so callers can promote it to the primary.
    Fallback { working_secret: String },
}

/// Credentials + HTTP client for talking to Qobuz.
#[derive(Clone)]
pub struct QobuzClient {
    http: reqwest::Client,
    app_id: String,
    /// Candidate app secrets to try when signing, in priority order. Auto-detection
    /// yields several (one per timezone seed); only one is actually valid.
    app_secrets: Vec<String>,
    /// Index of the last secret that signed successfully, cached so repeated
    /// signed calls in a session skip the known-bad candidates.
    working_secret: Arc<Mutex<Option<usize>>>,
    token: Option<String>,
}

impl QobuzClient {
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>) -> Result<Self> {
        let app_id = app_id.into();
        if app_id.trim().is_empty() {
            return Err(Error::MissingAppCredentials("app_id"));
        }
        let http = reqwest::Client::builder()
            .user_agent("qobuz-dl/0.1 (+https://github.com/)")
            .timeout(Duration::from_secs(30))
            .build()?;
        let mut app_secrets = Vec::new();
        let secret = app_secret.into();
        if !secret.trim().is_empty() {
            app_secrets.push(secret);
        }
        Ok(Self {
            http,
            app_id,
            app_secrets,
            working_secret: Arc::new(Mutex::new(None)),
            token: None,
        })
    }

    /// Add extra candidate app secrets (e.g. from auto-detection) to try when
    /// signing. Empty and duplicate values are ignored.
    pub fn with_secret_candidates(mut self, secrets: impl IntoIterator<Item = String>) -> Self {
        for s in secrets {
            if !s.trim().is_empty() && !self.app_secrets.contains(&s) {
                self.app_secrets.push(s);
            }
        }
        self
    }

    /// Attach an existing auth token (raw-token path or restored from keyring).
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut h = HeaderMap::new();
        h.insert(
            HeaderName::from_static("x-app-id"),
            HeaderValue::from_str(&self.app_id)
                .map_err(|_| Error::Auth("invalid app_id".into()))?,
        );
        if let Some(t) = &self.token {
            h.insert(
                HeaderName::from_static("x-user-auth-token"),
                HeaderValue::from_str(t).map_err(|_| Error::Auth("invalid token".into()))?,
            );
        }
        Ok(h)
    }

    /// Perform a GET against `endpoint` (relative to the API base) with query
    /// params, returning a deserialized `T`.
    async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> Result<T> {
        let url = format!("{API_BASE}{endpoint}");
        let mut query: Vec<(&str, String)> = params.to_vec();
        query.push(("app_id", self.app_id.clone()));

        let resp = self
            .http
            .get(&url)
            .headers(self.headers()?)
            .query(&query)
            .send()
            .await?;

        let status = resp.status();
        if status.as_u16() == 429 {
            let retry_after = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(std::time::Duration::from_secs);
            return Err(Error::RateLimited { retry_after });
        }
        let text = resp.text().await?;
        if !status.is_success() {
            // Try to surface a useful message; detect signature failures.
            if is_signature_error(&text) {
                return Err(Error::InvalidSignature);
            }
            // Prefer the API's `message` field over the raw JSON body.
            let message = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| truncate(&text, 300));
            return Err(Error::Http {
                status: status.as_u16(),
                message,
            });
        }
        Ok(serde_json::from_str(&text)?)
    }

    // ---- Authentication -------------------------------------------------

    // Email/password login is intentionally not supported: Qobuz's `user/login`
    // rejects it for partner/bundled accounts (e.g. Qobuz via a telco), which
    // have no Qobuz-native password. Authentication is via `user_auth_token`.

    /// Validate a raw token by fetching the user's profile. On success the token
    /// is retained for subsequent calls.
    pub async fn login_with_token(&mut self, token: &str) -> Result<()> {
        self.token = Some(token.to_string());
        // A signed-in-only call confirms the token is accepted.
        match self.get::<serde_json::Value>("user/get", &[]).await {
            Ok(_) => Ok(()),
            Err(Error::Http { status: 401, .. }) | Err(Error::Auth(_)) => {
                self.token = None;
                Err(Error::Auth("token was rejected".into()))
            }
            // `user/get` may not exist on all deployments; treat other errors as
            // non-fatal — the token is provisionally accepted.
            Err(_) => Ok(()),
        }
    }

    // ---- Metadata -------------------------------------------------------

    pub async fn album(&self, album_id: &str) -> Result<Album> {
        self.get("album/get", &[("album_id", album_id.to_string())])
            .await
    }

    pub async fn track(&self, track_id: &str) -> Result<Track> {
        self.get("track/get", &[("track_id", track_id.to_string())])
            .await
    }

    /// Fetch a playlist including all its tracks (paginating past 500).
    pub async fn playlist(&self, playlist_id: &str) -> Result<Playlist> {
        let mut playlist: Playlist = self
            .get(
                "playlist/get",
                &[
                    ("playlist_id", playlist_id.to_string()),
                    ("extra", "tracks".to_string()),
                    ("limit", PAGE_SIZE.to_string()),
                    ("offset", "0".to_string()),
                ],
            )
            .await?;

        let total = playlist.tracks.as_ref().and_then(|t| t.total).unwrap_or(0);
        let mut offset = playlist
            .tracks
            .as_ref()
            .map(|t| t.items.len() as u32)
            .unwrap_or(0);

        while offset < total {
            let page: Playlist = self
                .get(
                    "playlist/get",
                    &[
                        ("playlist_id", playlist_id.to_string()),
                        ("extra", "tracks".to_string()),
                        ("limit", PAGE_SIZE.to_string()),
                        ("offset", offset.to_string()),
                    ],
                )
                .await?;
            match page.tracks {
                Some(tl) if !tl.items.is_empty() => {
                    offset += tl.items.len() as u32;
                    if let Some(dst) = playlist.tracks.as_mut() {
                        dst.items.extend(tl.items);
                    }
                }
                _ => break,
            }
        }
        Ok(playlist)
    }

    pub async fn artist(&self, artist_id: &str) -> Result<Artist> {
        self.get(
            "artist/get",
            &[
                ("artist_id", artist_id.to_string()),
                ("extra", "albums".to_string()),
                ("limit", PAGE_SIZE.to_string()),
            ],
        )
        .await
    }

    // ---- Search ---------------------------------------------------------

    pub async fn search(&self, query: &str, limit: u32) -> Result<SearchResults> {
        self.get(
            "catalog/search",
            &[("query", query.to_string()), ("limit", limit.to_string())],
        )
        .await
    }

    // ---- Signed file URL ------------------------------------------------

    /// Sign and issue a single `track/getFileUrl` request with one specific
    /// secret. `Err(Error::InvalidSignature)` means that secret was rejected;
    /// `Err(Error::NoFileUrl)` means the track just isn't fetchable.
    async fn get_file_url_signed(
        &self,
        track_id: &str,
        format_id: u32,
        secret: &str,
    ) -> Result<FileUrl> {
        let ts = now_unix();
        let sig = signature::get_file_url_sig(track_id, format_id, ts, secret);
        let params = [
            ("track_id", track_id.to_string()),
            ("format_id", format_id.to_string()),
            ("intent", "stream".to_string()),
            ("request_ts", ts.to_string()),
            ("request_sig", sig),
        ];
        let file: FileUrl = self.get("track/getFileUrl", &params).await?;
        if file.url.is_none() {
            return Err(Error::NoFileUrl);
        }
        Ok(file)
    }

    /// Request a signed, temporary download URL for a track at the requested
    /// quality. The response reports the *actually delivered* quality, which may
    /// be lower (graceful downgrade). Tries each candidate secret, caching the
    /// one that works; `AllSignaturesRejected` if every candidate is refused.
    pub async fn file_url(&self, track_id: &str, quality: Quality) -> Result<FileUrl> {
        if self.app_secrets.is_empty() {
            return Err(Error::MissingAppCredentials("app_secret"));
        }
        let format_id = quality.format_id();

        // Try candidates starting from the last-known-good one, wrapping around.
        // A rejected signature just means "wrong secret" — move to the next; any
        // other error is fatal and propagated immediately.
        let n = self.app_secrets.len();
        let start = self.working_secret.lock().unwrap().unwrap_or(0) % n;
        for k in 0..n {
            let idx = (start + k) % n;
            match self
                .get_file_url_signed(track_id, format_id, &self.app_secrets[idx])
                .await
            {
                Ok(file) => {
                    *self.working_secret.lock().unwrap() = Some(idx);
                    return Ok(file);
                }
                // Wrong secret — try the next candidate.
                Err(Error::InvalidSignature) => continue,
                Err(e) => return Err(e),
            }
        }
        // Every candidate's signature was rejected. That is signing drift (the
        // formula moved) or genuinely wrong secrets — not "try one more secret".
        tracing::warn!(
            candidates = n,
            "all app_secret candidates rejected — possible signing-formula drift"
        );
        Err(Error::AllSignaturesRejected { candidates: n })
    }

    /// The app secret that last signed successfully in this session, if any.
    /// Callers can persist it so a future session starts from the known-good
    /// secret instead of re-probing every candidate.
    pub fn working_secret(&self) -> Option<String> {
        self.working_secret
            .lock()
            .unwrap()
            .and_then(|i| self.app_secrets.get(i).cloned())
    }

    /// Whether `secret`'s signature is accepted by `track/getFileUrl`. The API
    /// validates the signature before returning a URL, so an accepted signature
    /// yields either a URL (`Ok`) or "no file" for a non-streamable track
    /// (`NoFileUrl`) — both mean the secret is good. Only `InvalidSignature`
    /// means it is bad. A verdict is per-secret, not per-track, so the first
    /// non-404 response settles it; stale tracks (404) are skipped.
    async fn secret_signs(&self, track_ids: &[i64], format_id: u32, secret: &str) -> Result<bool> {
        for id in track_ids {
            match self.get_file_url_signed(&id.to_string(), format_id, secret).await {
                Ok(_) | Err(Error::NoFileUrl) => return Ok(true),
                Err(Error::InvalidSignature) => return Ok(false),
                // Stale track ID — try the next one for a verdict.
                Err(Error::Http { status: 404, .. }) => continue,
                // Bad app_id/token, ineligible account, network: report it.
                Err(e) => return Err(e),
            }
        }
        // Every track 404'd — no verdict possible.
        Err(Error::NoFileUrl)
    }

    /// Check request signing end-to-end, validating the *entered* (primary)
    /// secret specifically. Looks up real tracks via the unsigned `search`, then
    /// signs `track/getFileUrl`: if the primary secret signs, returns
    /// `SigningCheck::Primary`; if only a saved candidate signs, returns
    /// `SigningCheck::Fallback` with that secret (so the caller can adopt it).
    /// `AllSignaturesRejected` if none sign (formula drift or wrong secret); a
    /// bad `app_id`/token or ineligible account surfaces as its own error.
    pub async fn check_signing(&self) -> Result<SigningCheck> {
        if self.app_secrets.is_empty() {
            return Err(Error::MissingAppCredentials("app_secret"));
        }
        let results = self.search("music", 5).await?;
        let track_ids: Vec<i64> = results
            .tracks
            .map(|tl| tl.items.into_iter().map(|t| t.id).collect())
            .unwrap_or_default();
        if track_ids.is_empty() {
            return Err(Error::NoFileUrl);
        }
        let format_id = Quality::default().format_id();

        // The entered (primary) secret first.
        if self.secret_signs(&track_ids, format_id, &self.app_secrets[0]).await? {
            return Ok(SigningCheck::Primary);
        }
        // Entered secret rejected — does a saved candidate sign?
        for secret in &self.app_secrets[1..] {
            if self.secret_signs(&track_ids, format_id, secret).await? {
                return Ok(SigningCheck::Fallback {
                    working_secret: secret.clone(),
                });
            }
        }
        Err(Error::AllSignaturesRejected {
            candidates: self.app_secrets.len(),
        })
    }

    /// Shared HTTP client (for streaming downloads of already-signed URLs).
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }
}

/// Whether an error body reports a rejected request signature. Matches
/// case-insensitively — the API returns "Invalid Request Signature parameter
/// (request_sig)" (capital I) — so this must not be a case-sensitive check, or
/// `file_url`'s candidate-secret retry never triggers.
fn is_signature_error(body: &str) -> bool {
    let lower = body.to_lowercase();
    lower.contains("signature") && (lower.contains("invalid") || lower.contains("request_sig"))
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_signature_error_case_insensitively() {
        // The live API returns a capital "I" — the classifier must still match,
        // otherwise file_url's candidate-secret retry never triggers.
        assert!(is_signature_error(
            "HTTP 400: Invalid Request Signature parameter (request_sig)"
        ));
        assert!(is_signature_error(
            r#"{"code":400,"message":"Invalid Request Signature parameter (request_sig)"}"#
        ));
        assert!(is_signature_error("invalid signature"));
    }

    #[test]
    fn non_signature_errors_are_not_misclassified() {
        assert!(!is_signature_error(
            r#"{"code":401,"message":"User authentication is required."}"#
        ));
        assert!(!is_signature_error("Invalid or missing app_id parameter."));
    }
}
