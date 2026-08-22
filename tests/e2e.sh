#!/usr/bin/env bash
# E2E browser test: RAM-capped headless Chrome, clean exit guaranteed.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${E2E_PORT:-8765}"
DEBUG_PORT="${E2E_DEBUG_PORT:-9333}"
MAX_RSS_MB="${E2E_MAX_RSS_MB:-1500}"
TIMEOUT_S="${E2E_TIMEOUT_S:-220}"
CHROME_BIN="${CHROME_BIN:-google-chrome-stable}"
SERVER_PID=""
CHROME_PID=""
PASS=0
FAIL=0

cleanup() {
  # always kill chrome first (the RAM hog), then the server.
  # match by unique user-data-dir + debug port so we never touch a
  # user's real Chrome, and never leave orphans behind
  pkill -f "user-data-dir=/tmp/e2e-chrome-$DEBUG_PORT" 2>/dev/null
  pkill -f "remote-debugging-port=$DEBUG_PORT" 2>/dev/null
  if [[ -n "$CHROME_PID" ]] && kill -0 "$CHROME_PID" 2>/dev/null; then
    kill "$CHROME_PID" 2>/dev/null
    wait "$CHROME_PID" 2>/dev/null
  fi
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null
    wait "$SERVER_PID" 2>/dev/null
  fi
  # hard-kill anything that somehow survived
  pkill -f "http.server $PORT" 2>/dev/null
  rm -rf "/tmp/e2e-chrome-$DEBUG_PORT" 2>/dev/null
}
trap cleanup EXIT INT TERM

step() { echo -e "\n\033[1;36m== $1 ==\033[0m"; }
ok()   { echo -e "\033[1;32mPASS\033[0m $1"; PASS=$((PASS+1)); }
bad()  { echo -e "\033[1;31mFAIL\033[0m $1"; FAIL=$((FAIL+1)); }

step "1/4 cargo tests (no browser needed)"
if cargo test -p game --manifest-path "$ROOT/Cargo.toml" --quiet 2>/dev/null; then
  ok "cargo test -p game"
else
  bad "cargo test -p game"
  exit 1
fi

step "2/4 build wasm"
if "$ROOT/build.sh" >/dev/null 2>&1; then
  ok "wasm build"
else
  bad "wasm build"
  exit 1
fi

step "3/4 serve + launch RAM-capped Chrome"
python3 -m http.server "$PORT" -d "$ROOT/pkg" >/dev/null 2>&1 &
SERVER_PID=$!
sleep 1

"$CHROME_BIN" --headless=new \
  --no-sandbox --disable-dev-shm-usage \
  --disable-extensions --no-first-run --disable-background-networking \
  --js-flags="--max-old-space-size=256" \
  --enable-unsafe-webgpu --use-angle=swiftshader \
  --enable-features=Vulkan --use-vulkan=swiftshader \
  --remote-debugging-port="$DEBUG_PORT" \
  --user-data-dir="/tmp/e2e-chrome-$DEBUG_PORT" \
  --window-size=1280,720 \
  "http://localhost:$PORT/index.html" >/dev/null 2>&1 &
CHROME_PID=$!
sleep 2

step "4/4 CDP assertions (RAM watchdog: ${MAX_RSS_MB}MB cap, ${TIMEOUT_S}s)"
E2E_CHROME_PID="$CHROME_PID" E2E_MAX_RSS_MB="$MAX_RSS_MB" \
  node "$ROOT/tests/assert.js" "$DEBUG_PORT" "$TIMEOUT_S"
RESULT=$?
cleanup

echo
echo -e "======================================"
echo -e "  E2E: \033[1;32m$PASS passed\033[0m, \033[1;31m$FAIL failed\033[0m (chrome rss monitored)"
echo -e "======================================"
exit $RESULT