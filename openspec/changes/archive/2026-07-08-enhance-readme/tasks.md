# Tasks — enhance-readme

## 1. Screenshots

- [x] 1.1 Build and run the app (release profile) signed in, with realistic content: a search with album results, a queue with completed and in-progress items, filled-in settings
- [x] 1.2 Capture `search.png`, `queue.png`, `settings.png` at the same window size, default dark theme, into `docs/screenshots/`
- [x] 1.3 Verify the Settings capture exposes no `app_id`/`app_secret`/token value (empty, masked, or redact before commit)
- [x] 1.4 Losslessly optimize the PNGs (target well under ~500 KB each)

## 2. README rewrite

- [x] 2.1 Add header block: title, tagline, release + license badges (shields.io), legal note, hero image (`docs/screenshots/search.png`)
- [x] 2.2 Add a Screenshots section embedding all three captures with captions
- [x] 2.3 Add an Installation section linking GitHub Releases, naming per-OS artifacts (dmg / NSIS exe / AppImage / deb) and Linux runtime notes (Secret Service, GTK)
- [x] 2.4 Add a Usage walkthrough: sign in → search or paste URL/ID → choose quality → download location/templates
- [x] 2.5 Reorganize remaining sections per design order (Signing in, Build from source, Packaging, Configuration, macOS dev-keychain note), folding Prerequisites into Installation/Build-from-source without dropping any existing content

## 3. Verification

- [x] 3.1 Preview the rendered README (GitHub-flavored markdown) and check all image links, badges, and anchors resolve
- [x] 3.2 Cross-check against the spec scenarios: hero before build instructions, three screenshots present, install path before build-from-source, all prior content preserved
