#!/usr/bin/env python3
"""Scaffold the brief.json files for the nature/biome visual-QA rebuilds.

Run once. The build_<id>.py scripts live next to each brief and are authored
separately; `gam_cli.py all <brief>` then builds, validates (incl. Khronos),
reviews in Babylon and upserts the manifest.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "assets" / "src" / "props"

STYLE = ("Low-Poly Diorama: chunky, faceted, flat-shaded, palette-only, "
         "readable at ~60 px. 1u = 1 tile; player = 1.55u; base at z=0, "
         "centred on (0,0). Palette from docs/ART_DIRECTION.md. "
         "Never: plain cubes, featureless blobs, smooth normals, textures, "
         "invented hues, trees that are just cones/cylinders.")

# id, game filename stem, what, dims (x,y,z), primary view
ITEMS = [
    ("prop_hay_bale", "hay_bale", "A round straw bale: squat faceted cylinder with raised straw bands and a faceted spiral end - not a wrapped cube.", (1.10, 0.78, 0.72)),
    ("prop_dead_tree", "dead_tree", "A stark dead snag: flared root base, tapered trunk and two generations of forked bare branches, readable from the iso camera.", (1.40, 1.25, 2.60)),
    ("prop_mangrove_roots", "mangrove_roots", "A swamp mangrove root system: arched prop roots reaching the ground, wood stump and a small swamp-green foliage crown - no blue water hues in wood.", (1.55, 1.40, 1.80)),
    ("prop_ice_spire", "ice_spire", "A tundra ice spire: irregular faceted icy prisms with offset tips and scattered shards - no square display plinth, no traffic cones.", (1.10, 1.00, 2.05)),
    ("prop_pine", "pine", "A stylised conifer: overlapping offset needle tiers with visible branch stubs and exposed trunk - explicitly not three stacked cones.", (1.49, 1.10, 2.80)),
    ("prop_cactus", "cactus", "A desert cactus: tapered ribbed column with rounded, tapered arms and spine nubs - not three pieces of green plumbing.", (0.95, 0.44, 1.20)),
    ("prop_volcanic_bomb", "volcanic_bomb", "A volcanic bomb: angular dark stone projectile with glowing lava cracks across its facets and ash chips at its base - not an orange box.", (0.85, 0.80, 0.85)),
    ("prop_coral_brain", "coral_brain", "A brain coral: dome with winding raised sand-coloured ridges and swamp-green grooves - no pink or saturated yellow.", (0.95, 0.90, 0.85)),
    ("prop_vines", "vines", "A climbing vine: one continuous twisted strand with broad leaf pairs on a small wood support - not flat green crosses.", (0.90, 0.85, 0.85)),
    ("prop_potato", "potato", "A potato crop: half-buried lumpy tubers with eyes in dark soil and two modest leaf shoots - the tubers must be visible.", (0.95, 0.80, 0.62)),
]


def main():
    for ident, stem, what, (x, y, z) in ITEMS:
        d = SRC / ident
        d.mkdir(parents=True, exist_ok=True)
        brief = {
            "id": ident,
            "category": "prop",
            "tier": "S",
            "status": "done",
            "track": "invented",
            "source": "make",
            "what": what,
            "dims": {"x": x, "y": y, "z": z, "tol": 0.16,
                     "basis": "target from visual-QA brief; replaces assets/models/%s.glb" % stem},
            "pivot": "bottom_center",
            "budget_class": "prop",
            "density_class": "prop",
            "collider": "box",
            "props": {},
            "materials": [],
            "variants": [],
            "sockets": [],
            "views": {"primary": "iso",
                      "why": "commander camera at 45/30; gameplay-scale readability is the contract"},
            "reviews": "gameplay",
            "review_views": ["front", "three_quarter", "eye", "iso", "feed", "collision"],
            "references": [],
            "style_notes": STYLE,
            "notes": "Fresh-eyes visual QA rebuild: bmesh parts, bevels, weighted normals, packed UV0; re-exported to assets/models/%s.glb." % stem,
        }
        (d / "brief.json").write_text(json.dumps(brief, indent=2) + "\n")
        print("BRIEF", d / "brief.json")


if __name__ == "__main__":
    main()
