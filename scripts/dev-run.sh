#!/usr/bin/env bash
# Build, stable-sign, and run the GUI so the macOS Keychain stops re-prompting.
# Runs the signed binary directly (NOT `cargo run`, which would re-ad-hoc-sign it).
set -euo pipefail

cd "$(dirname "$0")/.."
cargo build -p qobuz-gui
./scripts/dev-sign.sh target/debug/qobuz-dl
exec target/debug/qobuz-dl "$@"
