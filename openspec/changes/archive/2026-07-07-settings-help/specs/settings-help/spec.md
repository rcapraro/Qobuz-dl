## ADDED Requirements

### Requirement: Per-card toggleable help

The Settings screen SHALL provide in-app help for the API credentials, Account, and
Options cards. Each card's help SHALL be hidden by default and SHALL be shown or
hidden by a control within that card, so the Settings screen stays uncluttered.

#### Scenario: Help hidden by default

- **WHEN** the Settings screen is first shown
- **THEN** no card's help content is displayed, and each of the API credentials,
  Account, and Options cards shows a control to reveal its help

#### Scenario: Toggle a card's help independently

- **WHEN** the user activates the help control on one card
- **THEN** that card's help content becomes visible and the other cards' help
  remains hidden, and activating the control again hides it

### Requirement: API credentials help

The API credentials card help SHALL explain what `app_id` and `app_secret` are and
how they are used, and SHALL provide step-by-step instructions for extracting them
from the Qobuz web player.

#### Scenario: Fields explained

- **WHEN** the user opens the API credentials help
- **THEN** it explains that `app_id` identifies the client (sent as the `x-app-id`
  header) and that `app_secret` is used to sign file-URL requests

#### Scenario: Extraction steps shown

- **WHEN** the user reads the API credentials help
- **THEN** it lists ordered steps to obtain `app_id` and `app_secret` from the Qobuz
  web player using the browser's developer tools

### Requirement: Account help

The Account card help SHALL explain the available sign-in methods and how the
session credential is stored.

#### Scenario: Sign-in methods explained

- **WHEN** the user opens the Account help
- **THEN** it explains that the user can sign in with email and password or by
  pasting a `user_auth_token`

#### Scenario: Token storage explained

- **WHEN** the user reads the Account help
- **THEN** it states that the authentication token is stored in the operating
  system keyring and is not written to the saved configuration

### Requirement: Options help

The Options card help SHALL explain the quality selector, the concurrency control,
and the cover-art toggle.

#### Scenario: Options explained

- **WHEN** the user opens the Options help
- **THEN** it describes the available quality tiers and notes that the delivered
  quality may be downgraded by the service, the meaning of the concurrency value and
  its allowed range, and the effect of the embed-cover-art toggle
