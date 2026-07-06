## ADDED Requirements

### Requirement: Continuous integration checks

The repository SHALL provide a GitHub Actions workflow that runs automated checks on every push and pull request targeting the `main` branch, covering formatting, linting, compilation, and tests on the platforms the app ships to.

#### Scenario: Checks run on pull request

- **WHEN** a pull request targeting `main` is opened or updated
- **THEN** the workflow runs `cargo fmt --check`, `cargo clippy`, `cargo build`, and `cargo test`, and reports pass/fail status on the pull request

#### Scenario: Checks cover shipped platforms

- **WHEN** the CI checks run
- **THEN** the build/clippy/test checks execute on both macOS and Windows runners

#### Scenario: Failing check blocks green status

- **WHEN** clippy reports a lint error or a test fails
- **THEN** the workflow run fails (non-success status)

### Requirement: Tag-triggered release build

The repository SHALL provide a GitHub Actions workflow that, when a version tag matching `v*` is pushed, builds optimized release installers for Windows x64 and macOS (Apple Silicon).

#### Scenario: Release triggered by version tag

- **WHEN** a git tag matching `v*` (e.g. `v0.1.0`) is pushed
- **THEN** the release workflow runs and builds the app in release mode on a macOS runner and a Windows runner

#### Scenario: Platform installers produced

- **WHEN** the release workflow builds the app
- **THEN** it produces a macOS Apple Silicon `.dmg` and a Windows x64 NSIS `.exe` installer via cargo-packager

#### Scenario: Not triggered by ordinary pushes

- **WHEN** a commit is pushed to a branch without a `v*` tag
- **THEN** the release workflow does not run

### Requirement: Version derived from tag

The release workflow SHALL use the pushed tag as the source of truth for the release version and apply it to the packaged artifacts, keeping the workspace and packaging versions consistent.

#### Scenario: Artifact version matches tag

- **WHEN** the tag `v1.2.3` is pushed
- **THEN** the packaged artifacts are built and named with version `1.2.3` (the workspace `Cargo.toml` and `Packager.toml` versions used for the build reflect `1.2.3`)

### Requirement: Published GitHub Release

The release workflow SHALL publish a GitHub Release for the pushed tag and attach the Windows and macOS installer artifacts to it.

#### Scenario: Release created with assets

- **WHEN** the release workflow finishes building both platforms
- **THEN** a GitHub Release exists for the tag with the macOS `.dmg` and the Windows `.exe` attached as downloadable assets

### Requirement: Optimized release artifacts

Released binaries SHALL be built with the project's optimized release profile (link-time optimization, single codegen unit, symbol stripping, abort-on-panic, size optimization).

#### Scenario: Release profile used

- **WHEN** the release workflow compiles the app
- **THEN** it builds in `--release` mode so the `[profile.release]` settings (`lto`, `codegen-units=1`, `strip`, `panic="abort"`, `opt-level="s"`) apply to the shipped binaries
