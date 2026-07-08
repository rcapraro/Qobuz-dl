# Tasks — add-linux-release

## 1. Release workflow

- [x] 1.1 Add a Linux matrix entry to `.github/workflows/release.yml`: `os: ubuntu-22.04`, `format: appimage,deb`, artifacts globs for `target/release/**/*.AppImage` and `target/release/**/*.deb`
- [x] 1.2 Add a Linux-only apt step (guarded by `runner.os == 'Linux'`) installing `libgtk-3-dev libssl-dev pkg-config libfuse2` (extend if the build reports more missing libs)

## 2. CI workflow

- [x] 2.1 Add `ubuntu-latest` to the check matrix in `.github/workflows/ci.yml` with the same apt step (minus `libfuse2`)

## 3. Verification

- [ ] 3.1 Validate both workflow files (YAML parse / actionlint if available) and push the change on `main`; confirm the CI Linux job passes
- [ ] 3.2 Push a test tag (or the next release tag) and confirm the GitHub Release carries the `.AppImage` and `.deb` assets alongside dmg and exe; iterate on missing system deps if the Linux job fails
