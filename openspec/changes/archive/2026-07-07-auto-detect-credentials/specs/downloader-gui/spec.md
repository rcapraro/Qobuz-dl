## ADDED Requirements

### Requirement: Auto-detect credentials control
The Settings screen SHALL provide a control that triggers automatic discovery
of the Qobuz `app_id` and `app_secret`, populates the credential fields with the
result, and communicates progress and outcome to the user.

#### Scenario: User triggers auto-detection
- **WHEN** the user activates the auto-detect control in Settings
- **THEN** the app runs discovery without blocking the UI and indicates that
  detection is in progress

#### Scenario: Fields populated on success
- **WHEN** discovery succeeds
- **THEN** the `app_id` and `app_secret` fields are filled with the discovered
  values and a success message is shown

#### Scenario: Error surfaced on failure
- **WHEN** discovery fails
- **THEN** the app shows a clear error message and the credential fields keep
  their previous contents so the user can still enter values manually
