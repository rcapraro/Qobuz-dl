## Why

The Settings screen asks for values users can't guess: `app_id` and `app_secret`
must be extracted from the Qobuz web player, and "Account" offers two different
sign-in paths (email/password vs. a pasted `user_auth_token`) with no explanation
of which to use or where the token is stored. First-time users get stuck with no
in-app guidance. The File organization card already has a toggleable template-help
panel; extending that same pattern to the other Settings cards makes the app
self-explanatory without cluttering the default view.

## What Changes

- Add a toggleable **"Show help" / "Hide help"** control inside the **API
  credentials**, **Account**, and **Options** cards (hidden by default), mirroring
  the existing template-help toggle in File organization.
- **API credentials help**: explain what `app_id` and `app_secret` are and how each
  is used, plus **step-by-step instructions to extract them from the Qobuz web
  player** (the part users get stuck on).
- **Account help**: explain the two sign-in methods (email/password vs. pasting a
  `user_auth_token`), that the token is stored in the OS keyring (not in config),
  and what "signed in / signed out" means.
- **Options help**: explain the quality tiers, the concurrency range, and the
  embed-cover-art toggle.
- Help panels render inside their card (consistent with the iced_aw card styling).

## Capabilities

### New Capabilities
- `settings-help`: in-app, per-card explanatory help for the API credentials,
  Account, and Options sections of the Settings screen, hidden by default and shown
  via a per-card toggle.

### Modified Capabilities
<!-- None: the Settings screen requirement in downloader-gui is unchanged; this adds
     new help behavior alongside it, parallel to the existing template-help capability. -->

## Impact

- **Crate:** `qobuz-gui` only. No changes to `qobuz-core`, config schema, auth, or
  the download engine.
- **Code:** `crates/qobuz-gui/src/app.rs` — new `Message` variants for the per-card
  help toggles, matching `App` state flags, and help-panel builder functions
  (parallel to the existing `template_help()` and `show_template_help`).
- **No new dependencies.** Reuses the existing iced/iced_aw widgets and `style.rs`
  card styling.
- **No breaking changes.**
