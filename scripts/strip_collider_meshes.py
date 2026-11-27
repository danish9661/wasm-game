#!/usr/bin/env python3
"""Strip renderable collision-proxy geometry from a runtime GLB.

The skill pipeline (gam.finish) exports a `_col` mesh node so gameplay can
query collision. The web 3D renderer walks EVERY glTF node and draws each
mesh primitive with its material base colour; a collider primitive has no
material, so it is drawn opaque white and completely hides the detailed
`<id>_lod0` render mesh. That is the source of the large white cubes in the
deployed village screenshot.

This keeps the collider NODE (name + `ig_collider`/extras intact) but removes
its `mesh` reference, so the proxy stays discoverable by name while no longer
being drawn. Geometry data is left in the binary chunk (harmless, and it keeps
accessor indices stable); only the scene graph changes.

Usage:
    python3 scripts/strip_collider_meshes.py assets/models/*.glb
"""
from __future__ import annotations

import json
import struct
import sys
from pathlib import Path

JSON_CHUNK = 0x4E4F534A


def is_collider(node: dict) -> bool:
    # Only genuine gam.finish proxies carry this extra. Names ending in _col
    # are NOT sufficient: town_hall has four real columns named colonnade_col.
    return "ig_collider" in node.get("extras", {})


def strip_file(path: str | Path) -> int:
    path = Path(path)
    blob = path.read_bytes()
    if blob[:4] != b"glTF":
        raise ValueError("%s: not a GLB" % path)
    total = struct.unpack_from("<I", blob, 8)[0]
    off = 12
    chunks = []
    while off < total:
        clen, ctype = struct.unpack_from("<II", blob, off)
        chunks.append((ctype, blob[off + 8: off + 8 + clen]))
        off += 8 + clen
    if not chunks or chunks[0][0] != JSON_CHUNK:
        raise ValueError("%s: missing JSON chunk" % path)
    doc = json.loads(chunks[0][1].decode("utf-8").rstrip(" \x00"))
    stripped = 0
    for node in doc.get("nodes", []):
        if is_collider(node) and "mesh" in node:
            node.pop("mesh")
            stripped += 1
    if stripped == 0:
        return 0
    js = json.dumps(doc, separators=(",", ":")).encode("utf-8")
    js += b" " * ((4 - len(js) % 4) % 4)
    out = bytearray(b"glTF" + struct.pack("<II", 2, 0))
    for ctype, data in [(JSON_CHUNK, js)] + chunks[1:]:
        out.extend(struct.pack("<II", len(data), ctype))
        out.extend(data)
    struct.pack_into("<I", out, 8, len(out))
    path.write_bytes(out)
    return stripped


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: strip_collider_meshes.py <glb> [glb ...]")
        return 1
    for arg in sys.argv[1:]:
        n = strip_file(arg)
        print("STRIPPED_COLLIDER", arg, n)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
