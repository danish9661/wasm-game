"""Re-open every fourth-wave GLB and re-measure it against its manifest.

    blender -b --python scripts/verify_wave4_animations.py

The rig derivation, re-import, slot binding, sampler and gate checks follow
verify_wave2_animations.py; the contact pivots are re-derived from the source
GLB with wave-4's per-clip contact choice, and the extra travel-direction and
wide-window foot-slide fields are checked as well.
"""

import hashlib
import json
import os
import sys

import bpy

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MANIFEST = os.path.join(ROOT, "assets", "animations", "wave4", "manifest.json")
FPS = 30
LABEL = "fourth-wave"

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import generate_wave2_animations as g2  # noqa: E402
import generate_wave4_animations as w4  # noqa: E402
import verify_wave2_animations as v2  # noqa: E402


def reload_info(clip):
    """Derive source tips, re-import the GLB and bind its slotted action."""
    src = w4.build(clip["model"], clip.get("contacts"))
    tips = {k: src["tips"][k] for k in clip["contacts"]}

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.render.fps = FPS
    bpy.ops.import_scene.gltf(filepath=os.path.join(ROOT, clip["path"]))
    objs = {o.name: o for o in bpy.data.objects}
    actions = [a for a in bpy.data.actions if a.name == clip["clip"]]
    errs = []
    if len(actions) != 1:
        return errs + ["imported action not found"], {}
    for slot in actions[0].slots:
        ident = slot.identifier[2:] if slot.identifier.startswith("OB") else slot.identifier
        o = objs.get(ident)
        if o and o.animation_data:
            o.animation_data.action = actions[0]
            o.animation_data.action_slot = slot
    names = ["root"] + [p[0] for p in g2.RIGS[clip["model"]]]
    empties = {n: v2.find_pivot(objs, n) for n in names}
    if any(v is None for v in empties.values()):
        errs.append("imported scene missing rig nodes: %s" %
                    [n for n, v in empties.items() if v is None])
        return errs, {}
    info = {"empties": empties,
            "meshes": {o.name: o for o in objs.values() if o.type == "MESH"},
            "tips": tips, "contacts": list(clip["contacts"])}
    return errs, info


def main():
    with open(MANIFEST) as fh:
        manifest = json.load(fh)
    clips = manifest["clips"]
    bad = []
    print("verifying %d %s clips" % (len(clips), LABEL))
    for clip in clips:
        path = os.path.join(ROOT, clip["path"])
        errs = v2.structural(clip, v2.glb_json(path))
        reload_errs, info = reload_info(clip)
        errs += reload_errs
        measured, warns = {}, []
        if info and not errs:
            measured = g2.measure(info, clip["frames"], clip["loop"])
            fast = w4.planted_slide_fast(info, clip["frames"], clip["loop"])
            direction = clip.get("direction")
            err = (w4.direction_error(measured, *w4.DIRS[direction])
                   if direction else 0.0)
            checks = dict(measured)
            checks["foot_slide_fast_cm"] = fast
            checks["direction_error_deg"] = err
            for key, tol in (("seam_position_cm", 0.5),
                             ("foot_slide_cm", 1.5),
                             ("ground_penetration_cm", 1.0),
                             ("float_p10_cm", 1.0),
                             ("end_settled_mps", 0.5),
                             ("foot_slide_fast_cm", 1.0),
                             ("direction_error_deg", 0.5)):
                if abs(checks[key] - clip[key]) > tol:
                    errs.append("%s manifest %.3f reloaded %.3f" %
                                (key, clip[key], checks[key]))
            fails, warns = w4.gate4(checks, clip["loop"], clip["clip"])
            errs += ["reloaded " + x for x in fails]
        status = "PASS" if not errs else "FAIL"
        print("[%s] %-30s slide=%.3fcm fast=%.3fcm pen=%.2fcm off=%.3fm%s" %
              (status, clip["clip"],
               measured.get("foot_slide_cm", 0.0),
               clip["foot_slide_fast_cm"],
               measured.get("ground_penetration_cm", 0.0),
               measured.get("root_offset_m", {}).get("magnitude", 0.0),
               "  warns=" + str(warns) if info and warns else ""))
        for e in errs:
            print("    -", e)
            bad.append((clip["clip"], e))

    for model, info in sorted(manifest["models"].items()):
        with open(os.path.join(ROOT, info["glb"]), "rb") as fh:
            digest = hashlib.sha256(fh.read()).hexdigest()
        if digest != info["sha256"]:
            bad.append((model, "source GLB changed since generation"))
    n = len(clips)
    print("RESULT %d/%d passed" % (n - len({c for c, _ in bad}), n))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
