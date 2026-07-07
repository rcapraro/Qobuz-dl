#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod style;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                // `cosmic_text` (iced's text renderer) logs a benign WARN when an
                // optional CJK system font is missing; `lofty` logs a benign WARN
                // when it pads a FLAC on tag write. Quiet both to errors only.
                .unwrap_or_else(|_| {
                    "warn,qobuz_core=info,qobuz_gui=info,cosmic_text=error,lofty=error".into()
                }),
        )
        .init();

    app::run()
}
