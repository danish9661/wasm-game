#!/usr/bin/env bash
# Launch Starfall for local play (and visual review).
# Builds wasm, serves pkg/, opens a real Chrome with WebGPU enabled and a
# remote-debugging port so automated screenshots can be taken.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
PORT="${PLAY_PORT:-8000}"
DEBUG_PORT="${PLAY_DEBUG_PORT:-9222}"
CHROME_BIN="${CHROME_BIN:-google-chrome-stable}"

"$ROOT/build.sh"

# serve from pkg/ (build.sh copies index.html + style.css there)
python3 -m http.server "$PORT" -d "$ROOT/pkg" >/dev/null 2>&1 &
SERVER_PID=$!

cleanup() {
  kill "$CHROME_PID" 2>/dev/null
  kill "$SERVER_PID" 2>/dev/null
}
trap cleanup EXIT INT TERM
sleep 1

echo "==> Starfall at http://localhost:$PORT (Ctrl+C to stop)"
echo "==> DevTools WebSocket: ws://127.0.0.1:$DEBUG_PORT (for visual review)"

"$CHROME_BIN" \
  --remote-debugging-port="$DEBUG_PORT" \
  --remote-allow-origins=* \
  --no-sandbox --disable-dev-shm-usage \
  --no-first-run --disable-background-networking \
  --enable-unsafe-webgpu --use-angle=swiftshader \
  --enable-features=Vulkan --use-vulkan=swiftshader \
  --disable-gpu-sandbox \
  ${EXTRA_FLAGS:-} \
  --user-data-dir="$HOME/.starfall-chrome-profile" \
  --disk-cache-dir="$HOME/.starfall-chrome-cache" \
  "http://localhost:$PORT/index.html" 2>/dev/null &
CHROME_PID=$!
wait $CHROME_PID
