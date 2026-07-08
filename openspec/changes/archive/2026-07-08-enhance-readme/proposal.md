# Enhance README with screenshots and richer structure

## Why

The README is text-only: a visitor cannot see what the app looks like before building it, which undersells a polished GUI application. The document also lacks the standard signals of a mature project (badges, download links to CI-built releases, a quick-start path), making it harder to evaluate and adopt.

## What Changes

- Add **screenshots** of the three app screens (Search, Queue, Settings) to the README, stored in the repository under `docs/screenshots/`.
- Add a **header block**: badges (release version, license, CI status), one-line tagline, and hero screenshot near the top.
- Add an **Installation / Download** section pointing to the prebuilt packages published by the `v*`-tag release workflow (dmg / NSIS / AppImage / deb), so users know they don't have to build from source.
- Reorganize into a clearer reading order: what it is → screenshots → install → sign in → usage → build from source → packaging → configuration → troubleshooting.
- Add a short **Usage** walkthrough (search or paste a URL/ID, pick quality, queue, resulting file layout from templates).
- Keep all existing content (legal note, token how-to, macOS keychain dev note) — reworded or relocated, not removed.

## Capabilities

### New Capabilities

- `project-documentation`: requirements for the README — what sections it must contain, that it shows current screenshots of each screen, and where screenshot assets live.

### Modified Capabilities

<!-- none — no runtime behavior changes -->

## Impact

- `README.md` — rewritten/extended.
- `docs/screenshots/` — new directory with PNG captures (checked into git; sized reasonably for repo weight).
- No code, API, or dependency changes; no impact on the build or release workflows.
