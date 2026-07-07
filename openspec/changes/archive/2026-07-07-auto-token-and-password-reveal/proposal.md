## Why

> Note: this change evolved during implementation. It began as "auto-discover the
> token via email/password + reveal the password", but live testing showed Qobuz
> email/password login does **not** work for partner/bundled accounts (e.g. Qobuz
> via a telecom), which have no Qobuz-native password. The final, verified outcome
> is **token-only authentication**.

Email/password sign-in cannot be supported reliably: Qobuz's `user/login` rejects
every email/password variant (GET/POST, plain/md5, even with web-player headers)
for partner/bundled accounts, which authenticate through the partner and only ever
yield a `user_auth_token`. The token, by contrast, works for every account type
and is easy to copy from the web player. So sign-in should be **token-only**, with
clear instructions for obtaining the token.

## What Changes

- **Remove** email/password sign-in entirely (UI fields, the reveal toggle, and
  the core `login` method). **BREAKING** for anyone who signed in with a password.
- Make the pasted **`user_auth_token`** the sole sign-in method.
- Document how to obtain the token from the Qobuz web player, both in the
  **account help** panel and in the **README**.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `qobuz-authentication`: remove the "Login with credentials" (email/password)
  requirement; token sign-in becomes the sole authentication method.
- `downloader-gui`: the Settings screen's account section becomes token-only (no
  email/password fields, no reveal toggle) and explains how to get the token.

## Impact

- **qobuz-core** (`client.rs`): remove `login`, `md5_hex`, and `json_id_to_string`.
  `login_with_token` is the sole auth entry point. (`md-5` stays — used by
  request signing.)
- **qobuz-gui** (`app.rs`): remove email/password state, messages
  (`EmailChanged`, `PasswordChanged`, `TogglePasswordVisibility`, `LoginPassword`),
  the `login_password` helper, and the password UI. `LoggedIn` carries just the
  token again. The account card is a single token field + Sign in/Sign out.
- **README.md**: new "Signing in" section with token-retrieval steps.
- No dependency changes.
