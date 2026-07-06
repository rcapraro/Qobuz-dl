//! Secure storage of the Qobuz `user_auth_token` via the OS keyring.
//!
//! macOS Keychain / Windows Credential Manager / Linux Secret Service. All
//! operations degrade gracefully: a keyring that is unavailable yields an error
//! the caller can surface, and the user can simply re-enter their token.

use crate::error::Result;

const SERVICE: &str = "com.qobuzdl.qobuz-dl";
const TOKEN_USER: &str = "user_auth_token";

fn entry() -> Result<keyring::Entry> {
    Ok(keyring::Entry::new(SERVICE, TOKEN_USER)?)
}

/// Persist the auth token in the OS keyring.
pub fn store_token(token: &str) -> Result<()> {
    entry()?.set_password(token)?;
    Ok(())
}

/// Retrieve the stored auth token, if any.
///
/// Returns `Ok(None)` when no token has been stored yet; `Err` only for genuine
/// keyring failures (e.g. no Secret Service provider on Linux).
pub fn load_token() -> Result<Option<String>> {
    match entry()?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Remove the stored token (sign out).
pub fn clear_token() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
