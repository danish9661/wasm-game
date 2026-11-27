"""Authoritative bmesh rebuilds for the visual-QA offenders.

Each builder fills an already-created collection; the caller exports it. All
parts use qa_mesh (bmesh boxes/cones/icos, packed UV0, weighted normals).
Bevels are applied only to silhouette-defining parts: bevelling every edge of
a 30-part assembly would blow the 500-triangle budget. Small detail parts are
plain flat-shaded boxes with hidden faces skipped. Base stays at z = 0 and the
pivot stays bottom-centre.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from common import mat, C  # noqa: E402
import qa_mesh as Q  # noqa: E402


def materials():
    """Flat palette materials, one set per repair run (no textures)."""
    return {
        "stone": mat("rep_stone", C["stone"]),
        "stone_dark": mat("rep_stone_dark", "#5C626A"),
        "stone_light": mat("rep_stone_light", "#99A0A8"),
        "wood": mat("rep_wood", C["wood"]),
        "wood_light": mat("rep_wood_light", "#8A5C31"),
        "wood_dark": mat("rep_leather", C["leather"]),
        "sand": mat("rep_sand", C["sand"]),
        "bone": mat("rep_bone", C["bone"]),
        "gold": mat("rep_gold", C["gold"], metallic=0.55, rough=0.45),
        "steel": mat("rep_steel", C["steel"], metallic=0.6, rough=0.5),
        "iron": mat("rep_iron", C["iron"], metallic=0.35),
        "cloth": mat("rep_cloth", C["cloth"]),
        "glass": mat("rep_glass", C["water"], rough=0.35),
        "red": mat("rep_red", C["enemy"]),
        "fire": mat("rep_fire", C["fire"], emission=C["fire"], strength=2.0),
    }


def finish_group(name, col, mats, bevel_bm=None, plain_bm=None, bevel=0.006):
    """Finish up to two bmesh groups; only the silhouette group is bevelled."""
    if bevel_bm is not None and len(bevel_bm.faces):
        Q.finish(name + "_hero", bevel_bm, mats, col, bevel=bevel)
    else:
        if bevel_bm is not None:
            bevel_bm.free()
    if plain_bm is not None and len(plain_bm.faces):
        Q.finish(name + "_detail", plain_bm, mats, col, bevel=0.0)
    else:
        if plain_bm is not None:
            plain_bm.free()


# ---------------------------------------------------------------------------
# Statue: a readable armoured figure, not a stack of grey blocks.
# ---------------------------------------------------------------------------
def build_statue(col, M=None):
    M = M or materials()
    order = ["stone_dark", "stone", "stone_light", "wood_dark", "gold",
             "steel", "cloth"]
    m = {k: i for i, k in enumerate(order)}
    hero, det = Q.make_bm(), Q.make_bm()
    # Silhouette parts get real bevels.
    Q.bm_box(hero, (-0.45, -0.45, 0.0), (0.45, 0.45, 0.22), m["stone_dark"])
    Q.bm_box(hero, (-0.35, -0.35, 0.22), (0.35, 0.35, 0.38), m["stone_light"])
    Q.bm_cone(hero, (0, 0, 0.80), 0.25, 0.35, 0.84, sides=4,
              mat=m["stone"], rot_z=math.pi / 4)
    Q.bm_box(hero, (-0.38, -0.16, 1.14), (0.38, 0.16, 1.28), m["stone"])
    Q.bm_ico(hero, (0, -0.01, 1.47), 0.19, m["stone_light"],
             scale=(0.92, 0.82, 1.14))
    # Small detail parts: no bevel, hidden faces omitted.
    for x in (-0.38, 0.38):
        Q.bm_box(det, (x - 0.07, -0.10, 0.66), (x + 0.07, 0.10, 1.18),
                 m["stone"], skip=("back",))
    Q.bm_box(det, (-0.15, -0.17, 1.52), (0.15, -0.14, 1.57), m["stone_dark"],
             skip=("top", "back"))
    Q.bm_box(det, (-0.04, -0.19, 1.43), (0.04, -0.15, 1.49), m["stone_light"],
             skip=("bottom", "back"))
    Q.bm_box(det, (-0.28, -0.19, 0.74), (0.28, 0.19, 0.84), m["wood_dark"],
             skip=("top", "bottom"))
    Q.bm_box(det, (-0.09, -0.22, 0.74), (0.09, -0.19, 0.84), m["gold"],
             skip=("top", "bottom", "back"))
    Q.bm_box(det, (-0.16, 0.19, 0.30), (0.16, 0.24, 0.76), m["cloth"],
             skip=("front", "top"))
    Q.bm_box(det, (0.52, -0.02, 0.52), (0.58, 0.02, 1.34), m["steel"],
             skip=("back",))
    Q.bm_box(det, (0.45, -0.10, 0.88), (0.65, 0.10, 0.95), m["gold"])
    Q.bm_box(det, (0.51, -0.03, 0.38), (0.59, 0.03, 0.52), m["wood_dark"],
             skip=("top", "bottom"))
    Q.bm_cone(det, (0, 0, 1.68), 0.18, 0.14, 0.10, sides=6, mat=m["gold"],
              skip_caps=True)
    for x in (-0.12, 0, 0.12):
        Q.bm_cone(det, (x, 0, 1.79), 0.04, 0.0, 0.14, sides=4, mat=m["gold"],
                  skip_caps=True)
    finish_group("statue", col, [M[k] for k in order], hero, det, bevel=0.006)
    return col


# ---------------------------------------------------------------------------
# Spike: four times the visual mass, still ~1.5 u tall.
# ---------------------------------------------------------------------------
def build_spike(col, M=None):
    M = M or materials()
    order = ["stone_dark", "stone", "wood", "wood_dark", "steel", "iron", "red"]
    m = {k: i for i, k in enumerate(order)}
    hero, det = Q.make_bm(), Q.make_bm()
    Q.bm_cone(hero, (0, 0, 0.11), 0.42, 0.36, 0.22, sides=8, mat=m["stone_dark"])
    Q.bm_cone(hero, (0, 0, 0.85), 0.21, 0.10, 1.08, sides=8, mat=m["wood"])
    Q.bm_cone(det, (0, 0, 0.26), 0.31, 0.25, 0.12, sides=8, mat=m["stone"],
              skip_caps=True)
    for z, r in ((0.42, 0.19), (1.02, 0.145)):
        Q.bm_cone(det, (0, 0, z), r, r, 0.07, sides=8, mat=m["iron"],
                  skip_caps=True)
    Q.bm_cone(det, (0, 0, 1.32), 0.155, 0.12, 0.18, sides=8, mat=m["red"],
              skip_caps=True)
    Q.bm_cone(det, (0, 0, 1.47), 0.13, 0.0, 0.24, sides=8, mat=m["steel"])
    for a in (0, math.pi / 2, math.pi, 3 * math.pi / 2):
        x, y = 0.27 * math.cos(a), 0.27 * math.sin(a)
        rot = (0.32 * math.sin(a), -0.32 * math.cos(a), 0.0)
        Q.bm_box(det, (x - 0.05, y - 0.05, 0.28), (x + 0.05, y + 0.05, 0.60),
                 m["wood_dark"], rot=rot, center=(x, y, 0.44), skip=("bottom",))
    finish_group("spike", col, [M[k] for k in order], hero, det, bevel=0.007)
    return col


# ---------------------------------------------------------------------------
# Interior floor: real plank seams, alternating tones, visible at 60 px.
# ---------------------------------------------------------------------------
def build_interior_floor(col, M=None):
    M = M or materials()
    order = ["wood_dark", "wood", "wood_light", "sand"]
    m = {k: i for i, k in enumerate(order)}
    hero, det = Q.make_bm(), Q.make_bm()
    Q.bm_box(hero, (-0.5, -0.5, 0.0), (0.5, 0.5, 0.03), m["wood_dark"])
    tones = ("wood", "wood_light", "sand", "wood")
    for i in range(6):
        y = -0.415 + i * 0.166
        for j, (xa, xb) in enumerate(((-0.49, -0.015), (0.015, 0.49))):
            Q.bm_box(det, (xa, y, 0.025), (xb, y + 0.146, 0.10),
                     m[tones[(i + j) % len(tones)]], skip=("bottom",))
    finish_group("interior_floor", col, [M[k] for k in order], hero, det,
                 bevel=0.004)
    return col


# ---------------------------------------------------------------------------
# Interior wall: studs, wainscot panels, rail and capping beam.
# ---------------------------------------------------------------------------
def build_interior_wall(col, M=None):
    M = M or materials()
    order = ["wood", "wood_light", "wood_dark", "bone", "stone_light"]
    m = {k: i for i, k in enumerate(order)}
    hero, det = Q.make_bm(), Q.make_bm()
    Q.bm_box(hero, (-0.5, -0.092, 2.42), (0.5, 0.092, 2.5), m["wood_dark"])
    Q.bm_box(hero, (-0.5, -0.092, 0.82), (0.5, 0.092, 0.88), m["wood_dark"])
    Q.bm_box(det, (-0.5, -0.075, 0.0), (0.5, 0.075, 2.5), m["bone"],
             skip=("bottom", "front", "back"))
    Q.bm_box(det, (-0.5, -0.080, 0.0), (0.5, 0.080, 0.82), m["wood"],
             skip=("back",))
    for x in (-0.43, 0.43):
        Q.bm_box(det, (x - 0.045, -0.094, 0.88), (x + 0.045, -0.074, 2.42),
                 m["wood_light"], skip=("back", "top"))
    for x in (-0.24, 0.24):
        Q.bm_box(det, (x - 0.16, -0.086, 0.14), (x + 0.16, -0.076, 0.70),
                 m["wood_light"], skip=("back", "top"))
    Q.bm_box(det, (-0.5, -0.088, 1.94), (0.5, -0.074, 2.00), m["wood_light"],
             skip=("back", "top"))
    Q.bm_box(det, (-0.32, -0.088, 1.12), (0.32, -0.078, 1.80), m["stone_light"],
             skip=("back", "top"))
    finish_group("interior_wall", col, [M[k] for k in order], hero, det,
                 bevel=0.005)
    return col


# ---------------------------------------------------------------------------
# Kit walls: real courses/planks inside the exact 1 x 0.15/0.18 x 2.5 contract.
# ---------------------------------------------------------------------------
def build_kit_wall_stone(col, M=None):
    M = M or materials()
    order = ["stone_dark", "stone", "stone_light"]
    m = {k: i for i, k in enumerate(order)}
    hero, det = Q.make_bm(), Q.make_bm()
    Q.bm_box(det, (-0.5, -0.09, 0.0), (0.5, 0.09, 2.5), m["stone_dark"],
             skip=("bottom", "front"))
    tones = ("stone", "stone_light", "stone")
    n = 0
    for course in range(5):
        z0 = 0.02 + course * 0.495
        z1 = min(2.5, z0 + 0.455)
        for block in range(3):
            xa = -0.49 + block * 0.33 + (0.16 if course % 2 else 0.0)
            xb = min(0.49, xa + 0.31)
            if xa >= 0.49:
                continue
            depth = 0.055 + ((n * 7) % 4) * 0.009
            # Only the top course is bevelled: that is the silhouette seen at
            # gameplay distance; lower courses stay cheap flat blocks.
            target = hero if course == 4 else det
            Q.bm_box(target, (xa, -0.09, z0), (xb, -0.09 + depth, z1),
                     m[tones[(course + block) % 3]],
                     skip=("back", "bottom") if target is det else ())
            n += 1
    finish_group("kit_wall_stone", col, [M[k] for k in order], hero, det,
                 bevel=0.006)
    return col


def build_kit_wall_wood(col, M=None):
    M = M or materials()
    order = ["wood_dark", "wood", "wood_light", "sand"]
    m = {k: i for i, k in enumerate(order)}
    hero, det = Q.make_bm(), Q.make_bm()
    Q.bm_box(det, (-0.5, -0.075, 0.0), (0.5, 0.075, 2.5), m["wood_dark"],
             skip=("bottom", "front"))
    for i in range(4):
        x = -0.485 + i * 0.245
        tone = "wood" if i % 2 == 0 else "wood_light"
        Q.bm_box(det, (x, -0.075, 0.02), (x + 0.225, -0.035, 0.84),
                 m[tone], skip=("back", "bottom"))
        Q.bm_box(det, (x, -0.075, 0.91), (x + 0.225, -0.035, 2.42),
                 m["sand" if i % 2 == 0 else "wood_light"],
                 skip=("back", "bottom"))
    Q.bm_box(hero, (-0.5, -0.09, 0.84), (0.5, 0.09, 0.91), m["wood_dark"])
    Q.bm_box(hero, (-0.5, -0.09, 2.42), (0.5, 0.09, 2.5), m["wood_dark"])
    Q.bm_box(det, (-0.54, -0.096, 0.24), (0.54, -0.076, 0.32), m["wood_dark"],
             rot=(0, math.atan2(0.52, 1.0), 0), center=(0, -0.086, 0.28),
             skip=("back",))
    finish_group("kit_wall_wood", col, [M[k] for k in order], hero, det,
                 bevel=0.005)
    return col


# ---------------------------------------------------------------------------
# House interior: cutaway shell + plank floor + readable windows.
# ---------------------------------------------------------------------------
def detail_house_interior(room, M=None):
    """Add bmesh detail to the assembled room collection from gen_interiors."""
    M = M or materials()
    order = ["wood_dark", "wood", "wood_light", "sand", "glass"]
    m = {k: i for i, k in enumerate(order)}
    det = Q.make_bm()
    # Plank floor over the existing 4x4 slab (top of slab is z=0.10).
    for i in range(8):
        y = -1.75 + i * 0.5
        Q.bm_box(det, (-1.98, y, 0.10), (1.98, y + 0.46, 0.135),
                 m["wood" if i % 2 else "wood_light"],
                 skip=("bottom",))
    # Corner posts break up the remaining three walls.
    for x, y in ((-1.98, 1.98), (-1.98, -1.98), (1.98, 1.98)):
        Q.bm_box(det, (x - 0.06, y - 0.06, 0.10), (x + 0.06, y + 0.06, 2.5),
                 m["wood_dark"], skip=("bottom",))
    Q.bm_box(det, (-2.02, 1.92, 2.38), (2.02, 2.02, 2.5), m["wood_dark"],
             skip=("top", "back"))
    # Two framed blue windows on the north wall and one on the west wall.
    for x in (-1.0, 1.0):
        Q.bm_box(det, (x - 0.30, 1.92, 1.28), (x + 0.30, 1.94, 2.00), m["glass"],
                 skip=("back",))
        for dx in (-0.35, 0.35):
            Q.bm_box(det, (x + dx - 0.045, 1.88, 1.20),
                     (x + dx + 0.045, 1.98, 2.08), m["wood_light"],
                     skip=("back",))
        Q.bm_box(det, (x - 0.35, 1.88, 1.20), (x + 0.35, 1.98, 1.28), m["wood"],
                 skip=("back", "top"))
    Q.bm_box(det, (-1.94, -0.3, 1.28), (-1.92, 0.3, 2.00), m["glass"],
             skip=("back",))
    Q.finish("house_interior_detail", det, [M[k] for k in order], room,
             bevel=0.0)
    return room
