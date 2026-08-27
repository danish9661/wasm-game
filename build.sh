#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "==> Building Rust -> WASM (wasm-pack, web target)"
wasm-pack build web --target web --out-dir ../pkg --release

echo "==> Copying static shell into pkg/"
cp web/static/index.html web/static/style.css pkg/
cp web/static/robots.txt web/static/sitemap.xml pkg/
cp documentation.html pkg/ 2>/dev/null || true
rm -rf pkg/element_previews
cp -r element_previews pkg/ 2>/dev/null || true

# Cache-bust: stamp the JS + WASM URLs with the commit SHA so a fresh GitHub
# Pages deploy is never served from a stale browser/CDN cache. The base-named
# files still exist, so any cached older index.html gracefully loads the new
# module instead of 404-ing.
SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "dev")
sed -i -E "s#(./wasm_game\.js)#\1?v=$SHA#g" pkg/index.html
sed -i -E "s#wasm_game_bg\.wasm#wasm_game_bg.wasm?v=$SHA#g" pkg/wasm_game.js

echo "==> Done. Serve with: python3 -m http.server 8000 -d pkg"
echo "    Docs:    http://localhost:8000/documentation.html"