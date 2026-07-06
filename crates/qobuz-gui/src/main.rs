#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,qobuz_core=info,qobuz_gui=info".into()),
        )
        .init();

    app::run()
}
