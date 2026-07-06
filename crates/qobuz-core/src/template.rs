//! Path templating and filesystem-safe sanitization.
//!
//! Templates use `{placeholder}` tokens. Track numbers support zero-padding via
//! `{tracknumber:02}`. Each rendered path *segment* is sanitized independently
//! so that separators inside a title never create unintended directories.

use std::collections::HashMap;

/// Values available to folder/track templates for a single track.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub values: HashMap<String, String>,
    pub track_number: Option<u32>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: &str, value: impl Into<String>) -> &mut Self {
        self.values.insert(key.to_string(), value.into());
        self
    }

    pub fn with_track_number(&mut self, n: u32) -> &mut Self {
        self.track_number = Some(n);
        self.values.insert("tracknumber".to_string(), n.to_string());
        self
    }
}

/// Characters illegal in path segments on Windows (a superset covering all
/// three target platforms).
const ILLEGAL: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

const MAX_SEGMENT_LEN: usize = 200;

/// Sanitize one path segment: strip illegal characters and control chars,
/// collapse whitespace, trim, and cap length. Never returns an empty string.
pub fn sanitize_segment(input: &str) -> String {
    let mut out: String = input
        .chars()
        .map(|c| {
            if ILLEGAL.contains(&c) || c.is_control() {
                ' '
            } else {
                c
            }
        })
        .collect();

    // Collapse runs of whitespace.
    let collapsed: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    out = collapsed.trim().trim_matches('.').trim().to_string();

    if out.len() > MAX_SEGMENT_LEN {
        out.truncate(MAX_SEGMENT_LEN);
        out = out.trim().to_string();
    }

    if out.is_empty() {
        out.push('_');
    }
    out
}

/// Render a single-segment template against the context. Supports `{key}` and
/// `{tracknumber:0N}` zero-padding. Unknown placeholders render as empty.
pub fn render_segment(template: &str, ctx: &TemplateContext) -> String {
    let rendered = render_raw(template, ctx);
    sanitize_segment(&rendered)
}

/// Render a multi-segment template (may contain `/`), sanitizing each segment.
pub fn render_path(template: &str, ctx: &TemplateContext) -> Vec<String> {
    let raw = render_raw(template, ctx);
    raw.split('/')
        .filter(|s| !s.trim().is_empty())
        .map(sanitize_segment)
        .collect()
}

fn render_raw(template: &str, ctx: &TemplateContext) -> String {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '{' {
            out.push(c);
            continue;
        }
        // Collect until '}'.
        let mut token = String::new();
        let mut closed = false;
        for tc in chars.by_ref() {
            if tc == '}' {
                closed = true;
                break;
            }
            token.push(tc);
        }
        if !closed {
            // Unterminated brace — emit literally.
            out.push('{');
            out.push_str(&token);
            continue;
        }
        out.push_str(&resolve_token(&token, ctx));
    }
    out
}

fn resolve_token(token: &str, ctx: &TemplateContext) -> String {
    // Support "name" and "name:0N" (pad width) forms, e.g. {tracknumber:02}.
    let (name, pad) = match token.split_once(':') {
        Some((n, spec)) => (n, spec.trim().parse::<usize>().ok()),
        None => (token, None),
    };

    if let (Some(width), Some(n)) = (pad, numeric(name, ctx)) {
        return format!("{n:0width$}");
    }

    ctx.values.get(name).cloned().unwrap_or_default()
}

fn numeric(name: &str, ctx: &TemplateContext) -> Option<u64> {
    if name == "tracknumber" {
        if let Some(n) = ctx.track_number {
            return Some(n as u64);
        }
    }
    ctx.values.get(name).and_then(|v| v.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> TemplateContext {
        let mut c = TemplateContext::new();
        c.set("albumartist", "Miles Davis")
            .set("album", "Kind of Blue")
            .set("year", "1959")
            .set("title", "So What")
            .with_track_number(1);
        c
    }

    #[test]
    fn renders_folder() {
        let segs = render_path("{albumartist} - {album} ({year})", &ctx());
        assert_eq!(segs, vec!["Miles Davis - Kind of Blue (1959)"]);
    }

    #[test]
    fn zero_pads_track_number() {
        let s = render_segment("{tracknumber:02}. {title}", &ctx());
        assert_eq!(s, "01. So What");
    }

    #[test]
    fn strips_illegal_chars() {
        let mut c = TemplateContext::new();
        c.set("title", "AC/DC: Back? \"Yes\"").with_track_number(2);
        let s = render_segment("{tracknumber:02} {title}", &c);
        assert!(!s.contains('/'));
        assert!(!s.contains(':'));
        assert!(!s.contains('?'));
        assert!(!s.contains('"'));
        assert_eq!(s, "02 AC DC Back Yes");
    }

    #[test]
    fn multi_segment_split() {
        let segs = render_path("{albumartist}/{album}", &ctx());
        assert_eq!(segs, vec!["Miles Davis", "Kind of Blue"]);
    }

    #[test]
    fn empty_segment_becomes_placeholder() {
        let c = TemplateContext::new();
        assert_eq!(sanitize_segment(""), "_");
        assert_eq!(render_segment("{missing}", &c), "_");
    }

    #[test]
    fn unknown_placeholder_is_empty() {
        let s = render_segment("{title}{nope}", &ctx());
        assert_eq!(s, "So What");
    }
}
