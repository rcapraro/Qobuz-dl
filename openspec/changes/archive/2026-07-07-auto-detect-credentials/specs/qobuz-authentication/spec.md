## ADDED Requirements

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
