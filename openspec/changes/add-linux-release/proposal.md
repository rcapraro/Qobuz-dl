# Add Linux release builds

## Why

The README (since v1.0.0) advertises Linux `.AppImage`/`.deb` downloads and the packaging docs claim cargo-packager produces them, but the release workflow only builds macOS (dmg) and Windows (NSIS) — Linux users currently have no prebuilt artifact and CI never compiles the app on Linux.

## What Changes

- Add a **Linux x64 job** to the release workflow matrix (`.github/workflows/release.yml`) that builds the app on an Ubuntu runner and packages **AppImage + deb** via cargo-packager, attaching both to the GitHub Release.
- Install the required Linux system dependencies in the workflow (GTK for file dialogs, OpenSSL headers for native-tls, FUSE for AppImage tooling).
- Add **Linux to the CI check matrix** (`.github/workflows/ci.yml`) so clippy/build/test run on the newly shipped platform, with the same system dependencies.
- No application code changes expected (the code already targets Linux — keyring via Secret Service, GTK dialogs are documented).

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `release-automation`: the CI-platform-coverage, tag-triggered-release-build, and published-release requirements gain Linux x64 (AppImage + deb) alongside macOS and Windows.

## Impact

- `.github/workflows/release.yml` — new matrix entry + Linux dependency install step.
- `.github/workflows/ci.yml` — `ubuntu-latest` added to the check matrix + dependency install.
- GitHub Releases gain two Linux assets per tag; release minutes increase by one job.
- `openspec/specs/release-automation/spec.md` — updated requirements.
