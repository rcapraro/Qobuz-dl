use md5::{Digest, Md5};

/// Build the `request_sig` for a signed Qobuz API call.
///
/// The signature is the MD5 hex digest of a string built by concatenating,
/// with no separators:
///   `object` + `method`
///   then each signed parameter as `name` + `value`, **sorted alphabetically
///   by parameter name** (the `app_id` and `user_auth_token` are excluded)
///   then the `request_ts` (unix seconds)
///   then the `app_secret`.
///
/// For `track/getFileUrl` this yields, e.g.:
/// `trackgetFileUrlformat_id7intentstreamtrack_id123<ts><secret>`.
///
/// NOTE: The exact string shape can drift between Qobuz web-player releases;
/// verify against the live `streamrip` / `qopy.py` sources when the API rejects
/// signatures.
pub fn request_sig(
    object: &str,
    method: &str,
    params: &[(&str, String)],
    request_ts: u64,
    app_secret: &str,
) -> String {
    let mut sorted: Vec<&(&str, String)> = params.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    let mut s = String::new();
    s.push_str(object);
    s.push_str(method);
    for (name, value) in sorted {
        s.push_str(name);
        s.push_str(value);
    }
    s.push_str(&request_ts.to_string());
    s.push_str(app_secret);

    let mut hasher = Md5::new();
    hasher.update(s.as_bytes());
    hex(&hasher.finalize())
}

/// Convenience helper for the `track/getFileUrl` endpoint.
pub fn get_file_url_sig(
    track_id: &str,
    format_id: u32,
    request_ts: u64,
    app_secret: &str,
) -> String {
    request_sig(
        "track",
        "getFileUrl",
        &[
            ("format_id", format_id.to_string()),
            ("intent", "stream".to_string()),
            ("track_id", track_id.to_string()),
        ],
        request_ts,
        app_secret,
    )
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_file_url_concatenation_order() {
        // Guards object+method + sorted params + ts + secret concatenation.
        let sig = get_file_url_sig("123", 7, 1234567890, "secret");
        let manual = {
            let mut h = Md5::new();
            h.update(
                "trackgetFileUrlformat_id7intentstreamtrack_id1231234567890secret".as_bytes(),
            );
            hex(&h.finalize())
        };
        assert_eq!(sig, manual);
    }

    #[test]
    fn params_are_sorted() {
        // Order of input params must not change the signature.
        let a = request_sig(
            "track",
            "getFileUrl",
            &[
                ("track_id", "1".into()),
                ("format_id", "7".into()),
                ("intent", "stream".into()),
            ],
            100,
            "sec",
        );
        let b = request_sig(
            "track",
            "getFileUrl",
            &[
                ("format_id", "7".into()),
                ("intent", "stream".into()),
                ("track_id", "1".into()),
            ],
            100,
            "sec",
        );
        assert_eq!(a, b);
    }

    #[test]
    fn hex_is_32_chars() {
        assert_eq!(get_file_url_sig("1", 6, 1, "x").len(), 32);
    }
}
