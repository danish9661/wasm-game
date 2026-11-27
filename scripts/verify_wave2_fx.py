"""Re-open every second-wave FX GLB and verify it against fx_manifest.json.

    blender -b --python scripts/verify_wave2_fx.py

Each GLB is imported into a fresh factory scene at 30 fps.  We assert exactly
one correctly named animation, that every animated mesh node has translation /
rotation / scale channels, that all sampler times span 0..(frames-1)/30 s, and
that the manifest frame count agrees with the reloaded keyframes and samplers.
"""

import json
import os
import struct
import sys

import bpy

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "assets", "animations", "wave2", "fx")
MANIFEST = os.path.join(ROOT, "assets", "animations", "wave2", "fx_manifest.json")
FPS = 30

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from generate_combat_animations import iter_fcurves  # noqa: E402

CHANNELS = {"translation", "rotation", "scale"}


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
    raise RuntimeError("no JSON chunk: " + path)


def structural(rec, j):
    """Check animation name, channel completeness and sampler time span."""
    errs = []
    anims = j.get("animations", [])
    if len(anims) != 1:
        return errs + ["expected 1 animation, found %d" % len(anims)]
    anim = anims[0]
    if anim.get("name") != rec["name"]:
        errs.append("animation name %r != %r" % (anim.get("name"), rec["name"]))

    nodes = j.get("nodes", [])
    mesh_nodes = {i for i, n in enumerate(nodes) if "mesh" in n}
    if len(mesh_nodes) != rec["primitive_count"]:
        errs.append("mesh nodes %d != manifest primitives %d" %
                    (len(mesh_nodes), rec["primitive_count"]))

    targets = {}
    for ch in anim.get("channels", []):
        idx = ch["target"]["node"]
        path = ch["target"]["path"]
        if idx not in mesh_nodes:
            errs.append("channel targets non-mesh node %d" % idx)
        targets.setdefault(idx, set()).add(path)
    for idx in sorted(mesh_nodes):
        missing = CHANNELS - targets.get(idx, set())
        if missing:
            errs.append("node %r missing %s" %
                        (nodes[idx].get("name"), sorted(missing)))
    for idx in sorted(set(targets) - mesh_nodes):
        errs.append("extra animated node %r" % nodes[idx].get("name"))

    want_max = (rec["frames"] - 1) / FPS
    if not anim.get("samplers"):
        errs.append("no samplers")
    for s_idx, sampler in enumerate(anim.get("samplers", [])):
        if sampler.get("interpolation", "LINEAR") != "LINEAR":
            errs.append("sampler %d interpolation is not LINEAR" % s_idx)
        acc = j["accessors"][sampler["input"]]
        tmin = acc.get("min", [float("nan")])[0]
        tmax = acc.get("max", [float("nan")])[0]
        if abs(tmin) > 1e-4 or abs(tmax - want_max) > 1e-3:
            errs.append("sampler %d time span %.5f..%.5f != 0..%.5f" %
                        (s_idx, tmin, tmax, want_max))
    return errs


def reload(rec):
    """Import into a clean scene; verify one Action and keyframe frame span."""
    errs = []
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.render.fps = FPS
    bpy.ops.import_scene.gltf(filepath=os.path.join(ROOT, rec["path"]))

    actions = [a for a in bpy.data.actions if a.name == rec["name"]]
    if len(actions) != 1:
        return errs + ["expected 1 imported action named %r, found %d" %
                       (rec["name"], len(actions))]
    action = actions[0]

    # The importer creates a slotted action; bind each slot so every channel
    # bundle is visible through the reloaded objects as well.
    objs = {o.name: o for o in bpy.data.objects}
    bound = 0
    for slot in action.slots:
        ident = slot.identifier[2:] if slot.identifier.startswith("OB") else slot.identifier
        o = objs.get(ident)
        if o is not None and o.animation_data is not None:
            o.animation_data.action = action
            o.animation_data.action_slot = slot
            bound += 1
    if bound != rec["primitive_count"]:
        errs.append("bound %d/%d animated nodes" % (bound, rec["primitive_count"]))

    fcurves = list(iter_fcurves(action))
    paths = {}
    for fc in fcurves:
        paths[fc.data_path] = paths.get(fc.data_path, 0) + 1
    n = rec["primitive_count"]
    # glTF stores rotation as a quaternion, so the importer re-creates four
    # rotation_quaternion curves per node (Euler authored, quaternion reloaded).
    if paths.get("location", 0) != 3 * n:
        errs.append("location fcurves %d != 3 * %d" % (paths.get("location", 0), n))
    if paths.get("scale", 0) != 3 * n:
        errs.append("scale fcurves %d != 3 * %d" % (paths.get("scale", 0), n))
    rot = 0
    if paths.get("rotation_euler", 0) == 3 * n:
        rot = 3
    elif paths.get("rotation_quaternion", 0) == 4 * n:
        rot = 4
    else:
        errs.append("rotation fcurves %s != 3/4 * %d per node" %
                    ({k: v for k, v in paths.items() if k.startswith("rotation")}, n))
    if len(fcurves) != n * (6 + rot):
        errs.append("fcurves %d != %d * (3 location + %d rotation + 3 scale)" %
                    (len(fcurves), n, rot or 3))

    frames = [kp.co.x for fc in fcurves for kp in fc.keyframe_points]
    if not frames:
        errs.append("no reloaded keyframes")
    else:
        lo, hi = min(frames), max(frames)
        if abs(lo) > 1e-4 or abs((hi + 1) - rec["frames"]) > 1e-4:
            errs.append("reloaded keys span frames %.3f..%.3f, expected 0..%d" %
                        (lo, hi, rec["frames"] - 1))
    for fc in fcurves:
        for kp in fc.keyframe_points:
            if kp.interpolation != "LINEAR":
                errs.append("non-LINEAR key on %s" % fc.data_path)
                break
    return errs


def main():
    with open(MANIFEST) as fh:
        manifest = json.load(fh)
    effects = manifest["effects"]
    print("verifying %d second-wave FX clips" % len(effects))
    failed = 0
    for rec in effects:
        errs = structural(rec, glb_json(os.path.join(ROOT, rec["path"])))
        errs += reload(rec)
        if rec["fps"] != FPS:
            errs.append("manifest fps %s != %d" % (rec["fps"], FPS))
        if abs(rec["duration_s"] - rec["frames"] / FPS) > 1e-4:
            errs.append("duration_s %s != frames/30" % rec["duration_s"])
        status = "PASS" if not errs else "FAIL"
        print("[%s] %-24s frames=%2d prims=%2d %s" %
              (status, rec["name"], rec["frames"], rec["primitive_count"],
               ("-- " + errs[0]) if errs else ""))
        for err in errs:
            print("    -", err)
        failed += 1 if errs else 0
    passed = len(effects) - failed
    print("RESULT %d/%d passed" % (passed, len(effects)))
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
