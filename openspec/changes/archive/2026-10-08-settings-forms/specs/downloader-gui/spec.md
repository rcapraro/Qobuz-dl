# Spec Delta

## ADDED Requirements

### Requirement: Unsaved settings indicator
The Settings screen SHALL show an "Unsaved changes" hint next to the Save settings control whenever the current settings differ from the persisted ones. Save settings SHALL be enabled only while there are unsaved changes. Any successful save, explicit or implicit, SHALL clear the hint.

#### Scenario: Editing a setting shows the hint
- **WHEN** the user changes any setting (credentials, directory, templates, quality, cover art, notifications, or concurrency) without saving
- **THEN** the "Unsaved changes" hint is shown and Save settings is enabled

#### Scenario: Saving clears the hint
- **WHEN** the user activates Save settings and the save succeeds
- **THEN** the hint disappears and Save settings is disabled

#### Scenario: Reverting an edit clears the hint
- **WHEN** the user changes a setting and then restores its persisted value
- **THEN** the hint disappears and Save settings is disabled

#### Scenario: Implicit save clears the hint
- **WHEN** an action that persists settings on its own succeeds (theme toggle, sign-in, credential auto-detection, or adopting a working fallback secret after a signing check or at the end of a download batch)
- **THEN** all current settings are persisted and the hint disappears

#### Scenario: Failed save keeps the hint
- **WHEN** a save fails
- **THEN** the error is reported on the status line and the hint stays shown

#### Scenario: Nothing to save at startup
- **WHEN** the app starts with its persisted settings
- **THEN** no hint is shown and Save settings is disabled

#### Scenario: Unreadable settings file stays replaceable
- **WHEN** the app starts with defaults because the persisted settings could not be read
- **THEN** the hint is shown and Save settings is enabled, so the defaults can replace the unreadable file

### Requirement: Labeled settings fields
Every text input on the Settings screen SHALL have a visible label that remains shown when the field holds a value. The labels SHALL share one fixed-width label column with the File organization fields, so all inputs on the screen start at the same horizontal position.

#### Scenario: Filled credentials keep their labels
- **WHEN** the `app_id` and `app_secret` fields contain values
- **THEN** each field still shows its label beside it

#### Scenario: Token input is labeled
- **WHEN** the user views the Account card
- **THEN** the token input has a visible label beside it

#### Scenario: Inputs align across cards
- **WHEN** the user views the Settings screen
- **THEN** the credential, token, folder and track inputs all start at the same horizontal position

### Requirement: One primary action per settings card
Each Settings card SHALL present at most one control in the primary button style, and that control SHALL be the card's main action. Other controls in the card SHALL use the secondary style. Helper text accompanying a button row SHALL describe every button in that row.

#### Scenario: Credentials card emphasis
- **WHEN** the user views the API credentials card
- **THEN** Auto-detect is the only primary button, Check signing is secondary, and the helper text explains both

#### Scenario: Account card emphasis
- **WHEN** the user views the Account card
- **THEN** Sign in is the only primary button and Sign out is secondary

### Requirement: Options card layout
The Options card SHALL lay out its controls on two lines: Quality and Concurrency on the first, the Embed cover art and Notify checkboxes on the second.

#### Scenario: Options on two lines
- **WHEN** the user views the Options card
- **THEN** Quality and Concurrency share the first line and both checkboxes share the second
