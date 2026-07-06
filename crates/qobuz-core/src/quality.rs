use serde::{Deserialize, Serialize};

/// A Qobuz download quality tier, mapped to its API `format_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Quality {
    /// MP3 320 kbps — `format_id` 5.
    Mp3,
    /// FLAC 16-bit / 44.1 kHz (CD) — `format_id` 6.
    FlacCd,
    /// FLAC 24-bit / ≤ 96 kHz — `format_id` 7.
    #[default]
    Flac24,
    /// FLAC 24-bit / ≤ 192 kHz (best available) — `format_id` 27.
    FlacHiRes,
}

impl Quality {
    /// The Qobuz `format_id` for this tier.
    pub fn format_id(self) -> u32 {
        match self {
            Quality::Mp3 => 5,
            Quality::FlacCd => 6,
            Quality::Flac24 => 7,
            Quality::FlacHiRes => 27,
        }
    }

    /// Parse a `format_id` back into a tier, if recognized.
    pub fn from_format_id(id: u32) -> Option<Quality> {
        match id {
            5 => Some(Quality::Mp3),
            6 => Some(Quality::FlacCd),
            7 => Some(Quality::Flac24),
            27 => Some(Quality::FlacHiRes),
            _ => None,
        }
    }

    /// The container/file extension a tier is delivered in.
    pub fn extension(self) -> &'static str {
        match self {
            Quality::Mp3 => "mp3",
            _ => "flac",
        }
    }

    /// Human-readable label for UI.
    pub fn label(self) -> &'static str {
        match self {
            Quality::Mp3 => "MP3 320",
            Quality::FlacCd => "FLAC 16/44.1 (CD)",
            Quality::Flac24 => "FLAC 24/≤96",
            Quality::FlacHiRes => "FLAC 24/≤192 (Hi-Res)",
        }
    }

    /// All tiers, best-to-worst, for building dropdowns.
    pub const ALL: [Quality; 4] = [
        Quality::FlacHiRes,
        Quality::Flac24,
        Quality::FlacCd,
        Quality::Mp3,
    ];
}

impl std::fmt::Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_id_roundtrip() {
        for q in Quality::ALL {
            assert_eq!(Quality::from_format_id(q.format_id()), Some(q));
        }
    }

    #[test]
    fn unknown_format_id() {
        assert_eq!(Quality::from_format_id(99), None);
    }

    #[test]
    fn extensions() {
        assert_eq!(Quality::Mp3.extension(), "mp3");
        assert_eq!(Quality::FlacHiRes.extension(), "flac");
    }
}
