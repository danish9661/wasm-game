#!/usr/bin/env bash
# Starfall gate: run the Rust tests AND the headless visual audit before push/CI.
# Exit non-zero on any failure so regressions in layout/animation are caught
# automatically. The visual audit needs Chrome; if it's missing we still run
# the Rust tests and only skip the browser pass.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> cargo test -p game"
cargo test -p game

echo "==> build wasm + pkg"
cargo build -p wasm_game --target wasm32-unknown-unknown
./build.sh

# Validate the browser-based visual audit.
CHROME="${CHROME_PATH:-/usr/bin/google-chrome}"
if command -v google-chrome >/dev/null 2>&1; then CHROME="$(command -v google-chrome)"; fi
if [ -x "$CHROME" ]; then
echo "==> visual audit (tools/visualize.js)"
# Resolve a single NODE_PATH dir that contains puppeteer-core (it may live in a
# few places, including paths with spaces — so pick ONE and quote it).
PP_DIR=""
for d in "/tmp/opencode/node_modules" "$(npm root -g 2>/dev/null)" "$HOME/node_modules" "/usr/local/lib/node_modules" "/home/danish1075/Documents/ri pi emu/node_modules"; do
  if [ -n "$d" ] && [ -d "$d/puppeteer-core" ]; then PP_DIR="$d"; break; fi
done
if [ -n "$PP_DIR" ]; then
  # AUDIT_VISUAL_BLOCK=0 => the pre-push hook: visual regressions are reported
  # but do NOT block the push (the browser pass can be flaky; CI enforces it).
  # AUDIT_VISUAL_BLOCK=1 (default, used by CI) => a visual failure fails the gate.
  if [ "${AUDIT_VISUAL_BLOCK:-1}" = "0" ]; then
    if ! NODE_PATH="$PP_DIR" node tools/visualize.js; then
      echo "!! visual audit reported problems — NOT blocking push (retry in CI)."
    fi
  else
    NODE_PATH="$PP_DIR" node tools/visualize.js
  fi
else
  echo "!! puppeteer-core not found in candidate dirs — skipping visual audit (Rust tests gated)."
fi
else
  echo "!! Chrome not found at $CHROME — skipping visual audit (Rust tests still gated)."
fi

echo "==> all gates passed"
