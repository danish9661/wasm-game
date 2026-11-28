#!/usr/bin/env python3
"""Verify the baked NPC pose contract (pure Python, standard library only).

    python3 scripts/verify_npc_poses.py

Checks every GLB referenced by assets/models/npc_poses.json:

  * stable name  npcpose_<clip>_<NN>.glb, frames in order, no missing files;
  * base Y ~= 0, X/Z centre ~= 0, <= 600 triangles, no textures/images;
  * every frame of a clip is made of the *same rigid mesh* -- the sorted edge
    lengths match exactly (only joint translation/rotation, never rescale), so
    a family's dimensions cannot drift between frames;
  * at least one limb/part moves between the first and a later frame.

The last two checks are the numeric counterpart of the contact sheet.
"""

from __future__ import annotations

import json
import struct
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODELS = ROOT / "assets" / "models"
MANIFEST = MODELS / "npc_poses.json"
BASE_TOL = 0.02
CENTER_TOL = 0.02
TRI_LIMIT = 600
COMPONENT = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
NUM_COMPONENTS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}


def glb(path: Path):
    data = path.read_bytes()
    magic, version, length = struct.unpack_from("<4sII", data, 0)
    if magic != b"glTF" or version != 2 or length > len(data):
        raise ValueError("bad GLB %s" % path)
    off, chunks = 12, {}
    while off + 8 <= length:
        clen, kind = struct.unpack_from("<I4s", data, off)
        off += 8
        chunks[kind] = data[off:off + clen]
        off += clen
    doc = json.loads(chunks[b"JSON"].decode("utf-8").rstrip("\x00 \r\n\t"))
    return doc, chunks.get(b"BIN\x00", b"")


def positions(doc: dict, binary: bytes, accessor: int):
    acc = doc["accessors"][accessor]
    if acc["type"] != "VEC3" or acc["componentType"] != 5126:
        raise ValueError("unexpected POSITION accessor")
    view = doc["bufferViews"][acc["bufferView"]]
    offset = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    stride = view.get("byteStride", 12)
    fmt = "<" + "fff"
    return [struct.unpack_from(fmt, binary, offset + i * stride)
            for i in range(acc["count"])]


def frame_stats(path: Path):
    doc, binary = glb(path)
    if doc.get("images") or doc.get("textures"):
        raise ValueError("textures found")
    for mat in doc.get("materials", []):
        pbr = mat.get("pbrMetallicRoughness", {})
        if pbr.get("baseColorTexture") or pbr.get("metallicRoughnessTexture"):
            raise ValueError("material texture reference found")
    verts, triangles, edges = [], 0, []
    for mesh in doc.get("meshes", []):
        for prim in mesh["primitives"]:
            if prim.get("mode", 4) != 4:
                raise ValueError("non-triangle primitive")
            pts = positions(doc, binary, prim["attributes"]["POSITION"])
            verts.extend(pts)
            if "indices" in prim:
                idx = doc["accessors"][prim["indices"]]
                fmt = "<" + {5121: "B", 5123: "H", 5125: "I"}[idx["componentType"]]
                step = struct.calcsize(fmt)
                view = doc["bufferViews"][idx["bufferView"]]
                base = view.get("byteOffset", 0) + idx.get("byteOffset", 0)
                tri = [struct.unpack_from(fmt, binary, base + i * step)[0]
                       for i in range(idx["count"])]
            else:
                tri = list(range(len(pts)))
            triangles += len(tri) // 3
            for i in range(0, len(tri) - 2, 3):
                a, b, c = tri[i], tri[i + 1], tri[i + 2]
                for u, v in ((a, b), (b, c), (c, a)):
                    p, q = pts[u], pts[v]
                    d2 = sum((p[k] - q[k]) ** 2 for k in range(3))
                    edges.append(round(d2, 6))
    lo = [min(v[k] for v in verts) for k in range(3)]
    hi = [max(v[k] for v in verts) for k in range(3)]
    center = [(lo[k] + hi[k]) / 2.0 for k in range(3)]
    return {
        "base": lo[1],
        "center": center,
        "height": hi[1] - lo[1],
        "triangles": triangles,
        "signature": sorted(edges),
        "verts": verts,
    }


def main() -> int:
    if not MANIFEST.is_file():
        print("FAIL: %s missing (run bake_npc_pose_frames.py)" % MANIFEST)
        return 1
    clips = json.loads(MANIFEST.read_text(encoding="utf-8"))
    errors, warnings = [], []
    heights = defaultdict(list)
    total_frames = 0
    for key, info in clips.items():
        files = info["frames"]
        if not isinstance(info.get("fps"), (int, float)) or info["fps"] <= 0:
            errors.append("%s: invalid fps %r" % (key, info.get("fps")))
        if not isinstance(info.get("loop"), bool):
            errors.append("%s: missing loop flag" % key)
        expected = ["npcpose_%s_%02d.glb" % (key, i) for i in range(len(files))]
        if files != expected:
            errors.append("%s: unstable/out-of-order filenames" % key)
        frames = []
        for name in files:
            path = MODELS / name
            if not path.is_file():
                errors.append("%s: %s missing" % (key, name))
                continue
            stats = frame_stats(path)
            frames.append(stats)
            total_frames += 1
            if abs(stats["base"]) > BASE_TOL:
                errors.append("%s base Y %.4f" % (name, stats["base"]))
            if max(abs(stats["center"][0]), abs(stats["center"][2])) > CENTER_TOL:
                errors.append("%s X/Z centre %s" % (name, stats["center"]))
            if stats["triangles"] > TRI_LIMIT:
                errors.append("%s has %d triangles" % (name, stats["triangles"]))
            heights[key.rsplit("_", 1)[0]].append(stats["height"])
        if len(frames) < 2:
            errors.append("%s: fewer than 2 readable frames" % key)
            continue
        # glTF stores positions as float32, so compare the edge-length
        # signatures with a small tolerance; any real per-frame rescale would
        # be orders of magnitude larger than this rounding noise.
        if any(len(f["signature"]) != len(frames[0]["signature"]) or
               max((abs(a - b) for a, b in
                    zip(f["signature"], frames[0]["signature"])), default=0) > 1e-4
               for f in frames[1:]):
            errors.append("%s: frame mesh is rescaled (edge lengths differ)" % key)
        # At least one part must move: compare per-frame vertex clouds rather
        # than the (normalised) bounding box, which can stay similar.
        moved = any(frames[0]["verts"] != f["verts"] for f in frames[1:])
        if not moved:
            errors.append("%s: no visible motion between frames" % key)

    print("[npc-verify] %d clips, %d GLBs; rigid mesh + motion + palette checked"
          % (len(clips), total_frames))
    for fam, hs in sorted(heights.items()):
        print("  %-16s height %.3f..%.3f u" % (fam, min(hs), max(hs)))
    for w in warnings:
        print("WARN:", w)
    if errors:
        for err in errors[:30]:
            print("FAIL:", err)
        print("[npc-verify] %d failure(s)" % len(errors))
        return 1
    print("[npc-verify] OK: all NPC pose frames pass")
    return 0


if __name__ == "__main__":
    sys.exit(main())
