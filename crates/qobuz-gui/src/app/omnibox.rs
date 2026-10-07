//! The Search screen's single field: deciding whether submitted text is a link
//! to add or a query to search.

use qobuz_core::catalog::{parse_input, Reference};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Submit {
    Empty,
    Add(Reference),
    BadUrl(String),
    /// `bare_id` is offered as an explicit "add by ID" action, never taken
    /// silently: `parse_input` accepts any single alphanumeric word as an
    /// album ID, so "Radiohead" or "1989" would otherwise skip the search.
    Search {
        query: String,
        bare_id: Option<Reference>,
    },
}

/// Classify by the input's shape, not by whether it parses — almost any single
/// word parses as a bare ID.
pub(super) fn classify(input: &str) -> Submit {
    let input = input.trim();
    if input.is_empty() {
        return Submit::Empty;
    }
    if !looks_like_url(input) {
        return Submit::Search {
            query: input.to_owned(),
            bare_id: parse_input(input).ok(),
        };
    }
    // `parse_input` ignores the host, so a foreign link with an `/album/` path
    // would otherwise be accepted.
    if !host(input).contains("qobuz.") {
        return Submit::BadUrl(format!("Not a Qobuz link: {input}"));
    }
    match parse_input(input) {
        Ok(reference) => Submit::Add(reference),
        Err(e) => Submit::BadUrl(e.to_string()),
    }
}

fn looks_like_url(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.contains("qobuz.com")
}

fn host(url: &str) -> String {
    let lower = url.to_ascii_lowercase();
    let after_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);
    after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn album_url_is_added() {
        assert_eq!(
            classify("https://open.qobuz.com/album/abc123"),
            Submit::Add(Reference::Album("abc123".into()))
        );
    }

    #[test]
    fn playlist_url_is_added() {
        assert_eq!(
            classify("  https://play.qobuz.com/playlist/4242  "),
            Submit::Add(Reference::Playlist("4242".into()))
        );
    }

    #[test]
    fn url_without_scheme_is_added() {
        assert_eq!(
            classify("open.qobuz.com/track/98765"),
            Submit::Add(Reference::Track("98765".into()))
        );
    }

    #[test]
    fn foreign_url_is_rejected_not_searched() {
        assert!(matches!(
            classify("https://example.com/album/123"),
            Submit::BadUrl(_)
        ));
    }

    #[test]
    fn unrecognised_qobuz_url_is_rejected() {
        assert!(matches!(
            classify("https://www.qobuz.com/us-en/discover"),
            Submit::BadUrl(_)
        ));
    }

    #[test]
    fn single_word_searches_and_offers_id() {
        assert_eq!(
            classify("Radiohead"),
            Submit::Search {
                query: "Radiohead".into(),
                bare_id: Some(Reference::Album("Radiohead".into())),
            }
        );
    }

    #[test]
    fn number_searches_and_offers_id() {
        assert_eq!(
            classify("1989"),
            Submit::Search {
                query: "1989".into(),
                bare_id: Some(Reference::Album("1989".into())),
            }
        );
    }

    #[test]
    fn multi_word_query_searches_without_id() {
        assert_eq!(
            classify("kind of blue"),
            Submit::Search {
                query: "kind of blue".into(),
                bare_id: None,
            }
        );
    }

    #[test]
    fn blank_is_empty() {
        assert_eq!(classify("   "), Submit::Empty);
    }
}
