#!/usr/bin/env python3
"""Define the visual-audit family: every character, NPC, enemy and boss GLB.

Reads assets/models/metrics.json and writes assets/qa/characters/family.json.
Kept explicit so both the render script and the critic use one frozen list.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
METRICS = ROOT / "assets" / "models" / "metrics.json"
OUT = ROOT / "assets" / "qa" / "characters" / "family.json"

# Creature-like models that the generator files categorise as "Other" but that
# still appear as living characters in the village / wilds.
EXTRA_CREATURES = {
    "fire_elemental", "stone_elemental", "forest_deer", "river_otter",
    "sea_turtle", "scarecrow",
}
# The rebuilt hero has its own strict animation binding contract; it is audited
# and re-rendered but only changed if the critic finds a blocking issue.
FIXABLE = True


def main() -> int:
    models = json.loads(METRICS.read_text())["models"]
    family = []
    for m in models:
        name = m["name"]
        if (m["category"] in {"Character", "Npc Variants"}
                or name.startswith("npc_")
                or name in EXTRA_CREATURES):
            family.append({
                "name": name,
                "category": m["category"],
                "height": m["height"],
                "triangles": m["triangles"],
                "materials": m["materials"],
            })
    family.sort(key=lambda e: (e["category"] != "Character", e["name"]))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"family": family, "count": len(family)}, indent=2) + "\n")
    print("family: %d models -> %s" % (len(family), OUT.relative_to(ROOT)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
