#!/usr/bin/env bash
# Publish to GitHub Pages: builds the wasm, commits the static pkg/ contents
# to the gh-pages branch, and pushes. Needs a git remote named "origin" and
# HTTPS credentials (or gh auth) configured.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

git rev-parse --git-dir >/dev/null 2>&1 || { echo "not a git repo — run from the repo root"; exit 1; }
git remote get-url origin >/dev/null 2>&1 || { echo "no origin remote configured"; exit 1; }

"$ROOT/build.sh"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT INT TERM

git fetch origin gh-pages 2>/dev/null || true
git worktree add "$WORK" gh-pages 2>/dev/null || git worktree add "$WORK" --orphan gh-pages
cd "$WORK"
rm -rf ./*
cp -r "$ROOT/pkg/." .
touch .nojekyll
git add -A
if git diff --cached --quiet; then
  echo "==> no changes to publish"
else
  git commit -m "deploy: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  git push -f origin gh-pages
  echo "==> pushed to gh-pages"
fi
cd "$ROOT"
git worktree remove "$WORK" --force 2>/dev/null || true
git branch -D gh-pages 2>/dev/null || true
echo "==> done — https://danish9661.github.io/wasm-game/"