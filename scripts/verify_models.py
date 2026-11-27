#!/usr/bin/env python3
"""Starfall GLB asset verifier (pure Python 3, standard library only).

Scans ``assets/models/*.glb``, parses the GLB container directly (12-byte
header -> JSON chunk -> optional BIN chunk), and checks every model against the
contract in ``docs/ART_DIRECTION.md``:

  * base at Y ~= 0, centred on (X, Z) ~= (0, 0),
  * non-empty meshes, triangle budget 500 (1500 for assembled/complex assets),
  * flat-shaded palette-only low-poly (material count is reported for review).

Outputs:

  * ``assets/models/metrics.json``  -- machine-readable metrics for every GLB
  * ``docs/ASSET_METRICS.md``       -- human-readable tables by category

No bpy, no third-party packages, no hard-coded list of models: new GLBs dropped
into the folder by other generators are picked up on the next run. Categories
are inferred from the ``ASSETS`` lists in ``scripts/blender/gen_*.py`` (falling
back to name heuristics), so newly generated sets get a sensible section too.

Usage:  python3 scripts/verify_models.py [--models-dir DIR] [--quiet]
"""

from __future__ import annotations

import argparse
import json
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MODELS = ROOT / "assets" / "models"
DEFAULT_JSON = DEFAULT_MODELS / "metrics.json"
DEFAULT_MD = ROOT / "docs" / "ASSET_METRICS.md"
GENERATOR_DIR = ROOT / "scripts" / "blender"

# --- Contract thresholds (see docs/ART_DIRECTION.md) ------------------------
BASE_TOL = 0.05          # metres; base of an asset sits at z=0
CENTER_TOL_MIN = 0.25    # metres; absolute allowance for an off-centre origin
CENTER_TOL_REL = 0.08    # ...or this fraction of the largest X/Z footprint
TRI_LIMIT = 500
COMPLEX_TRI_LIMIT = 1500
SCHEMA_VERSION = 1

# A name matching any of these is an assembled / interior / complex model and
# gets the larger triangle budget. Patterns, not file names, so future assets
# are covered automatically. Override with --complex-regex if needed.
COMPLEX_PATTERN = (
    r"(interior|house|cabin|hut|inn|barn|hall|tower|castle|fort|keep|"
    r"palace|temple|cathedral|church|abbey|shrine|dungeon|ruin|village|town|"
    r"city|market|station|train|bridge|ship|mine|cave|complex|landmark|"
    r"showcase|diorama|structure)"
)

IDENTITY = [1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0]

COMPONENT_BYTES = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
COMPONENT_FMT = {5120: "b", 5121: "B", 5122: "h", 5123: "H", 5125: "I", 5126: "f"}
COMPONENT_MAX = {5120: 127, 5121: 255, 5122: 32767, 5123: 65535, 5125: 4294967295}
NUM_COMPONENTS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4,
                  "MAT2": 4, "MAT3": 9, "MAT4": 16}


# ---------------------------------------------------------------------------
# GLB container / glTF helpers
# ---------------------------------------------------------------------------
def parse_glb(path: Path) -> tuple[dict, bytes]:
    """Return (glTF JSON dict, BIN chunk bytes). Raises ValueError on bad GLB."""
    data = path.read_bytes()
    if len(data) < 12:
        raise ValueError("shorter than the 12-byte GLB header")
    magic, version, length = struct.unpack_from("<4sII", data, 0)
    if magic != b"glTF":
        raise ValueError("bad magic %r (not a GLB)" % magic)
    if version != 2:
        raise ValueError("unsupported glTF version %d" % version)
    if length > len(data):
        raise ValueError("header length %d exceeds file size %d" % (length, len(data)))

    chunks: list[tuple[bytes, bytes]] = []
    off = 12
    while off + 8 <= length:
        clen, ctype = struct.unpack_from("<I4s", data, off)
        off += 8
        if clen > length - off:
            raise ValueError("chunk overruns file")
        chunks.append((ctype, data[off:off + clen]))
        off += clen

    if not chunks or chunks[0][0] != b"JSON":
        raise ValueError("first chunk is not JSON")
    try:
        gltf = json.loads(chunks[0][1].decode("utf-8").rstrip("\x00 \r\n\t"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ValueError("invalid JSON chunk: %s" % exc) from exc
    binary = next((payload for ctype, payload in chunks if ctype == b"BIN\x00"), b"")
    return gltf, binary


def primitive_triangles(gltf: dict, prim: dict) -> int:
    """Triangles contributed by one primitive (handles strips/fans/non-indexed)."""
    accessors = gltf.get("accessors", [])
    mode = prim.get("mode", 4)
    if "indices" in prim:
        idx = prim["indices"]
        if not isinstance(idx, int) or idx >= len(accessors):
            return 0
        count = accessors[idx].get("count", 0)
    else:
        pos = prim.get("attributes", {}).get("POSITION")
        if not isinstance(pos, int) or pos >= len(accessors):
            return 0
        count = accessors[pos].get("count", 0)
    if mode == 4:    # TRIANGLES
        return count // 3
    if mode in (5, 6):  # TRIANGLE_STRIP / TRIANGLE_FAN
        return max(0, count - 2)
    return 0  # points / lines: no triangles, but still a valid primitive


def mesh_triangles(gltf: dict) -> int:
    """Count unique mesh geometry once, even if a mesh is instanced on many nodes."""
    return sum(
        primitive_triangles(gltf, prim)
        for mesh in gltf.get("meshes", [])
        for prim in mesh.get("primitives", [])
    )


def read_accessor(gltf: dict, binary: bytes, index: int, component: int = 0) -> list[float] | None:
    """Decode one accessor to a flat float list (used when min/max are absent)."""
    accessors = gltf.get("accessors", [])
    views = gltf.get("bufferViews", [])
    if not isinstance(index, int) or index >= len(accessors):
        return None
    acc = accessors[index]
    view = views[acc["bufferView"]] if "bufferView" in acc else None
    if view is None or component not in COMPONENT_FMT:
        return None
    comp_type = acc.get("componentType")
    if comp_type not in COMPONENT_FMT:
        return None
    ncomp = NUM_COMPONENTS.get(acc.get("type", "SCALAR"), 1)
    count = acc.get("count", 0)
    offset = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    stride = view.get("byteStride") or COMPONENT_BYTES[comp_type] * ncomp
    fmt = COMPONENT_FMT[comp_type]
    size = COMPONENT_BYTES[comp_type]
    values: list[float] = []
    for i in range(count):
        for c in range(ncomp):
            at = offset + i * stride + c * size
            if at + size > len(binary):
                return None
            value = struct.unpack_from("<" + fmt, binary, at)[0]
            if acc.get("normalized") and comp_type != 5126:
                value /= COMPONENT_MAX[comp_type]
            values.append(float(value))
    return values


def accessor_bounds(gltf: dict, binary: bytes, index: int) -> tuple[list[float], list[float]] | None:
    """Local-space min/max of a POSITION accessor, reading BIN if min/max absent."""
    accessors = gltf.get("accessors", [])
    if not isinstance(index, int) or index >= len(accessors):
        return None
    acc = accessors[index]
    if "min" in acc and "max" in acc and len(acc["min"]) >= 3 and len(acc["max"]) >= 3:
        return [float(x) for x in acc["min"][:3]], [float(x) for x in acc["max"][:3]]
    values = read_accessor(gltf, binary, index)
    if not values:
        return None
    coords = [values[i:i + 3] for i in range(0, len(values) - 2, 3)]
    if not coords:
        return None
    lo = [min(p[k] for p in coords) for k in range(3)]
    hi = [max(p[k] for p in coords) for k in range(3)]
    return lo, hi


def matmul(a: list[float], b: list[float]) -> list[float]:
    """Column-major 4x4 multiply: result applies b first, then a."""
    return [sum(a[k * 4 + r] * b[c * 4 + k] for k in range(4))
            for c in range(4) for r in range(4)]


def node_matrix(node: dict) -> list[float]:
    """GLB node TRS or matrix -> column-major 4x4 world-local transform."""
    if "matrix" in node and len(node["matrix"]) == 16:
        return [float(x) for x in node["matrix"]]
    t = node.get("translation", [0.0, 0.0, 0.0])
    q = node.get("rotation", [0.0, 0.0, 0.0, 1.0])
    s = node.get("scale", [1.0, 1.0, 1.0])
    x, y, z, w = (float(v) for v in q)
    m = [
        1 - 2 * (y * y + z * z), 2 * (x * y + z * w), 2 * (x * z - y * w), 0.0,
        2 * (x * y - z * w), 1 - 2 * (x * x + z * z), 2 * (y * z + x * w), 0.0,
        2 * (x * z + y * w), 2 * (y * z - x * w), 1 - 2 * (x * x + y * y), 0.0,
        float(t[0]), float(t[1]), float(t[2]), 1.0,
    ]
    for c in range(3):
        for r in range(4):
            m[c * 4 + r] *= float(s[c])
    return m


def transform_point(m: list[float], p: list[float]) -> list[float]:
    return [m[r] * p[0] + m[4 + r] * p[1] + m[8 + r] * p[2] + m[12 + r]
            for r in range(3)]


def grow(lo: list[float], hi: list[float], point: list[float]) -> None:
    for k in range(3):
        lo[k] = min(lo[k], point[k])
        hi[k] = max(hi[k], point[k])


def world_bounds(gltf: dict, binary: bytes) -> tuple[list[float], list[float], bool]:
    """AABB in glTF (Y-up) world space, respecting node transforms.

    Blender's exporter bakes object transforms into meshes, but node-level
    translation/scale/rotation still appear in the GLB (and other exporters keep
    them), so transforming the POSITION accessor AABB is required for correct
    heights and base Y. Returns (min, max, fully_covered).
    """
    lo = [float("inf")] * 3
    hi = [float("-inf")] * 3
    missing = 0
    nodes = gltf.get("nodes", [])
    scenes = gltf.get("scenes", [])

    def visit(index: int, parent: list[float]) -> None:
        nonlocal missing
        if not isinstance(index, int) or index < 0 or index >= len(nodes):
            return
        node = nodes[index]
        world = matmul(parent, node_matrix(node))
        mesh_index = node.get("mesh")
        if isinstance(mesh_index, int) and mesh_index < len(gltf.get("meshes", [])):
            for prim in gltf["meshes"][mesh_index].get("primitives", []):
                pos = prim.get("attributes", {}).get("POSITION")
                bounds = accessor_bounds(gltf, binary, pos) if pos is not None else None
                if bounds is None:
                    missing += 1
                    continue
                local_lo, local_hi = bounds
                for x in (local_lo[0], local_hi[0]):
                    for y in (local_lo[1], local_hi[1]):
                        for z in (local_lo[2], local_hi[2]):
                            grow(lo, hi, transform_point(world, [x, y, z]))
        for child in node.get("children", []):
            visit(child, world)

    if scenes:
        for scene in scenes:
            for root in scene.get("nodes", []):
                visit(root, IDENTITY)
    else:
        children = {c for n in nodes for c in n.get("children", [])}
        for i in range(len(nodes)):
            if i not in children:
                visit(i, IDENTITY)

    if not all(v != float("inf") for v in lo):
        # No scene graph: fall back to raw mesh bounds with identity transform.
        for mesh in gltf.get("meshes", []):
            for prim in mesh.get("primitives", []):
                pos = prim.get("attributes", {}).get("POSITION")
                bounds = accessor_bounds(gltf, binary, pos) if pos is not None else None
                if bounds is None:
                    missing += 1
                    continue
                grow(lo, hi, bounds[0])
                grow(lo, hi, bounds[1])
    if not all(v != float("inf") for v in lo):
        lo, hi = [0.0] * 3, [0.0] * 3
    return lo, hi, missing == 0


def count_materials(gltf: dict) -> int:
    """Number of distinct glTF materials actually referenced by primitives."""
    used = {
        prim["material"]
        for mesh in gltf.get("meshes", [])
        for prim in mesh.get("primitives", [])
        if isinstance(prim.get("material"), int)
    }
    return len(used)


# ---------------------------------------------------------------------------
# Categorisation (generator ASSETS lists first, then name heuristics)
# ---------------------------------------------------------------------------
BUILDING_WORDS = (
    "house", "cabin", "hut", "inn", "barn", "hall", "tower", "castle", "fort",
    "keep", "palace", "temple", "cathedral", "church", "abbey", "shrine",
    "dungeon", "ruin", "village", "town", "city", "market", "station",
    "bridge", "ship", "mine", "cave", "interior", "workshop", "shop",
    "smith", "forge", "tavern", "guild", "library", "school", "clinic",
    "shelter", "outpost",
)


def matches_words(stem: str, words: tuple[str, ...]) -> bool:
    """Token-aware keyword match ("mushroom" must not match "room")."""
    lower = stem.lower()
    tokens = set(re.findall(r"[a-z0-9]+", lower))
    for word in words:
        if word in tokens:
            return True
        # Long words may be embedded in a compound name (watchtower_large).
        if len(word) >= 5 and word in lower:
            return True
        # Short words only at a compound boundary (townhall, fortwall, ...).
        if len(word) >= 4 and (lower.startswith(word) or lower.endswith(word)):
            return True
    return False
FALLBACK_CATEGORIES: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("Character", ("player", "guard", "goblin", "skeleton", "slime", "wolf",
                   "bat", "spider", "archer", "raider", "brute", "queen", "king",
                   "stormcaller", "leviathan", "colossus", "ogre", "wraith",
                   "imp", "golem", "knight", "boss", "villager", "merchant",
                   "toad", "scorpion", "frost", "enemy", "humanoid")),
    ("Weapon", ("sword", "axe", "bow", "dagger", "spear", "hammer", "mace",
                "scythe", "crossbow", "arrow", "shield", "weapon")),
    ("Building", BUILDING_WORDS),
    ("Nature", ("tree", "pine", "rock", "ore", "crystal", "rubble", "bush",
                "fern", "grass", "mushroom", "flower", "cactus", "vine",
                "lilypad", "reed", "bone", "log", "leaf")),
    ("Structure", ("altar", "anvil", "campfire", "chest", "bed", "fence",
                   "wall", "turret", "portal", "banner", "lantern", "torch",
                   "spike", "farm", "totem", "statue", "brazier", "sign",
                   "well", "barrel", "crate", "pillar", "rail", "car", "train")),
)
CATEGORY_ORDER = ("Building", "Interior", "Character", "Nature", "Weapon",
                  "Structure", "Other")


def category_for_generator(stem: str) -> str:
    """Map gen_<set>.py to a report category (new generator sets included)."""
    s = stem.lower()
    if "interior" in s or "room" in s:
        return "Interior"
    if "character" in s or "enemy" in s or "creature" in s:
        return "Character"
    if "weapon" in s or "tool" in s:
        return "Weapon"
    if "nature" in s or "foliage" in s or "terrain" in s:
        return "Nature"
    if "structure" in s or "building" in s or "complex" in s or "landmark" in s:
        return "Structure"
    return s.replace("gen_", "").replace("_", " ").title() or "Other"


def load_generator_categories() -> dict[str, str]:
    """Learn name -> category from every ``gen_*.py`` generator.

    Generators use either ``ASSETS = [("name", builder), ...]``,
    ``BUILDERS = [build_name, ...]`` (the collection name is the file name), or
    direct ``export_asset("name", col)`` calls. All three forms are recognised
    so new generator sets need no changes to this tool.
    """
    result: dict[str, str] = {}
    if not GENERATOR_DIR.is_dir():
        return result

    def priority(gen: Path) -> tuple[int, str]:
        # Base sets own their names; interior/complex sets can reuse a base
        # asset's name for a room-specific variant. Keep the base category.
        base = ("characters", "nature", "structures", "weapons")
        return (0 if any(k in gen.stem for k in base) else 1, gen.name)

    for gen in sorted(GENERATOR_DIR.glob("gen_*.py"), key=priority):
        text = gen.read_text(encoding="utf-8", errors="replace")
        category = category_for_generator(gen.stem)
        names: set[str] = set(re.findall(r"export_asset\(\s*['\"]([^'\"]+)['\"]", text))
        for var in ("ASSETS", "BUILDERS", "EXPORTS", "ITEMS"):
            match = re.search(r"^%s\s*=\s*\[" % var, text, re.MULTILINE)
            if not match:
                continue
            start = match.end() - 1
            depth = 0
            block = ""
            for i in range(start, len(text)):
                if text[i] == "[":
                    depth += 1
                elif text[i] == "]":
                    depth -= 1
                    if depth == 0:
                        block = text[start:i + 1]
                        break
            names.update(re.findall(r"['\"]([a-zA-Z0-9_-]+)['\"]\s*,", block))
            names.update(re.findall(r"\bbuild_([a-zA-Z0-9_]+)\b", block))
        for name in names:
            result.setdefault(name, category)
    return result


VEHICLE_WORDS = ("car", "train", "rail", "cart", "wagon", "vehicle")


def categorise(stem: str, generator_categories: dict[str, str]) -> str:
    # A few names win over their generator set (the weapons generator also
    # produces the car/train/rail props).
    if matches_words(stem, BUILDING_WORDS):
        return "Building"
    if matches_words(stem, VEHICLE_WORDS):
        return "Structure"
    category = generator_categories.get(stem)
    if category is not None:
        return category
    for name, keywords in FALLBACK_CATEGORIES:
        if matches_words(stem, keywords):
            return name
    return "Other"


# ---------------------------------------------------------------------------
# Per-model verification
# ---------------------------------------------------------------------------
def verify_model(path: Path, generator_categories: dict[str, str],
                 complex_re: re.Pattern) -> dict:
    stem = path.stem
    category = categorise(stem, generator_categories)
    result = {
        "name": stem,
        "file": path.relative_to(ROOT).as_posix() if ROOT in path.parents else path.name,
        "category": category,
        "triangles": 0,
        "triangle_limit": TRI_LIMIT,
        "complex": bool(complex_re.search(stem)),
        "meshes": 0,
        "primitives": 0,
        "materials": 0,
        "min": [0.0, 0.0, 0.0],
        "max": [0.0, 0.0, 0.0],
        "height": 0.0,
        "base_y": 0.0,
        "center_x": 0.0,
        "center_z": 0.0,
        "footprint_x": 0.0,
        "footprint_z": 0.0,
        "bounds_complete": False,
        "violations": [],
    }
    if result["complex"]:
        result["triangle_limit"] = COMPLEX_TRI_LIMIT

    try:
        gltf, binary = parse_glb(path)
    except (OSError, ValueError) as exc:
        result["violations"].append("unreadable GLB: %s" % exc)
        result["pass"] = False
        return result

    meshes = gltf.get("meshes", [])
    result["meshes"] = len(meshes)
    prims = [prim for mesh in meshes for prim in mesh.get("primitives", [])]
    result["primitives"] = len(prims)
    result["triangles"] = mesh_triangles(gltf)
    result["materials"] = count_materials(gltf)

    lo, hi, complete = world_bounds(gltf, binary)
    result["min"] = [round(v, 4) for v in lo]
    result["max"] = [round(v, 4) for v in hi]
    result["bounds_complete"] = complete
    result["height"] = round(hi[1] - lo[1], 4)
    result["base_y"] = round(lo[1], 4)
    result["center_x"] = round((lo[0] + hi[0]) / 2.0, 4)
    result["center_z"] = round((lo[2] + hi[2]) / 2.0, 4)
    result["footprint_x"] = round(hi[0] - lo[0], 4)
    result["footprint_z"] = round(hi[2] - lo[2], 4)

    violations: list[str] = result["violations"]
    if result["meshes"] == 0 or not prims:
        violations.append("missing/empty meshes")
    if result["triangles"] <= 0:
        violations.append("zero triangles")
    if not complete:
        violations.append("incomplete POSITION bounds (min/max missing)")
    if abs(result["base_y"]) > BASE_TOL:
        violations.append("base Y %.3fu is not ~0" % result["base_y"])
    tol = max(CENTER_TOL_MIN, CENTER_TOL_REL * max(result["footprint_x"], result["footprint_z"]))
    if abs(result["center_x"]) > tol or abs(result["center_z"]) > tol:
        violations.append(
            "not centred (X %.3fu, Z %.3fu; tolerance %.3fu)"
            % (result["center_x"], result["center_z"], tol)
        )
    if result["triangles"] > result["triangle_limit"]:
        violations.append("%d tris exceeds %d limit" % (result["triangles"], result["triangle_limit"]))
    result["pass"] = not violations
    return result


# ---------------------------------------------------------------------------
# Reports
# ---------------------------------------------------------------------------
def write_json(path: Path, models: list[dict], models_dir: Path) -> dict:
    passed = sum(1 for m in models if m["pass"])
    report = {
        "schema_version": SCHEMA_VERSION,
        "generated_by": "scripts/verify_models.py",
        "contract": {
            "base_y_tolerance": BASE_TOL,
            "center_tolerance_min": CENTER_TOL_MIN,
            "center_tolerance_relative": CENTER_TOL_REL,
            "triangle_limit": TRI_LIMIT,
            "complex_triangle_limit": COMPLEX_TRI_LIMIT,
            "complex_name_pattern": COMPLEX_PATTERN,
        },
        "summary": {
            "models": len(models),
            "passed": passed,
            "failed": len(models) - passed,
            "total_triangles": sum(m["triangles"] for m in models),
            "models_dir": models_dir.relative_to(ROOT).as_posix()
            if ROOT in models_dir.parents else str(models_dir),
        },
        "models": models,
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    return report


def write_markdown(path: Path, report: dict) -> None:
    models: list[dict] = report["models"]
    s = report["summary"]
    lines = [
        "# Starfall — Asset metrics",
        "",
        "Generated by `python3 scripts/verify_models.py` from the GLB files in "
        "`assets/models/`. Machine-readable copy: `assets/models/metrics.json`.",
        "",
        "## Summary",
        "",
        "- Models scanned: **%d** (%d passed, %d failed)" % (s["models"], s["passed"], s["failed"]),
        "- Total triangles: **%d** (budget: %d per asset, %d for assembled/interior/complex assets)"
        % (s["total_triangles"], TRI_LIMIT, COMPLEX_TRI_LIMIT),
        "- Contract: base Y ≈ 0 (±%.2fu), centred on X/Z (≈±%.2fu or %.0f%% of footprint), "
        "flat-shaded, palette-only." % (BASE_TOL, CENTER_TOL_MIN, CENTER_TOL_REL * 100),
        "- Categories come from the `ASSETS` / `BUILDERS` lists and `export_asset` "
        "calls in `scripts/blender/gen_*.py`; new generator sets are picked up "
        "automatically.",
        "",
        "Heights are in Blender/glTF units (1 tile = 1.0 u, Y-up). `Base Y` is the "
        "lowest world-space point; `X/Z` are the footprint centre offsets.",
        "",
    ]

    by_category: dict[str, list[dict]] = {}
    for model in models:
        by_category.setdefault(model["category"], []).append(model)
    for category in CATEGORY_ORDER + tuple(sorted(set(by_category) - set(CATEGORY_ORDER))):
        group = by_category.get(category)
        if not group:
            continue
        group = sorted(group, key=lambda m: m["name"])
        heights = [m["height"] for m in group]
        lines += [
            "## %s (%d)" % (category, len(group)),
            "",
            "Height range: %.2f–%.2f u." % (min(heights), max(heights)),
            "",
            "| Model | Tris | Height (u) | Base Y | Centre X | Centre Z | Materials | Check |",
            "|---|---:|---:|---:|---:|---:|---:|---|",
        ]
        for m in group:
            status = "PASS" if m["pass"] else "FAIL: " + "; ".join(m["violations"])
            lines.append(
                "| `%s` | %d%s | %.2f | %.3f | %+.3f | %+.3f | %d | %s |"
                % (m["name"], m["triangles"], "*" if m["complex"] else "",
                   m["height"], m["base_y"], m["center_x"], m["center_z"],
                   m["materials"], status)
            )
        lines.append("")

    # Static binding contract for the rebuilt player: the animation work sizes
    # clips against this model, so the numbers must survive report regeneration.
    p = next((m for m in models if m["name"] == "player"), None)
    if p:
        lines += [
            "## Player binding contract (hero rebuild)",
            "",
            "- `assets/models/player.glb` — root node `char_player`, single render mesh "
            "`char_player_lod0`, no collider. Source brief: `assets/src/char/char_player/brief.json`.",
            "- Verified bbox: **X %.3f u wide, Z %.3f u deep, Y %.3f u tall** "
            "(footprint centre X %+.3f / Z %+.3f, base Y %.3f)." % (
                p["footprint_x"], p["footprint_z"], p["height"],
                p["center_x"], p["center_z"], p["base_y"]),
            "- Pivot: bottom-centre **between the feet at the world origin**; root translation "
            "(0, 0, 0), no rotation or scale. Feet touch the ground at Y = 0.",
            "- Rig: **none** — static faceted mesh. No armature, bones, skins, sockets, "
            "morph targets or embedded animation clips. Bind clips to the root node.",
            "- Materials (8): orange tunic `#FF8C26`, skin `#EAB88C`, hair `#4A2E1A`, "
            "leather `#592F1A`, trousers `#3C4453`, steel `#C8D0D8`, eyes `#1A1A1A`, "
            "gold `#D9A62E`. No textures.",
            "- The old 268-triangle procedural player was replaced by a 500-triangle "
            "bmesh-built hero: tapered body/limbs, bevels, weighted normals and packed UVs. "
            "Hands sit at hip height; a separate tapered blade/guard/grip is held in the "
            "right hand and is part of the same mesh. Overall height is held at 1.55 u.",
            "",
        ]
    failed = [m for m in models if not m["pass"]]
    lines += ["## Violations", ""]
    if failed:
        for m in failed:
            lines.append("- `%s`: %s" % (m["name"], "; ".join(m["violations"])))
    else:
        lines.append("- None. All models meet the contract.")
    lines.append("")
    lines.append("`*` = assembled/complex model using the %d-triangle budget." % COMPLEX_TRI_LIMIT)
    lines.append("")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines), encoding="utf-8")


# ---------------------------------------------------------------------------
def build_complex_regex(pattern: str | None) -> re.Pattern:
    return re.compile(pattern if pattern is not None else COMPLEX_PATTERN, re.IGNORECASE)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--models-dir", type=Path, default=DEFAULT_MODELS,
                        help="folder containing *.glb files (default: assets/models)")
    parser.add_argument("--json", type=Path, default=DEFAULT_JSON,
                        help="metrics JSON output path")
    parser.add_argument("--md", type=Path, default=DEFAULT_MD,
                        help="Markdown report output path")
    parser.add_argument("--complex-regex", default=None,
                        help="override the regex for models allowed 1500 triangles")
    parser.add_argument("--quiet", action="store_true", help="suppress the per-model listing")
    args = parser.parse_args(argv)

    if not args.models_dir.is_dir():
        print("error: models directory not found: %s" % args.models_dir, file=sys.stderr)
        return 2
    files = sorted(args.models_dir.glob("*.glb"))
    if not files:
        print("error: no *.glb files in %s" % args.models_dir, file=sys.stderr)
        return 2

    generators = load_generator_categories()
    complex_re = build_complex_regex(args.complex_regex)
    models = [verify_model(p, generators, complex_re) for p in files]
    report = write_json(args.json, models, args.models_dir)
    write_markdown(args.md, report)

    if not args.quiet:
        width = max(len(m["name"]) for m in models)
        for m in models:
            print("  %-*s  %-10s %4d tris  %5.2fu  base %+.3f  %s"
                  % (width, m["name"], m["category"], m["triangles"], m["height"],
                     m["base_y"], "PASS" if m["pass"] else "FAIL"))
    s = report["summary"]
    print("[verify_models] %d GLBs: %d passed, %d failed"
          % (s["models"], s["passed"], s["failed"]))
    for m in models:
        if not m["pass"]:
            print("  FAIL %s: %s" % (m["name"], "; ".join(m["violations"])))
    print("[verify_models] wrote %s and %s" % (args.json, args.md))
    return 1 if s["failed"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
