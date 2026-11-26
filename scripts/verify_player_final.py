"""Re-open every final player GLB and re-measure it against the manifest.

    blender -b --python scripts/verify_player_final.py
"""

import hashlib
import json
import os
import struct
import sys

import bpy

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "assets", "animations", "player")
MANIFEST = os.path.join(OUT, "manifest.json")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import generate_player_final as gen  # noqa: E402


def glb_json(path):
    with open(path, "rb") as fh:
        data = fh.read()
    off = 12
    length = len(data)
    while off < length:
        clen, ctype = struct.unpack_from("<II", data, off)
        off += 8
        if ctype == 0x4E4F534A:
            return json.loads(data[off:off + clen].decode("utf-8"))
        off += clen
    raise RuntimeError("no JSON chunk: " + path)


def structural(clip, path, j):
    errs = []
    anims = j.get("animations", [])
    if len(anims) != 1 or anims[0].get("name") != clip["clip"]:
        errs.append("animation name/count mismatch")
        return errs
    for ch in anims[0]["channels"]:
        if ch["target"]["path"] == "scale":
            errs.append("scale key on node %s" % ch["target"]["node"])
    nodes = {n.get("name") for n in j.get("nodes", [])}
    for key in gen.PARTS:
        if key not in nodes:
            errs.append("missing rig node " + key)
    return errs


def main():
    with open(MANIFEST) as fh:
        manifest = json.load(fh)
    bad = []
    print("verifying %d final player clips" % len(manifest["clips"]))
    for clip in manifest["clips"]:
        path = os.path.join(ROOT, clip.get("path", ""))
        if not os.path.exists(path):
            path = os.path.join(OUT, clip["clip"] + ".glb")
        errs = structural(clip, path, glb_json(path))
        bpy.ops.wm.read_factory_settings(use_empty=True)
        bpy.context.scene.render.fps = 30   # GLB times were exported at 30 fps
        bpy.ops.import_scene.gltf(filepath=path)
        objs = {o.name: o for o in bpy.data.objects}
        actions = [a for a in bpy.data.actions if a.name == clip["clip"]]
        if len(actions) != 1:
            errs.append("imported action not found")
            measured = {}
        else:
            for slot in actions[0].slots:
                ident = slot.identifier[2:] if slot.identifier.startswith("OB") else slot.identifier
                o = objs.get(ident)
                if o and o.animation_data:
                    o.animation_data.action = actions[0]
                    o.animation_data.action_slot = slot
            rig = {n: objs[n] for n in gen.PARTS if n in objs}
            meshes = {o.name: o for o in objs.values() if o.type == "MESH"}
            frames = round(clip["duration_s"] * 30) - (0 if clip["loop"] else 0)
            # loops carry one extra key; one-shots carry a settle tail.
            frames = frames - 1 if clip["loop"] else frames
            measured = gen.measure(rig, meshes, frames, clip["loop"])
            for key in ("seam_position_cm", "foot_slide_cm",
                        "ground_penetration_cm", "float_p10_cm"):
                if abs(measured[key] - clip[key]) > 1.5:
                    errs.append("%s manifest %.3f reloaded %.3f" %
                                (key, clip[key], measured[key]))
            for key, (warn, fail) in gen.GATES.items():
                if measured[key] > fail:
                    errs.append("reloaded %s %.3f > %s" % (key, measured[key], fail))
        status = "PASS" if not errs else "FAIL"
        print("[%s] %-14s %s" % (status, clip["clip"], measured))
        for e in errs:
            print("    -", e)
            bad.append((clip["clip"], e))

    player = os.path.join(ROOT, "assets", "models", "player.glb")
    with open(player, "rb") as fh:
        digest = hashlib.sha256(fh.read()).hexdigest()
    ok = digest == manifest["source"]["sha256"]
    print("source player.glb sha256 %s" % ("UNCHANGED" if ok else "CHANGED - REBIND"))
    if not ok:
        bad.append(("source", "player.glb changed since generation"))
    print("RESULT %d/%d passed" % (len(manifest["clips"]) - len({c for c, _ in bad}),
                                   len(manifest["clips"])))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
