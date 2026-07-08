# Design — add-linux-release

## Context

`release.yml` runs a per-OS matrix (macOS dmg, Windows NSIS) that: checks out, syncs the version from the tag into the workspace `Cargo.toml`, installs cargo-packager, builds `--release`, packages with `cargo packager --formats <fmt>`, and uploads via `softprops/action-gh-release` with `fail_on_unmatched_files: true`. `ci.yml` runs clippy/build/test on macOS and Windows only. The packager config lives in `[package.metadata.packager]` (icon: `assets/icon.png`); cargo-packager supports `appimage` and `deb` formats out of the box. The dependency tree includes `native-tls` (needs OpenSSL headers on Linux) and GTK-based file dialogs are documented as a runtime requirement.

## Goals / Non-Goals

**Goals:**
- Ship Linux x64 `.AppImage` and `.deb` on every `v*` tag, attached to the GitHub Release.
- Compile/lint/test on Linux in CI so the shipped platform can't silently break.

**Non-Goals:**
- No ARM Linux (aarch64) builds, no Flatpak/Snap/AUR, no code signing for Linux.
- No changes to app code, packager metadata beyond what Linux packaging strictly requires, or to the macOS/Windows jobs.

## Decisions

1. **One Linux matrix entry building both formats** — `cargo packager --formats appimage,deb` in a single job (`format: appimage,deb`), rather than two jobs: one compile, two package outputs; the existing per-entry `artifacts` glob simply lists both patterns (`*.AppImage`, `*.deb`).
2. **Runner: `ubuntu-22.04`, not `ubuntu-latest`** — AppImage/deb binaries link against the build machine's glibc; building on the oldest supported LTS runner maximizes the range of distros the artifacts run on. CI (`ci.yml`) can use `ubuntu-latest` since those binaries aren't shipped.
3. **System dependencies installed via apt in a Linux-only step** (guarded by `runner.os == 'Linux'` in release.yml so the matrix stays one job template): `libgtk-3-dev` (file dialogs), `libssl-dev`/`pkg-config` (native-tls), `libfuse2` (linuxdeploy AppImage tooling used by cargo-packager). Adjust from the first CI run if the build reports other missing libs (iced/winit/wgpu use pure-Rust windowing but may need `libxkbcommon-dev`/`libwayland-dev` at build time on some setups) — the task list includes a verify-and-iterate step.
4. **CI matrix gains `ubuntu-latest`** with the same apt step (minus `libfuse2`, which is packaging-only). This mirrors the spec's "checks cover shipped platforms" requirement.
5. **Version-sync step unchanged** — it is shell-portable (`sed -E` + `mktemp`) and already runs under `shell: bash` on all matrix OSes.

## Risks / Trade-offs

- [AppImage tooling fails headless on the runner (FUSE/linuxdeploy quirks)] → install `libfuse2`; if linuxdeploy still fails, cargo-packager respects `APPIMAGE_EXTRACT_AND_RUN=1` style workarounds — verify on a test tag before considering done.
- [Missing build-time system library only discovered in CI] → the plan explicitly budgets an iterate-on-CI task; failures are contained to the Linux matrix entry (`fail-fast: false` already set).
- [Older-glibc users still can't run the AppImage] → building on ubuntu-22.04 covers most; documenting a minimum glibc is out of scope.
- [Release job count +1 → longer wall-clock/minutes] → acceptable; jobs run in parallel.

## Open Questions

- None blocking. (aarch64 Linux could be a follow-up change if requested.)
