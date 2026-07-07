#!/usr/bin/env bash
# Sign a locally-built qobuz-dl binary with a STABLE self-signed identity so the
# macOS Keychain trusts it across rebuilds (ad-hoc signatures change every build,
# which is why the Keychain re-prompts and the stored token appears to vanish).
#
# One-time setup — create the self-signed code-signing certificate:
#   Keychain Access → Certificate Assistant → Create a Certificate…
#     Name:            Qobuz-dl Dev
#     Identity Type:   Self Signed Root
#     Certificate Type: Code Signing
#   (leave it in the "login" keychain)
#
# Then run this after each build (or use scripts/dev-run.sh), and click
# "Always Allow" the first time the app reads the token — it now sticks.
set -euo pipefail

IDENTITY="${QOBUZ_DL_SIGN_ID:-Qobuz-dl Dev}"
BIN="${1:-target/debug/qobuz-dl}"

if [ ! -f "$BIN" ]; then
  echo "binary not found: $BIN (build it first, e.g. cargo build -p qobuz-gui)" >&2
  exit 1
fi

# --identifier must match the keyring service in crates/qobuz-core/src/auth.rs
codesign --force --sign "$IDENTITY" --identifier com.qobuzdl.qobuz-dl "$BIN"
echo "Signed $BIN with identity '$IDENTITY' (identifier com.qobuzdl.qobuz-dl)"
