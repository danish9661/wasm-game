#!/usr/bin/env python3
"""Lift the root node of a GLB by a Y offset, rewriting only the JSON chunk.

Used when a held item is authored with its pivot at the grip (so Babylon's
first-person review frames it correctly) but the shipped asset in
assets/models/ must satisfy the Starfall contract "base at Y = 0". The mesh
and every other node are untouched; only the single root node translation
changes, so the base-at-zero / centred pivot contract is preserved.

Usage: python3 scripts/lift_glb.py IN.glb OUT.glb 0.118
"""
from __future__ import annotations

import json
import struct
import sys
from pathlib import Path


def lift(src: Path, dst: Path, dy: float) -> None:
    data = src.read_bytes()
    magic, version, length = struct.unpack_from("<4sII", data, 0)
    if magic != b"glTF" or version != 2:
        raise SystemExit("not a GLB v2: %s" % src)
    chunks = []
    off = 12
    while off + 8 <= length:
        clen, ctype = struct.unpack_from("<I4s", data, off)
        off += 8
        chunks.append([ctype, bytearray(data[off:off + clen])])
        off += clen
    js = json.loads(chunks[0][1].decode("utf-8").rstrip("\x00 \r\n\t"))
    scene = js["scenes"][js.get("scene", 0)]
    for idx in scene["nodes"]:
        node = js["nodes"][idx]
        if "translation" in node:
            node["translation"][1] = round(node["translation"][1] + dy, 6)
        elif "matrix" in node:
            node["matrix"][13] = round(node["matrix"][13] + dy, 6)
        else:
            node["translation"] = [0.0, dy, 0.0]
    raw = json.dumps(js, separators=(",", ":")).encode("utf-8")
    raw += b" " * ((-len(raw)) % 4)
    chunks[0][1] = bytearray(raw)
    out = bytearray(struct.pack("<4sII", b"glTF", 2, 0))
    for ctype, payload in chunks:
        out += struct.pack("<I4s", len(payload), ctype) + payload
    struct.pack_into("<I", out, 8, len(out))
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(out)


if __name__ == "__main__":
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    lift(Path(sys.argv[1]), Path(sys.argv[2]), float(sys.argv[3]))
