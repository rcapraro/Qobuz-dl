## REMOVED Requirements

### Requirement: Login with credentials
**Reason**: Qobuz's `user/login` rejects email/password authentication for
partner/bundled accounts (e.g. Qobuz via a telecom), which have no Qobuz-native
password. Supporting a path that silently fails for a large class of accounts is
worse than not offering it.
**Migration**: Sign in with your `user_auth_token` (the "Login with raw token"
path). Copy the `x-user-auth-token` value from the Qobuz web player's DevTools →
Network tab; see the account help panel and the README for step-by-step
instructions.
