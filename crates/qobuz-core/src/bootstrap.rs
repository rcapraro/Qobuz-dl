//! Automatic discovery of Qobuz web-player API credentials.
//!
//! `app_id` and `app_secret` are not bundled with this app; historically the
//! user extracts them by hand from the Qobuz web player. This module reproduces
//! the well-known `streamrip`/`qobuz-dl` technique: fetch the public web-player
//! login page, locate its hashed `bundle.js`, and parse the `app_id` plus the
//! timezone-seeded `app_secret` fragments out of it.
//!
//! Extraction yields ONE `app_id` but SEVERAL candidate secrets (one per
//! timezone seed). The correct one is identified later, at request-signing
//! time, by the client trying each until `track/getFileUrl` is accepted (see
//! [`crate::client::QobuzClient::file_url`]).
//!
//! NOTE: the bundle layout can drift between Qobuz web-player releases; if the
//! regexes stop matching, cross-check the live `streamrip` / `qopy.py` sources
//! (same maintenance posture as [`crate::signature`]).

use crate::error::{Error, Result};
use base64::Engine;
use regex::Regex;

const LOGIN_URL: &str = "https://play.qobuz.com/login";
const PLAY_BASE: &str = "https://play.qobuz.com";
/// The base64-encoded secret fragments carry a fixed trailing block that is not
/// part of the secret; `streamrip` drops the last 44 characters before decoding.
const SECRET_TRAILER_LEN: usize = 44;

/// Credentials discovered from the Qobuz web player.
#[derive(Debug, Clone)]
pub struct AppCredentials {
    /// The web-player API `app_id`.
    pub app_id: String,
    /// Candidate `app_secret`s to try, in priority order. The signing path
    /// attempts each until one is accepted.
    pub app_secrets: Vec<String>,
}

/// Fetch the Qobuz web-player bundle and extract the `app_id` and candidate
/// `app_secret`s. Requires no prior credentials or authentication.
pub async fn discover_app_credentials() -> Result<AppCredentials> {
    let http = reqwest::Client::builder()
        .user_agent("qobuz-dl/0.1 (+https://github.com/)")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let login_page = http
        .get(LOGIN_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let bundle_path = find_bundle_path(&login_page).ok_or_else(|| {
        Error::CredentialDiscovery("could not find the web-player bundle URL".into())
    })?;

    let bundle = http
        .get(format!("{PLAY_BASE}{bundle_path}"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    parse_credentials(&bundle)
}

/// Locate the hashed `bundle.js` path referenced by the login page.
fn find_bundle_path(login_page: &str) -> Option<String> {
    let re = Regex::new(r#"<script src="(/resources/\d+\.\d+\.\d+-[a-z]\d+/bundle\.js)""#).unwrap();
    re.captures(login_page).map(|c| c[1].to_string())
}

/// Parse the `app_id` and candidate `app_secret`s out of a web-player bundle.
///
/// Split out from the network fetch so it can be unit-tested offline.
fn parse_credentials(bundle: &str) -> Result<AppCredentials> {
    let app_id = extract_app_id(bundle)
        .ok_or_else(|| Error::CredentialDiscovery("app_id not found in bundle".into()))?;
    // Modern bundles expose the real secret inline in the production block; older
    // ones only carry the timezone-seeded fragments. Try the inline one first,
    // then fall back to the assembled candidates.
    let mut app_secrets = Vec::new();
    if let Some(inline) = extract_inline_secret(bundle) {
        app_secrets.push(inline);
    }
    for s in extract_secrets(bundle) {
        if !app_secrets.contains(&s) {
            app_secrets.push(s);
        }
    }
    if app_secrets.is_empty() {
        return Err(Error::CredentialDiscovery(
            "no app_secret candidates found in bundle".into(),
        ));
    }
    Ok(AppCredentials {
        app_id,
        app_secrets,
    })
}

/// Extract the secret embedded directly in the production API block, if present.
fn extract_inline_secret(bundle: &str) -> Option<String> {
    let re = Regex::new(r#"production:\{api:\{appId:"\d+",appSecret:"(\w+)""#).unwrap();
    re.captures(bundle).map(|c| c[1].to_string())
}

fn extract_app_id(bundle: &str) -> Option<String> {
    // Preferred: the production API block.
    let primary = Regex::new(r#"production:\{api:\{appId:"(\d+)""#).unwrap();
    if let Some(c) = primary.captures(bundle) {
        return Some(c[1].to_string());
    }
    // Fallback for layout drift.
    let fallback = Regex::new(r#"appId:"(\d+)""#).unwrap();
    fallback.captures(bundle).map(|c| c[1].to_string())
}

/// Reconstruct the candidate secrets from the per-timezone seed/info/extras
/// fragments, mirroring `streamrip`'s `Spoofer`.
fn extract_secrets(bundle: &str) -> Vec<String> {
    // Collect (timezone, [seed, info?, extras?]) preserving insertion order.
    let seed_re =
        Regex::new(r#"[a-z]\.initialSeed\("([\w=]+)",window\.utimezone\.([a-z]+)\)"#).unwrap();
    let mut secrets: Vec<(String, Vec<String>)> = Vec::new();
    for c in seed_re.captures_iter(bundle) {
        secrets.push((c[2].to_string(), vec![c[1].to_string()]));
    }

    // streamrip reorders the second discovered timezone to the front.
    if secrets.len() >= 2 {
        let second = secrets.remove(1);
        secrets.insert(0, second);
    }

    // Append the info/extras fragments, constrained to the known timezones.
    let tz_alt: Vec<String> = secrets.iter().map(|(tz, _)| capitalize(tz)).collect();
    if !tz_alt.is_empty() {
        let pattern = format!(
            r#"name:"\w+/({})",info:"([\w=]+)",extras:"([\w=]+)""#,
            tz_alt.join("|")
        );
        if let Ok(ie_re) = Regex::new(&pattern) {
            for c in ie_re.captures_iter(bundle) {
                let tz = c[1].to_lowercase();
                if let Some(entry) = secrets.iter_mut().find(|(t, _)| *t == tz) {
                    entry.1.push(c[2].to_string());
                    entry.1.push(c[3].to_string());
                }
            }
        }
    }

    // Assemble each secret: join fragments, drop the trailing block, decode.
    let engine = base64::engine::general_purpose::STANDARD;
    let mut out = Vec::new();
    for (_tz, parts) in secrets {
        let joined: String = parts.concat();
        if joined.len() <= SECRET_TRAILER_LEN {
            continue;
        }
        let trimmed = &joined[..joined.len() - SECRET_TRAILER_LEN];
        if let Ok(bytes) = engine.decode(trimmed) {
            if let Ok(s) = String::from_utf8(bytes) {
                if !s.is_empty() && !out.contains(&s) {
                    out.push(s);
                }
            }
        }
    }
    out
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic bundle whose `berlin` timezone carries `plaintext` as
    /// its (base64-encoded, 44-char-trailered) secret, plus a bare `paris` seed
    /// that decodes to nothing.
    fn bundle_with_secret(plaintext: &str) -> String {
        let engine = base64::engine::general_purpose::STANDARD;
        let b64 = engine.encode(plaintext);
        let full = format!("{b64}{}", "A".repeat(SECRET_TRAILER_LEN));
        let third = full.len() / 3;
        let (seed, rest) = full.split_at(third);
        let (info, extras) = rest.split_at(rest.len() / 2);
        format!(
            concat!(
                r#"var x=production:{{api:{{appId:"123456789",appSecret:"unused"}}}};"#,
                r#"a.initialSeed("AAAA",window.utimezone.paris);"#,
                r#"b.initialSeed("{seed}",window.utimezone.berlin);"#,
                r#"var m={{name:"Europe/Berlin",info:"{info}",extras:"{extras}"}};"#,
            ),
            seed = seed,
            info = info,
            extras = extras,
        )
    }

    #[test]
    fn parses_app_id_and_secret_from_bundle() {
        // base64 of all-'A' plaintext stays within the [\w=] class the regexes accept.
        let plaintext = "A".repeat(30);
        let bundle = bundle_with_secret(&plaintext);
        let creds = parse_credentials(&bundle).unwrap();
        assert_eq!(creds.app_id, "123456789");
        assert!(
            creds.app_secrets.iter().any(|s| s == &plaintext),
            "expected decoded secret in {:?}",
            creds.app_secrets
        );
    }

    #[test]
    fn finds_bundle_path_in_login_page() {
        let page = r#"<html><script src="/resources/7.1.3-b011/bundle.js"></script></html>"#;
        assert_eq!(
            find_bundle_path(page).as_deref(),
            Some("/resources/7.1.3-b011/bundle.js")
        );
    }

    #[test]
    fn malformed_bundle_yields_discovery_error() {
        for input in ["", "totally unrelated content", r#"appId:"" no secrets"#] {
            match parse_credentials(input) {
                Err(Error::CredentialDiscovery(_)) => {}
                other => panic!("expected CredentialDiscovery for {input:?}, got {other:?}"),
            }
        }
    }
}
