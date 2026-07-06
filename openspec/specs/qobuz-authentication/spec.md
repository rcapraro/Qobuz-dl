# qobuz-authentication Specification

## Purpose
TBD - created by archiving change add-qobuz-downloader. Update Purpose after archive.
## Requirements
### Requirement: Login with credentials
The system SHALL allow the user to authenticate to Qobuz using an email and
password, and SHALL obtain and retain a `user_auth_token` for subsequent API
calls.

#### Scenario: Successful credential login
- **WHEN** the user submits a valid Qobuz email and password with a configured `app_id`
- **THEN** the system calls `user/login`, receives a `user_auth_token`, and reports the account as signed in

#### Scenario: Free or ineligible account
- **WHEN** the user logs in with an account lacking streaming/download credentials
- **THEN** the system surfaces a clear error and does not mark the account as signed in

#### Scenario: Invalid credentials
- **WHEN** the user submits an incorrect email or password
- **THEN** the system displays an authentication-failed message without crashing

### Requirement: Login with raw token
The system SHALL allow the user to paste an existing `user_auth_token` (with
`user_id`) and use it directly without submitting a password.

#### Scenario: Token accepted
- **WHEN** the user provides a valid `user_auth_token` and `user_id`
- **THEN** the system uses the token on subsequent calls and reports the account as signed in

### Requirement: Manage app credentials
The system SHALL allow the user to enter and persist an `app_id` and
`app_secret` used for API access and request signing.

#### Scenario: App credentials required for signed calls
- **WHEN** the user attempts a download without a configured `app_id` and `app_secret`
- **THEN** the system prompts the user to enter them before proceeding

### Requirement: Secure token storage
The system SHALL store the `user_auth_token` in the operating system keyring and
SHALL NOT persist the password or token in plaintext configuration files.

#### Scenario: Token persisted securely across restarts
- **WHEN** the user has signed in and restarts the app
- **THEN** the token is retrieved from the OS keyring and the user remains signed in without re-entering credentials

#### Scenario: Signature failure surfaced
- **WHEN** a signed API call is rejected due to an invalid signature or expired secret
- **THEN** the system reports the failure and prompts the user to verify the `app_secret`

