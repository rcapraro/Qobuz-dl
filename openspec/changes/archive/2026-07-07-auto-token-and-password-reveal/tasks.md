## 1. Investigation

- [x] 1.1 Confirmed via live API tests that `user/login` rejects all email/password variants (GET/POST × plain/md5, with browser headers) for a Qobuz-via-Orange partner account
- [x] 1.2 Confirmed the `user_auth_token` from the web player works (`user/get` returns the profile), so token sign-in is the reliable, universal path

## 2. Core: remove email/password login

- [x] 2.1 Remove `QobuzClient::login` from `client.rs`; `login_with_token` is the sole auth entry point
- [x] 2.2 Remove the now-unused `md5_hex` and `json_id_to_string` helpers and the md5 unit test (keep the `md-5` dependency — request signing still uses it)

## 3. GUI: token-only sign-in

- [x] 3.1 Remove email/password state (`email`, `password`, `reveal_password`) and their messages (`EmailChanged`, `PasswordChanged`, `TogglePasswordVisibility`, `LoginPassword`) and the `login_password` helper
- [x] 3.2 Revert `Message::LoggedIn` to `Result<String, String>`; `login_token` returns the token; the handler stores it in the keyring
- [x] 3.3 Make the account card a single secure token field + Sign in (token) + Sign out
- [x] 3.4 Rewrite the account help panel: sign-in uses a `user_auth_token`, with step-by-step DevTools instructions to obtain it

## 4. Docs

- [x] 4.1 Update `README.md`: features line + a new "Signing in" section with token-retrieval steps; note email/password is unsupported and why

## 5. Verification

- [x] 5.1 `cargo fmt`, `cargo clippy --workspace`, and `cargo test -p qobuz-core` all pass
- [ ] 5.2 Run the GUI: paste the `user_auth_token`, press Sign in, confirm signed-in state and a successful download — left for the user to verify.
