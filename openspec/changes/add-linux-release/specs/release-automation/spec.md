# release-automation — delta

## MODIFIED Requirements

### Requirement: Continuous integration checks

The repository SHALL provide a GitHub Actions workflow that runs automated checks on every push and pull request targeting the `main` branch, covering formatting, linting, compilation, and tests on the platforms the app ships to.

#### Scenario: Checks run on pull request

- **WHEN** a pull request targeting `main` is opened or updated
- **THEN** the workflow runs `cargo fmt --check`, `cargo clippy`, `cargo build`, and `cargo test`, and reports pass/fail status on the pull request

#### Scenario: Checks cover shipped platforms

- **WHEN** the CI checks run
- **THEN** the build/clippy/test checks execute on macOS, Windows, and Linux runners

#### Scenario: Failing check blocks green status

- **WHEN** clippy reports a lint error or a test fails
- **THEN** the workflow run fails (non-success status)

### Requirement: Tag-triggered release build

The repository SHALL provide a GitHub Actions workflow that, when a version tag matching `v*` is pushed, builds optimized release installers for Windows x64, macOS (Apple Silicon), and Linux x64.

#### Scenario: Release triggered by version tag

- **WHEN** a git tag matching `v*` (e.g. `v0.1.0`) is pushed
- **THEN** the release workflow runs and builds the app in release mode on a macOS runner, a Windows runner, and a Linux runner

#### Scenario: Platform installers produced

- **WHEN** the release workflow builds the app
- **THEN** it produces a macOS Apple Silicon `.dmg`, a Windows x64 NSIS `.exe` installer, and Linux x64 `.AppImage` and `.deb` packages via cargo-packager

#### Scenario: Not triggered by ordinary pushes

- **WHEN** a commit is pushed to a branch without a `v*` tag
- **THEN** the release workflow does not run

### Requirement: Published GitHub Release

The release workflow SHALL publish a GitHub Release for the pushed tag and attach the Windows, macOS, and Linux installer artifacts to it.

#### Scenario: Release created with assets

- **WHEN** the release workflow finishes building all platforms
- **THEN** a GitHub Release exists for the tag with the macOS `.dmg`, the Windows `.exe`, and the Linux `.AppImage` and `.deb` attached as downloadable assets
