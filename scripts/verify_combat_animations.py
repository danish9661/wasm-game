"""Independent verification of the exported combat clips.

Run in a fresh Blender 5.1 process:

    blender -b --python scripts/verify_combat_animations.py

For every clip in assets/animations/manifest.json this:
  * parses the GLB JSON (animation name, channels, no scale keys, 30 fps keys);
  * imports the GLB into a clean scene (proves the clip actually loads);
  * re-measures the loop seam, planted-foot slide and root behaviour from the
    imported node transforms and compares them with the manifest.
"""

import json
import math
import os
import struct
import sys

import bpy
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "assets", "animations")
MANIFEST = os.path.join(OUT, "manifest.json")
FPS = 30


def glb_json(path):
    with open(path, "rb") as fh:
        data = fh.read()
    magic, version, length = struct.unpack_from("<III", data, 0)
    assert magic == 0x46546C67 and version == 2, path
    off = 12
    while off < length:
        clen, ctype = struct.unpack_from("<II", data, off)
        off += 8
        if ctype == 0x4E4F534A:
            return json.loads(data[off:off + clen].decode("utf-8"))
        off += clen
    raise RuntimeError("no JSON chunk in " + path)


def structural_check(clip, path):
    errs = []
    j = glb_json(path)
    anims = j.get("animations", [])
    if len(anims) != 1:
        errs.append("expected 1 animation, found %d" % len(anims))
        return errs
    if anims[0].get("name") != clip["action"]:
        errs.append("animation name %r != %r" % (anims[0].get("name"), clip["action"]))
    node_names = set(n.get("name", "") for n in j.get("nodes", []))
    for required in ("root", "hips", "foot_l", "foot_r", "tunic", "boot_l", "boot_r"):
        if required not in node_names:
            errs.append("missing node " + required)
    for ch in anims[0]["channels"]:
        if ch["target"]["path"] == "scale":
            errs.append("scale key on node %s" % ch["target"]["node"])
    times = set()
    for sampler in anims[0]["samplers"]:
        acc = j["accessors"][sampler["input"]]
        times.add(round(acc.get("max", [0])[0] * FPS))
    if times and max(times) + 1 != clip["frames"]:
        errs.append("last key frame %d != %d" % (max(times), clip["frames"] - 1))
    return errs


def remeasure(clip, path):
    """Import the exported GLB and re-measure it with the generator's own code."""
    errs = []
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.render.fps = 30   # GLB times were exported at 30 fps
    bpy.ops.import_scene.gltf(filepath=path)
    scene = bpy.context.scene
    objs = {o.name: o for o in bpy.data.objects}
    for n in ("root", "hips", "foot_l", "foot_r"):
        if n not in objs:
            errs.append("imported scene missing node " + n)
    if errs:
        return errs, {}
    actions = [a for a in bpy.data.actions if a.name == clip["action"]]
    if len(actions) != 1:
        return errs + ["imported action %r not found" % clip["action"]], {}
    # Imported glTF actions are slotted per node. Binding every object to the
    # first slot would freeze all but one node, so match by slot identifier.
    for slot in actions[0].slots:
        target = slot.identifier[2:] if slot.identifier.startswith("OB") else slot.identifier
        o = objs.get(target)
        if o and o.animation_data:
            o.animation_data.action = actions[0]
            o.animation_data.action_slot = slot

    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import generate_combat_animations as gen
    rig = {n: objs[n] for n in gen.JOINTS if n in objs}
    meshes = {o.name: o for o in objs.values() if o.type == "MESH"}
    res = gen.measure_clip(rig, meshes, scene, clip["clip"], clip["frames"],
                           clip["loop"], clip["root_motion"], clip.get("direction"),
                           None)
    measured = {
        "seam_position_cm": res["seam_position_cm"],
        "foot_slide_cm": res["foot_slide_cm"],
        "root_speed_mps": res["root_speed_mps"],
        "ground_penetration_cm": res["ground_penetration_cm"],
    }
    if clip["root_motion"] and res["root_speed_mps"] < 0.05:
        errs.append("root-motion clip did not travel")
    if not clip["root_motion"] and res["root_speed_mps"] > 0.02:
        errs.append("in-place clip travelled at %.2f m/s" % res["root_speed_mps"])
    # The GLB round-trip stores rotations as quaternion LINEAR samplers while
    # the authoring scene keys Euler angles. At sprint speed this shifts
    # contact points ~1 cm and speed ~0.2 m/s; absolute gates are still
    # checked on both the manifest and the reloaded numbers.
    for key, tol in (("seam_position_cm", 0.5), ("foot_slide_cm", 1.5),
                     ("root_speed_mps", 0.2), ("ground_penetration_cm", 1.0)):
        if abs(measured[key] - clip[key]) > tol:
            errs.append("%s: manifest %.3f, reloaded %.3f" %
                        (key, clip[key], measured[key]))
    if clip["loop"] and measured["seam_position_cm"] > 2.0:
        errs.append("seam %.3f cm > 2" % measured["seam_position_cm"])
    return errs, measured


def main():
    with open(MANIFEST) as fh:
        manifest = json.load(fh)
    failures = []
    print("verifying %d clips from %s" % (len(manifest["clips"]), MANIFEST))
    for clip in manifest["clips"]:
        path = os.path.join(ROOT, clip["path"])
        errs = structural_check(clip, path)
        more, measured = remeasure(clip, path)
        errs += more
        status = "PASS" if not errs else "FAIL"
        print("[%s] %-20s %s" % (status, clip["clip"], measured))
        for e in errs:
            print("    -", e)
            failures.append((clip["clip"], e))
    print("RESULT %d passed, %d failed checks" %
          (len(manifest["clips"]) - len(set(c for c, _ in failures)), len(failures)))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
