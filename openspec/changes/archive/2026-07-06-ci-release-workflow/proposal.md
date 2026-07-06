## Why

The project has no CI and no automated releases — building and distributing the app is entirely manual (`cargo packager` on each developer's machine). Adding GitHub Actions gives repeatable, contributor-independent builds, catches regressions on every change, and produces ready-to-install Windows and macOS artifacts on demand. The release build is already size-optimized in `Cargo.toml`; CI ensures those optimized, stripped binaries are what ships.

## What Changes

- Add a **CI checks workflow** (`.github/workflows/ci.yml`) that runs on push/PR to `main`: `cargo fmt --check`, plus `cargo clippy`, `cargo build`, and `cargo test` on the two shipped platforms (macOS + Windows), so regressions are caught before a release.
- Add a **release workflow** (`.github/workflows/release.yml`) triggered by pushing a `v*` git tag: it builds the app in `--release` mode on native runners and produces installers via `cargo-packager` — a **macOS (Apple Silicon / aarch64) `.dmg`** and a **Windows x64 NSIS `.exe`** — then publishes them to a GitHub Release for the tag.
- **Derive the release version from the git tag** and apply it to the packaged artifacts (syncing the workspace `Cargo.toml` and `Packager.toml` versions, which are currently unlinked), so artifact names/versions match the tag.
- Confirm/keep the **optimized release profile** (`lto`, `codegen-units=1`, `strip`, `panic="abort"`, `opt-level="s"`) as the basis for released binaries; optionally commit `Cargo.lock` for reproducible CI builds.

## Capabilities

### New Capabilities
- `release-automation`: GitHub Actions CI checks on push/PR and tag-triggered release builds that produce and publish optimized Windows x64 and macOS (Apple Silicon) installers.

### Modified Capabilities
<!-- None. openspec/specs/ contains only `gui-theming`; this adds a new, self-contained build/release-automation capability. No application behavior changes. -->

## Impact

- **New files:** `.github/workflows/ci.yml`, `.github/workflows/release.yml`. Optionally un-ignore + commit `Cargo.lock` (edit `.gitignore`).
- **Possibly touched:** `crates/qobuz-gui/Packager.toml` and root `Cargo.toml` version handling (updated at release time from the tag, not committed per-release).
- **Build targets:** native runners only (`macos-latest` for aarch64, `windows-latest` for x64) — required because `reqwest` uses native-tls and `keyring`/`iced`/`rfd` use OS-native backends, making cross-compilation impractical. Linux packaging (deb/appimage in `Packager.toml`) is out of scope for this change.
- **No code changes** to `qobuz-core`/`qobuz-gui` runtime; no new crate dependencies (uses GitHub-hosted actions + `cargo-packager`).
- **Distribution note:** artifacts are unsigned/unnotarized (no code-signing certificates configured); macOS users may need to bypass Gatekeeper. Signing is out of scope.
