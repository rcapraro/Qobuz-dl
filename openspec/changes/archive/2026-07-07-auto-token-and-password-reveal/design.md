## Context

This change went through two pivots during implementation as the real behavior of
Qobuz's `user/login` was discovered:

1. Original plan: reveal the password + auto-discover the token via email/password
   and drop the manual token path.
2. Live testing (documented below) showed email/password login is rejected for
   partner/bundled accounts. First correction: keep the token path.
3. Final: since email/password cannot work for those accounts and the token works
   universally, remove email/password entirely — **token-only** sign-in.

## Investigation (why email/password is unsupported)

Tested against the live API with the maintainer's account (`@orange.fr`, a Qobuz
via Orange bundle):

- `user/login` with email + password was tried as GET and POST, with the password
  both plaintext and MD5-hashed, and with full browser-like headers
  (`origin`/`referer`/`user-agent`). **Every** variant returned
  `401 {"message":"User authentication is required."}`.
- The web player's own captured login carried `extra=partner` and an existing
  `x-user-auth-token` — i.e. it authenticates via the partner, not a password.
- The `user_auth_token` from the web player works: `user/get` returns the profile.

Conclusion: partner/bundled accounts have no Qobuz-native password. Token auth is
the only universally-working method.

## Goals / Non-Goals

**Goals:**
- Token-only sign-in (paste `user_auth_token`).
- Clear instructions for obtaining the token, in the account help and the README.

**Non-Goals:**
- Email/password login (removed).
- Browser/localStorage scraping or an embedded webview.
- Any change to token storage (OS keyring) or how it is sent (`x-user-auth-token`).

## Decisions

### Decision: Remove email/password from core and GUI
Delete `QobuzClient::login` and its helpers (`md5_hex`, `json_id_to_string`) from
`client.rs`; `login_with_token` becomes the sole auth entry point. In the GUI,
remove the email/password state, the reveal toggle, the `LoginPassword` flow, and
`login_password`. `Message::LoggedIn` reverts to `Result<String, String>` (just
the token). Rationale: keeping a path that silently 401s for partner accounts is a
worse UX than not offering it.

### Decision: Token field as the single account control
The account card is one secure text field ("paste your user_auth_token") plus
Sign in / Sign out. `login_token` validates via `login_with_token` and returns the
token.

### Decision: Document token retrieval in two places
The account help panel and the README both give the DevTools → Network →
`x-user-auth-token` steps, so users can self-serve without external docs.

## Risks / Trade-offs

- [Removing password login breaks users who signed in with a password] → BREAKING,
  and intentional: password login did not actually work for the reporting account,
  and the token path is a strict superset of what works. Documented in the spec as
  a REMOVED requirement with migration steps.
- [Users must fetch a token manually] → Mitigated with explicit, copy-paste
  instructions in-app and in the README; it is a one-time step (token persists in
  the keyring).
