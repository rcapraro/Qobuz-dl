# downloader-gui delta

## ADDED Requirements

### Requirement: Account card shows stored-token status
The Settings screen's Account card SHALL display the stored-token status line
(token saved with masked preview and session origin, or no token saved),
serving as the detailed counterpart to the header's at-a-glance signed-in
indicator.

#### Scenario: Status line with a saved token
- **WHEN** the user opens Settings while a token is stored
- **THEN** the Account card shows that a token is saved in the system keyring, a masked preview, and whether it was restored at startup or validated this session

#### Scenario: Status line without a token
- **WHEN** the user opens Settings while no token is stored
- **THEN** the Account card states that no token is saved

### Requirement: Token actions follow token state
The Account card SHALL enable Sign out only while a token is stored, and SHALL
enable Sign in only while the token input is non-empty.

#### Scenario: Sign out disabled without a token
- **WHEN** no token is stored
- **THEN** the Sign out button is disabled

#### Scenario: Sign in disabled with an empty input
- **WHEN** the token input field is empty
- **THEN** the Sign in button is disabled

#### Scenario: Actions re-enable as state changes
- **WHEN** the user pastes a token into the input, or a sign-in stores a token
- **THEN** Sign in (respectively Sign out) becomes enabled accordingly
