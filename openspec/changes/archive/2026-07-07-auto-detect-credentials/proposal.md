## Why

Today the user must manually extract the `app_id` and `app_secret` from the
Qobuz web player's JavaScript bundle by hand — opening DevTools, reading a
request header, and scraping a hex string out of a minified bundle. This is
error-prone, intimidating for non-technical users, and breaks silently whenever
Qobuz ships a new web-player release with rotated credentials. Automating the
extraction removes the single biggest onboarding hurdle and lets the app
self-heal when Qobuz rotates its credentials.

## What Changes

- Add an **auto-detect credentials** capability to `qobuz-core`: fetch the Qobuz
  web-player login page, locate its JavaScript bundle, and parse out the
  `app_id` and `app_secret` (the same seed/timezone-based extraction used by
  `streamrip`/`qobuz-dl`).
- Expose this via the core public API as a single async function returning the
  discovered `app_id` and `app_secret`.
- Add a GUI action in the Settings screen ("Auto-detect") that runs the
  extraction, fills the `app_id`/`app_secret` fields, and reports success or a
  clear failure message; manual entry remains available as a fallback.
- Keep the existing manual-entry help text but reframe it as the fallback path.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `qobuz-authentication`: the "Manage app credentials" requirement gains the
  ability to automatically discover `app_id` and `app_secret` from the Qobuz
  web player, in addition to manual entry.
- `downloader-gui`: the Settings screen gains an auto-detect control that
  populates the credential fields and surfaces progress/errors.

## Impact

- **qobuz-core**: new module (e.g. `bootstrap.rs`) performing the bundle fetch
  and parse; new function re-exported from `lib.rs`. Depends on the existing
  `reqwest` client and `regex` (new dependency if not already present).
- **qobuz-gui**: new `Message` variants and a button in `settings_view`; async
  call wrapped in `Task::perform`.
- **Network**: adds outbound requests to `play.qobuz.com` (login page + bundle)
  performed only on explicit user action.
- No config schema change — the discovered values are written into the existing
  `app_id`/`app_secret` fields.
