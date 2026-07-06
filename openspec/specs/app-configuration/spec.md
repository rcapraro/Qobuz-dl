# app-configuration Specification

## Purpose
TBD - created by archiving change add-qobuz-downloader. Update Purpose after archive.
## Requirements
### Requirement: Persist settings
The system SHALL persist user settings — download directory, quality tier,
folder template, track template, cover-art embedding toggle, download
concurrency, `app_id`, and `app_secret` — to the platform configuration
directory, and SHALL reload them on startup.

#### Scenario: Settings survive restart
- **WHEN** the user changes settings and restarts the app
- **THEN** the previously saved settings are loaded and applied

#### Scenario: Sensible defaults
- **WHEN** the app runs for the first time with no saved configuration
- **THEN** it starts with default templates, default quality, and a default download directory without erroring

### Requirement: Secrets excluded from config file
The system SHALL NOT store the `user_auth_token` or password in the plaintext
configuration file; the token is delegated to the keyring.

#### Scenario: Config file contains no token
- **WHEN** the user inspects the saved configuration file
- **THEN** it contains settings and app credentials but no `user_auth_token` or password

### Requirement: Live template preview
The system SHALL show a preview of the rendered path for the current folder and
track templates as the user edits them.

#### Scenario: Preview updates
- **WHEN** the user edits the folder or track template in settings
- **THEN** the system displays an example rendered path reflecting the current templates

