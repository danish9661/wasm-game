#!/usr/bin/env python3
"""Scaffold game-asset-modeling briefs for the expansion hero pieces.

The brief records intent (what/dims/view/style) and points at the exported GLB
measurements. Run once; the files are checked in alongside the generators.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STYLE = ("Low-Poly Diorama: chunky, faceted, flat-shaded, saturated accent "
         "colours, readable at ~60 px. Palette-only (docs/ART_DIRECTION.md), "
         "no textures, no smooth shading. 1 tile = 1 u; player = 1.55 u; "
         "storey = 2.5 u; doors 0.9 x 2.0 u. Base at z=0, centred on (0,0). "
         "Never: smooth normals, external textures, invented hues, hidden "
         "geometry nobody sees.")

BRIEFS = [
    {
        "id": "mangrove", "category": "env", "tier": "S", "status": "done",
        "track": "invented", "source": "make",
        "what": "Swamp biome hero tree: stilt roots and a flat-topped canopy, "
                "a landmark the player can shelter under.",
        "dims": {"x": 2.59, "y": 1.75, "z": 2.72, "tol": 0.05,
                 "basis": "measured from exported GLB; 1.75x player height"},
        "pivot": "bottom_center", "budget_class": "prop", "density_class": "prop",
        "collider": "convex", "props": {}, "materials": [], "variants": [],
        "sockets": [],
        "views": {"primary": "iso",
                  "why": "seen from the 45/30 isometric camera as a biome landmark"},
        "references": [], "style_notes": STYLE,
        "notes": "Hero finishing: bevelled silhouette parts, weighted normals, "
                 "flat shading, smart UV pack; 460 tris. Crown widened, "
                 "seam capped and feet grounded after the critic rounds.",
    },
    {
        "id": "npc_forest_warden", "category": "char", "tier": "S",
        "status": "done", "track": "character", "source": "make",
        "what": "Forest warden mini-boss: antlered humanoid in a leaf cloak "
                "carrying a glowing rune staff.",
        "dims": {"x": 1.28, "y": 0.69, "z": 2.32, "tol": 0.05,
                 "basis": "measured from exported GLB; 1.50x player including antlers, body ~2.2u"},
        "pivot": "bottom_center", "budget_class": "char_body",
        "density_class": "char_body", "collider": "box", "props": {},
        "materials": [], "variants": [], "sockets": [],
        "views": {"primary": "eye",
                  "why": "fought from the player's eye view; silhouette must "
                         "read as a boss at once"},
        "references": [], "style_notes": STYLE,
        "notes": "Hero finishing: bevel on torso/staff/pauldrons/mask only, "
                 "weighted normals, flat shading, smart UV pack; 450 tris. "
                 "Branching skull-attached antlers and front-facing leaf cloak "
                 "added after the first critic round.",
    },
    {
        "id": "blacksmith_hall_interior", "category": "env", "tier": "S",
        "status": "done", "track": "kit", "source": "make",
        "what": "Complete 5x4-tile forge hall assembled from the expansion "
                "kit pieces: forge, anvil, bellows, tool rack and water trough.",
        "dims": {"x": 5.15, "y": 4.17, "z": 2.50, "tol": 0.05,
                 "basis": "measured from exported GLB; exactly one storey"},
        "pivot": "bottom_center", "budget_class": "kit",
        "density_class": "kit", "collider": "box", "props": {},
        "materials": [], "variants": [], "sockets": [],
        "views": {"primary": "iso",
                  "why": "diorama cutaway from the isometric camera"},
        "references": [], "style_notes": STYLE,
        "notes": "Built from the same Frame-based piece builders as the "
                 "standalone kit; 0.9x2.0 door with 0.5u transom, two windows "
                 "at 1.1u sill, walls cut with real openings. 1380 tris.",
    },
    {
        "id": "temple_hall_interior", "category": "env", "tier": "S",
        "status": "done", "track": "kit", "source": "make",
        "what": "Complete 5x5-tile temple hall assembled from the expansion "
                "kit pieces: stone colonnade, altar, braziers and banners.",
        "dims": {"x": 5.18, "y": 5.19, "z": 2.50, "tol": 0.05,
                 "basis": "measured from exported GLB; exactly one storey"},
        "pivot": "bottom_center", "budget_class": "kit",
        "density_class": "kit", "collider": "box", "props": {},
        "materials": [], "variants": [], "sockets": [],
        "views": {"primary": "iso",
                  "why": "diorama cutaway from the isometric camera"},
        "references": [], "style_notes": STYLE,
        "notes": "Built from the same Frame-based piece builders as the "
                 "standalone kit; 0.9x2.0 door with 0.5u transom, two windows "
                 "at 1.1u sill, walls cut with real openings. 1476 tris.",
    },
]

for brief in BRIEFS:
    out = ROOT / "assets" / "src" / brief["category"] / brief["id"] / "brief.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(brief, indent=2) + "\n", encoding="utf-8")
    print("BRIEF", out.relative_to(ROOT))
