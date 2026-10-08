# Spec Delta

## MODIFIED Requirements

### Requirement: Settings screen
The system SHALL provide a settings screen exposing Qobuz sign-in via a
`user_auth_token`, `app_id`/`app_secret`, download-directory picker, quality
selector, cover-art toggle, folder/track template fields with a live preview, and
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

### Requirement: Labeled settings fields
Every text input on the Settings screen SHALL have a visible label that remains shown when the field holds a value. Each row's first input SHALL start on one fixed-width label column shared with the File organization fields. A second input on the same row, such as `app_secret` after `app_id`, SHALL carry its own inline label.

#### Scenario: Filled credentials keep their labels
- **WHEN** the `app_id` and `app_secret` fields contain values
- **THEN** each field still shows its label beside it

#### Scenario: Token input is labeled
- **WHEN** the user views the Account card
- **THEN** the token input has a visible label beside it

#### Scenario: Inputs align across cards
- **WHEN** the user views the Settings screen
- **THEN** the `app_id`, token, folder and track inputs all start at the same horizontal position

#### Scenario: Credentials share a row
- **WHEN** the user views the API credentials card
- **THEN** `app_id` and `app_secret` are on one row, `app_id` on the label column and `app_secret` after its inline label

### Requirement: One primary action per settings card
Each Settings card SHALL present at most one control in the primary button style, and that control SHALL be the card's main action. Other controls in the card SHALL use the secondary style. When a card explains its buttons, the explanation SHALL be in the card's help panel and SHALL describe every button in that card.

#### Scenario: Credentials card emphasis
- **WHEN** the user views the API credentials card
- **THEN** Auto-detect is the only primary button, Check signing is secondary, and no explanatory text sits under the buttons

#### Scenario: Credentials help explains both buttons
- **WHEN** the user opens the API credentials help panel
- **THEN** it explains what Auto-detect and Check signing each do

#### Scenario: Account card emphasis
- **WHEN** the user views the Account card
- **THEN** Sign in is the only primary button and Sign out is secondary

## ADDED Requirements

### Requirement: Single-line Options card
The Options card SHALL lay out its controls on one line: Quality on the label column, then Concurrency, then the Embed cover art and Notify checkboxes.

#### Scenario: Options on one line
- **WHEN** the user views the Options card at the default window size
- **THEN** Quality, Concurrency and both checkboxes share one line, with Quality starting on the label column

## REMOVED Requirements

### Requirement: Options card layout
**Reason**: It required a two-line Options card, which is condensed to one line so the Settings screen fits the default window height.
**Migration**: Replaced by "Single-line Options card".
