#!/usr/bin/env bash
# Local play launcher: builds the wasm, serves pkg/, opens a real (non-headless)
# Chrome with WebGPU enabled. Ctrl+C stops everything.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${PLAY_PORT:-8000}"
CHROME_BIN="${CHROME_BIN:-google-chrome-stable}"

"$ROOT/build.sh"

python3 -m http.server "$PORT" -d "$ROOT/pkg" >/dev/null 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null' EXIT INT TERM
sleep 1

echo "==> Playing Starfall at http://localhost:$PORT (Ctrl+C to stop)"
"$CHROME_BIN" \
  --no-sandbox --disable-dev-shm-usage \
  --enable-unsafe-webgpu --use-angle=swiftshader \
  --enable-features=Vulkan --use-vulkan=swiftshader \
  "http://localhost:$PORT/index.html" 2>/dev/null &
wait $SERVER_PID