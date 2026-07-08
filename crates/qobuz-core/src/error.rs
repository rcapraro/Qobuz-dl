use std::time::Duration;
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

    #[error("all {candidates} candidate app_secret(s) were rejected — the Qobuz signing formula may have changed (not your credentials). Try Auto-detect, or update the app.")]
    AllSignaturesRejected { candidates: usize },

    #[error("app credentials missing: {0}")]
    MissingAppCredentials(&'static str),

    #[error("could not auto-detect credentials: {0}")]
    CredentialDiscovery(String),

    #[error("rate limited by the Qobuz API")]
    RateLimited {
        /// Server-advertised delay before retrying, from a `Retry-After` header.
        retry_after: Option<Duration>,
    },

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

impl Error {
    /// Whether this error is worth retrying: rate limiting, network failures
    /// (including timeouts), and HTTP 5xx responses. All other errors are
    /// treated as permanent.
    pub fn is_transient(&self) -> bool {
        matches!(self, Error::RateLimited { .. } | Error::Network(_))
            || matches!(self, Error::Http { status, .. } if *status >= 500)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_transient_errors() {
        assert!(Error::RateLimited { retry_after: None }.is_transient());
        assert!(Error::Http {
            status: 503,
            message: "boom".into()
        }
        .is_transient());
    }

    #[test]
    fn classifies_permanent_errors() {
        assert!(!Error::Auth("nope".into()).is_transient());
        assert!(!Error::NoFileUrl.is_transient());
        assert!(!Error::InvalidSignature.is_transient());
        assert!(!Error::AllSignaturesRejected { candidates: 3 }.is_transient());
        assert!(!Error::Http {
            status: 404,
            message: "missing".into()
        }
        .is_transient());
    }
}
