## Context

Grounding from the repo:

- **Workspace** (`Cargo.toml`): resolver 2, members `crates/qobuz-core` (lib) + `crates/qobuz-gui` (binary `qobuz-dl`), `[workspace.package] version = "0.1.0"` inherited via `version.workspace = true`. Release profile already optimized: `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`, `opt-level = "s"`.
- **Packaging** (`crates/qobuz-gui/Packager.toml`): flat cargo-packager schema — `product-name = "Qobuz-dl"`, `identifier = "com.qobuzdl.qobuz-dl"`, `binaries = [{ path = "qobuz-dl", main = true }]`, `formats = ["dmg", "nsis", "deb", "appimage"]`, `[macos] minimum-system-version = "11.0"`. Hardcodes its own `version = "0.1.0"` — **not linked** to the workspace version. No icons/resources. `cargo packager --release` (run from `crates/qobuz-gui/`) builds the binary and bundles it; bundles land under the workspace `target/release/`.
- **Dependency constraints**: `reqwest` uses default **native-tls** (SChannel on Windows, Secure Transport on macOS); `keyring` uses `apple-native`/`windows-native`; `iced` (wgpu) and `rfd` use OS-native backends. → **build on native runners**, cross-compilation is impractical.
- **No `.github/` yet.** No git tags. `Cargo.lock` is gitignored.

User decisions for this change: macOS = **Apple Silicon (aarch64) only**; release trigger = **`v*` tag**; **also add CI checks**.

## Goals / Non-Goals

**Goals:**
- CI checks (fmt/clippy/build/test) on push/PR to `main`, on the shipped platforms.
- Tag-driven release producing a macOS aarch64 `.dmg` and a Windows x64 NSIS `.exe`, published to a GitHub Release.
- Version taken from the tag; artifacts reflect it.
- Ship the already-optimized release profile.

**Non-Goals:**
- Linux packaging (deb/appimage) — out of scope though still declared in `Packager.toml`.
- macOS universal/Intel builds; code signing / notarization; auto-versioning of `Cargo.toml` commits; changelog generation.

## Decisions

### 1. Two workflows: `ci.yml` and `release.yml`
Separate concerns: fast per-PR checks vs. heavier tag-driven packaging.

- **`ci.yml`** — `on: push` (branches: main) and `on: pull_request` (branches: main).
  - `fmt` job on `ubuntu-latest`: `cargo fmt --all --check` (formatting is platform-independent and Ubuntu is cheapest; no system libs needed for fmt).
  - `check` job with `strategy.matrix.os: [macos-latest, windows-latest]`: `dtolnay/rust-toolchain@stable` (+ clippy), `Swatinem/rust-cache@v2`, then `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`.
  - *Why macOS+Windows and not Ubuntu for compile/test:* the app targets only those two; building on Linux would require `libdbus-1-dev`/`libgtk-3-dev`/Vulkan/X11 dev packages (keyring/rfd/iced). Testing the real shipped platforms is both simpler and more relevant.

- **`release.yml`** — `on: push: tags: ['v*']`; `permissions: contents: write` (needed to create the Release).

*Alternative considered:* a single workflow with conditional jobs — rejected; two files are clearer and let CI stay lightweight.

### 2. Release jobs = per-platform matrix on native runners
Two jobs (or a matrix) producing one installer each:

- **macOS** (`macos-latest`, which is Apple Silicon → `aarch64-apple-darwin` host build): `cargo packager --release --formats dmg` from `crates/qobuz-gui/`.
- **Windows** (`windows-latest`, x64 host): `cargo packager --release --formats nsis` from `crates/qobuz-gui/`.

Each installs the toolchain, `Swatinem/rust-cache@v2`, and `cargo-packager` (`cargo install cargo-packager --locked`, cached). Explicit `--formats` avoids cargo-packager attempting host-incompatible bundles (deb/appimage) declared in `Packager.toml`.

### 3. Version from tag, applied before build
Compute `VERSION` = tag without leading `v` (e.g. `v1.2.3` → `1.2.3`). Before building, rewrite the version in both `Cargo.toml` (`[workspace.package] version`) and `crates/qobuz-gui/Packager.toml` (`version`) so artifacts match the tag. Use platform-appropriate in-place edits:
- macOS/Linux shell: `sed -i '' -E 's/^version = ".*"/version = "'$VERSION'"/'` scoped to the right line.
- Windows: a PowerShell `(Get-Content ...) -replace ... | Set-Content` step.

Because these edits are ephemeral (in the runner only, not committed), the two-source-of-truth problem is resolved per-build without churn. *Alternative:* pass `cargo packager --version` — rejected as less reliable across cargo-packager versions and it doesn't fix the workspace `Cargo.toml` used for the compile.

### 4. Publish with `softprops/action-gh-release@v2`
Each platform job, after packaging, uploads its installer to the same tag's Release via `softprops/action-gh-release@v2` with a `files:` glob. The action is idempotent per tag (creates the Release once, appends assets), so both jobs can target it without a separate aggregation job. Globs (robust to exact filenames):
- macOS: `target/release/**/*.dmg`
- Windows: `target/release/**/*setup.exe` (NSIS installer; excludes the raw `qobuz-dl.exe`).

*Alternative:* `actions/upload-artifact` + a final release job — more moving parts; use only if asset-name collisions appear.

### 5. Optimization = ship the existing release profile (+ optional lockfile)
The `[profile.release]` is already tuned for small, stripped, LTO'd binaries; the deliverable is ensuring CI builds `--release` (it does, via `cargo packager --release`). Optionally **commit `Cargo.lock`** (remove it from `.gitignore`) for reproducible, cache-friendlier CI builds — recommended for a distributed application. No source/profile changes are required to satisfy the optimization goal.

## Risks / Trade-offs

- **cargo-packager artifact filenames vary by version** → use globs, not hardcoded names, when collecting assets.
- **Version edit via sed/PowerShell is brittle if `Cargo.toml` gains other `version = ` lines** → scope the replacement to the `[workspace.package]` block / the `Packager.toml` top-level `version` key, and verify by echoing the file after edit.
- **`cargo install cargo-packager` is slow (~minutes)** → cache with `Swatinem/rust-cache` and/or `cargo-bins/cargo-binstall` for a prebuilt binary to cut minutes.
- **Unsigned macOS `.dmg`** → Gatekeeper will warn; documented as a known limitation. Signing/notarization is a future change requiring secrets.
- **`Cargo.lock` gitignored** → without committing it, CI resolves fresh deps each run (slower, less reproducible); mitigated by the optional lockfile task.
- **Apple Silicon-only** → Intel Macs need Rosetta or won't run; explicitly chosen by the user, revisit with a universal build later if needed.

## Migration Plan

Additive only. Merge the workflows; the first `v*` tag push exercises the release path. Rollback = delete the workflow files. No runtime or dependency impact.

## Open Questions

- Whether to commit `Cargo.lock` now (recommended) or leave gitignored — included as an optional task; default to committing it.
- Whether to speed up cargo-packager install with `cargo-binstall` — optional optimization, not required for correctness.
