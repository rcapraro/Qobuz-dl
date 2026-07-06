//! Resolving user input (Qobuz URLs or bare IDs) into typed references.

use crate::error::{Error, Result};

/// A typed reference to a Qobuz entity the user wants to act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    Album(String),
    Track(String),
    Playlist(String),
    Artist(String),
}

impl Reference {
    pub fn kind(&self) -> &'static str {
        match self {
            Reference::Album(_) => "album",
            Reference::Track(_) => "track",
            Reference::Playlist(_) => "playlist",
            Reference::Artist(_) => "artist",
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Reference::Album(id)
            | Reference::Track(id)
            | Reference::Playlist(id)
            | Reference::Artist(id) => id,
        }
    }
}

/// Parse a pasted Qobuz URL or bare ID into a [`Reference`].
///
/// Recognizes URLs like:
///   `https://open.qobuz.com/album/{id}`
///   `https://play.qobuz.com/album/{id}`
///   `https://www.qobuz.com/xx-xx/album/slug/{id}`
///   `.../track/{id}`, `.../playlist/{id}`, `.../artist/{id}`
///
/// A bare numeric string is treated as an **album** id (the most common case);
/// an alphanumeric hash id (albums often use these) is likewise treated as an
/// album. Callers wanting a different default can construct a [`Reference`]
/// directly.
pub fn parse_input(input: &str) -> Result<Reference> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::UnrecognizedInput(input.to_string()));
    }

    // Bare id (numeric or album hash) → default to album.
    if is_bare_id(trimmed) {
        return Ok(Reference::Album(trimmed.to_string()));
    }

    if let Some(rest) = strip_scheme_host(trimmed) {
        let segments: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
        // Locate a known kind keyword; the id is the last bare-id segment that
        // follows it (handles both `.../album/{id}` and, for www.qobuz.com,
        // `.../album/{slug}/{id}`). Fall back to the immediate next segment.
        for (i, seg) in segments.iter().enumerate() {
            let make = match *seg {
                "album" => Reference::Album as fn(String) -> Reference,
                "track" => Reference::Track,
                "playlist" => Reference::Playlist,
                "artist" => Reference::Artist,
                _ => continue,
            };
            let after = &segments[i + 1..];
            let id = after
                .iter()
                .rev()
                .find(|s| is_bare_id(&clean_id(s)))
                .or_else(|| after.first())
                .copied();
            if let Some(id) = id {
                return Ok(make(clean_id(id)));
            }
        }
    }

    Err(Error::UnrecognizedInput(input.to_string()))
}

fn clean_id(s: &str) -> String {
    s.split(['?', '#']).next().unwrap_or(s).to_string()
}

fn is_bare_id(s: &str) -> bool {
    !s.is_empty()
        && !s.contains('/')
        && !s.contains('.')
        && s.chars().all(|c| c.is_ascii_alphanumeric())
        // Reject pure kind keywords.
        && !matches!(s, "album" | "track" | "playlist" | "artist")
}

fn strip_scheme_host(url: &str) -> Option<String> {
    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    // Drop the host component.
    let (_host, rest) = after_scheme.split_once('/')?;
    Some(rest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_qobuz_album() {
        assert_eq!(
            parse_input("https://open.qobuz.com/album/abc123").unwrap(),
            Reference::Album("abc123".into())
        );
    }

    #[test]
    fn play_qobuz_track() {
        assert_eq!(
            parse_input("https://play.qobuz.com/track/98765").unwrap(),
            Reference::Track("98765".into())
        );
    }

    #[test]
    fn www_qobuz_with_slug_and_query() {
        let r = parse_input("https://www.qobuz.com/us-en/album/some-slug/xyz789?foo=1").unwrap();
        assert_eq!(r, Reference::Album("xyz789".into()));
    }

    #[test]
    fn playlist_url() {
        assert_eq!(
            parse_input("https://open.qobuz.com/playlist/555").unwrap(),
            Reference::Playlist("555".into())
        );
    }

    #[test]
    fn bare_numeric_id_is_album() {
        assert_eq!(
            parse_input("  123456 ").unwrap(),
            Reference::Album("123456".into())
        );
    }

    #[test]
    fn empty_input_rejected() {
        assert!(matches!(parse_input("   "), Err(Error::UnrecognizedInput(_))));
    }

    #[test]
    fn garbage_url_rejected() {
        assert!(parse_input("https://example.com/foo/bar").is_err());
    }
}
