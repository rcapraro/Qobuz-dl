//! Small crate-internal helpers.

/// Truncate `s` to at most `max_bytes` bytes without splitting a UTF-8
/// codepoint: the cut falls on the nearest char boundary at or below the
/// limit. Returns `s` unchanged when it already fits.
pub(crate) fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_cut_exactly_at_the_limit() {
        assert_eq!(truncate_at_char_boundary("abcdef", 4), "abcd");
    }

    #[test]
    fn short_input_is_returned_unchanged() {
        assert_eq!(truncate_at_char_boundary("abc", 10), "abc");
        assert_eq!(truncate_at_char_boundary("", 0), "");
    }

    #[test]
    fn cut_never_splits_a_codepoint() {
        // 'é' is 2 bytes; a limit of 3 lands mid-codepoint and must back off.
        let s = "ééé";
        assert_eq!(truncate_at_char_boundary(s, 3), "é");
        assert_eq!(truncate_at_char_boundary(s, 4), "éé");
        // 4-byte emoji.
        let e = "🎵🎵";
        assert_eq!(truncate_at_char_boundary(e, 5), "🎵");
    }

    #[test]
    fn exact_boundary_is_kept() {
        assert_eq!(truncate_at_char_boundary("éé", 2), "é");
    }
}
