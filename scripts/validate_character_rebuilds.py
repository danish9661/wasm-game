#!/usr/bin/env python3
"""Khronos + Babylon review evidence for the rebuilt character models.

    python3 scripts/validate_character_rebuilds.py

Khronos glTF-Validator checks the exported GLB containers. Babylon (the project
engine) loads each GLB in the skill's Playwright harness; engine console errors
fail the step. Evidence lands in assets/qa/characters/review/.
"""
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODELS = ROOT / "assets" / "models"
QA = ROOT / "assets" / "qa" / "characters"
REVIEW = QA / "review"
SKILL = Path(os.path.expanduser(
    "~/.config/opencode/skills/game-asset-modeling"))
TOOLS = SKILL / "tools"
SPEC = ROOT / "assets" / "spec.json"
NAMES = ["wolf", "wraith", "bat", "scorpion_queen", "colossus", "spider",
         "npc_desert_scorpion", "npc_merchant", "sea_turtle", "npc_wolf_pup",
         "scarecrow", "npc_farmer", "npc_fisher", "npc_bard", "npc_sailor",
         "npc_traveling_scholar", "npc_innkeeper"]


def main() -> int:
    REVIEW.mkdir(parents=True, exist_ok=True)
    metrics = {m["name"]: m for m in
               json.loads((MODELS / "metrics.json").read_text())["models"]}
    failed = []
    for name in NAMES:
        glb = MODELS / (name + ".glb")
        brief_dir = QA / "briefs"
        brief_dir.mkdir(parents=True, exist_ok=True)
        m = metrics[name]
        brief = {
            "id": name, "category": "char", "status": "done",
            "track": "invented", "source": "make",
            "what": "Rebuilt low-poly character for the visual-quality audit.",
            "dims": {"x": m["footprint_x"], "y": m["footprint_z"],
                     "z": m["height"], "tol": 0.35,
                     "basis": "verified GLB bounds from scripts/verify_models.py"},
            "pivot": "bottom_center", "views": {"primary": "iso"},
            "style_notes": "ART_DIRECTION.md: flat-shaded diorama, palette only.",
            "review_views": ["front", "three_quarter", "iso", "feed"],
        }
        bpath = brief_dir / (name + ".json")
        bpath.write_text(json.dumps(brief, indent=2) + "\n")

        raw = subprocess.run(
            ["node", str(TOOLS / "khronos_validate.mjs"), str(glb)],
            capture_output=True, text=True)
        try:
            report = json.loads(raw.stdout)
            issues = report.get("issues", {})
            errs = len(issues.get("errors", []))
            warns = len(issues.get("warnings", []))
        except (ValueError, AttributeError):
            errs, warns = -1, -1
        print("[khronos] %-22s errors=%d warnings=%d" % (name, errs, warns))
        if errs != 0:
            failed.append((name, "khronos errors=%d" % errs))

        out = REVIEW / name
        shot = subprocess.run(
            ["node", str(TOOLS / "review" / "shoot.mjs"), str(glb),
             "--spec", str(SPEC), "--brief", str(bpath),
             "--out", str(out), "--sheet", str(REVIEW / (name + "_sheet.png")),
             "--views", "front,three_quarter,iso,feed", "--size", "900x640"],
            capture_output=True, text=True, timeout=300)
        ok = False
        rpath = out / "review.json"
        if rpath.exists():
            rr = json.loads(rpath.read_text())
            ok = not rr.get("errors")
            if not ok:
                failed.append((name, "Babylon console errors: %s" % rr["errors"][:2]))
        else:
            failed.append((name, "Babylon review did not produce review.json"))
        print("[babylon] %-22s %s" % (name, "PASS" if ok else "FAIL"))
        if not ok and shot.stderr:
            print("   ", shot.stderr[-300:].replace("\n", " "))
    print("RESULT", "PASS" if not failed else "FAIL", failed)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
