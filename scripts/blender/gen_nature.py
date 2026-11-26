"""Starfall nature, resources & props — low-poly asset generator.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_nature.py

Style contract: docs/ART_DIRECTION.md. All geometry is flat-shaded, sits on
z = 0, is centred on (0, 0) and stays well below 500 triangles per asset.
"""

import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from common import *

import math

# ---------------------------------------------------------------------------
# Scene + shared palette materials
# ---------------------------------------------------------------------------

new_scene()


def M(name, colour, **kw):
    return mat(name, colour, **kw)


MAT = {
    # Earth / stone
    "wood":       M("nat_wood", C["wood"]),
    "leather":    M("nat_leather", C["leather"]),
    "stone":      M("nat_stone", C["stone"]),
    "stone_dark": M("nat_stone_dark", "#5E646B"),
    "stone_light": M("nat_stone_light", "#9BA3AB"),
    "iron":       M("nat_iron", C["iron"], metallic=0.25),
    "gold":       M("nat_gold", C["gold"], metallic=0.4),
    "bone":       M("nat_bone", C["bone"]),
    # Foliage (lightness variants of the palette greens)
    "leaf":       M("nat_leaf", C["leaf"]),
    "leaf_dark":  M("nat_leaf_dark", "#2C6327"),
    "leaf_light": M("nat_leaf_light", "#55953F"),
    "grass":      M("nat_grass", C["grass"]),
    "swamp":      M("nat_swamp", C["swamp"]),
    "sand":       M("nat_sand", C["sand"]),
    "snow":       M("nat_snow_tint", "#D8E4F2"),
    # Accents
    "arcane":      M("nat_arcane", C["arcane"], emission=C["arcane"], strength=1.4),
    "arcane_light": M("nat_arcane_light", "#9068C4",
                      emission="#9068C4", strength=0.8),
    "petal":      M("nat_petal", "#FFB066"),
    "cap":        M("nat_cap", C["enemy"]),
}


def tri_count(col):
    """True triangulated face count (n-gon caps included)."""
    return sum(sum(len(p.vertices) - 2 for p in o.data.polygons)
               for o in col.objects if o.type == "MESH")


# ---------------------------------------------------------------------------
# Trees
# ---------------------------------------------------------------------------

def build_tree():
    col = collection("tree")
    cyl("trunk", (0, 0, 0.72), 0.15, 1.44, MAT["wood"], col, 6)
    ico("canopy_core", (0, 0, 1.95), 0.62, MAT["leaf"], col, 1)
    ico("canopy_left", (-0.40, 0.06, 1.76), 0.48, MAT["leaf_dark"], col, 1)
    ico("canopy_right", (0.40, -0.10, 1.80), 0.50, MAT["leaf_light"], col, 1)
    ico("canopy_top", (0.06, 0.26, 2.32), 0.46, MAT["leaf"], col, 1)
    return col


def build_pine():
    col = collection("pine")
    cyl("trunk", (0, 0, 0.42), 0.11, 0.84, MAT["wood"], col, 6)
    cone("tier_low", (0, 0, 1.02), 0.62, 0.08, 0.92, MAT["leaf_dark"], col, 8)
    cone("tier_mid", (0, 0, 1.62), 0.50, 0.06, 0.88, MAT["leaf"], col, 8)
    cone("tier_top", (0, 0, 2.25), 0.35, 0.0, 1.00, MAT["leaf_light"], col, 8)
    return col


# ---------------------------------------------------------------------------
# Rocks & minerals
# ---------------------------------------------------------------------------

def build_rock():
    col = collection("rock")
    ico("boulder", (0, 0, 0.325), 0.5, MAT["stone"], col, 0,
        scale=(1.05, 0.85, 0.65))
    ico("chip", (0.28, 0.12, 0.096), 0.16, MAT["stone_dark"], col, 0,
        scale=(1.0, 0.9, 0.6))
    return col


def build_rock_pile():
    col = collection("rock_pile")
    ico("stone_a", (0, 0, 0.273), 0.42, MAT["stone"], col, 0,
        scale=(1.05, 0.9, 0.65))
    ico("stone_b", (-0.28, 0.12, 0.198), 0.30, MAT["stone_dark"], col, 0,
        scale=(1.0, 0.9, 0.66))
    ico("stone_c", (0.26, -0.12, 0.238), 0.34, MAT["stone_light"], col, 0,
        scale=(1.0, 0.85, 0.7))
    # Snow-tinted stone on top gives the pile a cold-biome reading.
    ico("stone_snow", (0.0, 0.20, 0.42), 0.26, MAT["snow"], col, 0,
        scale=(1.0, 0.9, 0.7))
    return col


def build_ore():
    col = collection("ore")
    ico("host_rock", (0, 0, 0.26), 0.44, MAT["stone_dark"], col, 0,
        scale=(1.05, 0.9, 0.59))
    ico("fleck_gold", (0.06, 0.12, 0.46), 0.10, MAT["gold"], col, 0,
        scale=(1.0, 1.0, 0.6))
    ico("fleck_iron", (-0.18, 0.05, 0.41), 0.09, MAT["iron"], col, 0,
        scale=(1.0, 1.0, 0.6))
    ico("fleck_gold_2", (0.22, -0.08, 0.45), 0.08, MAT["gold"], col, 0,
        scale=(1.0, 1.0, 0.6))
    ico("fleck_iron_2", (0.02, -0.22, 0.29), 0.08, MAT["iron"], col, 0,
        scale=(1.0, 1.0, 0.6))
    return col


def build_crystal():
    col = collection("crystal")
    ico("base_rock", (0, 0, 0.12), 0.26, MAT["stone_dark"], col, 0,
        scale=(1.1, 1.0, 0.46))
    cone("shard_main", (0, 0, 0.55), 0.20, 0.0, 0.86, MAT["arcane"], col, 6)
    left = cone("shard_left", (-0.24, 0.06, 0.38), 0.13, 0.0, 0.56,
                MAT["arcane_light"], col, 6)
    left.rotation_euler = (0.18, 0, 0)
    right = cone("shard_right", (0.22, -0.10, 0.34), 0.11, 0.0, 0.48,
                 MAT["arcane"], col, 6)
    right.rotation_euler = (-0.16, 0, 0)
    cone("shard_small", (0.05, 0.22, 0.28), 0.08, 0.0, 0.38,
         MAT["arcane_light"], col, 6)
    return col


def build_rubble():
    col = collection("rubble")
    chips = [
        ((-0.26, 0.10, 0.105), 0.18, (1.1, 0.9, 0.58), "stone"),
        ((0.22, -0.06, 0.082), 0.14, (1.0, 1.0, 0.58), "stone_light"),
        ((0.04, 0.28, 0.060), 0.10, (1.0, 0.9, 0.6), "stone_dark"),
        ((-0.10, -0.24, 0.070), 0.12, (1.1, 0.8, 0.58), "stone"),
        ((0.32, 0.20, 0.045), 0.08, (1.0, 1.0, 0.56), "stone_light"),
    ]
    for i, (loc, radius, scale, colour) in enumerate(chips):
        ico("chip_%d" % i, loc, radius, MAT[colour], col, 0, scale=scale)
    plank = box("splinter", (0.02, -0.05, 0.03), (0.44, 0.09, 0.06),
                MAT["wood"], col)
    plank.rotation_euler = (0, 0, 0.5)
    return col


# ---------------------------------------------------------------------------
# Foliage
# ---------------------------------------------------------------------------

def build_bush():
    col = collection("bush")
    ico("bush_core", (0, 0, 0.342), 0.38, MAT["leaf"], col, 0,
        scale=(1.1, 1.1, 0.9))
    ico("bush_left", (-0.30, 0.06, 0.252), 0.28, MAT["leaf_dark"], col, 0,
        scale=(1.0, 1.0, 0.9))
    ico("bush_right", (0.28, -0.10, 0.270), 0.30, MAT["leaf_light"], col, 0,
        scale=(1.0, 1.0, 0.9))
    ico("bush_back", (0.04, 0.28, 0.239), 0.26, MAT["leaf"], col, 0,
        scale=(1.0, 1.0, 0.92))
    return col


def build_fern():
    col = collection("fern")
    cyl("stem", (0, 0, 0.22), 0.035, 0.44, MAT["grass"], col, 4)
    for i in range(6):
        side = 1 if i % 2 else -1
        z = 0.11 + i * 0.065
        leaf = ico("frond_%d" % i, (side * 0.15, 0.0, z + 0.03), 0.17,
                   MAT["leaf" if i % 3 else "leaf_light"], col, 0,
                   scale=(1.0, 0.55, 0.14))
        leaf.rotation_euler = (0, side * 0.35, 0)
    return col


def build_grass_tuft():
    col = collection("grass_tuft")
    blades = [
        ((0.00, 0.00), 0.42, 0.0, 0.0),
        ((0.12, 0.05), 0.34, 0.0, 0.16),
        ((-0.10, 0.08), 0.36, -0.14, 0.0),
        ((0.03, -0.13), 0.31, 0.12, 0.0),
        ((-0.06, -0.08), 0.40, 0.0, -0.12),
    ]
    for i, ((x, y), height, rx, ry) in enumerate(blades):
        tilt = math.hypot(rx, ry)  # bottom rim sits exactly on z = 0
        z = (height / 2) * math.cos(tilt) + 0.07 * math.sin(tilt)
        blade = cone("blade_%d" % i, (x, y, z),
                     0.07, 0.0, height,
                     MAT["grass" if i % 2 == 0 else "leaf_light"], col, 4)
        blade.rotation_euler = (rx, ry, 0)
    return col


def build_mushroom():
    col = collection("mushroom")
    cyl("stem", (0, 0, 0.16), 0.085, 0.32, MAT["bone"], col, 6)
    ico("cap", (0, 0, 0.36), 0.30, MAT["cap"], col, 0, scale=(1.0, 1.0, 0.55))
    ico("spot_a", (-0.13, 0.08, 0.51), 0.055, MAT["bone"], col, 0,
        scale=(1.0, 1.0, 0.3))
    ico("spot_b", (0.12, -0.06, 0.50), 0.05, MAT["bone"], col, 0,
        scale=(1.0, 1.0, 0.3))
    # Small companion makes the resource read as a harvestable cluster.
    cyl("stem_small", (0.28, 0.12, 0.10), 0.055, 0.20, MAT["bone"], col, 4)
    ico("cap_small", (0.28, 0.12, 0.22), 0.18, MAT["cap"], col, 0,
        scale=(1.0, 1.0, 0.55))
    return col


def build_flower():
    col = collection("flower")
    cyl("stem", (0, 0, 0.22), 0.045, 0.44, MAT["grass"], col, 5)
    ico("leaf", (0.12, 0.0, 0.18), 0.14, MAT["leaf"], col, 0,
        scale=(1.0, 0.5, 0.25))
    for i in range(5):
        a = i * 2 * math.pi / 5
        ico("petal_%d" % i, (math.cos(a) * 0.16, math.sin(a) * 0.16, 0.47),
            0.11, MAT["petal"], col, 0, scale=(1.0, 1.0, 0.4))
    ico("centre", (0, 0, 0.49), 0.10, MAT["gold"], col, 0,
        scale=(1.0, 1.0, 0.6))
    return col


def build_cactus():
    col = collection("cactus")
    cyl("trunk", (0, 0, 0.55), 0.20, 1.10, MAT["leaf"], col, 8)
    arm_l = cyl("arm_left", (-0.24, 0, 0.62), 0.10, 0.30,
                MAT["leaf_dark"], col, 6)
    arm_l.rotation_euler = (0, math.pi / 2, 0)
    cyl("arm_left_tip", (-0.38, 0, 0.78), 0.10, 0.34,
        MAT["leaf_dark"], col, 6)
    arm_r = cyl("arm_right", (0.24, 0, 0.80), 0.09, 0.28,
                MAT["leaf_light"], col, 6)
    arm_r.rotation_euler = (0, math.pi / 2, 0)
    cyl("arm_right_tip", (0.36, 0, 0.94), 0.09, 0.30,
        MAT["leaf_light"], col, 6)
    for i, (x, y, z, rot) in enumerate([
            (0.20, 0.02, 0.52, 0.5), (-0.18, 0.06, 0.90, -0.6),
            (0.06, -0.19, 0.72, 2.4)]):
        spine = cone("spine_%d" % i, (x, y, z), 0.025, 0.0, 0.10,
                     MAT["sand"], col, 4)
        spine.rotation_euler = (math.pi / 2, 0, rot)
    return col


def build_vines():
    col = collection("vines")
    vines = [
        ((-0.24, 0.02), 0.50, -0.20, 0.6),
        ((0.02, -0.14), 0.42, 0.22, 0.5),
        ((0.22, 0.14), 0.56, -0.16, 0.7),
    ]
    for i, ((x, y), height, lean, width) in enumerate(vines):
        cyl("vine_%d_base" % i, (x, y, height * 0.25), 0.035,
            height * 0.5, MAT["swamp"], col, 4)
        upper = cyl("vine_%d_top" % i,
                    (x + lean * 0.12, y, height * 0.72), 0.035,
                    height * 0.46, MAT["swamp"], col, 4)
        upper.rotation_euler = (lean, 0, 0)
        for j, side in enumerate((-1, 1)):
            leaf = ico("leaf_%d_%d" % (i, j),
                       (x + side * width * 0.18, y, height * (0.4 + 0.22 * j)),
                       0.13, MAT["leaf" if j else "leaf_light"], col, 0,
                       scale=(1.0, 0.6, 0.18))
            leaf.rotation_euler = (0, side * 0.4, 0)
    return col


def build_lilypad():
    col = collection("lilypad")
    cyl("pad", (0, 0, 0.022), 0.42, 0.044, MAT["leaf"], col, 12)
    ico("pad_highlight", (0.05, 0.05, 0.05), 0.14, MAT["leaf_light"], col, 0,
        scale=(1.2, 1.0, 0.15))
    ico("bloom", (0.12, 0.10, 0.09), 0.09, MAT["petal"], col, 0,
        scale=(1.0, 1.0, 0.7))
    ico("bloom_centre", (0.12, 0.10, 0.12), 0.04, MAT["gold"], col, 0)
    return col


def build_reed():
    col = collection("reed")
    stems = [
        (-0.20, 0.04, 0.78, 0.10),
        (0.00, -0.06, 0.94, -0.06),
        (0.18, 0.10, 0.70, 0.13),
        (0.07, 0.22, 0.58, -0.10),
    ]
    for i, (x, y, height, lean) in enumerate(stems):
        z = (height / 2) * math.cos(lean) + 0.025 * abs(math.sin(lean))
        stem = cyl("stem_%d" % i, (x, y, z), 0.025,
                   height, MAT["grass" if i % 2 else "swamp"], col, 4)
        stem.rotation_euler = (lean, 0, 0)
        cone("seed_%d" % i, (x + lean * 0.04, y, height + 0.07),
             0.045, 0.0, 0.14, MAT["sand"], col, 4)
    for i, side in enumerate((-1, 1)):
        leaf = ico("blade_%d" % i, (side * 0.16, 0.02, 0.20), 0.19,
                   MAT["leaf"], col, 0, scale=(1.0, 0.45, 0.10))
        leaf.rotation_euler = (0, side * 0.5, 0)
    return col


# ---------------------------------------------------------------------------
# Props
# ---------------------------------------------------------------------------

def build_bone_pile():
    col = collection("bone_pile")
    for i, (x, y, length, rot) in enumerate([
            (0.10, 0.02, 0.62, 0.35),
            (-0.12, -0.16, 0.50, -0.75),
            (-0.02, 0.20, 0.44, 1.25)]):
        bone = cyl("bone_%d" % i, (x, y, 0.045), 0.045, length,
                   MAT["bone"], col, 5)
        bone.rotation_euler = (math.pi / 2, 0, rot)
        dx, dy = math.sin(rot) * length / 2, -math.cos(rot) * length / 2
        for j, sign in enumerate((-1, 1)):
            ico("knob_%d_%d" % (i, j),
                (x + sign * dx, y + sign * dy, 0.06), 0.075,
                MAT["bone"], col, 0, scale=(1.0, 1.0, 0.8))
    ico("skull", (0.16, 0.22, 0.084), 0.14, MAT["bone"], col, 0,
        scale=(1.0, 0.85, 0.6))
    for i, sx in enumerate((-1, 1)):
        ico("eye_%d" % i, (0.16 + sx * 0.06, 0.32, 0.115), 0.026,
            MAT["stone_dark"], col, 0)
    return col


def build_barrel():
    col = collection("barrel")
    cyl("body", (0, 0, 0.32), 0.30, 0.64, MAT["wood"], col, 10)
    cyl("band_low", (0, 0, 0.10), 0.315, 0.06, MAT["iron"], col, 10)
    cyl("band_high", (0, 0, 0.54), 0.315, 0.06, MAT["iron"], col, 10)
    cyl("lid", (0, 0, 0.645), 0.28, 0.03, MAT["leather"], col, 10)
    return col


def build_crate_box():
    col = collection("crate_box")
    box("crate", (0, 0, 0.35), (0.70, 0.70, 0.70), MAT["wood"], col)
    for sx in (-1, 1):
        for sy in (-1, 1):
            box("post_%d_%d" % (sx, sy), (sx * 0.33, sy * 0.33, 0.36),
                (0.06, 0.06, 0.72), MAT["leather"], col)
    for z in (0.06, 0.64):
        for axis in ("x", "y"):
            scale = (0.72, 0.06, 0.07) if axis == "x" else (0.06, 0.72, 0.07)
            loc = (0, -0.33, z) if axis == "x" else (-0.33, 0, z)
            box("rail_%s_%s" % (axis, z), loc, scale, MAT["leather"], col)
    brace = box("brace", (0, -0.355, 0.35), (0.66, 0.03, 0.08),
                MAT["leather"], col)
    brace.rotation_euler = (0, 0, 0.42)
    return col


def build_pillar():
    col = collection("pillar")
    box("foot", (0, 0, 0.07), (0.56, 0.56, 0.14), MAT["stone_dark"], col)
    cyl("shaft", (0, 0, 0.78), 0.19, 1.28, MAT["stone"], col, 8)
    cyl("ring_low", (0, 0, 0.20), 0.23, 0.08, MAT["iron"], col, 8)
    cyl("ring_high", (0, 0, 1.34), 0.23, 0.08, MAT["iron"], col, 8)
    box("capital", (0, 0, 1.49), (0.52, 0.52, 0.14), MAT["stone_light"], col)
    box("cap_trim", (0, 0, 1.575), (0.30, 0.30, 0.03), MAT["gold"], col)
    return col


# ---------------------------------------------------------------------------
# Build, export, preview
# ---------------------------------------------------------------------------

BUILDERS = [
    build_tree, build_pine, build_rock, build_rock_pile, build_bush,
    build_mushroom, build_crystal, build_flower, build_grass_tuft, build_fern,
    build_ore, build_cactus, build_vines, build_lilypad, build_reed,
    build_rubble, build_bone_pile, build_barrel, build_crate_box, build_pillar,
]

sets = []
for build in BUILDERS:
    col = build()
    export_asset(col.name, col)
    print("TRIS", col.name, tri_count(col))
    sets.append((col.name, col))

# contact_sheet repositions objects for the group shot, so it runs last.
contact_sheet(sets, "nature.png", width=2400, height=420)
