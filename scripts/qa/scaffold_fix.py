#!/usr/bin/env python3
"""Create a game-asset-modeling brief scaffold for a visual-QA rebuild.

Usage: python3 scripts/qa/scaffold_fix.py NAME [prop|wpn]
Reads the shipped GLB's measured dims from assets/models/metrics.json so the
replacement keeps the same footprint and height (base-at-zero contract).
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


def main() -> int:
    name = sys.argv[1]
    category = sys.argv[2] if len(sys.argv) > 2 else "prop"
    metrics = json.loads((ROOT / "assets/models/metrics.json").read_text())
    m = next((x for x in metrics["models"] if x["name"] == name), None)
    if m is None:
        raise SystemExit("no metrics for %s" % name)
    aid = ("prop_" if category == "prop" else "wpn_") + name
    d = ROOT / "assets/src/props" / aid
    d.mkdir(parents=True, exist_ok=True)
    if category == "wpn":
        pivot, point, on_ground, collider = "custom", [0.0, 0.0, 0.06], False, "none"
    else:
        pivot, point, on_ground, collider = "bottom_center", None, True, "box"
    brief = {
        "id": aid,
        "category": category,
        "tier": "S",
        "status": "done",
        "track": "invented",
        "source": "make",
        "what": "Visual-QA rebuild of %s; low-poly diorama prop, readable from the isometric camera." % name,
        "dims": {
            "x": round(m["footprint_x"], 3),
            "y": round(m["footprint_z"], 3),
            "z": round(m["height"], 3),
            "tol": 0.14,
            "basis": "Shipped %s.glb measured by verify_models.py: %.3f x %.3f x %.3f u; keep the same footprint and height." % (
                name, m["footprint_x"], m["footprint_z"], m["height"]),
        },
        "pivot": pivot,
        "budget_class": "prop" if category == "prop" else "hero",
        "density_class": "prop",
        "collider": collider,
        "props": {},
        "materials": [],
        "variants": [],
        "sockets": [],
        "views": {"primary": "held" if category == "wpn" else "iso",
                  "why": "Reads at gameplay scale beside a 1.55 u human; 60 px inset used in the contact sheet."},
        "references": [],
        "style_notes": "Low-poly diorama, flat palette colours from docs/ART_DIRECTION.md, no textures. Never: smooth shading, plain unbroken boxes, invented hues.",
        "notes": "Visual-QA rebuild: bmesh parts, bevels, weighted normals, packed UV0; replaces assets/models/%s.glb." % name,
    }
    if point is not None:
        brief["pivot_point"] = point
        brief["on_ground"] = on_ground
    path = d / "brief.json"
    path.write_text(json.dumps(brief, indent=2) + "\n", encoding="utf-8")
    print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
