# project-documentation — delta

## ADDED Requirements

### Requirement: README presents the application visually
The README SHALL include screenshots of the application's three screens (Search, Queue, Settings), stored in-repo under `docs/screenshots/` and referenced by relative path, with a hero screenshot visible before any build or installation instructions. Screenshots MUST reflect the current UI and MUST NOT expose secrets (`app_id`, `app_secret`, `user_auth_token` values).

#### Scenario: Visitor previews the app on GitHub
- **WHEN** a visitor opens the repository page on GitHub without cloning or building
- **THEN** the rendered README shows a screenshot of the app near the top and one screenshot per screen (Search, Queue, Settings) further down

#### Scenario: Settings screenshot contains no credentials
- **WHEN** the Settings screenshot is inspected at full resolution
- **THEN** no usable `app_id`, `app_secret`, or `user_auth_token` value is readable (fields empty, masked, or redacted)

### Requirement: README provides an installation path without building from source
The README SHALL contain an Installation section that links to the project's GitHub Releases and names the prebuilt artifacts per platform (macOS dmg, Windows NSIS installer, Linux AppImage/deb), including platform-specific runtime notes (Linux Secret Service provider and GTK).

#### Scenario: Non-developer wants the app
- **WHEN** a user without a Rust toolchain reads the README
- **THEN** they find a section directing them to the GitHub Releases page with the artifact matching their OS, before the build-from-source instructions

### Requirement: README documents the end-to-end usage flow
The README SHALL include a short usage walkthrough covering: signing in with the `user_auth_token`, finding music by search or pasted Qobuz URL/ID, selecting quality, and where downloads land (directory + path templates). Existing content — the legal notice, the token-retrieval steps, configuration/keyring notes, and the macOS dev-keychain workaround — MUST be preserved.

#### Scenario: New user goes from install to first download
- **WHEN** a new user follows the README top to bottom after installing
- **THEN** the documented steps (sign in → search or paste URL → choose quality → download) match the app's actual flow, with no removed prior content

### Requirement: README carries project status badges
The README header SHALL display badges for the latest release version and the license, and MUST NOT display badges that reference non-existent or perpetually-stale signals (e.g., a branch CI badge when CI only runs on tags).

#### Scenario: Visitor checks project maturity
- **WHEN** a visitor views the top of the README
- **THEN** they see a latest-release badge and a license badge that resolve to accurate values
