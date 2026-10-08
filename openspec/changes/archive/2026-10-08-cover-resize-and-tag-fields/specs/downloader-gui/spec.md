# Spec Delta

## MODIFIED Requirements

### Requirement: Settings screen
The system SHALL provide a settings screen exposing Qobuz sign-in via a
`user_auth_token`, `app_id`/`app_secret`, download-directory picker, quality
selector, Cover art selector (Off, 400 px, 500 px, 600 px),
folder/track template fields with a live preview, and
a bounded numeric concurrency control that accepts only values in the range 1–10.
A persisted concurrency above 10 SHALL be treated as 10.
The account section SHALL NOT offer email/password login (unsupported by Qobuz for
partner/bundled accounts) and SHALL explain how to obtain the token from the Qobuz
web player.

#### Scenario: Configure and save
- **WHEN** the user fills in credentials and preferences and saves
- **THEN** the settings are persisted and the app reflects the signed-in state

#### Scenario: Token sign-in
- **WHEN** the user pastes a valid `user_auth_token` and presses Sign in
- **THEN** the token is validated, stored in the OS keyring, and the account is reported as signed in

#### Scenario: Guidance for obtaining the token
- **WHEN** the user opens the account help panel
- **THEN** the app explains that sign-in uses a `user_auth_token` and how to copy it from the web player's developer tools

#### Scenario: Concurrency is bounded
- **WHEN** the user adjusts the concurrency control
- **THEN** the value is constrained to the range 1–10 and cannot be set to a non-numeric or out-of-range value

#### Scenario: Saved concurrency above the range
- **WHEN** the app starts with a persisted concurrency above 10
- **THEN** the concurrency control shows 10, downloads run at most 10 tracks at once, and no unsaved-changes hint is shown

#### Scenario: Choose a cover size
- **WHEN** the user picks 400 px in the Cover art selector and saves
- **THEN** later downloads embed covers scaled down to 400 px

### Requirement: Single-line Options card
The Options card SHALL lay out its controls on one line: Quality on the label column, then Concurrency, then the Cover art selector and the Notify checkbox.

#### Scenario: Options on one line
- **WHEN** the user views the Options card at the default window size
- **THEN** Quality, Concurrency, the Cover art selector and the Notify checkbox share one line, with Quality starting on the label column
