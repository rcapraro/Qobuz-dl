## Context

The Settings tab (`crates/qobuz-gui/src/app.rs`, `settings_view`) renders four
`iced_aw::Card`s: API credentials, Account, File organization, and Options. The
File organization card already carries a toggleable help panel: a `show_template_help:
bool` flag on `App`, a `Message::ToggleTemplateHelp`, a `secondary_button` toggle,
and a `template_help()` builder that returns a styled `Card`. This change replicates
that proven pattern for the other three cards.

`app_id`/`app_secret` are user-supplied and extracted from the Qobuz web player
(header `x-app-id`; `app_secret` used only for request signing). Auth supports
email/password login or a pasted `user_auth_token`; the token lives in the OS
keyring, never in config. These are the facts the help must convey.

## Goals / Non-Goals

**Goals:**
- Per-card, toggleable help for API credentials, Account, and Options, hidden by
  default, consistent with the existing template help in look and interaction.
- Accurate content, including concrete web-player extraction steps for the
  credentials.
- Confine changes to `qobuz-gui`.

**Non-Goals:**
- No changes to `qobuz-core`, config, auth, or the engine.
- No external links or embedded images; text-only guidance (the app has no
  hyperlink widget and the CSP/offline nature argues for plain text).
- No changes to the existing template-help behavior.

## Decisions

### Decision: One toggle + help flag per card, reusing the template-help pattern
Add three `bool` flags to `App` (`show_credentials_help`, `show_account_help`,
`show_options_help`) and three `Message` variants (`ToggleCredentialsHelp`,
`ToggleAccountHelp`, `ToggleOptionsHelp`), each flipping its flag in `update`.
Each card gains a `secondary_button` labeled "Show help"/"Hide help"; when on, the
card's body includes a help panel built by a dedicated function
(`credentials_help()`, `account_help()`, `options_help()`).
- **Alternatives considered:** a single shared help flag/toggle for the whole
  screen (rejected — less discoverable and mixes unrelated content); a separate Help
  tab (rejected per user — per-card is more contextual and matches the existing
  pattern).

### Decision: Help content rendered inline within each card's body
The help panel is appended to the card body `column` (as template help is), not a
nested card, so it reads as part of the section it explains and inherits the card's
surface. Use `style::mono` for literal values (header/field names) and `TEXT_SM`
body text, matching `template_help()`.

### Decision: Content scope
- **API credentials:** what `app_id` (sent as `x-app-id`) and `app_secret` (used to
  sign `track/getFileUrl` requests) are; numbered steps to obtain them from the
  Qobuz web player (open the web player while logged in, open browser dev tools →
  Network, filter for the bundle/API calls, read the `x-app-id` header and the
  app secret from the player bundle). Steps kept generic enough to survive minor
  web-player changes.
- **Account:** email/password vs. pasting a `user_auth_token`; token stored in the OS
  keyring, excluded from the saved config; meaning of the signed-in indicator.
- **Options:** the four quality tiers (MP3-320, FLAC-CD 16/44.1, FLAC-24/≤96,
  FLAC-Hi-Res 24/≤192) and that the API may downgrade delivered quality; the
  concurrency range (1–16 simultaneous downloads); embed-cover-art.

## Risks / Trade-offs

- [Web-player extraction steps may drift as Qobuz updates its player] → Mitigation:
  describe the approach (dev tools → find `x-app-id` / app secret) rather than exact
  pixel/menu paths, and keep it in help text that is easy to edit later.
- [Three more toggles add UI state] → Mitigation: identical, well-understood pattern
  to the existing template help; low complexity, no new dependencies.

## Open Questions

None — placement (per-card toggles) and credential-help depth (include extraction
steps) confirmed with the user.
