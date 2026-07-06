use thiserror::Error;

/// Errors produced by the Qobuz core engine.
#[derive(Debug, Error)]
pub enum Error {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("account is not eligible for streaming/download (free or restricted account)")]
    IneligibleAccount,

    #[error("request signature rejected — verify the app_secret")]
    InvalidSignature,

    #[error("app credentials missing: {0}")]
    MissingAppCredentials(&'static str),

    #[error("rate limited by the Qobuz API")]
    RateLimited,

    #[error("could not recognize input as a Qobuz URL or ID: {0}")]
    UnrecognizedInput(String),

    #[error("no downloadable file URL was returned for this track")]
    NoFileUrl,

    #[error("failed to parse API response: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("secure token storage error: {0}")]
    Keyring(String),

    #[error("tagging error: {0}")]
    Tagging(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("configuration error: {0}")]
    Config(String),
}

impl From<keyring::Error> for Error {
    fn from(e: keyring::Error) -> Self {
        Error::Keyring(e.to_string())
    }
}

impl From<lofty::error::LoftyError> for Error {
    fn from(e: lofty::error::LoftyError) -> Self {
        Error::Tagging(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
