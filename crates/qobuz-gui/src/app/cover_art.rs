//! The Settings Cover art choice and how it maps onto the config.

use qobuz_core::{Config, CoverSize};

/// One Cover art choice: Off, or a size. The config keeps them apart
/// (`embed_art`, `cover_size`), so picking Off doesn't forget the size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CoverArt {
    Off,
    Size(CoverSize),
}

impl CoverArt {
    pub(super) fn of(config: &Config) -> Self {
        if config.embed_art {
            CoverArt::Size(config.cover_size)
        } else {
            CoverArt::Off
        }
    }

    pub(super) fn all() -> Vec<CoverArt> {
        std::iter::once(CoverArt::Off)
            .chain(CoverSize::ALL.map(CoverArt::Size))
            .collect()
    }
}

impl std::fmt::Display for CoverArt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverArt::Off => f.write_str("Off"),
            CoverArt::Size(size) => size.fmt(f),
        }
    }
}
