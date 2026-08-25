#!/usr/bin/env bash
# Launch Starfall with WebGPU + Vulkan enabled (fast backend) and a remote
# debugging port. By default it builds the wasm, serves pkg/ locally and opens
# http://localhost:<PORT>/index.html. Pass a URL to open that instead, e.g.
#
#   ./launch.sh                                   # local build + localhost
#   ./launch.sh https://danish9661.github.io/wasm-game/   # github (Vulkan)
#
# The key flag is --enable-features=Vulkan: without it, Chrome's WebGPU uses
# the default Linux backend, which can't present a canvas and falls back to a
# slow GPU->CPU readback (this is why the plain github page was ~40fps while
# this script gave ~80fps).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
PORT="${PLAY_PORT:-8000}"
DEBUG_PORT="${PLAY_DEBUG_PORT:-9222}"
CHROME_BIN="${CHROME_BIN:-google-chrome-stable}"
CHROME_CACHE="$(mktemp -d)"

URL="${1:-}"

# Only build + serve locally when we're not given an explicit URL to open.
if [ -z "$URL" ]; then
    "$ROOT/build.sh"
    python3 -m http.server "$PORT" -d "$ROOT/pkg" >/dev/null 2>&1 &
    SERVER_PID=$!
    URL="http://localhost:$PORT/index.html"
fi

cleanup() {
    kill "${CHROME_PID:-}" 2>/dev/null
    kill "${SERVER_PID:-}" 2>/dev/null
    rm -rf "$CHROME_CACHE" 2>/dev/null
}
trap cleanup EXIT INT TERM
sleep 1

echo "==> Starfall: $URL"
echo "==> DevTools WebSocket: ws://127.0.0.1:$DEBUG_PORT (for visual review)"

"$CHROME_BIN" \
    --remote-debugging-port="$DEBUG_PORT" \
    --remote-allow-origins=* \
    --no-sandbox --disable-dev-shm-usage \
    --no-first-run --disable-background-networking \
    --enable-unsafe-webgpu \
    --enable-features=Vulkan \
    --disable-gpu-sandbox \
    ${EXTRA_FLAGS:-} \
    --user-data-dir="$HOME/.starfall-chrome-profile" \
    --disk-cache-dir="$CHROME_CACHE" \
    "$URL" 2>/dev/null &
CHROME_PID=$!
wait $CHROME_PID
