# qobuz-authentication delta

## ADDED Requirements

### Requirement: Stored-token visibility
The system SHALL make the stored `user_auth_token` state discoverable by the
user: whether a token is currently saved in the OS keyring, a masked preview
sufficient to recognize it (at most the last 4 characters), and how the current
session was established (restored from the keyring at startup, or validated by
a sign-in during this session). The system SHALL NOT display the full token
back to the user.

#### Scenario: Saved token is visible after restart
- **WHEN** a token is stored in the OS keyring and the app starts
- **THEN** the user can see that a token is saved, a masked preview of it, and that the session was restored from the keyring

#### Scenario: No token saved
- **WHEN** no token is stored in the OS keyring
- **THEN** the user can see that no token is saved and that signing in requires pasting a `user_auth_token`

#### Scenario: Token saved during this session
- **WHEN** the user signs in with a pasted token and it is validated and stored
- **THEN** the stored-token state updates to show a token is saved and that it was validated this session

#### Scenario: Full token never echoed
- **WHEN** any stored-token state is displayed
- **THEN** at most the last 4 characters of the token are shown; the rest is masked

### Requirement: Truthful sign-out state
After a sign-out, the displayed stored-token state SHALL reflect what actually
happened in the keyring: removed on success, still present if removal failed.

#### Scenario: Sign-out removes the token
- **WHEN** the user signs out and the keyring removal succeeds
- **THEN** the stored-token state shows no token saved

#### Scenario: Sign-out fails to remove the token
- **WHEN** the user signs out and the keyring removal fails
- **THEN** the failure is reported and the stored-token state continues to show that a token is saved
