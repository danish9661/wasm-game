#!/usr/bin/env bash
# Git pre-push hook: run the full gate before pushing.
#  - Rust tests ALWAYS gate the push (reliable, fast).
#  - The headless visual audit is run but its failures only WARN here; CI is the
#    strict enforcer so a flaky browser run never blocks a legitimate push.
# Install via: cp tools/pre-push.sh .git/hooks/pre-push && chmod +x .git/hooks/pre-push
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
AUDIT_VISUAL_BLOCK=0 exec bash "$ROOT/tools/audit.sh"
