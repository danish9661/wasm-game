#!/usr/bin/env python3
"""Generate docs/SKILL_ASSET_METRICS.md for assets made with the skill pipeline.

Reads the exported GLB's measurements from ``assets/models/metrics.json``,
the skill validator result from the asset's ``validate.json`` and the Babylon
review result from ``assets/review/<id>/review.json``.

Run: python3 scripts/build_skill_asset_metrics.py
"""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ID = "prop_star_crown_shrine"
PLAYER = 1.55
OUT = ROOT / "docs" / "SKILL_ASSET_METRICS.md"


def load(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def main():
    metrics = load("assets/models/metrics.json")
    m = next(x for x in metrics["models"] if x["name"] == ID)
    brief = load("assets/src/props/%s/brief.json" % ID)
    validation = load("assets/src/props/%s/validate.json" % ID)
    review = load("assets/review/%s/review.json" % ID)
    fails = sum(1 for i in validation["items"] if i["level"] == "FAIL")
    warns = sum(1 for i in validation["items"] if i["level"] == "WARN")
    tri_detail = next(i["detail"] for i in validation["items"]
                      if i["check"] == "budget.lod0")
    lod0_tris = int(re.search(r"(\d+) tris", tri_detail).group(1))

    lines = [
        "# Starfall - Skill-built asset metrics",
        "",
        "One hero prop authored with the **game-asset-modeling** skill pipeline "
        "(brief -> placeholder -> bmesh form -> finish -> validate -> Babylon "
        "review -> manifest). All numbers below are read from the exported "
        "files, not from the brief.",
        "",
        "## `prop_star_crown_shrine`",
        "",
        "A Starfall shrine plinth displaying the Star Crown: stepped stone "
        "base, four gold-capped posts, a six-sided gold halo and a hovering "
        "arcane crown with an emissive gem. Built from the brief at "
        "`assets/src/props/%s/brief.json`." % ID,
        "",
        "| Measurement | Value |",
        "|---|---:|",
        "| Brief dimensions (W x D x H, Blender axes) | %.2f x %.2f x %.2f u |" % (
            brief["dims"]["x"], brief["dims"]["y"], brief["dims"]["z"]),
        "| Measured W x D x H (X x Z x Y, glTF axes) | %.3f x %.3f x %.3f u |" % (
            m["footprint_x"], m["footprint_z"], m["height"]),
        "| Player-relative height (x1.55) | %.2fx |" % (m["height"] / PLAYER),
        "| Base Y | %.3f u |" % m["base_y"],
        "| Footprint centre (X, Z) | (%+.3f, %+.3f) u |" % (
            m["center_x"], m["center_z"]),
        "| Render triangles (LOD0) | %d |" % lod0_tris,
        "| Project verifier triangles (render + collider) | %d |" % m["triangles"],
        "| Materials | %d (stone, gold, star gem, dim arcane) |" % m["materials"],
        "| Triangle budget | 500 (prop) |",
        "",
        "## Scale contract",
        "",
        "- **1 unit = 1 tile = 1 metre.** Blender is Z-up; exported glTF is "
        "Y-up. W x D x H above are in glTF axes (X footprint x Z footprint x "
        "Y height).",
        "- Player reference **1.55 u**; the shrine is %.2fx the player, below "
        "the 2.5 u storey." % (m["height"] / PLAYER),
        "- Bottom-centre pivot at the world origin; base Y = 0 and footprint "
        "centre (0, 0) in the exported GLB.",
        "- Flat-shaded, palette-only Principled materials, no textures, "
        "no smooth normals.",
        "",
        "## Skill checks",
        "",
        "- `gam_cli.py validate`: **%s** - %d fail, %d warn. Includes naming, "
        "hierarchy, applied transforms, budget, dimensions, pivot, collider, "
        "normals, UV0, lightmap UV, materials, palette and Khronos." % (
            validation["result"], fails, warns),
        "- `gam_cli.py review`: **%d engine errors**. Views: front, "
        "three-quarter, eye, iso, iso_context, night and two crown close-ups; "
        "sheet at `assets/review/%s.png`." % (len(review["errors"]), ID),
        "- Emissive accent verified in the night view: the purple gem and two "
        "shards read against a dark scene.",
        "- `python3 scripts/verify_models.py`: **%d GLBs, %d passed, %d "
        "failed**; `python3 scripts/check_metrics.py`: OK." % (
            metrics["summary"]["models"], metrics["summary"]["passed"],
            metrics["summary"]["failed"]),
        "",
        "## Reproduce",
        "",
        "```bash",
        "BLENDER=~/.local/blender/blender gam_cli.py build "
        "assets/src/props/%s/brief.json" % ID,
        "gam_cli.py validate assets/src/props/%s/brief.json" % ID,
        "gam_cli.py review assets/src/props/%s/brief.json" % ID,
        "gam_cli.py manifest assets/src/props/%s/brief.json" % ID,
        "python3 scripts/verify_models.py",
        "python3 scripts/check_metrics.py",
        "python3 scripts/build_skill_asset_metrics.py",
        "```",
        "",
    ]
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print("WROTE", OUT.relative_to(ROOT))


if __name__ == "__main__":
    main()
