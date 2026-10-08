# Spec Delta

## MODIFIED Requirements

### Requirement: Settings screen
The system SHALL provide a settings screen exposing Qobuz sign-in via a
`user_auth_token`, `app_id`/`app_secret`, download-directory picker, quality
selector, Cover art selector (Off, 400 px, 500 px, 600 px),
folder/track template fields with a live preview, a rename template field with
a live preview, and
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

#### Scenario: Rename template field
- **WHEN** the user views the File organization card
- **THEN** a labeled Rename field for the rename template is shown below the folder and track templates, on the same label column

## ADDED Requirements

### Requirement: Rename an album's folder from the queue
A queue group SHALL offer a Rename folder control under the same conditions as Edit tags. Activating it SHALL show, in the group header, a name field prefilled with the rename template's suggestion and controls to confirm or cancel. Confirming SHALL rename the folder; the status line SHALL report success or the reason it was refused.

#### Scenario: Suggested name shown
- **WHEN** the user activates Rename folder on a finished album
- **THEN** a field prefilled with the suggested name appears in its header, with Rename and Cancel controls

#### Scenario: Free name
- **WHEN** the user replaces the suggestion with their own name and confirms
- **THEN** the folder gets that name, sanitized

#### Scenario: Cancel
- **WHEN** the user cancels
- **THEN** the field closes and nothing is renamed

#### Scenario: Not offered while downloading
- **WHEN** any track of a group is queued, downloading or tagging
- **THEN** its header does not offer Rename folder

#### Scenario: Empty name
- **WHEN** the name field is empty or sanitizes to nothing
- **THEN** the Rename control is unavailable
