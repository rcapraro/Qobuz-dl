# qobuz-authentication Specification

## Purpose
TBD - created by archiving change add-qobuz-downloader. Update Purpose after archive.
## Requirements
### Requirement: Login with raw token
The system SHALL allow the user to paste an existing `user_auth_token` (with
`user_id`) and use it directly without submitting a password.

#### Scenario: Token accepted
- **WHEN** the user provides a valid `user_auth_token` and `user_id`
- **THEN** the system uses the token on subsequent calls and reports the account as signed in

### Requirement: Manage app credentials
The system SHALL allow the user to enter and persist an `app_id` and
`app_secret` used for API access and request signing.

#### Scenario: App credentials required for signed calls
- **WHEN** the user attempts a download without a configured `app_id` and `app_secret`
- **THEN** the system prompts the user to enter them before proceeding

### Requirement: Secure token storage
The system SHALL store the `user_auth_token` in the operating system keyring and
SHALL NOT persist the password or token in plaintext configuration files.

#### Scenario: Token persisted securely across restarts
- **WHEN** the user has signed in and restarts the app
- **THEN** the token is retrieved from the OS keyring and the user remains signed in without re-entering credentials

#### Scenario: Signature failure surfaced
- **WHEN** a signed API call is rejected due to an invalid signature or expired secret
- **THEN** the system reports the failure and prompts the user to verify the `app_secret`

### Requirement: Auto-detect app credentials
The system SHALL be able to automatically discover the `app_id` and
`app_secret` from the Qobuz web player, so the user does not have to extract
them by hand. Discovery SHALL run only on explicit user request and SHALL NOT
require any prior credentials or authentication.

#### Scenario: Successful auto-detection
- **WHEN** the user requests credential auto-detection
- **THEN** the system fetches the Qobuz web-player bundle, extracts a valid
  `app_id` and `app_secret`, and returns them for the user to review and persist

#### Scenario: Detection fills the credential fields
- **WHEN** auto-detection succeeds
- **THEN** the discovered `app_id` and `app_secret` populate the configured
  credential values, replacing any previously entered values

#### Scenario: Detection failure leaves manual entry intact
- **WHEN** auto-detection fails (network error, bundle unreachable, or the
  bundle format is unrecognized)
- **THEN** the system reports a clear error, does not alter the existing
  credential values, and manual entry remains available as a fallback

### Requirement: Manual credential entry remains available
The system SHALL continue to allow the user to enter and persist `app_id` and
`app_secret` manually, independent of the auto-detection path.

#### Scenario: Manual entry after or instead of detection
- **WHEN** the user types `app_id` and `app_secret` directly
- **THEN** the system accepts and persists them exactly as before, whether or
  not auto-detection was ever attempted

### Requirement: Stored-token visibility
The system SHALL make the stored `user_auth_token` state discoverable by the
user: whether a token is currently saved in the OS keyring, a masked preview
sufficient to recognize it (at most the last 4 characters), and how the current
session was established (restored from the keyring at startup, or validated by
a sign-in during this session). The system SHALL NOT display the full token
back to the user.

#### Scenario: Saved token is visible after restart
- **WHEN** a token is stored in the OS keyring and the app starts
- **THEN** the user can see that a token is saved, a masked preview of it, and that the session was restored from the keyring

#### Scenario: No token saved
- **WHEN** no token is stored in the OS keyring
- **THEN** the user can see that no token is saved and that signing in requires pasting a `user_auth_token`

#### Scenario: Token saved during this session
- **WHEN** the user signs in with a pasted token and it is validated and stored
- **THEN** the stored-token state updates to show a token is saved and that it was validated this session

#### Scenario: Full token never echoed
- **WHEN** any stored-token state is displayed
- **THEN** at most the last 4 characters of the token are shown; the rest is masked

### Requirement: Truthful sign-out state
After a sign-out, the displayed stored-token state SHALL reflect what actually
happened in the keyring: removed on success, still present if removal failed.

#### Scenario: Sign-out removes the token
- **WHEN** the user signs out and the keyring removal succeeds
- **THEN** the stored-token state shows no token saved

#### Scenario: Sign-out fails to remove the token
- **WHEN** the user signs out and the keyring removal fails
- **THEN** the failure is reported and the stored-token state continues to show that a token is saved

