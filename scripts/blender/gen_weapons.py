"""Starfall — weapons, shield & old-world vehicles/town props.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_weapons.py

Style contract: docs/ART_DIRECTION.md. Every mesh is flat-shaded, sits on
z = 0, is centred on (0, 0) and stays far below 500 triangles per asset.
The contact sheet is rendered last because it repositions the objects.
"""

import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from common import *

import math
import mathutils

# ---------------------------------------------------------------------------
# Scene + shared palette materials
# ---------------------------------------------------------------------------

new_scene()

MAT = {
    "steel":      mat("wp_steel", C["steel"], metallic=0.75),
    "steel_dark": mat("wp_steel_dark", "#8E99A3", metallic=0.35),
    "iron":       mat("wp_iron", C["iron"], metallic=0.2),
    "gold":       mat("wp_gold", C["gold"], metallic=0.7),
    "leather":    mat("wp_leather", C["leather"]),
    "wood":       mat("wp_wood", C["wood"]),
    "bone":       mat("wp_bone", C["bone"]),
    "cloth":      mat("wp_cloth", C["cloth"]),
    "red":        mat("wp_red", C["enemy"]),
}


# ---------------------------------------------------------------------------
# Small local helpers (common.py is untouched)
# ---------------------------------------------------------------------------

def cyl_r(name, loc, radius, depth, material, col, vertices=8, rotation=None):
    """cyl() with the optional Euler rotation that only cone() exposes."""
    o = cyl(name, loc, radius, depth, material, col, vertices)
    if rotation:
        o.rotation_euler = rotation
    return o


def seg(name, p0, p1, width, thickness, material, col):
    """Flat bar from (x0,z0) to (x1,z1); width lies in the XZ plane."""
    dx, dz = p1[0] - p0[0], p1[1] - p0[1]
    length = math.hypot(dx, dz)
    o = box(name, ((p0[0] + p1[0]) * 0.5, 0, (p0[1] + p1[1]) * 0.5),
            (width, thickness, length * 1.04), material, col)
    o.rotation_euler = (0, math.atan2(dx, dz), 0)
    return o


def finalize(col):
    """Centre on (0,0) and drop the lowest point exactly onto z = 0."""
    bpy.context.view_layer.update()
    pts = [o.matrix_world @ mathutils.Vector(c)
           for o in col.objects if o.type == "MESH"
           for c in o.bound_box]
    if pts:
        cx = (min(p.x for p in pts) + max(p.x for p in pts)) * 0.5
        cy = (min(p.y for p in pts) + max(p.y for p in pts)) * 0.5
        cz = min(p.z for p in pts)
        for o in col.objects:
            o.location.x -= cx
            o.location.y -= cy
            o.location.z -= cz
    return col


def tri_count(col):
    """True triangulated face count (n-gon caps included)."""
    return sum(sum(len(p.vertices) - 2 for p in o.data.polygons)
               for o in col.objects if o.type == "MESH")


# ---------------------------------------------------------------------------
# Melee weapons
# ---------------------------------------------------------------------------

def build_sword():
    col = collection("sword")
    ico("sword_pommel", (0, 0, 0.07), 0.065, MAT["gold"], col, 0)
    cyl("sword_grip", (0, 0, 0.25), 0.038, 0.34, MAT["leather"], col, 6)
    box("sword_guard", (0, 0, 0.44), (0.34, 0.075, 0.055), MAT["gold"], col)
    for s in (-1, 1):
        ico("sword_quillon_%d" % s, (s * 0.175, 0, 0.47), 0.038,
            MAT["gold"], col, 0)
    blade = cone("sword_blade", (0, 0, 0.77), 0.05, 0.05, 0.62,
                 MAT["steel"], col, 4)
    blade.scale = (1.0, 0.35, 1.0)
    tip = cone("sword_tip", (0, 0, 1.16), 0.05, 0.0, 0.18,
               MAT["steel"], col, 4)
    tip.scale = (1.0, 0.35, 1.0)
    return finalize(col)


def build_axe():
    col = collection("axe")
    cyl("axe_haft", (0, 0, 0.47), 0.035, 0.90, MAT["wood"], col, 6)
    cyl("axe_wrap", (0, 0, 0.19), 0.041, 0.18, MAT["leather"], col, 6)
    box("axe_hub", (0, 0, 0.86), (0.11, 0.15, 0.24), MAT["iron"], col)
    blade = cone("axe_blade", (0.13, 0, 0.86), 0.07, 0.24, 0.30,
                 MAT["steel"], col, 4, rotation=(0, math.pi / 2, 0))
    blade.scale = (1.0, 0.20, 1.0)
    box("axe_poll", (-0.12, 0, 0.86), (0.14, 0.13, 0.19), MAT["iron"], col)
    cyl("axe_collar", (0, 0, 0.74), 0.05, 0.05, MAT["gold"], col, 6)
    return finalize(col)


def build_spear():
    col = collection("spear")
    cyl("spear_shaft", (0, 0, 0.55), 0.028, 1.00, MAT["wood"], col, 6)
    cyl("spear_wrap", (0, 0, 0.17), 0.034, 0.16, MAT["leather"], col, 6)
    cyl("spear_collar", (0, 0, 1.045), 0.046, 0.06, MAT["gold"], col, 6)
    head = cone("spear_head", (0, 0, 1.15), 0.075, 0.0, 0.24,
                MAT["steel"], col, 4)
    head.scale = (1.0, 0.45, 1.0)
    cone("spear_butt", (0, 0, 0.045), 0.0, 0.035, 0.09,
         MAT["iron"], col, 6)
    return finalize(col)


def build_hammer():
    col = collection("hammer")
    cyl("hammer_handle", (0, 0, 0.42), 0.035, 0.80, MAT["wood"], col, 6)
    cyl("hammer_grip", (0, 0, 0.19), 0.042, 0.18, MAT["leather"], col, 6)
    box("hammer_head", (0, 0, 0.88), (0.42, 0.17, 0.19), MAT["iron"], col)
    for s in (-1, 1):
        box("hammer_face_%d" % s, (s * 0.235, 0, 0.88),
            (0.055, 0.19, 0.21), MAT["steel"], col)
    box("hammer_band", (0, 0, 0.88), (0.075, 0.185, 0.205), MAT["gold"], col)
    cone("hammer_spike", (0, 0, 1.03), 0.045, 0.0, 0.14,
         MAT["steel"], col, 4)
    return finalize(col)


def build_dagger():
    col = collection("dagger")
    ico("dagger_pommel", (0, 0, 0.065), 0.05, MAT["gold"], col, 0)
    cyl("dagger_grip", (0, 0, 0.17), 0.03, 0.22, MAT["leather"], col, 6)
    box("dagger_guard", (0, 0, 0.29), (0.22, 0.055, 0.04), MAT["gold"], col)
    for s in (-1, 1):
        box("dagger_quillon_%d" % s, (s * 0.12, 0, 0.32),
            (0.035, 0.055, 0.075), MAT["gold"], col)
    blade = cone("dagger_blade", (0, 0, 0.60), 0.055, 0.055, 0.56,
                 MAT["steel"], col, 4)
    blade.scale = (1.0, 0.30, 1.0)
    tip = cone("dagger_tip", (0, 0, 0.93), 0.055, 0.0, 0.14,
               MAT["steel"], col, 4)
    tip.scale = (1.0, 0.30, 1.0)
    return finalize(col)


def build_mace():
    col = collection("mace")
    cyl("mace_haft", (0, 0, 0.42), 0.033, 0.80, MAT["wood"], col, 6)
    cyl("mace_grip", (0, 0, 0.19), 0.040, 0.18, MAT["leather"], col, 6)
    cyl("mace_collar", (0, 0, 0.81), 0.055, 0.05, MAT["gold"], col, 6)
    ico("mace_head", (0, 0, 0.93), 0.145, MAT["iron"], col, 0)
    spikes = [
        ((0.22, 0, 0.93), (0, math.pi / 2, 0)),
        ((-0.22, 0, 0.93), (0, -math.pi / 2, 0)),
        ((0, 0.22, 0.93), (-math.pi / 2, 0, 0)),
        ((0, -0.22, 0.93), (math.pi / 2, 0, 0)),
        ((0, 0, 1.07), None),
    ]
    for i, (loc, rot) in enumerate(spikes):
        cone("mace_spike_%d" % i, loc, 0.045, 0.0, 0.18,
             MAT["steel"], col, 4, rotation=rot)
    return finalize(col)


def build_scythe():
    col = collection("scythe")
    cyl("scythe_shaft", (0, 0, 0.60), 0.030, 1.20, MAT["wood"], col, 6)
    cyl("scythe_grip", (0, 0, 0.20), 0.038, 0.18, MAT["leather"], col, 6)
    box("scythe_bracket", (0.03, 0, 1.16), (0.13, 0.10, 0.10), MAT["gold"], col)
    pts = [(0.03, 1.21), (0.20, 1.28), (0.38, 1.26),
           (0.55, 1.18), (0.68, 1.05), (0.75, 0.92)]
    widths = [0.13, 0.12, 0.10, 0.075, 0.05]
    for i in range(len(pts) - 1):
        seg("scythe_blade_%d" % i, pts[i], pts[i + 1], widths[i],
            0.028, MAT["steel"], col)
    ang = math.atan2(0.07, -0.13)
    tip = cone("scythe_point", (0.778, 0, 0.867), 0.02, 0.0, 0.12,
               MAT["steel"], col, 4, rotation=(0, ang, 0))
    tip.scale = (1.0, 0.55, 1.0)
    return finalize(col)


# ---------------------------------------------------------------------------
# Ranged weapons
# ---------------------------------------------------------------------------

def build_bow():
    col = collection("bow")
    ts = [-1.0, -0.66, -0.32, 0.0, 0.32, 0.66, 1.0]
    pts = [(-0.20 * t * t, 0.62 + 0.58 * t) for t in ts]
    for i in range(len(pts) - 1):
        seg("bow_stave_%d" % i, pts[i], pts[i + 1],
            0.055, 0.05, MAT["wood"], col)
    box("bow_grip", (0, 0, 0.62), (0.085, 0.075, 0.28), MAT["leather"], col)
    for i, z in enumerate((0.04, 1.20)):
        box("bow_nock_%d" % i, (-0.20, 0, z),
            (0.055, 0.06, 0.08), MAT["gold"], col)
    cyl("bow_string", (-0.205, 0, 0.62), 0.006, 1.16, MAT["bone"], col, 4)
    return finalize(col)


def build_crossbow():
    col = collection("crossbow")
    box("crossbow_stock", (0, 0, 0.46), (0.10, 0.78, 0.11), MAT["wood"], col)
    box("crossbow_butt", (0, 0.40, 0.47), (0.15, 0.13, 0.18), MAT["leather"], col)
    box("crossbow_grip", (0, 0.22, 0.24), (0.095, 0.11, 0.48), MAT["wood"], col)
    box("crossbow_trigger", (0, -0.02, 0.40), (0.02, 0.10, 0.06), MAT["gold"], col)
    box("crossbow_rail", (0, -0.12, 0.53), (0.075, 0.52, 0.05),
        MAT["steel_dark"], col)
    for s in (-1, 1):
        limb = box("crossbow_limb_%d" % s, (s * 0.30, -0.33, 0.53),
                   (0.52, 0.075, 0.08), MAT["steel"], col)
        limb.rotation_euler = (0, 0, -s * 0.16)
        box("crossbow_tip_%d" % s, (s * 0.56, -0.40, 0.53),
            (0.06, 0.08, 0.09), MAT["gold"], col)
    box("crossbow_string", (0, -0.40, 0.53), (1.12, 0.012, 0.012),
        MAT["bone"], col)
    bolt = cyl("crossbow_bolt", (0, -0.14, 0.585), 0.013, 0.46,
               MAT["wood"], col, 6)
    bolt.rotation_euler = (math.pi / 2, 0, 0)
    cone("crossbow_bolt_tip", (0, -0.40, 0.585), 0.035, 0.0, 0.08,
         MAT["steel"], col, 4, rotation=(math.pi / 2, 0, 0))
    box("crossbow_sight", (0, 0.34, 0.56), (0.03, 0.03, 0.07), MAT["gold"], col)
    return finalize(col)


def build_arrow():
    col = collection("arrow")
    cyl("arrow_nock", (0, 0, 0.045), 0.022, 0.09, MAT["bone"], col, 6)
    cyl("arrow_shaft", (0, 0, 0.46), 0.016, 0.80, MAT["wood"], col, 6)
    cyl("arrow_collar", (0, 0, 0.865), 0.026, 0.035, MAT["gold"], col, 6)
    head = cone("arrow_head", (0, 0, 0.945), 0.045, 0.0, 0.15,
                MAT["steel"], col, 4)
    head.scale = (1.0, 0.30, 1.0)
    for i, a in enumerate((0, 2 * math.pi / 3, 4 * math.pi / 3)):
        f = box("arrow_fletch_%d" % i,
                (0.048 * math.cos(a), 0.048 * math.sin(a), 0.19),
                (0.11, 0.012, 0.20), MAT["cloth"], col)
        f.rotation_euler = (0, 0, a)
    return finalize(col)


# ---------------------------------------------------------------------------
# Shield
# ---------------------------------------------------------------------------

def build_block_shield():
    col = collection("block_shield")
    rim = cyl_r("shield_rim", (0, 0.02, 0.50), 0.55, 0.16,
                MAT["iron"], col, 8, rotation=(math.pi / 2, 0, 0))
    rim.scale = (1.14, 1.0, 1.0)
    body = cyl_r("shield_body", (0, 0, 0.50), 0.50, 0.14,
                 MAT["wood"], col, 8, rotation=(math.pi / 2, 0, 0))
    body.scale = (1.14, 1.0, 1.0)
    cyl_r("shield_boss_ring", (0, -0.075, 0.50), 0.20, 0.035,
          MAT["gold"], col, 8, rotation=(math.pi / 2, 0, 0))
    boss = ico("shield_boss", (0, -0.115, 0.50), 0.14, MAT["gold"], col, 0)
    boss.scale = (1.0, 0.55, 1.0)
    for i, a in enumerate((math.pi / 4, 3 * math.pi / 4,
                           5 * math.pi / 4, 7 * math.pi / 4)):
        box("shield_stud_%d" % i,
            (0.34 * math.cos(a) * 1.14, -0.085,
             0.50 + 0.34 * math.sin(a)),
            (0.055, 0.03, 0.055), MAT["gold"], col)
    return finalize(col)


# ---------------------------------------------------------------------------
# Old-world town props
# ---------------------------------------------------------------------------

def build_car():
    col = collection("car")
    box("car_chassis", (0, 0, 0.34), (1.95, 0.92, 0.34), MAT["red"], col)
    box("car_hood", (0.55, 0, 0.62), (0.75, 0.88, 0.34), MAT["red"], col)
    box("car_cabin", (-0.30, 0, 0.78), (0.85, 0.82, 0.56), MAT["red"], col)
    box("car_roof", (-0.30, 0, 1.09), (0.92, 0.88, 0.08), MAT["leather"], col)
    for s in (-1, 1):
        box("car_window_%d" % s, (-0.30, s * 0.42, 0.83),
            (0.62, 0.02, 0.30), MAT["cloth"], col)
    for x in (-0.65, 0.65):
        for s in (-1, 1):
            cyl_r("car_wheel_%s_%d" % (x, s), (x, s * 0.51, 0.28),
                  0.28, 0.15, MAT["iron"], col, 6, (math.pi / 2, 0, 0))
            cyl_r("car_hub_%s_%d" % (x, s), (x, s * 0.60, 0.28),
                  0.09, 0.035, MAT["gold"], col, 4, (math.pi / 2, 0, 0))
    box("car_grille", (1.00, 0, 0.46), (0.05, 0.55, 0.26), MAT["iron"], col)
    for s in (-1, 1):
        box("car_bumper_%d" % s, (s * 1.02, 0, 0.22),
            (0.08, 0.98, 0.09), MAT["iron"], col)
        cyl_r("car_lamp_%d" % s, (0.99, s * 0.30, 0.68),
              0.09, 0.08, MAT["gold"], col, 4, (0, math.pi / 2, 0))
    return finalize(col)


def build_train():
    col = collection("train")
    box("train_chassis", (0, 0, 0.42), (3.55, 0.85, 0.26), MAT["iron"], col)
    cyl_r("train_boiler", (0.55, 0, 0.94), 0.40, 2.30,
          MAT["red"], col, 6, (0, math.pi / 2, 0))
    cyl_r("train_smokebox", (1.72, 0, 0.94), 0.41, 0.08,
          MAT["iron"], col, 6, (0, math.pi / 2, 0))
    cyl_r("train_door", (1.77, 0, 0.94), 0.20, 0.05,
          MAT["gold"], col, 6, (0, math.pi / 2, 0))
    box("train_cabin", (-1.18, 0, 1.04), (1.05, 0.88, 1.25), MAT["red"], col)
    box("train_cabin_roof", (-1.18, 0, 1.70), (1.18, 0.98, 0.10),
        MAT["leather"], col)
    for s in (-1, 1):
        box("train_window_%d" % s, (-1.18, s * 0.45, 1.20),
            (0.52, 0.02, 0.42), MAT["cloth"], col)
    cyl_r("train_stack", (1.32, 0, 1.50), 0.14, 0.50, MAT["iron"], col, 6)
    cone("train_stack_top", (1.32, 0, 1.79), 0.14, 0.21, 0.14,
         MAT["iron"], col, 6)
    cone("train_dome", (0.18, 0, 1.30), 0.17, 0.12, 0.18,
         MAT["gold"], col, 6)
    for x in (-1.12, -0.10, 0.92):
        for s in (-1, 1):
            cyl_r("train_wheel_%s_%d" % (x, s), (x, s * 0.50, 0.34),
                  0.34, 0.13, MAT["iron"], col, 6, (math.pi / 2, 0, 0))
            cyl_r("train_hub_%s_%d" % (x, s), (x, s * 0.57, 0.34),
                  0.10, 0.035, MAT["gold"], col, 4, (math.pi / 2, 0, 0))
    cow = cone("train_cowcatcher", (2.00, 0, 0.36), 0.42, 0.06, 0.46,
               MAT["iron"], col, 4, rotation=(0, math.pi / 2, 0))
    cow.scale = (0.55, 0.9, 1.0)
    return finalize(col)


def build_rail():
    col = collection("rail")
    for i in range(9):
        box("rail_tie_%d" % i, (-1.6 + i * 0.4, 0, 0.05),
            (0.20, 1.25, 0.10), MAT["wood"], col)
    for s in (-1, 1):
        box("rail_runner_%d" % s, (0, s * 0.45, 0.15),
            (3.55, 0.10, 0.10), MAT["steel"], col)
        box("rail_head_%d" % s, (0, s * 0.45, 0.21),
            (3.55, 0.065, 0.025), MAT["steel_dark"], col)
    for s in (-1, 1):
        for x in (-1.6, 1.6):
            box("rail_plate_%d_%s" % (s, x), (x, s * 0.45, 0.105),
                (0.22, 0.16, 0.02), MAT["gold"], col)
    return finalize(col)


# ---------------------------------------------------------------------------
# Build, export, preview
# ---------------------------------------------------------------------------

BUILDERS = [
    build_sword, build_axe, build_spear, build_hammer, build_bow,
    build_dagger, build_crossbow, build_mace, build_scythe,
    build_block_shield, build_arrow, build_car, build_train, build_rail,
]

sets = []
for build in BUILDERS:
    col = build()
    export_asset(col.name, col)
    print("TRIS", col.name, tri_count(col))
    sets.append((col.name, col))

# contact_sheet repositions objects for the group shot, so it runs last.
contact_sheet(sets, "weapons.png", width=2100, height=480)
