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

echo "==> Done. Serve with: python3 -m http.server 8000 -d pkg"
echo "    Docs:    http://localhost:8000/documentation.html"