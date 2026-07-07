## Context

`app_id` and `app_secret` are the two Qobuz web-player API credentials the app
needs before any signed call (`track/getFileUrl`) works. They are deliberately
NOT bundled (see CLAUDE.md domain quirks): the user extracts them by hand from
the web player's DevTools and pastes them into Settings. This is the single
biggest onboarding friction point and it silently breaks whenever Qobuz ships a
new web-player release with a rotated `app_secret`.

Established open-source tools (`streamrip`, `qobuz-dl`, `qopy.py`) solve this by
scraping the public web-player assets: the login page references a hashed
`bundle.js`; the `app_id` is a plain string in that bundle, and the
`app_secret` is assembled from base64 fragments keyed by an app "seed" and
timezone-labelled `info`/`extras` segments, then base64-decoded. This change
ports that well-understood technique into `qobuz-core`.

## Goals / Non-Goals

**Goals:**
- Provide a single async core function that returns a freshly discovered
  `app_id` + `app_secret` with no prior credentials required.
- Give the GUI a one-click "Auto-detect" action in Settings that fills the
  fields, with clear success/error feedback.
- Keep manual entry fully working as the fallback.
- Fail loudly and specifically when the bundle format drifts, mirroring the
  existing `Error::InvalidSignature` philosophy.

**Non-Goals:**
- No automatic/periodic re-detection or background refresh — detection runs only
  on explicit user action.
- No bundling of credentials in the binary.
- Not attempting to validate the discovered credentials against a live signed
  call within this change (the user can sign in / download as before to verify).

## Decisions

### Decision: Put extraction in a new `qobuz-core` module
Add `crates/qobuz-core/src/bootstrap.rs` with a public async
`fn discover_app_credentials(http: &reqwest::Client) -> Result<AppCredentials>`
(struct `{ app_id: String, app_secret: String }`), re-exported from `lib.rs`.
Rationale: keeps the GUI free of HTTP/parsing per the core separation-of-concerns
rule; the function takes a borrowed `reqwest::Client` so it reuses the app's
existing client and stays testable. It does not need a `QobuzClient` because no
`app_id`/signing is available yet.

Alternatives considered: a method on `QobuzClient` — rejected because the client
is constructed *from* credentials we don't have yet (chicken-and-egg).

### Decision: Extraction algorithm (mirror streamrip)
1. `GET https://play.qobuz.com/login` → parse HTML for the bundle URL matching
   `/resources/\d+\.\d+\.\d+-[a-z]\d+/bundle\.js`.
2. `GET` that bundle.
3. `app_id`: regex for the production app id, e.g.
   `production:\{api:\{appId:"(\d+)",appSecret:"(\w+)"` (with fallbacks for
   layout drift).
4. `app_secret`: locate the timezone `seed`/`info`/`extras` segments
   (`[a-z]\.initialSeed\("(...)",window\.utimezone\.(...)\)` and the
   `name:"\w+/(...)",info:"(...)",extras:"(...)"` map), concatenate
   `seed + info + extras`, drop the trailing 44 characters, and base64-decode to
   the ASCII secret.
Rationale: this is the canonical, battle-tested shape; keeping the regexes in
one module with a cross-reference comment to `streamrip` matches the existing
`signature.rs` guidance.

### Decision: Error handling
Reuse the central `thiserror` `Error`. Add a dedicated variant (e.g.
`Error::CredentialDiscovery(String)`) so the GUI can show a specific message and
so format drift is diagnosable, consistent with `Error::InvalidSignature`.

### Decision: GUI wiring
Add `Message::AutoDetectCredentials` (button pressed) and
`Message::CredentialsDetected(Result<AppCredentials, String>)` (async result).
The button lives next to the existing credential fields in `settings_view`;
`update` runs `Task::perform(discover..., ...)`, and on success writes the
fields and saves config, on failure sets `self.status`. A transient
"Detecting…" status covers progress. The existing `credentials_help()` block is
retained and reworded as the manual fallback.

## Risks / Trade-offs

- [Qobuz changes the bundle layout, breaking the regexes] → Failure is explicit
  (`CredentialDiscovery` error with context); manual entry still works, and the
  regexes are isolated in one module with a `streamrip` cross-reference for quick
  repair. Same maintenance posture as `signature.rs`.
- [Terms-of-service / scraping concerns] → We only read publicly served static
  assets, on explicit user action, mirroring existing community tooling; no
  change to what the app ultimately does with the credentials.
- [Network dependency at detection time] → Bounded to two GETs on user request;
  errors are surfaced, not fatal to the app.
- [New `regex` dependency] → Small, ubiquitous crate; acceptable. If already
  present transitively we add it as a direct dep of `qobuz-core`.

## Open Questions

- Should a successful detection also immediately persist config, or only fill
  the fields and let the user press Save? (Leaning: fill + auto-save, matching
  how other settings persist.)
