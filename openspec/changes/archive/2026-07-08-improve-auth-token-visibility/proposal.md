# Improve auth-token visibility

## Why

A saved `user_auth_token` is invisible today: after a restart the Account card's token field is empty (it is an input, never a display), so the only hint that a session exists is the small "● signed in / ○ signed out" dot in the header. Users cannot tell whether a token is stored, where it lives, or whether the one they see applies — which makes sign-in state confusing and sign-out feel like a no-op.

## What Changes

- The Account card (Settings) gains an explicit stored-token status line: whether a token is saved in the OS keyring, shown with a masked preview (e.g. `••••…k3Zq`, last 4 characters only) so the user can recognize which token is active without exposing it.
- The status line distinguishes how the current session came to be: token restored from the keyring at startup vs. validated by a Sign in during this session.
- Action gating follows token state: "Sign out" is only enabled when a token is stored; "Sign in" is only enabled when the token input is non-empty.
- Signing out updates the status line immediately (token removed → "No token saved"), and a keyring removal failure keeps the saved-state display truthful instead of claiming the token is gone.
- The header indicator is unchanged (it stays the at-a-glance global signal); the Account card becomes the detailed source of truth.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `qobuz-authentication`: new requirement — the stored-token state (present/absent, masked identity, origin of the current session) SHALL be discoverable by the user; the full token SHALL never be displayed back.
- `downloader-gui`: the Account card SHALL display the stored-token status and SHALL gate Sign in / Sign out availability on token state.

## Impact

- `crates/qobuz-gui/src/app.rs` — track token origin (restored vs. validated this session) in `App` state; SignOut/LoggedIn arms update it.
- `crates/qobuz-gui/src/app/view/settings.rs` — Account card renders the status line and gates the buttons.
- `crates/qobuz-core/src/auth.rs` — no storage changes; possibly a small helper for a masked token suffix (or keep masking in the GUI).
- No API, config-format, or keyring changes; no breaking changes.
