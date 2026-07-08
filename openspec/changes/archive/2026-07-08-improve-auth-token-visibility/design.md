# Design: improve-auth-token-visibility

## Context

The token lives only in the OS keyring (`qobuz-core/src/auth.rs`) and, at
runtime, in `App.token: Option<String>` (`qobuz-gui/src/app.rs`). The GUI just
refactored to derived state (`signed_in()` reads `token.is_some()`) and split
views into `app/view/`; the Account card is built in
`app/view/settings.rs`. The header shows "● signed in / ○ signed out"; nothing
else reveals whether a token exists, and the token input is always empty on
startup.

Constraints: no traits/dyn dispatch in core; GUI does no keyring/FS access
directly; the full token must never be echoed back to the UI.

## Goals / Non-Goals

**Goals**
- Make stored-token presence, identity (masked), and session origin visible in the Account card.
- Gate Sign in / Sign out on actual state.
- Keep sign-out reporting truthful when keyring removal fails.

**Non-Goals**
- No change to token storage, validation, or the login flow itself.
- No periodic token-health checking or automatic re-validation.
- No change to the header indicator (spec'd elsewhere; stays as-is).

## Decisions

1. **Track session origin with a small enum in `App`, not new core state.**
   `enum TokenOrigin { Restored, ValidatedThisSession }` stored alongside
   `token` (e.g. `token: Option<(String, TokenOrigin)>` or a parallel field set
   in `new()` and `LoggedIn(Ok)`). Alternative — persisting origin/timestamps in
   config — rejected: it duplicates keyring truth and adds config churn for a
   display-only concern.

2. **Masking lives in the GUI.** A tiny `masked_token(&str) -> String` helper in
   `app/view/settings.rs` (or `app/view/mod.rs`) producing `••••…k3Zq` (last 4
   chars; fewer if the token is shorter than 8, fully masked below 4).
   Alternative — a core `auth::masked()` helper — rejected: masking is a
   presentation rule; core stays UI-agnostic.

3. **Sign-out keeps `token` on keyring-removal failure.** Today `SignOut`
   clears `self.token` unconditionally. To satisfy "truthful sign-out state",
   clear the in-memory token only when `auth::clear_token()` succeeds; on
   failure, report the error and leave the state showing a saved token.
   Alternative — clear memory but display "keyring still holds a token" —
   rejected: two sources of truth is exactly the smell the refactor removed.

4. **Button gating via `on_press_maybe`.** Same pattern already used by
   "Start downloads" / "Retry": `Sign out` gets
   `.on_press_maybe(app.signed_in().then_some(Message::SignOut))`; `Sign in`
   gets `(!app.token_input.trim().is_empty()).then_some(Message::LoginToken)`.
   Requires switching the two buttons from `action_button`/`secondary_button`
   (which take a concrete `Message`) to maybe-press variants — add
   `action_button_maybe`/`secondary_button_maybe` in `style.rs` or build the
   buttons locally like the queue view does.

5. **Status line placement**: a single small text row at the top of the Account
   card body, e.g. `Token: saved in keyring (••••…k3Zq) — restored at startup`
   or `Token: none saved — paste a user_auth_token below`. Rendered from
   `app.token` + origin; no new messages needed.

## Risks / Trade-offs

- [Masked suffix leaks 4 characters of a secret] → Standard practice (card
  numbers, API-key dashboards); 4 chars of a long opaque token is not usable,
  and the token is already on this machine's keyring.
- [Keyring removal failure leaves the user "stuck signed in"] → The status line
  plus error message explain why; retrying Sign out stays available since the
  button remains enabled.
- [Buttons gated off could confuse if state is stale] → State is derived
  directly from `token`/input on every view rebuild; no staleness possible.

## Migration Plan

Pure additive GUI change; no data migration. Rollback = revert the commit.

## Open Questions

None — scope is deliberately display + gating only.
