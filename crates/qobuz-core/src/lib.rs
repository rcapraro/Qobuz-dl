//! `qobuz-core` — a UI-agnostic Qobuz API client and download engine.
//!
//! The GUI (and any future CLI) is built on top of these modules:
//! authentication, catalog browsing/search, signed file-URL requests, streamed
//! downloads with progress, path templating, and audio tagging.

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
pub mod signature;
pub mod tagging;
pub mod template;

mod util;

pub use bootstrap::{discover_app_credentials, AppCredentials};
pub use catalog::Reference;
pub use client::{QobuzClient, SigningCheck};
pub use config::Config;
pub use download::fetch_bytes;
pub use engine::{download_all, resolve, Job, JobEvent};
pub use error::{Error, Result};
pub use quality::Quality;
