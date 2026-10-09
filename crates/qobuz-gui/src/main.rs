#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod style;

// `cosmic_text` (iced's text renderer) logs a benign WARN when an optional CJK
// system font is missing; `lofty` logs a benign WARN when it pads a FLAC on tag
// write. Quiet both to errors only.
const LOG_FILTER: &str = "warn,qobuz_core=info,qobuz_gui=info,cosmic_text=error,lofty=error";
// The MusicBrainz lookup's steps are for development; release builds compile
// debug logs out.
const DEV_LOG_FILTER: &str = "qobuz_core::musicbrainz=debug";

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                if cfg!(debug_assertions) {
                    format!("{LOG_FILTER},{DEV_LOG_FILTER}").into()
                } else {
                    LOG_FILTER.into()
                }
            }),
        )
        .init();

    app::run()
}
