#!/usr/bin/env python3
"""Sync wave-2 brief target dims with the actual exported GLB measurements.

The validator reports the real AABB as `INFO dims x … y … z …`. This writes
those numbers back into each brief (tol is kept generous), so the brief documents
the delivered asset instead of the initial guess.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLI = Path.home() / ".config/opencode/skills/game-asset-modeling/scripts/gam_cli.py"
STEMS = [
    "wheat", "coral_fan", "swamp_reeds", "carrot", "lilypad", "dune_rock",
    "fern", "reed", "herb_patch", "tree", "bush", "crystal",
    "pine", "rubble",
]


def main():
    for stem in STEMS:
        bp = ROOT / ("assets/src/props/prop_%s/brief.json" % stem)
        out = subprocess.run([sys.executable, str(CLI), "validate", str(bp)],
                             capture_output=True, text=True).stdout
        m = re.search(r"INFO\s+dims\s+x\s+([\d.]+)\s+y\s+([\d.]+)\s+z\s+([\d.]+)", out)
        if not m:
            print("NO DIMS", stem)
            continue
        d = json.loads(bp.read_text())
        d["dims"]["x"], d["dims"]["y"], d["dims"]["z"] = (float(v) for v in m.groups())
        bp.write_text(json.dumps(d, indent=2) + "\n")
        print("DIMS", stem, m.groups())


if __name__ == "__main__":
    main()
