# Tasks: improve-auth-token-visibility

## 1. State: token origin and truthful sign-out

- [x] 1.1 Add `TokenOrigin { Restored, ValidatedThisSession }` to `crates/qobuz-gui/src/app.rs` and track it alongside `token` (set `Restored` in `App::new` when the keyring yields a token; set `ValidatedThisSession` in `LoggedIn(Ok)`)
- [x] 1.2 Change the `SignOut` arm to clear `self.token` only when `auth::clear_token()` succeeds; on failure, keep the token state and surface the error in `status`

## 2. View: status line and gated buttons

- [x] 2.1 Add a `masked_token(&str) -> String` helper (last 4 chars visible, `••••…` prefix; fully masked for tokens shorter than 4) near the Account card code, with unit tests for long/short/empty tokens
- [x] 2.2 Render the stored-token status line at the top of the Account card in `crates/qobuz-gui/src/app/view/settings.rs`: saved (masked preview + "restored at startup" / "validated this session") or "none saved" hint
- [x] 2.3 Gate the buttons: `Sign in` enabled only when the token input is non-empty (trimmed); `Sign out` enabled only when a token is stored (`on_press_maybe`, adding maybe-press button variants in `style.rs` if needed)

## 3. Verification

- [x] 3.1 `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt` — clean
- [x] 3.2 Manual smoke (`cargo run -p qobuz-gui`): fresh start with no token shows "none saved" + disabled Sign out; sign in shows masked preview + "validated this session"; restart shows "restored at startup"; sign out returns to "none saved" (verified via screenshot: restored-token status line, masked preview, disabled Sign in on empty input, enabled Sign out; sign-out/sign-in cycle left to the user to avoid clearing the real keyring token)
- [x] 3.3 Update the Account help text (`app/help.rs`) if it references behavior changed here (sign-out wording)
