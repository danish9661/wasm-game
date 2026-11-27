"""Wave-2 rebuild of the remaining nature/biome visual-QA offenders.

Runs headless in Blender. For each model it writes a brief, builds bmesh parts,
bevels, packs UV0, applies weighted normals via gam.finish, exports through the
gam pipeline and copies the GLB to assets/models/<name>.glb (base at z=0,
footprint centred). Khronos validation and Babylon review are then run on each
brief with gam_cli.py (see docs/VISUAL_QA_NATURE.md).

Usage:
    BLENDER=~/.local/blender/blender python3 -m ...   (or just run blender -b)
    ~/.local/blender/blender -b --python scripts/blender/rebuild_nature_wave2.py
"""
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SKILL = os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts")
sys.path.insert(0, SKILL)
sys.path.insert(0, os.path.join(ROOT, "assets", "src"))

import bpy
import gam
import _family_hero as F
import _family_nature as N

from mathutils import Vector

# (stem, dims x/y/z, builder key)
TARGETS = [
    ("coral_brain", (0.95, 0.90, 0.82), "coral_brain"),
    ("wheat",       (0.70, 0.65, 1.05), "wheat"),
    ("coral_fan",   (0.85, 0.70, 0.92), "coral_fan"),
    ("flower",      (0.55, 0.50, 0.64), "flower"),
    ("shell_cluster", (0.70, 0.60, 0.52), "shell_cluster"),
    ("hay_bale",    (0.96, 0.72, 0.70), "hay_bale"),
    ("swamp_reeds", (0.95, 0.85, 1.52), "swamp_reeds"),
    ("basalt_columns", (1.10, 0.95, 1.82), "basalt_columns"),
    ("carrot",      (0.75, 0.65, 0.58), "carrot"),
    ("lilypad",     (0.90, 0.88, 0.22), "lilypad"),
    ("pine",        (1.49, 1.10, 2.98), "pine"),
    ("volcanic_bomb", (0.85, 0.80, 0.85), "volcanic_bomb"),
    ("vines",       (0.90, 0.85, 1.05), "vines"),
    ("potato",      (1.00, 0.85, 0.58), "potato"),
    ("ice_crystal", (0.78, 0.72, 1.35), "ice_crystal"),
    ("coral",       (0.90, 0.78, 1.00), "coral"),
    ("dune_rock",   (1.05, 0.88, 0.62), "dune_rock"),
    ("grass_tuft",  (0.45, 0.42, 0.52), "grass_tuft"),
    ("fern",        (0.62, 0.55, 0.60), "fern"),
    ("reed",        (0.60, 0.52, 1.15), "reed"),
    ("herb_patch",  (0.72, 0.65, 0.48), "herb_patch"),
    ("rubble",      (0.95, 0.85, 0.46), "rubble"),
    ("haystack",    (1.02, 0.92, 1.02), "haystack"),
    ("mushroom",    (0.58, 0.52, 0.58), "mushroom"),
    ("mushroom_cluster", (0.78, 0.70, 0.64), "mushroom_cluster"),
    ("tree",        (1.29, 0.82, 2.97), "tree"),
    ("bush",        (1.05, 0.95, 0.78), "bush"),
    ("crystal",     (0.78, 0.72, 1.02), "crystal"),
]

STYLE = ("Low-Poly Diorama: chunky, faceted, flat-shaded, palette-only, "
         "readable at ~60 px. 1u = 1 tile; player 1.55u; base at z=0, "
         "centred on (0,0). Never: plain cubes, featureless blobs, smooth "
         "normals, textures, invented hues.")


def write_brief(stem, dims):
    ident = "prop_" + stem
    d = os.path.join(ROOT, "assets", "src", "props", ident)
    os.makedirs(d, exist_ok=True)
    brief = {
        "id": ident, "category": "prop", "tier": "S", "status": "done",
        "track": "invented", "source": "make",
        "what": "Visual-QA wave-2 rebuild of %s at gameplay scale." % stem,
        "dims": {"x": dims[0], "y": dims[1], "z": dims[2], "tol": 0.18,
                 "basis": "target silhouette from fresh-eyes QA sheets"},
        "pivot": "bottom_center", "budget_class": "prop",
        "density_class": "prop", "collider": "box", "props": {},
        "materials": [], "variants": [], "sockets": [],
        "views": {"primary": "iso", "why": "commander camera 45/30"},
        "review_views": ["front", "three_quarter", "eye", "iso", "feed"],
        "references": [], "style_notes": STYLE,
        "notes": "Rebuilt with bmesh parts, bevels, weighted normals, packed UVs.",
    }
    p = os.path.join(d, "brief.json")
    with open(p, "w", encoding="utf-8") as f:
        json.dump(brief, f, indent=2)
    return p


def palette():
    return {
        "sand": gam.material("m_sand", "#D9B46A", roughness=0.95),
        "sand_dark": gam.material("m_sand_dark", "#B8944E", roughness=0.95),
        "gold": gam.material("m_gold", "#D9A62E", roughness=0.9),
        "bone": gam.material("m_bone", "#E8E0CC", roughness=0.95),
        "wood": gam.material("m_wood", "#6B4423", roughness=0.95),
        "leather": gam.material("m_leather", "#592F1A", roughness=0.95),
        "leaf": gam.material("m_leaf", "#3A7A33", roughness=0.95),
        "leaf_dark": gam.material("m_leaf_dark", "#2C6327", roughness=0.95),
        "leaf_light": gam.material("m_leaf_light", "#55953F", roughness=0.95),
        "grass": gam.material("m_grass", "#5B8C3A", roughness=0.95),
        "swamp": gam.material("m_swamp", "#4A6741", roughness=0.95),
        "stone": gam.material("m_stone", "#7A7F87", roughness=0.95),
        "stone_dark": gam.material("m_stone_dark", "#5E646B", roughness=0.95),
        "stone_light": gam.material("m_stone_light", "#9BA3AB", roughness=0.95),
        "ice": gam.material("m_ice", "#A9C8E0", roughness=0.55),
        "ice_light": gam.material("m_ice_light", "#BFD9EE", roughness=0.5),
        "lava": gam.material("m_lava", "#D9561F", roughness=0.65,
                             emissive="#D9561F", emissive_strength=2.0),
        "coral": gam.material("m_coral", "#C4451A", roughness=0.9),
        "arcane": gam.material("m_arcane", "#6B3FA0", roughness=0.8,
                               emissive="#6B3FA0", emissive_strength=0.5),
        "mush": gam.material("m_mush", "#C0392B", roughness=0.9),
        "wood_light": gam.material("m_wood_light", "#7C5633", roughness=0.95),
    }


def part(name, mats, fn, bevel=0.0):
    return F.part(name, [mats[k] for k in (fn.__mats__ if hasattr(fn, "__mats__")
                                            else ("sand",))], fn, bevel=bevel)


def build(key, M):
    """Return a list of gam part objects for `key`."""
    parts = []

    def add(name, keys, fn, bevel=0.0):
        parts.append(F.part(name, [M[k] for k in keys], fn, bevel=bevel))

    if key == "coral_brain":
        add("body", ["arcane"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.30), 0.42, mat=0, seed=11, scale=(1.10, 1.0, 0.80)),
            bevel=0.008)
        def ridge(bm, i):
            a = i * math.tau / 8.0 + 0.2
            x, y = math.cos(a) * 0.38, math.sin(a) * 0.35
            N.tapered_blade(bm, (x, y, 0.10),
                            (x * 0.55 - math.sin(a) * 0.12,
                             y * 0.55 + math.cos(a) * 0.12, 0.76),
                            0.058, mat=0, n=3)
        for i in range(8):
            add("ridge_%d" % i, ["coral" if i % 2 else "bone"],
                lambda bm, i=i: ridge(bm, i))
        def groove(bm, i):
            a = i * math.tau / 4.0 + 0.7
            x, y = math.cos(a) * 0.30, math.sin(a) * 0.28
            N.tapered_blade(bm, (x, y, 0.20), (x * 0.55, y * 0.55, 0.66),
                            0.028, mat=0, n=3)
        for i in range(4):
            add("groove_%d" % i, ["wood"], lambda bm, i=i: groove(bm, i))

    elif key == "wheat":
        def stalk(bm, i):
            x = -0.24 + 0.12 * i
            y = 0.22 * ((i % 3) - 1)
            h = 0.86 + 0.06 * (i % 3)
            N.branch(bm, (x, y, 0.01), (x + 0.03, y, h), 0.018, 0.010,
                     mat=0, n=4)
            for k in range(4):
                z = h - 0.22 + k * 0.07
                side = 1 if k % 2 else -1
                N.tapered_blade(bm, (x + 0.03, y, z),
                                (x + 0.03 + side * 0.09, y + 0.02, z + 0.05),
                                0.042, mat=0, n=3)
            N.tapered_blade(bm, (x, y, 0.16), (x - 0.16, y + 0.02, 0.42),
                            0.032, mat=0, n=3)
        for i in range(5):
            add("stalk_%d" % i, ["gold" if i % 2 else "sand"],
                lambda bm, i=i: stalk(bm, i))

    elif key == "coral_fan":
        add("base", ["stone_dark"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.09), 0.20, mat=0, seed=7, scale=(1.1, 0.9, 0.6)))
        def blade(bm, i):
            a = -1.0 + i * 0.4
            N.tapered_blade(bm, (math.sin(a) * 0.10, 0.0, 0.12),
                            (math.sin(a) * 0.55, 0.02 * (i % 2), 0.86 - abs(a) * 0.16),
                            0.085, mat=0, n=3)
        for i in range(6):
            add("fan_%d" % i, ["coral" if i % 2 else "arcane"],
                lambda bm, i=i: blade(bm, i))
        for i in (1, 3, 5):
            add("polyp_%d" % i, ["bone"], lambda bm, i=i: N.angular_rock(
                bm, (math.sin(-1.0 + i * 0.4) * 0.5, 0.02, 0.74 - abs(-1.0 + i * 0.4) * 0.16),
                0.06, mat=0, seed=40 + i))

    elif key == "flower":
        add("stem", ["grass"], lambda bm: N.branch(
            bm, (0, 0, 0.01), (0, 0, 0.52), 0.030, 0.022, mat=0, n=5))
        def leaf(bm, side):
            N.tapered_blade(bm, (0, 0, 0.20 + 0.10 * side),
                            (side * 0.26, 0.04, 0.34 + 0.08 * side),
                            0.075, mat=0, n=3)
        for s in (-1, 1):
            add("leaf_%s" % s, ["leaf"], lambda bm, s=s: leaf(bm, s))
        def petal(bm, i):
            a = i * math.tau / 6.0
            N.tapered_blade(bm, (0, 0, 0.55),
                            (math.cos(a) * 0.28, math.sin(a) * 0.28, 0.59),
                            0.085, mat=0, n=3)
        for i in range(6):
            add("petal_%d" % i, ["gold" if i % 2 else "bone"],
                lambda bm, i=i: petal(bm, i))
        add("centre", ["wood"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.60), 0.10, mat=0, seed=3, scale=(1, 1, 0.7)))

    elif key == "shell_cluster":
        add("sand_base", ["sand"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.07), 0.32, mat=0, seed=2, scale=(1.1, 1.0, 0.4)))
        for i, (x, y, h, r) in enumerate(((-0.20, 0.06, 0.34, 0.14),
                                          (0.18, -0.08, 0.48, 0.16),
                                          (0.02, 0.22, 0.26, 0.11))):
            add("shell_%d" % i, ["gold" if i % 2 else "sand"],
                lambda bm, x=x, y=y, h=h, r=r: N.crystal_shard(
                    bm, (x, y, 0.10), r, h, mat=0, n=6, tilt=(0.08, -0.05), seed=i))
        add("curl", ["sand"], lambda bm: N.angular_rock(
            bm, (0.24, 0.18, 0.12), 0.12, mat=0, seed=9, scale=(1.2, 0.9, 0.6)))

    elif key == "hay_bale":
        add("bale", ["sand"], lambda bm: gam.bm_box(
            bm, (-0.48, -0.36, 0.0), (0.48, 0.36, 0.70)), bevel=0.055)
        def band(bm, x):
            gam.bm_box(bm, (x - 0.035, -0.375, 0.0), (x + 0.035, 0.375, 0.71))
        for x in (-0.26, 0.26):
            add("band_%s" % x, ["leather"], lambda bm, x=x: band(bm, x))
        def straw(bm, i, end):
            x = -0.40 + 0.13 * (i % 7)
            z = 0.08 + 0.11 * (i // 7)
            y = end * 0.365
            N.tapered_blade(bm, (x, y, z), (x + 0.05, y + end * 0.04, z + 0.06),
                            0.022, mat=0, n=3)
        for i in range(14):
            add("straw_%d" % i, ["sand_dark"],
                lambda bm, i=i: straw(bm, i, 1 if i % 2 else -1))

    elif key == "swamp_reeds":
        add("mud", ["swamp"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.06), 0.38, mat=0, seed=5, scale=(1.1, 1.0, 0.3)))
        def reed(bm, i):
            a = i * math.tau / 9.0
            x, y = math.cos(a) * 0.30, math.sin(a) * 0.26
            h = 1.02 + 0.16 * (i % 4)
            N.branch(bm, (x, y, 0.06), (x + 0.05, y, h), 0.030, 0.018,
                     mat=0, n=4)
            if i % 2 == 0:
                N.tapered_blade(bm, (x + 0.05, y, h - 0.16),
                                (x + 0.05, y, h + 0.14), 0.062, mat=0, n=4)
            else:
                N.tapered_blade(bm, (x, y, 0.22),
                                (x + 0.18, y + 0.04, 0.48), 0.055, mat=0, n=3)
        for i in range(9):
            add("reed_%d" % i, ["grass" if i % 3 else "leaf_dark"],
                lambda bm, i=i: reed(bm, i))

    elif key == "basalt_columns":
        add("base", ["stone_dark"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.06), 0.52, mat=0, seed=8, scale=(1.1, 0.95, 0.28)))
        cols = [(-0.32, 0.06, 1.72, 0.19), (0.02, -0.06, 1.38, 0.17),
                (0.34, 0.10, 0.98, 0.15), (-0.08, 0.30, 0.72, 0.14),
                (0.26, -0.28, 0.56, 0.12)]
        for i, (x, y, h, r) in enumerate(cols):
            def col(bm, x=x, y=y, h=h, r=r, i=i):
                lean = 0.05 if i == 1 else 0.0
                F.loft(bm, [F.ring(x, y, 0.02, r, r * 0.95, n=6),
                            F.ring(x + lean, y, h * 0.62, r * 0.96, r * 0.92, n=6),
                            F.ring(x + lean * 1.5, y, h,
                                   r * (0.6 if i in (2, 4) else 0.86),
                                   r * (0.6 if i in (2, 4) else 0.86), n=6)], mat=0)
            add("column_%d" % i, ["stone" if i % 2 else "stone_dark"], col)
        for i, (x, y) in enumerate(((-0.44, -0.24), (0.42, 0.28), (0.10, 0.42))):
            add("rubble_%d" % i, ["stone_light"], lambda bm, x=x, y=y: N.angular_rock(
                bm, (x, y, 0.05), 0.11, mat=0, seed=50 + i, scale=(1, 0.9, 0.6)))

    elif key == "carrot":
        for i, (x, y, h) in enumerate(((-0.22, 0.04, 0.52),
                                       (0.16, -0.10, 0.56), (0.02, 0.22, 0.46))):
            def root(bm, x=x, y=y, h=h):
                F.loft(bm, [F.ring(x, y, 0.02, 0.025, 0.025, n=6),
                            F.ring(x, y, h * 0.55, 0.10, 0.095, n=6),
                            F.ring(x, y, h, 0.115, 0.105, n=6)], mat=0)
            add("root_%d" % i, ["gold" if i % 2 else "sand"], root)
            def frond(bm, k, x=x, y=y, h=h):
                a = k * math.tau / 3.0
                N.tapered_blade(bm, (x, y, h - 0.02),
                                (x + math.cos(a) * 0.16, y + math.sin(a) * 0.14,
                                 h + 0.16), 0.038, mat=0, n=3)
            for k in range(3):
                add("frond_%d_%d" % (i, k), ["leaf" if k % 2 else "grass"],
                    lambda bm, i=i, k=k: frond(bm, k))
        add("soil", ["leather"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.04), 0.34, mat=0, seed=4, scale=(1.1, 0.9, 0.3)))

    elif key == "lilypad":
        def pad(bm):
            n = 10
            cx = bm.verts.new((0, 0, 0.045))
            rim = [bm.verts.new((math.cos(0.34 + k * (math.tau - 0.68) / (n - 1)) * 0.44,
                                 math.sin(0.34 + k * (math.tau - 0.68) / (n - 1)) * 0.44,
                                 0.045)) for k in range(n)]
            for k in range(n - 1):
                gam._face(bm, [cx, rim[k], rim[k + 1]], (0, 0, 1),
                          [(0, 0), (rim[k].co.x, rim[k].co.y),
                           (rim[k + 1].co.x, rim[k + 1].co.y)], 0)
        add("pad", ["leaf"], pad)
        def vein(bm, i):
            a = 0.4 + i * (math.tau - 0.8) / 5.0
            N.tapered_blade(bm, (math.cos(a) * 0.08, math.sin(a) * 0.08, 0.05),
                            (math.cos(a) * 0.38, math.sin(a) * 0.38, 0.075),
                            0.026, mat=0, n=3)
        for i in range(5):
            add("vein_%d" % i, ["leaf_light"], lambda bm, i=i: vein(bm, i))
        add("bud", ["bone"], lambda bm: N.angular_rock(
            bm, (0.12, 0.10, 0.12), 0.09, mat=0, seed=6, scale=(1, 1, 0.8)))

    elif key == "pine":
        # Branch-first conifer: no clean stacked cones. Visible forked boughs
        # carry irregular faceted needle clumps at different heights.
        add("trunk", ["wood"], lambda bm: N.faceted_stump(
            bm, (0, 0, 0), 0.13, 1.30, mat=0, n=7, seed=7))
        boughs = [(-0.52, 0.10, 0.78), (0.48, -0.12, 1.12),
                  (-0.34, -0.40, 1.48), (0.30, 0.38, 1.82),
                  (0.02, 0.02, 2.30)]
        for i, (x, y, z) in enumerate(boughs):
            add("bough_%d" % i, ["wood"],
                lambda bm, x=x, y=y, z=z: N.branch(
                    bm, (0, 0, z - 0.18), (x, y, z), 0.055, 0.022, mat=0, n=5))
        clumps = [(-0.55, 0.10, 0.86, 0.46, 0.40, 0.50, 1),
                  (0.50, -0.12, 1.20, 0.44, 0.40, 0.52, 2),
                  (-0.36, -0.42, 1.56, 0.40, 0.38, 0.48, 3),
                  (0.32, 0.40, 1.90, 0.37, 0.35, 0.46, 4),
                  (0.0, 0.02, 2.36, 0.30, 0.29, 0.56, 5)]
        for i, (x, y, z, rx, ry, h, seed) in enumerate(clumps):
            add("needles_%d" % i,
                ["leaf_dark" if i < 2 else "leaf" if i < 4 else "leaf_light"],
                lambda bm, x=x, y=y, z=z, rx=rx, ry=ry, h=h, seed=seed:
                N.leaf_lobe(bm, x, y, z, rx, ry, h, mat=0, n=6, seed=seed))

    elif key == "volcanic_bomb":
        add("body", ["stone_dark"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.42), 0.42, mat=0, seed=11, scale=(1, 0.92, 0.88)),
            bevel=0.006)
        def crack(bm, i):
            a = i * math.tau / 6.0 + 0.3
            x, y = math.cos(a) * 0.36, math.sin(a) * 0.34
            N.tapered_blade(bm, (x * 0.35, y * 0.35, 0.78), (x, y, 0.12),
                            0.045, mat=0, n=3)
        for i in range(6):
            add("crack_%d" % i, ["lava"], lambda bm, i=i: crack(bm, i))
        for i in range(3):
            add("ash_%d" % i, ["stone"], lambda bm, i=i: N.angular_rock(
                bm, (math.cos(i * 2.1) * 0.40, math.sin(i * 2.1) * 0.36, 0.05),
                0.10, mat=0, seed=30 + i, scale=(1, 0.9, 0.6)))

    elif key == "vines":
        add("support", ["wood"], lambda bm: N.faceted_stump(
            bm, (0, 0, 0), 0.11, 0.32, mat=0, n=6, seed=4))
        def strand(bm, i):
            z0 = 0.06 + i * 0.24
            a0, a1 = i * 1.2, i * 1.2 + 1.2
            r = 0.17
            N.branch(bm, (math.cos(a0) * r, math.sin(a0) * r, z0),
                     (math.cos(a1) * r, math.sin(a1) * r, z0 + 0.26),
                     0.040, 0.032, mat=0, n=5)
        for i in range(4):
            add("strand_%d" % i, ["leaf"], lambda bm, i=i: strand(bm, i))
        def leaf(bm, i):
            z = 0.18 + 0.11 * i
            a = i * math.tau / 8.0 + 0.3
            x, y = math.cos(a) * 0.17, math.sin(a) * 0.17
            N.tapered_blade(bm, (x, y, z),
                            (x + math.cos(a) * 0.32, y + math.sin(a) * 0.30, z + 0.12),
                            0.09, mat=0, n=3)
        for i in range(8):
            add("leaf_%d" % i, ["leaf_light" if i % 2 else "leaf"],
                lambda bm, i=i: leaf(bm, i))

    elif key == "potato":
        add("soil", ["leather"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.10), 0.44, mat=0, seed=6, scale=(1.05, 0.88, 0.30)))
        tubers = [(-0.28, 0.06, 0.24, 0.20), (0.10, -0.18, 0.22, 0.18),
                  (0.30, 0.12, 0.21, 0.17), (-0.06, 0.27, 0.20, 0.16),
                  (-0.20, -0.20, 0.22, 0.15)]
        for i, (x, y, z, r) in enumerate(tubers):
            add("tuber_%d" % i, ["wood" if i % 2 else "sand_dark"],
                lambda bm, x=x, y=y, z=z, r=r: N.angular_rock(
                    bm, (x, y, z), r, mat=0, seed=40 + i,
                    scale=(1.35, 0.95, 0.75)))
        for i in range(6):
            x, y, z, r = tubers[i // 2]
            a = (i % 2) * math.pi + 0.6
            add("eye_%d" % i, ["leather"], lambda bm, x=x, y=y, z=z, r=r, a=a,
                i=i: N.angular_rock(bm, (x + math.cos(a) * r * 0.75,
                                         y + math.sin(a) * r * 0.65,
                                         z + r * 0.55), 0.028, mat=0, seed=60 + i))
        for i in range(2):
            x, y = (-0.14, 0.10) if i == 0 else (0.20, -0.02)
            add("shoot_%d" % i, ["leaf"],
                lambda bm, x=x, y=y: N.tapered_blade(
                    bm, (x, y, 0.34), (x + 0.04, y + 0.02, 0.55), 0.05, mat=0, n=3))

    elif key == "ice_crystal":
        add("snow_base", ["ice_light"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.09), 0.35, mat=0, seed=2, scale=(1.1, 1.0, 0.45)))
        def spire(bm, i):
            x, y, h, r = ((0, 0, 1.28, 0.20), (-0.26, 0.06, 0.66, 0.13),
                          (0.24, -0.08, 0.80, 0.14), (0.04, 0.26, 0.52, 0.11))[i]
            N.crystal_shard(bm, (x, y, 0.05), r, h, mat=0, n=6,
                            tilt=(0.08 * (i % 2), -0.06), seed=i + 3)
        for i in range(4):
            add("spire_%d" % i, ["ice" if i % 2 else "ice_light"],
                lambda bm, i=i: spire(bm, i))
        add("glow", ["ice_light"], lambda bm: N.crystal_shard(
            bm, (0.02, 0.14, 0.06), 0.07, 0.42, mat=0, n=5, seed=9))

    elif key == "coral":
        add("base", ["stone_dark"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.09), 0.30, mat=0, seed=7, scale=(1.15, 1.0, 0.55)))
        def branch(bm, i):
            a = i * math.tau / 5.0 + 0.3
            x, y = math.cos(a) * 0.20, math.sin(a) * 0.18
            h = 0.66 + 0.16 * (i % 3)
            N.branch(bm, (x, y, 0.12), (x * 1.6, y * 1.6, h), 0.065, 0.035,
                     mat=0, n=5)
            N.tapered_blade(bm, (x * 1.6, y * 1.6, h - 0.10),
                            (x * 2.0, y * 2.0, h + 0.12), 0.075, mat=0, n=3)
        for i in range(5):
            add("branch_%d" % i, ["coral" if i % 2 else "arcane"],
                lambda bm, i=i: branch(bm, i))

    elif key == "dune_rock":
        add("main", ["sand"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.26), 0.50, mat=0, seed=9, scale=(1.20, 0.95, 0.62)),
            bevel=0.010)
        add("ledge", ["sand_dark"], lambda bm: N.angular_rock(
            bm, (0.26, 0.06, 0.18), 0.30, mat=0, seed=12, scale=(1.1, 1.0, 0.55)))
        add("cap", ["stone_light"], lambda bm: N.angular_rock(
            bm, (-0.20, -0.08, 0.42), 0.26, mat=0, seed=13, scale=(1.1, 0.9, 0.48)))

    elif key == "grass_tuft":
        def blade(bm, i):
            a = i * math.tau / 7.0
            x, y = math.cos(a) * 0.13, math.sin(a) * 0.11
            h = 0.38 + 0.10 * (i % 3)
            N.tapered_blade(bm, (x, y, 0.01),
                            (x + math.cos(a) * 0.12, y + math.sin(a) * 0.10, h),
                            0.080, mat=0, n=3)
        for i in range(7):
            add("blade_%d" % i, ["grass" if i % 2 else "leaf_light"],
                lambda bm, i=i: blade(bm, i))

    elif key == "fern":
        add("stem", ["grass"], lambda bm: N.branch(
            bm, (0, 0, 0.01), (0, 0, 0.50), 0.035, 0.020, mat=0, n=4))
        def frond(bm, i):
            side = 1 if i % 2 else -1
            z = 0.12 + 0.055 * i
            ytip = 0.24 if i % 3 == 0 else (-0.20 if i % 3 == 1 else 0.04)
            N.tapered_blade(bm, (0, 0, z),
                            (side * 0.30, ytip, z + 0.16), 0.085, mat=0, n=3)
        for i in range(8):
            add("frond_%d" % i, ["leaf" if i % 3 else "leaf_light"],
                lambda bm, i=i: frond(bm, i))

    elif key == "reed":
        def stem(bm, i):
            a = i * math.tau / 5.0
            x, y = math.cos(a) * 0.16, math.sin(a) * 0.14
            h = 0.88 + 0.10 * (i % 3)
            N.branch(bm, (x, y, 0.01), (x + 0.04, y, h), 0.032, 0.020,
                     mat=0, n=4)
            N.tapered_blade(bm, (x + 0.04, y, h - 0.12),
                            (x + 0.04, y, h + 0.16), 0.070, mat=0, n=4)
            N.tapered_blade(bm, (x, y, 0.20), (x + 0.22, y + 0.03, 0.42),
                            0.070, mat=0, n=3)
        for i in range(5):
            add("stem_%d" % i, ["grass" if i % 2 else "swamp"],
                lambda bm, i=i: stem(bm, i))

    elif key == "herb_patch":
        def herb(bm, i):
            a = i * math.tau / 5.0
            x, y = math.cos(a) * 0.20, math.sin(a) * 0.18
            h = 0.28 + 0.08 * (i % 3)
            N.branch(bm, (x, y, 0.01), (x, y, h), 0.030, 0.020, mat=0, n=4)
            for s in (-1, 1):
                N.tapered_blade(bm, (x, y, h * 0.6),
                                (x + s * 0.16, y + 0.02, h * 0.9),
                                0.070, mat=0, n=3)
            N.tapered_blade(bm, (x, y, h), (x, y, h + 0.14), 0.050, mat=0, n=4)
        for i in range(5):
            add("herb_%d" % i, ["leaf" if i % 2 else "leaf_light"],
                lambda bm, i=i: herb(bm, i))

    elif key == "rubble":
        chips = [(-0.26, 0.10, 0.20, 0.26), (0.22, -0.06, 0.16, 0.21),
                 (0.04, 0.28, 0.13, 0.18), (-0.10, -0.24, 0.15, 0.20),
                 (0.32, 0.20, 0.10, 0.15), (-0.34, -0.10, 0.11, 0.14)]
        for i, (x, y, z, r) in enumerate(chips):
            add("chip_%d" % i, ["stone" if i % 2 else "stone_light"],
                lambda bm, x=x, y=y, z=z, r=r: N.angular_rock(
                    bm, (x, y, z), r, mat=0, seed=20 + i,
                    scale=(1.1, 0.9, 0.6)))
        add("splinter", ["wood"], lambda bm: gam.bm_box(
            bm, (-0.20, -0.06, 0.01), (0.24, 0.03, 0.05)))
    elif key == "haystack":
        def stack(bm):
            F.loft(bm, [F.ring(0, 0, 0.01, 0.50, 0.44, n=9),
                        F.ring(0, 0, 0.30, 0.44, 0.40, n=9),
                        F.ring(0, 0, 0.66, 0.30, 0.28, n=9),
                        F.ring(0, 0, 1.02, 0.05, 0.05, n=9)], mat=0)
        add("stack", ["sand"], stack, bevel=0.010)
        def straw(bm, i):
            a = i * math.tau / 7.0
            z = 0.22 + 0.14 * (i % 5)
            r = 0.44 - z * 0.30
            N.tapered_blade(bm, (math.cos(a) * r, math.sin(a) * r * 0.9, z),
                            (math.cos(a) * (r + 0.10), math.sin(a) * (r + 0.09),
                             z + 0.06), 0.028, mat=0, n=3)
        for i in range(7):
            add("straw_%d" % i, ["gold" if i % 2 else "sand_dark"],
                lambda bm, i=i: straw(bm, i))

    elif key in ("mushroom", "mushroom_cluster"):
        specs = ([(0, 0, 0.52, 0.30)] if key == "mushroom" else
                 [(-0.22, 0.05, 0.34, 0.20), (0.22, -0.08, 0.50, 0.24),
                  (0.02, 0.24, 0.26, 0.15)])
        for i, (x, y, h, r) in enumerate(specs):
            def stem(bm, x=x, y=y, h=h, r=r):
                N.faceted_stump(bm, (x, y, 0.01), r * 0.30, h * 0.86,
                                mat=0, n=6, seed=i + 2)
            add("stem_%d" % i, ["wood_light"], stem)
            add("cap_%d" % i, ["mush" if i % 2 else "coral"],
                lambda bm, x=x, y=y, h=h, r=r: N.cap(
                    bm, (x, y, h * 0.84), r, mat=0, n=8))
            for k in range(3):
                a = k * math.tau / 3.0 + 0.3
                add("spot_%d_%d" % (i, k), ["bone"],
                    lambda bm, x=x, y=y, h=h, r=r, a=a: N.angular_rock(
                        bm, (x + math.cos(a) * r * 0.55,
                             y + math.sin(a) * r * 0.55, h * 0.92),
                        0.038, mat=0, seed=70 + k, scale=(1, 1, 0.4)))

    elif key == "bush":
        add("underside", ["leaf_dark"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.18), 0.42, mat=0, seed=5, scale=(1.2, 1.1, 0.45)))
        def stem(bm, i):
            a = i * math.tau / 4.0 + 0.3
            N.branch(bm, (0, 0, 0.10),
                     (math.cos(a) * 0.26, math.sin(a) * 0.24, 0.52),
                     0.045, 0.025, mat=0, n=5)
        for i in range(4):
            add("stem_%d" % i, ["wood"], lambda bm, i=i: stem(bm, i))
        lobes = [(-0.30, 0.08, 0.52, 0.34, 0.32, 0.44, 1),
                 (0.30, -0.06, 0.50, 0.35, 0.33, 0.42, 2),
                 (0.02, 0.32, 0.60, 0.32, 0.31, 0.46, 3),
                 (0.05, -0.30, 0.44, 0.30, 0.29, 0.40, 4)]
        for i, (x, y, z, rx, ry, h, seed) in enumerate(lobes):
            add("lobe_%d" % i, ["leaf" if i % 2 else "leaf_light"],
                lambda bm, x=x, y=y, z=z, rx=rx, ry=ry, h=h, seed=seed:
                N.leaf_lobe(bm, x, y, z, rx, ry, h, mat=0, n=6, seed=seed))
        def sprig(bm, i):
            a = i * math.tau / 5.0 + 0.5
            N.tapered_blade(bm, (math.cos(a) * 0.20, math.sin(a) * 0.18, 0.42),
                            (math.cos(a) * 0.46, math.sin(a) * 0.42, 0.68),
                            0.075, mat=0, n=3)
        for i in range(5):
            add("sprig_%d" % i, ["grass"], lambda bm, i=i: sprig(bm, i))

    elif key == "crystal":
        add("base", ["stone_dark"], lambda bm: N.angular_rock(
            bm, (0, 0, 0.10), 0.28, mat=0, seed=6, scale=(1.1, 1.0, 0.45)))
        def shard(bm, i):
            x, y, h, r = ((0, 0, 0.96, 0.20), (-0.26, 0.07, 0.60, 0.14),
                          (0.24, -0.10, 0.52, 0.13), (0.06, 0.24, 0.42, 0.10))[i]
            N.crystal_shard(bm, (x, y, 0.04), r, h, mat=0, n=6,
                            tilt=(0.10 * (i % 2), -0.06), seed=i + 4)
        for i in range(4):
            add("shard_%d" % i, ["arcane" if i % 2 else "arcane"],
                lambda bm, i=i: shard(bm, i))
        add("core", ["arcane"], lambda bm: N.crystal_shard(
            bm, (0, 0.04, 0.06), 0.075, 0.50, mat=0, n=5, seed=9))

    elif key == "tree":
        add("trunk", ["wood"], lambda bm: F.loft(
            bm, [F.ring(0, 0, 0.0, 0.19, 0.17, n=7),
                 F.ring(0, 0, 0.22, 0.15, 0.14, n=7),
                 F.ring(0, 0, 1.60, 0.125, 0.115, n=7),
                 F.ring(0, 0, 2.10, 0.09, 0.085, n=7)], mat=0))
        def bough(bm, i):
            a = i * math.tau / 3.0 + 0.4
            N.branch(bm, (0, 0, 1.72),
                     (math.cos(a) * 0.38, math.sin(a) * 0.34, 2.30),
                     0.052, 0.018, mat=0, n=5)
        for i in range(3):
            add("bough_%d" % i, ["wood"], lambda bm, i=i: bough(bm, i))
        add("crown", ["leaf"], lambda bm: N.leaf_lobe(
            bm, 0, 0, 2.18, 0.62, 0.56, 0.79, mat=0, n=8, seed=1))
        add("crown_left", ["leaf_dark"], lambda bm: N.leaf_lobe(
            bm, -0.42, 0.08, 2.02, 0.40, 0.38, 0.56, mat=0, n=7, seed=2))
        add("crown_right", ["leaf_light"], lambda bm: N.leaf_lobe(
            bm, 0.42, -0.10, 2.08, 0.42, 0.39, 0.54, mat=0, n=7, seed=3))

    else:
        raise ValueError("unknown builder " + key)
    return parts


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    only = None
    if "--only" in argv:
        only = set(argv[argv.index("--only") + 1].split(","))
    for stem, dims, key in TARGETS:
        if only and stem not in only:
            continue
        brief_path = write_brief(stem, dims)
        ctx = gam.Ctx(brief_path)
        gam.reset()
        lib = gam.load_material_lib(ctx)
        M = palette()
        parts = build(key, M)
        N.finish(ctx, parts, name=stem, pack_margin=0.01)
        obj = bpy.data.objects.get("prop_" + stem + "_lod0")
        tris = sum((len(p.vertices) - 2) for p in obj.data.polygons) if obj else 0
        print("WAVE2_BUILT", stem, "tris", tris)
    print("WAVE2_DONE", len(TARGETS))


if __name__ == "__main__":
    main()
