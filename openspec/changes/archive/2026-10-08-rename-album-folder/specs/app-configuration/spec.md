# Spec Delta

## MODIFIED Requirements

### Requirement: Persist settings
The system SHALL persist user settings — download directory, quality tier,
folder template, track template, rename template, cover art setting (Off or a
size), download concurrency, `app_id`, and `app_secret` — to the platform
configuration directory, and SHALL reload them on startup.

#### Scenario: Settings survive restart
- **WHEN** the user changes settings and restarts the app
- **THEN** the previously saved settings are loaded and applied

#### Scenario: Sensible defaults
- **WHEN** the app runs for the first time with no saved configuration
- **THEN** it starts with default templates, default quality, and a default download directory without erroring

#### Scenario: Cover art setting from an older configuration
- **WHEN** the app starts with a configuration saved before the cover art size existed
- **THEN** cover art is Off if embedding was disabled there, and 600 px otherwise

#### Scenario: Rename template from an older configuration
- **WHEN** the app starts with a configuration saved before the rename template existed
- **THEN** the rename template is `{albumartist} - {album} ({year})`

### Requirement: Live template preview
The system SHALL show a preview of the rendered path for the current folder and
track templates, and of the folder name for the current rename template, as the
user edits them.

#### Scenario: Preview updates
- **WHEN** the user edits the folder or track template in settings
- **THEN** the system displays an example rendered path reflecting the current templates

#### Scenario: Rename preview updates
- **WHEN** the user edits the rename template
- **THEN** the system displays an example folder name reflecting it
