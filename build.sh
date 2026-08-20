#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "==> Building Rust -> WASM (wasm-pack, web target)"
wasm-pack build web --target web --out-dir ../pkg --release

echo "==> Copying static shell into pkg/"
cp web/static/index.html web/static/style.css pkg/

echo "==> Done. Serve with: python3 -m http.server 8000 -d pkg"