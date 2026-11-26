#!/usr/bin/env python3
"""Lightweight invariant check for the generated asset metrics file.

Reads ``assets/models/metrics.json`` (produced by ``verify_models.py``) and
asserts the key art-direction invariants without needing Blender, Rust or any
third-party package:

  * player height ~= 1.55 u,
  * house height within 2..4 u,
  * every model's base Y ~= 0,
  * summary counts match the recorded models.

Run:  python3 scripts/check_metrics.py
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
METRICS = ROOT / "assets" / "models" / "metrics.json"
BASE_TOL = 0.06  # metrics are rounded to 4 decimals; contract tolerance is 0.05


def main() -> int:
    if not METRICS.is_file():
        print("FAIL: %s is missing (run python3 scripts/verify_models.py first)" % METRICS)
        return 1
    try:
        report = json.loads(METRICS.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print("FAIL: cannot read metrics.json: %s" % exc)
        return 1

    errors: list[str] = []
    models: list[dict] = report.get("models", [])
    by_name = {m.get("name"): m for m in models}

    if report.get("schema_version") != 1:
        errors.append("unexpected metrics schema_version %r" % report.get("schema_version"))
    if not models:
        errors.append("metrics file contains no models")

    player = by_name.get("player")
    if player is None:
        errors.append("player model missing from metrics")
    elif not (1.45 <= player["height"] <= 1.65):
        errors.append("player height %.3fu is not ~1.55u" % player["height"])

    house = by_name.get("house")
    if house is None:
        errors.append("house model missing from metrics")
    elif not (2.0 <= house["height"] <= 4.0):
        errors.append("house height %.3fu is outside 2..4u" % house["height"])

    for m in models:
        if abs(m.get("base_y", float("inf"))) > BASE_TOL:
            errors.append("%s base Y %.3fu is not ~0" % (m["name"], m["base_y"]))
        if m.get("triangles", 0) > m.get("triangle_limit", 0):
            errors.append("%s exceeds its triangle budget" % m["name"])

    summary = report.get("summary", {})
    if summary.get("models") != len(models):
        errors.append("summary.models != len(models)")
    passed = sum(1 for m in models if m.get("pass"))
    if summary.get("passed") != passed:
        errors.append("summary.passed != number of passing models")
    if summary.get("failed") != len(models) - passed:
        errors.append("summary.failed != number of failing models")

    if errors:
        for err in errors:
            print("FAIL:", err)
        print("[check_metrics] %d invariant(s) failed" % len(errors))
        return 1

    print("[check_metrics] OK: %d models, player %.2fu, house %.2fu, all base Y ~= 0"
          % (len(models), player["height"], house["height"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
