"""Re-open every second-wave GLB and re-measure it against its manifest.

    blender -b --python scripts/verify_wave2_animations.py

Every clip is imported into a clean scene at 30 fps, its slotted glTF action is
bound back to the matching node, and the generator's own gates are re-run on
the reloaded transforms.  The source model bounds are re-derived independently
so the contact tips are not read from the manifest.
"""

import json
import os
import struct
import sys

import bpy

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "assets", "animations", "wave2")
MANIFEST = os.path.join(OUT, "manifest.json")
FPS = 30

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import generate_wave2_animations as gen  # noqa: E402


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


def find_pivot(objs, name):
    """Blender suffixes an empty when a mesh node already owns the name."""
    candidates = [name] + ["%s.%03d" % (name, i) for i in range(1, 20)]
    for cand in candidates:
        o = objs.get(cand)
        if o is not None and o.type == "EMPTY":
            return o
    return None


def structural(clip, j):
    errs = []
    anims = j.get("animations", [])
    if len(anims) != 1 or anims[0].get("name") != clip["clip"]:
        errs.append("animation name/count mismatch")
        return errs
    for ch in anims[0]["channels"]:
        if ch["target"]["path"] == "scale":
            errs.append("scale key on node %s" % ch["target"]["node"])
    nodes = {n.get("name") for n in j.get("nodes", [])}
    for name in ["root"] + [p[0] for p in gen.RIGS[clip["model"]]]:
        if not any(n == name or n.startswith(name + ".") for n in nodes):
            errs.append("missing rig node " + name)
    for sampler in anims[0]["samplers"]:
        acc = j["accessors"][sampler["input"]]
        if abs(acc.get("min", [0])[0]) > 1e-4 or \
           abs(acc.get("max", [0])[0] - (clip["frames"] - 1) / FPS) > 1e-3:
            errs.append("sampler time range != 0..%d frames at 30 fps" %
                        (clip["frames"] - 1))
    return errs


def reload_and_measure(clip):
    """Import the GLB, bind its action slots, re-measure with generator gates."""
    # Re-derive contact tips from the *source* model's real bounds.
    src = gen.build_rig(clip["model"])
    tips = {k: src["tips"][k] for k in clip["contacts"]}
    contacts = list(clip["contacts"])

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.render.fps = FPS
    path = os.path.join(ROOT, clip["path"])
    bpy.ops.import_scene.gltf(filepath=path)
    objs = {o.name: o for o in bpy.data.objects}
    errs = []
    actions = [a for a in bpy.data.actions if a.name == clip["clip"]]
    if len(actions) != 1:
        return errs + ["imported action not found"], {}
    for slot in actions[0].slots:
        ident = slot.identifier[2:] if slot.identifier.startswith("OB") else slot.identifier
        o = objs.get(ident)
        if o and o.animation_data:
            o.animation_data.action = actions[0]
            o.animation_data.action_slot = slot
    names = ["root"] + [p[0] for p in gen.RIGS[clip["model"]]]
    empties = {n: find_pivot(objs, n) for n in names}
    if any(v is None for v in empties.values()):
        errs.append("imported scene missing rig nodes: %s" %
                    [n for n, v in empties.items() if v is None])
        return errs, {}
    info = {"empties": empties,
            "meshes": {o.name: o for o in objs.values() if o.type == "MESH"},
            "tips": tips, "contacts": contacts}
    measured = gen.measure(info, clip["frames"], clip["loop"])
    return errs, measured


def main():
    with open(MANIFEST) as fh:
        manifest = json.load(fh)
    bad = []
    clips = manifest["clips"]
    print("verifying %d second-wave clips" % len(clips))
    for clip in clips:
        path = os.path.join(ROOT, clip["path"])
        errs = structural(clip, glb_json(path))
        errs, measured = reload_and_measure(clip)
        for key, tol in (("seam_position_cm", 0.5), ("foot_slide_cm", 1.5),
                         ("ground_penetration_cm", 1.0), ("float_p10_cm", 1.0),
                         ("end_settled_mps", 0.5)):
            if abs(measured[key] - clip[key]) > tol:
                errs.append("%s manifest %.3f reloaded %.3f" %
                            (key, clip[key], measured[key]))
        fails, warns = gen.gate(measured, clip["loop"], clip["clip"])
        errs += ["reloaded " + x for x in fails]
        status = "PASS" if not errs else "FAIL"
        print("[%s] %-26s slide=%.3fcm pen=%.2fcm off=%.3fm%s" %
              (status, clip["clip"], measured["foot_slide_cm"],
               measured["ground_penetration_cm"],
               measured["root_offset_m"]["magnitude"],
               "  warns=" + str(warns) if warns else ""))
        for e in errs:
            print("    -", e)
            bad.append((clip["clip"], e))

    for model, info in sorted(manifest["models"].items()):
        path = os.path.join(ROOT, info["glb"])
        with open(path, "rb") as fh:
            import hashlib
            digest = hashlib.sha256(fh.read()).hexdigest()
        if digest != info["sha256"]:
            bad.append((model, "source GLB changed since generation"))
    print("RESULT %d/%d passed" % (len(clips) - len({c for c, _ in bad}), len(clips)))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
