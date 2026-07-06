## 1. CI checks workflow

- [x] 1.1 Create `.github/workflows/ci.yml` triggered on `push` and `pull_request` to `main`
- [x] 1.2 Add a `fmt` job on `ubuntu-latest`: checkout, `dtolnay/rust-toolchain@stable` with `rustfmt`, `cargo fmt --all --check`
- [x] 1.3 Add a `check` job with `strategy.matrix.os: [macos-latest, windows-latest]`: checkout, `dtolnay/rust-toolchain@stable` (+ `clippy`), `Swatinem/rust-cache@v2`
- [x] 1.4 In the `check` job run `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`

## 2. Release workflow — scaffolding

- [x] 2.1 Create `.github/workflows/release.yml` triggered on `push` tags `['v*']`, with `permissions: contents: write`
- [x] 2.2 Add a step (or reusable snippet) to compute `VERSION` from the tag (`v1.2.3` → `1.2.3`) and expose it to later steps
- [x] 2.3 Add a matrix/two jobs: macOS (`macos-latest`) and Windows (`windows-latest`), each with checkout + `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2`

## 3. Release workflow — version sync & build

- [x] 3.1 Before building, rewrite the version from the tag into root `Cargo.toml` (`[workspace.package] version`) and `crates/qobuz-gui/Packager.toml` (`version`) — `sed` on macOS, PowerShell `-replace` on Windows; echo the files to verify
- [x] 3.2 Install `cargo-packager` (`cargo install cargo-packager --locked`; optionally via `cargo-binstall` to save time)
- [x] 3.3 macOS job: from `crates/qobuz-gui/` run `cargo packager --release --formats dmg` (Apple Silicon / aarch64 host build)
- [x] 3.4 Windows job: from `crates/qobuz-gui/` run `cargo packager --release --formats nsis`

## 4. Release workflow — publish

- [x] 4.1 In each platform job, publish with `softprops/action-gh-release@v2` targeting the tag, with `files:` globs (`target/release/**/*.dmg` on macOS, `target/release/**/*setup.exe` on Windows)
- [x] 4.2 Confirm the action creates the Release once and appends assets from both jobs (idempotent per tag)

## 5. Build optimization / reproducibility

- [x] 5.1 Verify `[profile.release]` in root `Cargo.toml` remains `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`, `opt-level = "s"` (no change expected; artifacts must build via `--release`)
- [x] 5.2 (Optional, recommended) Remove `Cargo.lock` from `.gitignore` and commit it for reproducible, cache-friendly CI builds

## 6. Verification

- [x] 6.1 Lint the workflow YAML locally (e.g. `actionlint` if available) or validate structure; ensure both files parse
- [x] 6.2 Push a branch/PR and confirm `ci.yml` runs fmt + macOS/Windows clippy/build/test and reports status — verified green on push to main
- [x] 6.3 Push a test tag (e.g. `v0.1.0`) and confirm `release.yml` builds both platforms, produces the `.dmg` + NSIS `.exe`, and publishes a GitHub Release with both assets attached — verified: run 28824067077 published Qobuz-dl_0.1.0_aarch64.dmg + qobuz-dl_0.1.0_x64-setup.exe
- [ ] 6.4 Download and sanity-check each installer runs on its platform (macOS Apple Silicon `.dmg`, Windows x64 `.exe`) *(manual human check)*
