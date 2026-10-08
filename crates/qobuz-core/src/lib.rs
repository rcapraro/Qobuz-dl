//! `qobuz-core` — a UI-agnostic Qobuz API client and download engine.
//!
//! The GUI (and any future CLI) is built on top of these modules:
//! authentication, catalog browsing/search, signed file-URL requests, streamed
//! downloads with progress, path templating, and audio tagging.

pub mod artwork;
pub mod auth;
pub mod bootstrap;
pub mod catalog;
pub mod client;
pub mod config;
pub mod download;
pub mod engine;
pub mod error;
pub mod models;
pub mod quality;
pub mod rename;
pub mod signature;
pub mod tag_edit;
pub mod tagging;
pub mod template;

#[cfg(test)]
mod test_support;
mod util;

pub use artwork::CoverSize;
pub use bootstrap::{discover_app_credentials, AppCredentials};
pub use catalog::Reference;
pub use client::{QobuzClient, SigningCheck};
pub use config::Config;
pub use download::fetch_bytes;
pub use engine::{download_all, resolve, Job, JobEvent};
pub use error::{Error, Result};
pub use quality::Quality;
pub use rename::rename_folder;

/// Re-exported so callers can cancel a batch without depending on `tokio-util`
/// themselves. Pass one to [`download_all`] and call `cancel()` to stop it.
pub use tokio_util::sync::CancellationToken;
