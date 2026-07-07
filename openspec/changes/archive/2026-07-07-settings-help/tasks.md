## 1. State and messages

- [x] 1.1 Add `show_credentials_help`, `show_account_help`, `show_options_help` bool fields to `App` (default `false`) and initialize them in `App::new`
- [x] 1.2 Add `Message::ToggleCredentialsHelp`, `Message::ToggleAccountHelp`, `Message::ToggleOptionsHelp`
- [x] 1.3 Handle the three new messages in `update` by flipping their respective flags

## 2. Help panel builders

- [x] 2.1 Add `credentials_help()` — explains `app_id` (`x-app-id` header) and `app_secret` (request signing), plus ordered steps to extract both from the Qobuz web player via browser dev tools; use `style::mono` for literal tokens and `TEXT_SM` body text
- [x] 2.2 Add `account_help()` — explains email/password vs. pasted `user_auth_token`, that the token is stored in the OS keyring and excluded from saved config, and the signed-in indicator
- [x] 2.3 Add `options_help()` — explains the four quality tiers and possible downgrade, the concurrency range (1–16), and the embed-cover-art toggle

## 3. Wire toggles into the Settings cards

- [x] 3.1 In `settings_view`, append a "Show help"/"Hide help" `secondary_button` to the API credentials card body and include `credentials_help()` when its flag is set
- [x] 3.2 Do the same for the Account card (`account_help()`)
- [x] 3.3 Do the same for the Options card (`options_help()`)

## 4. Quality gates

- [x] 4.1 `cargo fmt`
- [x] 4.2 `cargo clippy --workspace` (no warnings)
- [x] 4.3 `cargo build --release -p qobuz-gui`

## 5. Verification

- [x] 5.1 Smoke-launched the release binary: Settings tab renders with the three help toggles, help hidden by default, no panic
- [ ] 5.2 Toggling one card's help reveals its panel and leaves the other cards' help hidden; toggling again hides it *(needs hands-on interaction)*
- [ ] 5.3 Confirm the credentials help lists the extraction steps and the account/options help read correctly in both light and dark themes *(needs hands-on interaction)*
- [x] 5.4 `openspec validate settings-help --strict`
