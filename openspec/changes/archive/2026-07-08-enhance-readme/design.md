# Design — enhance-readme

## Context

`README.md` is accurate but text-only and organized around building from source. The app now has three polished screens (Search, Queue, Settings — Catppuccin-themed iced UI) and CI-built installers on every `v*` tag, none of which is visible from the README. There is no `docs/` directory yet; the only images in the repo are the app icon under `crates/qobuz-gui/assets/`.

## Goals / Non-Goals

**Goals:**
- Show the app visually (one screenshot per screen) before any build instructions.
- Give non-Rust users a download path (GitHub Releases artifacts).
- Improve scanability: badges, tagline, logical section order, short usage walkthrough.

**Non-Goals:**
- No code or UI changes; no changes to the release workflow.
- No automated screenshot generation/CI screenshot refresh — captures are manual.
- No multi-language README, no separate docs site.

## Decisions

1. **Screenshot storage: `docs/screenshots/*.png`** — a conventional, code-free location; keeps `crates/qobuz-gui/assets/` reserved for assets bundled into the binary. Alternative considered: `.github/assets/` (works, but less discoverable) and external image hosting (rejected: link rot, not versioned with the UI).
2. **Three captures, fixed names**: `search.png`, `queue.png`, `settings.png`; the hero image at the top of the README reuses `search.png` (the most representative screen — album grid with artwork). Referenced with relative paths so they render on GitHub and in local clones.
3. **Consistent capture setup**: same window size for all three, one theme (the app's default dark Catppuccin variant), realistic content (a search with results, a queue with completed/in-progress items, settings filled in). Captured on macOS; PNGs run through lossless optimization to keep the repo light (target well under ~500 KB each).
4. **Redaction rule**: the Settings capture must not expose secrets — `app_id`/`app_secret`/token fields must be empty, masked by the UI, or blurred before commit. This is a hard requirement in the spec.
5. **Badges: release version + license only** (shields.io, static/GitHub-backed). A CI "build passing" badge is deliberately omitted: the only workflow runs on tags, so a branch badge would be meaningless or stale.
6. **Section order**: title + badges + tagline → legal note → hero screenshot → Features → Screenshots → Installation (Releases downloads, per-OS notes incl. Linux Secret Service/GTK) → Signing in → Usage → Build from source → Packaging → Configuration → macOS dev-keychain note. All current content is preserved; the Prerequisites section folds into Installation/Build-from-source.

## Risks / Trade-offs

- [Screenshots go stale as the UI evolves] → keep only three canonical captures with fixed filenames so refreshing is a drop-in replacement; the spec requires they reflect the current UI, making staleness an auditable spec violation.
- [Repo weight growth from binary assets] → three optimized PNGs, replaced in place (history growth bounded by refresh frequency).
- [Screenshots require a signed-in account with real content] → capture locally with the maintainer's account; redaction rule (Decision 4) prevents secret leakage.

## Open Questions

- None blocking. (If a light-theme variant is ever wanted, it can be a follow-up; GitHub `picture`/`prefers-color-scheme` markup is compatible with the chosen layout.)
