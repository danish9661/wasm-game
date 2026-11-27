"""Starfall asset expansion — wave 4: all remaining focus areas.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_models_v4.py

Fifty-one new low-poly, flat-shaded, palette-only assets spanning every gap
called out for this expansion: dungeon structures + an assembled treasury
interior, farm/town props, ocean props, ten humanoid NPC variants, and new
ruin / vehicle / creature families. Every name was checked against the 171
GLBs already in ``assets/models``; nothing is duplicated.

Assets are re-based to z = 0 / (0, 0) before export, measured against the
1.55 u player and recorded in ``assets/models/expansion_models_v4.json``.
``dungeon_treasury_interior`` is the hero piece (selective bevel, weighted
normals, UV pack) using the 1500-triangle assembled-asset budget.
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import new_scene, box, cyl, cone, ico, contact_sheet
from expansion_common import P, collection, finalize, write_records

new_scene()
RECORDS = []
SETS = []
PLAYER = 1.55


def slab(p, name, loc, scale, key):
    return box(name, loc, scale, P[key], p)


def rod(p, name, loc, radius, depth, key, verts=6, tilt=(0, 0, 0)):
    o = cyl(name, loc, radius, depth, P[key], p, verts)
    o.rotation_euler = tilt
    return o


def tip(p, name, loc, r1, r2, depth, key, verts=5, tilt=(0, 0, 0)):
    o = cone(name, loc, r1, r2, depth, P[key], p, verts)
    o.rotation_euler = tilt
    return o


def lump(p, name, loc, radius, key, scale=None):
    return ico(name, loc, radius, P[key], p, 0, scale)


def flat(p, name, loc, radius, depth, key, verts=5, angle=0.0):
    """A rod lying in the ground plane, optionally angled (Euler Y then Z)."""
    return rod(p, name, loc, radius, depth, key, verts, (0, math.pi / 2, angle))


# ===========================================================================
# A - dungeon structures & interior (11)
# ===========================================================================
def build_dungeon_stairs():
    p = collection("dungeon_stairs")
    for i in range(6):
        h = (i + 1) * 0.25
        y = -0.625 + i * 0.25
        slab(p, "riser_%d" % i, (0, y, (h - 0.03) / 2),
             (1.0, 0.25, h - 0.03), "stone_dark")
        slab(p, "tread_%d" % i, (0, y, h - 0.015),
             (1.0, 0.25, 0.03), "stone_light")
    for side in (-1, 1):
        slab(p, "cheek_%d" % side, (side * 0.52, 0, 0.42),
             (0.08, 1.50, 0.84), "stone")
    return p


def build_dungeon_cage():
    p = collection("dungeon_cage")
    slab(p, "floor", (0, 0, 0.04), (1.10, 1.10, 0.08), "stone_dark")
    slab(p, "top", (0, 0, 1.50), (1.10, 1.10, 0.08), "iron")
    for i, (x, y) in enumerate(((-0.50, -0.50), (0.50, -0.50),
                                (0.50, 0.50), (-0.50, 0.50))):
        rod(p, "post_%d" % i, (x, y, 0.77), 0.055, 1.46, "iron", 5)
    for i in range(4):
        t = -0.35 + i * 0.23
        flat(p, "bar_x_%d" % i, (0, -0.50, 0.20 + i * 0.28), 0.03, 1.0,
             "iron", 4)
        flat(p, "bar_z_%d" % i, (0.50, 0, 0.28 + i * 0.28), 0.03, 1.0,
             "iron", 4, math.pi / 2)
    slab(p, "door", (-0.56, 0, 0.78), (0.06, 0.70, 1.40), "iron")
    lump(p, "lock", (-0.62, 0, 0.78), 0.09, "gold")
    return p


def build_dungeon_lever():
    p = collection("dungeon_lever")
    slab(p, "mount", (0, 0.10, 0.62), (0.34, 0.14, 0.52), "stone_dark")
    rod(p, "pivot", (0, -0.02, 0.72), 0.09, 0.16, "iron", 6,
        (math.pi / 2, 0, 0))
    rod(p, "handle", (0, -0.14, 0.92), 0.045, 0.55, "wood", 5,
        (0.55, 0, 0.10))
    lump(p, "grip", (0, -0.30, 1.16), 0.10, "wood_light")
    for i, z in enumerate((0.38, 0.48, 0.58)):
        slab(p, "chain_%d" % i, (0.12, 0.12, z), (0.10, 0.03, 0.07), "iron")
    return p


def build_dungeon_rack():
    p = collection("dungeon_rack")
    slab(p, "base", (0, 0, 0.06), (1.40, 0.60, 0.12), "wood_dark")
    for i, x in enumerate((-0.60, 0.60)):
        slab(p, "post_%d" % i, (x, 0, 0.70), (0.10, 0.14, 1.28), "wood")
    for i, z in enumerate((0.40, 1.10)):
        slab(p, "cross_%d" % i, (0, 0, z), (1.30, 0.10, 0.10), "wood")
    for i, x in enumerate((-0.30, 0.30)):
        lump(p, "clamp_%d" % i, (x, -0.10, 1.06), 0.12, "iron")
        for j, z in enumerate((0.72, 0.86, 1.00)):
            slab(p, "strap_%d_%d" % (i, j), (x, 0, z),
                 (0.10, 0.03, 0.06), "leather" if "leather" in P else "wood_dark")
    return p


def build_coffin_stack():
    p = collection("coffin_stack")
    for i, (x, y, z) in enumerate(((-0.28, 0, 0.18), (0.22, 0.10, 0.18),
                                   (0, 0, 0.62))):
        slab(p, "coffer_%d" % i, (x, y, z), (1.10, 0.56, 0.34), "stone")
        slab(p, "lid_%d" % i, (x, y, z + 0.22), (1.24, 0.64, 0.10),
             "stone_light" if i == 2 else "stone_dark")
    lump(p, "skull", (-0.62, 0.14, 0.86), 0.14, "bone")
    for i, (x, y, a) in enumerate(((0.54, -0.22, 0.5), (0.30, 0.30, -0.6))):
        flat(p, "bone_%d" % i, (x, y, 0.86), 0.035, 0.40, "bone", 4, a)
    return p


def build_dungeon_broken_pillar():
    p = collection("dungeon_broken_pillar")
    slab(p, "plinth", (0, 0, 0.08), (0.52, 0.52, 0.16), "stone_dark")
    rod(p, "shaft", (0, 0, 0.72), 0.17, 1.22, "stone", 6, (0.05, 0, 0.04))
    tip(p, "break", (0.06, 0, 1.40), 0.17, 0.09, 0.20, "stone_light", 6,
        (0.12, 0, 0.10))
    for i, (x, y, r) in enumerate(((-0.38, 0.14, 0.15), (0.32, -0.22, 0.12),
                                   (0.20, 0.36, 0.10))):
        lump(p, "rubble_%d" % i, (x, y, r * 0.6), r, "stone_dark")
    for i, x in enumerate((-0.12, 0.14)):
        slab(p, "crack_%d" % i, (x, -0.17, 0.84), (0.04, 0.02, 0.38),
             "stone_dark")
    return p


def build_dungeon_floor_grate():
    p = collection("dungeon_floor_grate")
    for i, (x, y, sx, sy) in enumerate(((-0.48, 0, 0.06, 1.0),
                                        (0.48, 0, 0.06, 1.0),
                                        (0, -0.48, 1.0, 0.06),
                                        (0, 0.48, 1.0, 0.06))):
        slab(p, "frame_%d" % i, (x, y, 0.05), (sx, sy, 0.10), "iron")
    for i in range(7):
        flat(p, "bar_%d" % i, (0, -0.40 + i * 0.133, 0.06),
             0.025, 0.96, "iron", 4)
    for i, x in enumerate((-0.44, 0.44)):
        lump(p, "hinge_%d" % i, (x, -0.48, 0.08), 0.07, "steel")
    return p


def build_dungeon_crank():
    p = collection("dungeon_crank")
    slab(p, "mount", (0, 0.12, 0.92), (0.36, 0.12, 0.44), "stone_dark")
    rod(p, "axle", (0, -0.02, 0.92), 0.07, 0.30, "iron", 6,
        (math.pi / 2, 0, 0))
    rod(p, "arm", (0, -0.18, 1.10), 0.045, 0.44, "iron", 5,
        (0.30, 0, 0))
    lump(p, "grip", (0, -0.28, 1.30), 0.09, "wood")
    rod(p, "gear", (0, -0.10, 0.92), 0.16, 0.06, "iron", 8,
        (math.pi / 2, 0, 0))
    for i, z in enumerate((0.54, 0.66, 0.78)):
        slab(p, "chain_%d" % i, (0.14, 0.10, z), (0.10, 0.03, 0.07), "iron")
    return p


def build_barred_door():
    p = collection("barred_door")
    for i, x in enumerate((-0.48, 0.48)):
        slab(p, "jamb_%d" % i, (x, 0, 1.0), (0.08, 0.14, 2.0), "stone_dark")
    slab(p, "lintel", (0, 0, 2.02), (1.04, 0.16, 0.12), "stone")
    slab(p, "leaf", (0, 0, 0.96), (0.88, 0.10, 1.90), "wood_dark")
    for i in range(5):
        slab(p, "bar_%d" % i, (-0.36 + i * 0.18, -0.09, 0.98),
             (0.06, 0.06, 1.90), "iron")
    for i, z in enumerate((0.56, 1.42)):
        slab(p, "brace_%d" % i, (0, -0.12, z), (0.92, 0.05, 0.08), "iron")
    for i, z in enumerate((0.52, 1.48)):
        lump(p, "hinge_%d" % i, (-0.43, -0.10, z), 0.07, "steel")
    slab(p, "lock", (0.30, -0.14, 0.96), (0.16, 0.06, 0.20), "gold")
    return p


def build_dungeon_arch():
    p = collection("dungeon_arch")
    for i, x in enumerate((-0.62, 0.62)):
        slab(p, "pier_%d" % i, (x, 0, 0.92), (0.24, 0.36, 1.84), "stone_dark")
        slab(p, "foot_%d" % i, (x, 0, 0.09), (0.34, 0.44, 0.18), "stone")
    for i, x in enumerate((-0.40, -0.16, 0.16, 0.40)):
        h = 1.85 + (0.24 if abs(x) < 0.3 else 0.08)
        stone = slab(p, "voussoir_%d" % i, (x, 0, h), (0.26, 0.36, 0.20),
                     "stone" if i % 2 else "stone_light")
        stone.rotation_euler.z = x * 0.65
    lump(p, "keystone", (0, -0.04, 2.18), 0.16, "stone_light")
    slab(p, "cornice", (0, 0, 2.32), (1.56, 0.40, 0.10), "stone_dark")
    return p


def build_dungeon_treasury_interior():
    """Hero: 4x4 vaulted store room with chests, bullion and a barred door."""
    p = collection("dungeon_treasury_interior")
    slab(p, "floor_slab", (0, 0, 0.05), (4.0, 4.0, 0.10), "stone_dark")
    for side, fixed in (("x", -2.0), ("x", 2.0), ("y", -2.0), ("y", 2.0)):
        for i in (-1.5, -0.5, 0.5, 1.5):
            x, y = (i, fixed) if side == "x" else (fixed, i)
            suffix = "%s%s_%s" % (side, i, fixed)
            rot = 0.0 if side == "x" else math.pi / 2
            for key, z, h, mat_key in (("low", 0.38, 0.76, "stone_dark"),
                                       ("mid", 1.25, 0.98, "stone"),
                                       ("top", 2.15, 0.70, "stone_light")):
                o = slab(p, "wall_%s_%s" % (key, suffix), (x, y, z),
                         (1.0, 0.18, h), mat_key)
                o.rotation_euler.z = rot
    # Door and transom in the south wall.
    for i, x in enumerate((-0.46, 0.46)):
        slab(p, "door_jamb_%d" % i, (x, -1.94, 1.0), (0.08, 0.10, 2.0), "wood")
    slab(p, "door_leaf", (0, -2.0, 0.95), (0.84, 0.10, 1.90), "wood_dark")
    for i in range(3):
        slab(p, "door_bar_%d" % i, (-0.28 + i * 0.28, -2.07, 1.0),
             (0.05, 0.04, 1.80), "iron")
    slab(p, "transom", (0, -1.94, 2.24), (0.94, 0.12, 0.48), "stone")
    # Two pillars, three strongboxes, bullion and sconces.
    for i, (x, y) in enumerate(((-1.4, 1.1), (1.4, 1.1))):
        rod(p, "pillar_%d" % i, (x, y, 1.25), 0.15, 2.50, "stone", 6)
        slab(p, "capital_%d" % i, (x, y, 2.46), (0.38, 0.38, 0.10), "stone_light")
    for i, (x, y) in enumerate(((-1.30, -0.90), (0, -1.05), (1.30, -0.90))):
        slab(p, "chest_%d" % i, (x, y, 0.28), (0.72, 0.48, 0.44), "wood")
        slab(p, "chest_lid_%d" % i, (x, y, 0.54), (0.76, 0.50, 0.12), "wood_dark")
        slab(p, "chest_band_%d" % i, (x, y - 0.25, 0.34), (0.76, 0.04, 0.08), "gold")
    for i, (x, y, r) in enumerate(((-0.55, 0.40, 0.24), (0.50, 0.50, 0.20),
                                   (0.05, 0.95, 0.18))):
        lump(p, "bullion_%d" % i, (x, y, r * 0.5), r, "gold",
             (1.2, 1.0, 0.55))
    for i, (x, y) in enumerate(((-1.80, -0.20), (1.80, -0.20))):
        rod(p, "torch_%d" % i, (x, y, 1.10), 0.05, 0.66, "wood", 5)
        tip(p, "flame_%d" % i, (x, y, 1.52), 0.12, 0.0, 0.30, "fire", 5)
    return p


DUNGEON = [
    ("dungeon_stairs", build_dungeon_stairs, (1.10, 1.50, 1.50),
     "cellar kit", "six stone treads"),
    ("dungeon_cage", build_dungeon_cage, (1.10, 1.10, 1.60),
     "prison prop", "iron barred cage"),
    ("dungeon_lever", build_dungeon_lever, (0.45, 0.40, 1.25),
     "mechanism", "wall lever + chain"),
    ("dungeon_rack", build_dungeon_rack, (1.40, 0.60, 1.40),
     "holding rack", "braced timber frame"),
    ("coffin_stack", build_coffin_stack, (1.30, 0.70, 1.00),
     "crypt prop", "three stacked coffers"),
    ("dungeon_broken_pillar", build_dungeon_broken_pillar, (0.90, 0.85, 1.55),
     "ruin prop", "snapped column"),
    ("dungeon_floor_grate", build_dungeon_floor_grate, (1.00, 1.00, 0.12),
     "floor kit", "seven-bar grate"),
    ("dungeon_crank", build_dungeon_crank, (0.45, 0.40, 1.40),
     "mechanism", "gate winch + gear"),
    ("barred_door", build_barred_door, (1.04, 0.20, 2.10),
     "door variant", "oak + five iron bars"),
    ("dungeon_arch", build_dungeon_arch, (1.60, 0.45, 2.40),
     "doorway", "voussoir arch"),
    ("dungeon_treasury_interior", build_dungeon_treasury_interior,
     (4.20, 4.20, 2.50), "interior", "HERO 4x4 vault; bullion"),
]


# ===========================================================================
# B - farm & town (12)
# ===========================================================================
def build_haystack():
    p = collection("haystack")
    lump(p, "base", (0, 0, 0.18), 0.50, "sand", (1.2, 1.1, 0.5))
    lump(p, "middle", (-0.10, 0.05, 0.42), 0.38, "sand_dark",
         (1.1, 1.0, 0.85))
    lump(p, "tip", (0.04, -0.04, 0.72), 0.24, "sand", (1.0, 1.0, 1.1))
    for i, (x, y, a) in enumerate(((-0.42, 0.12, 0.5), (0.38, -0.18, -0.6))):
        tip(p, "straw_%d" % i, (x, y, 0.36), 0.06, 0.0, 0.30, "sand", 4,
            (0, 0, a))
    return p


def build_hay_bale():
    p = collection("hay_bale")
    slab(p, "bale", (0, 0, 0.30), (0.90, 0.62, 0.60), "sand")
    for i, x in enumerate((-0.26, 0.26)):
        slab(p, "band_%d" % i, (x, 0, 0.30), (0.06, 0.66, 0.64), "leather" if "leather" in P else "wood_dark")
    for i, (x, y) in enumerate(((-0.32, 0.20), (0.30, -0.18))):
        lump(p, "tie_%d" % i, (x, y, 0.62), 0.09, "sand_dark")
    for i, x in enumerate((-0.36, 0.36)):
        tip(p, "straw_%d" % i, (x, 0, 0.62), 0.05, 0.0, 0.22, "sand", 4,
            (0, 0, x * 0.8))
    return p


def build_chicken_coop():
    p = collection("chicken_coop")
    slab(p, "floor", (0, 0, 0.10), (1.30, 1.00, 0.20), "wood")
    for i, (x, y) in enumerate(((-0.56, -0.42), (0.56, -0.42),
                                (0.56, 0.42), (-0.56, 0.42))):
        slab(p, "post_%d" % i, (x, y, 0.72), (0.09, 0.09, 1.04), "wood_dark")
    for i, side in enumerate((-1, 1)):
        slab(p, "roof_%d" % i, (side * 0.34, 0, 1.42),
             (0.72, 1.16, 0.06), "stone_dark")
    for i, y in enumerate((-0.16, 0.20)):
        flat(p, "perch_%d" % i, (0, y, 0.56), 0.045, 1.10, "wood_light", 5)
    slab(p, "door", (0, -0.52, 0.56), (0.44, 0.04, 0.58), "wood_dark")
    for i, (x, y) in enumerate(((-0.22, -0.16), (0.20, 0.20))):
        lump(p, "hen_%d" % i, (x, y, 0.46), 0.16, "bone", (1.1, 0.85, 0.9))
        tip(p, "beak_%d" % i, (x, y - 0.16, 0.46), 0.05, 0.0, 0.14,
            "player", 4, (math.pi / 2, 0, 0))
    return p


def build_water_trough():
    p = collection("water_trough")
    slab(p, "base", (0, 0, 0.20), (1.50, 0.60, 0.18), "wood")
    slab(p, "water", (0, 0, 0.31), (1.32, 0.46, 0.04), "water")
    for i, x in enumerate((-0.72, 0.72)):
        slab(p, "end_%d" % i, (x, 0, 0.32), (0.10, 0.60, 0.34), "wood_dark")
        slab(p, "leg_%d" % i, (x, 0, 0.09), (0.12, 0.42, 0.18), "wood_dark")
    rod(p, "pump", (-0.60, 0.24, 0.62), 0.055, 0.66, "iron", 6)
    slab(p, "spout", (-0.46, 0.20, 0.90), (0.30, 0.08, 0.08), "iron")
    return p


def build_apple_tree():
    p = collection("apple_tree")
    rod(p, "trunk", (0, 0, 0.72), 0.14, 1.44, "wood_dark", 6)
    for i, (x, y, z, r) in enumerate(((-0.42, 0.06, 1.76, 0.50),
                                      (0.40, -0.10, 1.72, 0.48),
                                      (0.02, 0.20, 2.06, 0.46))):
        lump(p, "canopy_%d" % i, (x, y, z), r,
             "leaf" if i != 1 else "leaf_dark", (1.1, 1.0, 0.85))
    for i, (x, y, z) in enumerate(((-0.36, -0.24, 1.66), (0.30, 0.18, 1.84),
                                   (0.06, 0.36, 1.92), (0.46, -0.28, 1.60),
                                   (-0.12, -0.34, 1.82))):
        lump(p, "apple_%d" % i, (x, y, z), 0.085, "enemy")
    for i, side in enumerate((-1, 1)):
        rod(p, "branch_%d" % i, (side * 0.22, 0, 1.20), 0.05, 0.60,
            "wood", 5, (0, 0, side * 0.7))
    return p


def build_farm_windmill():
    p = collection("farm_windmill")
    slab(p, "base", (0, 0, 0.08), (1.00, 1.00, 0.16), "stone_dark")
    tip(p, "tower", (0, 0, 1.06), 0.42, 0.30, 1.84, "stone", 8)
    tip(p, "roof", (0, 0, 2.10), 0.36, 0.0, 0.42, "wood_dark", 8)
    rod(p, "axle", (0, -0.34, 1.90), 0.055, 0.26, "wood", 6,
        (math.pi / 2, 0, 0))
    lump(p, "hub", (0, -0.48, 1.90), 0.12, "iron")
    for i, a in enumerate((0, math.pi / 2, math.pi, 3 * math.pi / 2)):
        blade = slab(p, "blade_%d" % i,
                     (math.sin(a) * 0.46, -0.48, 1.90 + math.cos(a) * 0.46),
                     (0.22, 0.04, 0.72), "cloth")
        blade.rotation_euler.y = a
    slab(p, "door", (0, -0.40, 0.30), (0.36, 0.06, 0.60), "wood_dark")
    lump(p, "window", (0, -0.42, 1.18), 0.10, "gold")
    return p


def build_town_fountain():
    p = collection("town_fountain")
    rod(p, "basin", (0, 0, 0.18), 0.56, 0.36, "stone", 8)
    rod(p, "water", (0, 0, 0.36), 0.48, 0.04, "water", 8)
    rod(p, "column", (0, 0, 0.72), 0.11, 0.76, "stone_light", 6)
    rod(p, "bowl", (0, 0, 1.08), 0.26, 0.10, "stone", 6)
    tip(p, "finial", (0, 0, 1.26), 0.13, 0.0, 0.24, "gold", 6)
    for i in range(4):
        a = i * math.tau / 4.0
        tip(p, "spout_%d" % i, (math.cos(a) * 0.22, math.sin(a) * 0.22, 0.90),
            0.06, 0.0, 0.20, "water", 5,
            (math.sin(a) * 0.8, -math.cos(a) * 0.8, 0))
        lump(p, "rim_%d" % i, (math.cos(a) * 0.50, math.sin(a) * 0.50, 0.40),
             0.10, "stone_light")
    return p


def build_market_umbrella():
    p = collection("market_umbrella")
    rod(p, "pole", (0, 0, 0.92), 0.045, 1.84, "wood", 6)
    lump(p, "foot", (0, 0, 0.06), 0.24, "stone_dark", (1.0, 1.0, 0.4))
    tip(p, "canopy", (0, 0, 1.92), 0.74, 0.10, 0.34, "enemy", 8)
    for i in range(8):
        a = i * math.tau / 8.0
        rib = slab(p, "rib_%d" % i,
                   (math.cos(a) * 0.44, math.sin(a) * 0.44, 1.76),
                   (0.70, 0.05, 0.05), "wood_light")
        rib.rotation_euler.z = a
    for i in (0, 2, 4, 6):
        a = i * math.tau / 8.0
        lump(p, "fringe_%d" % i,
             (math.cos(a) * 0.68, math.sin(a) * 0.68, 1.74), 0.07, "gold")
    lump(p, "finial", (0, 0, 2.14), 0.08, "gold")
    return p


def build_market_scale():
    p = collection("market_scale")
    slab(p, "base", (0, 0, 0.06), (0.70, 0.44, 0.12), "wood")
    rod(p, "post", (0, 0, 0.72), 0.05, 1.32, "wood_dark", 6)
    flat(p, "beam", (0, 0, 1.36), 0.035, 1.10, "gold", 5)
    for i, x in enumerate((-0.50, 0.50)):
        for j, z in enumerate((1.02, 1.20)):
            slab(p, "chain_%d_%d" % (i, j), (x, 0, z),
                 (0.03, 0.03, 0.18), "iron")
        rod(p, "pan_%d" % i, (x, 0, 0.90), 0.18, 0.05, "iron", 6)
    for i, x in enumerate((-0.20, 0.18)):
        lump(p, "weight_%d" % i, (x, 0.10, 0.16), 0.10, "stone")
    lump(p, "pivot", (0, -0.04, 1.36), 0.07, "gold")
    return p


def build_inn_sign():
    p = collection("inn_sign")
    rod(p, "post", (-0.44, 0, 1.00), 0.065, 2.00, "wood_dark", 6)
    slab(p, "arm", (-0.18, 0, 1.82), (0.62, 0.08, 0.08), "wood")
    slab(p, "board", (0.34, 0, 1.42), (0.62, 0.08, 0.56), "wood")
    slab(p, "rim", (0.34, -0.05, 1.42), (0.54, 0.02, 0.46), "gold")
    for i, x in enumerate((0.16, 0.52)):
        slab(p, "chain_%d" % i, (x, 0, 1.66), (0.03, 0.03, 0.22), "iron")
    lump(p, "mug", (0.34, -0.08, 1.42), 0.13, "sand", (0.8, 0.4, 1.0))
    slab(p, "brace", (-0.28, 0, 1.58), (0.05, 0.05, 0.42), "wood")
    return p


def build_street_bollard():
    p = collection("street_bollard")
    slab(p, "base", (0, 0, 0.06), (0.42, 0.42, 0.12), "stone_dark")
    rod(p, "post", (0, 0, 0.44), 0.15, 0.70, "iron", 8)
    rod(p, "cap", (0, 0, 0.82), 0.18, 0.10, "steel", 8)
    for i, z in enumerate((0.24, 0.64)):
        rod(p, "band_%d" % i, (0, 0, z), 0.16, 0.06, "gold", 8)
    for i, x in enumerate((-0.30, 0.30)):
        lump(p, "ring_%d" % i, (x, 0, 0.56), 0.08, "iron")
    return p


def build_town_clock():
    p = collection("town_clock")
    slab(p, "foot", (0, 0, 0.06), (0.42, 0.42, 0.12), "stone_dark")
    slab(p, "post", (0, 0, 1.10), (0.18, 0.18, 2.08), "wood")
    slab(p, "case", (0, -0.06, 2.18), (0.56, 0.20, 0.56), "wood_dark")
    rod(p, "face", (0, -0.18, 2.18), 0.22, 0.05, "bone", 8,
        (math.pi / 2, 0, 0))
    slab(p, "hand_long", (0, -0.22, 2.28), (0.03, 0.02, 0.24), "charcoal" if "charcoal" in P else "stone_dark")
    slab(p, "hand_short", (0.08, -0.22, 2.14), (0.16, 0.02, 0.03),
         "charcoal" if "charcoal" in P else "stone_dark")
    for i in range(4):
        a = i * math.pi / 2
        lump(p, "tick_%d" % i,
             (math.sin(a) * 0.16, -0.22, 2.18 + math.cos(a) * 0.16),
             0.025, "gold")
    tip(p, "cap", (0, -0.06, 2.58), 0.32, 0.0, 0.24, "stone_dark", 6)
    lump(p, "finial", (0, -0.06, 2.76), 0.06, "gold")
    return p


FARM_TOWN = [
    ("haystack", build_haystack, (1.25, 1.10, 0.95), "crop", "straw stack"),
    ("hay_bale", build_hay_bale, (0.95, 0.70, 0.70), "crop", "bound bale"),
    ("chicken_coop", build_chicken_coop, (1.35, 1.20, 1.50),
     "farm prop", "perches + two hens"),
    ("water_trough", build_water_trough, (1.55, 0.70, 1.00),
     "farm prop", "pumped trough"),
    ("apple_tree", build_apple_tree, (1.85, 1.60, 2.55),
     "orchard", "fruit tree"),
    ("farm_windmill", build_farm_windmill, (1.30, 1.20, 2.40),
     "landmark", "four sails"),
    ("town_fountain", build_town_fountain, (1.15, 1.15, 1.42),
     "town prop", "basin + jets"),
    ("market_umbrella", build_market_umbrella, (1.50, 1.50, 2.20),
     "stall dressing", "eight ribs"),
    ("market_scale", build_market_scale, (1.10, 0.50, 1.45),
     "stall dressing", "balance + pans"),
    ("inn_sign", build_inn_sign, (1.05, 0.25, 2.05),
     "hanging sign", "mug board"),
    ("street_bollard", build_street_bollard, (0.40, 0.40, 0.90),
     "street prop", "iron post"),
    ("town_clock", build_town_clock, (0.65, 0.45, 2.80),
     "street prop", "post clock"),
]


# ===========================================================================
# C - ocean (8)
# ===========================================================================
def build_ship_wheel():
    p = collection("ship_wheel")
    rod(p, "hub", (0, 0, 0.08), 0.13, 0.16, "wood_dark", 8,
        (math.pi / 2, 0, 0))
    lump(p, "boss", (0, -0.12, 0.08), 0.10, "gold")
    for i in range(8):
        a = i * math.tau / 8.0
        x, z = math.sin(a) * 0.50, 0.08 + math.cos(a) * 0.50
        spoke = slab(p, "spoke_%d" % i, (x / 2, 0, (z + 0.08) / 2),
                     (0.05, 0.05, 1.0), "wood")
        spoke.rotation_euler.y = a
        seg = slab(p, "rim_%d" % i, (x, 0, z), (0.28, 0.07, 0.07), "wood_dark")
        seg.rotation_euler.y = a
        rod(p, "grip_%d" % i, (x * 1.16, 0, z + (z - 0.08) * 0.16),
            0.035, 0.16, "wood_light", 4, (0, a, 0))
    return p


def build_rope_coil():
    p = collection("rope_coil")
    for i, r in enumerate((0.42, 0.32, 0.22)):
        for j in range(8):
            a = j * math.tau / 8.0
            seg = slab(p, "coil_%d_%d" % (i, j),
                       (math.cos(a) * r, math.sin(a) * r, 0.05 + i * 0.07),
                       (0.26, 0.10, 0.07), "sand")
            seg.rotation_euler.z = a
    rod(p, "tail", (0.46, -0.18, 0.08), 0.035, 0.52, "sand", 4,
        (0, math.pi / 2, 0.5))
    lump(p, "knot", (-0.30, 0.24, 0.22), 0.08, "sand_dark")
    return p


def build_fishing_rod():
    p = collection("fishing_rod")
    tip(p, "rod", (0, 0.10, 1.06), 0.045, 0.015, 2.00, "wood_light", 6,
        (0.12, 0, 0.10))
    rod(p, "grip", (0, -0.02, 0.30), 0.065, 0.58, "leather" if "leather" in P else "wood_dark", 6)
    rod(p, "reel", (0.06, -0.08, 0.62), 0.11, 0.08, "iron", 6,
        (math.pi / 2, 0, 0))
    rod(p, "line", (0.22, 0.14, 0.46), 0.008, 1.20, "bone", 4)
    tip(p, "hook", (0.28, 0.14, 0.10), 0.035, 0.0, 0.14, "iron", 4,
        (math.pi, 0, 0))
    lump(p, "float", (0.24, 0.12, 0.24), 0.09, "enemy")
    return p


def build_life_ring():
    p = collection("life_ring")
    for i in range(8):
        a = i * math.tau / 8.0
        seg = slab(p, "ring_%d" % i,
                   (math.cos(a) * 0.42, math.sin(a) * 0.42, 0.08),
                   (0.34, 0.14, 0.14), "player" if i % 4 else "bone")
        seg.rotation_euler.z = a
    for i in (0, 2, 4, 6):
        a = i * math.tau / 8.0
        band = slab(p, "band_%d" % i,
                    (math.cos(a) * 0.42, math.sin(a) * 0.42, 0.08),
                    (0.14, 0.16, 0.16), "bone")
        band.rotation_euler.z = a
    lump(p, "rope", (0.44, 0.22, 0.14), 0.08, "sand")
    return p


def build_dock_bollard():
    p = collection("dock_bollard")
    slab(p, "base", (0, 0, 0.06), (0.46, 0.46, 0.12), "wood_dark")
    rod(p, "post", (0, 0, 0.40), 0.16, 0.68, "wood", 8)
    rod(p, "cap", (0, 0, 0.78), 0.20, 0.10, "wood_dark", 8)
    flat(p, "horn", (0, 0, 0.66), 0.07, 0.66, "iron", 6)
    lump(p, "rope", (0.16, 0.18, 0.52), 0.11, "sand", (1.2, 1.0, 0.6))
    for i, x in enumerate((-0.16, 0.16)):
        rod(p, "bolt_%d" % i, (x, -0.18, 0.14), 0.025, 0.04, "iron", 4)
    return p


def build_ship_lantern():
    p = collection("ship_lantern")
    slab(p, "base", (0, 0, 0.05), (0.28, 0.28, 0.08), "iron")
    for i, (x, y) in enumerate(((-0.11, -0.11), (0.11, -0.11),
                                (0.11, 0.11), (-0.11, 0.11))):
        rod(p, "frame_%d" % i, (x, y, 0.36), 0.022, 0.56, "iron", 4)
    lump(p, "glass", (0, 0, 0.38), 0.15, "gold", (0.9, 0.9, 1.4))
    tip(p, "flame", (0, 0, 0.38), 0.07, 0.0, 0.22, "fire", 5)
    tip(p, "hood", (0, 0, 0.70), 0.21, 0.06, 0.18, "iron", 6)
    lump(p, "hook", (0, 0, 0.84), 0.07, "iron")
    flat(p, "guard", (0, -0.12, 0.20), 0.02, 0.30, "iron", 4)
    return p


def build_shell_cluster():
    p = collection("shell_cluster")
    lump(p, "sand", (0, 0, 0.05), 0.44, "sand", (1.2, 1.05, 0.3))
    for i, (x, y, r) in enumerate(((-0.18, 0.04, 0.20), (0.16, 0.12, 0.17),
                                   (0.04, -0.20, 0.15))):
        lump(p, "shell_%d" % i, (x, y, r * 0.8), r,
             "bone" if i % 2 else "coral", (1.1, 1.0, 0.75))
    for i, (x, y, a) in enumerate(((-0.18, 0.04, 0.3), (0.16, 0.12, -0.2))):
        for j in range(2):
            ridge = slab(p, "ridge_%d_%d" % (i, j),
                         (x - 0.06 + j * 0.12, y, 0.22),
                         (0.03, 0.20, 0.03), "sand_dark")
            ridge.rotation_euler.z = a
    tip(p, "spire", (0.28, -0.14, 0.16), 0.09, 0.0, 0.26, "stone_light", 5)
    return p


def build_driftwood():
    p = collection("driftwood")
    trunk = rod(p, "trunk", (0, 0, 0.12), 0.10, 1.30, "wood_light", 6,
                (0, math.pi / 2, 0.12))
    for i, (x, y, a) in enumerate(((-0.34, 0.10, 0.8), (0.30, -0.10, -0.7))):
        rod(p, "branch_%d" % i, (x, y, 0.18), 0.05, 0.54, "wood", 5,
            (0, math.pi / 2, a))
    for i, (x, y) in enumerate(((-0.46, 0.02), (0.44, -0.04))):
        lump(p, "barnacle_%d" % i, (x, y, 0.18), 0.08, "stone_light")
    lump(p, "seaweed", (0.08, 0.20, 0.12), 0.16, "swamp", (1.0, 0.4, 0.3))
    return p


OCEAN = [
    ("ship_wheel", build_ship_wheel, (1.30, 0.30, 1.15),
     "helm", "eight spokes + grips"),
    ("rope_coil", build_rope_coil, (1.00, 0.95, 0.30),
     "deck prop", "three coils"),
    ("fishing_rod", build_fishing_rod, (0.55, 0.40, 2.16),
     "tackle", "rod + reel + float"),
    ("life_ring", build_life_ring, (1.00, 0.95, 0.20),
     "safety", "segmented ring"),
    ("dock_bollard", build_dock_bollard, (0.70, 0.55, 0.90),
     "mooring", "horn + rope"),
    ("ship_lantern", build_ship_lantern, (0.35, 0.35, 0.95),
     "lighting", "caged lamp"),
    ("shell_cluster", build_shell_cluster, (1.05, 0.95, 0.40),
     "beach prop", "three shells"),
    ("driftwood", build_driftwood, (1.45, 0.50, 0.30),
     "beach prop", "salt-bleached log"),
]


# ===========================================================================
# D - ten more NPC variants
# ===========================================================================
def humanoid(p, legs, torso, head, arms=True):
    for i, x in enumerate((-0.17, 0.17)):
        rod(p, "leg_%d" % i, (x, 0, legs / 2), 0.095, legs, "wood_dark", 5)
        slab(p, "boot_%d" % i, (x, -0.06, 0.06), (0.20, 0.28, 0.12),
             "leather" if "leather" in P else "wood_dark")
    slab(p, "torso", (0, 0, torso), (0.62, 0.38, 0.68), "cloth")
    if arms:
        for i, x in enumerate((-0.39, 0.39)):
            rod(p, "arm_%d" % i, (x, 0, torso + 0.02), 0.08, 0.54,
                "cloth", 5, (0, -0.22 if i == 0 else 0.22, 0))
    lump(p, "head", (0, -0.02, head), 0.19, "skin")


def build_npc_mayor():
    p = collection("npc_mayor")
    humanoid(p, 0.62, 0.96, 1.46)
    slab(p, "robe", (0, 0, 0.62), (0.70, 0.44, 0.66), "cloth")
    slab(p, "sash", (0, -0.21, 0.98), (0.62, 0.05, 0.12), "enemy")
    slab(p, "hat_brim", (0, 0, 1.64), (0.56, 0.34, 0.05), "charcoal" if "charcoal" in P else "stone_dark")
    tip(p, "hat_crown", (0, 0, 1.76), 0.20, 0.16, 0.22,
        "charcoal" if "charcoal" in P else "stone_dark", 6)
    lump(p, "medal", (0, -0.23, 1.04), 0.10, "gold")
    rod(p, "cane", (0.46, -0.04, 0.72), 0.025, 1.40, "wood", 5)
    lump(p, "cane_knob", (0.46, -0.04, 1.44), 0.07, "gold")
    return p


def build_npc_priest():
    p = collection("npc_priest")
    humanoid(p, 0.58, 0.92, 1.42)
    tip(p, "robe", (0, 0, 0.60), 0.34, 0.22, 0.86, "bone", 6)
    slab(p, "stole", (0, -0.22, 1.02), (0.16, 0.05, 0.66), "arcane")
    tip(p, "hood", (0, 0.02, 1.56), 0.24, 0.16, 0.30, "bone", 6)
    slab(p, "book", (0.36, -0.16, 0.96), (0.26, 0.18, 0.06), "enemy")
    slab(p, "cross_v", (0, -0.26, 1.18), (0.04, 0.03, 0.22), "gold")
    slab(p, "cross_h", (0, -0.26, 1.22), (0.16, 0.03, 0.04), "gold")
    for i, z in enumerate((0.74, 0.84, 0.94)):
        lump(p, "bead_%d" % i, (0.14, -0.22, z), 0.035, "arcane")
    return p


def build_npc_bard():
    p = collection("npc_bard")
    humanoid(p, 0.60, 0.96, 1.44)
    slab(p, "cape", (0, 0.26, 1.00), (0.66, 0.08, 0.84), "enemy")
    lump(p, "feather", (0.16, 0.04, 1.66), 0.12, "player", (0.5, 0.7, 1.4))
    slab(p, "hat", (0, 0, 1.62), (0.44, 0.28, 0.10), "wood_dark")
    lump(p, "lute_body", (-0.42, -0.14, 0.88), 0.20, "wood", (1.0, 0.6, 1.2))
    rod(p, "lute_neck", (-0.42, -0.16, 1.22), 0.035, 0.62, "wood_light", 5)
    for i, x in enumerate((-0.48, -0.42, -0.36)):
        rod(p, "string_%d" % i, (x, -0.26, 1.02), 0.006, 0.54, "bone", 4)
    slab(p, "belt", (0, 0, 0.68), (0.64, 0.40, 0.09), "gold")
    return p


def build_npc_hunter():
    p = collection("npc_hunter")
    humanoid(p, 0.62, 0.98, 1.46)
    tip(p, "hood", (0, 0.02, 1.60), 0.25, 0.17, 0.32, "leaf_dark", 6)
    slab(p, "cloak", (0, 0.24, 1.02), (0.64, 0.08, 0.80), "leaf_dark")
    rod(p, "bow_top", (0.48, -0.10, 1.20), 0.025, 0.70, "wood_light", 4,
        (0, 0, 0.28))
    rod(p, "bow_low", (0.48, -0.10, 0.78), 0.025, 0.70, "wood_light", 4,
        (0, 0, -0.28))
    rod(p, "bow_string", (0.56, -0.10, 0.99), 0.008, 0.80, "bone", 4)
    rod(p, "quiver", (-0.42, 0.16, 1.06), 0.10, 0.50, "wood_dark", 6,
        (0.12, 0, 0.16))
    for i, x in enumerate((-0.48, -0.42, -0.36)):
        rod(p, "arrow_%d" % i, (x, 0.16, 1.42), 0.012, 0.36, "bone", 4)
    return p


def build_npc_fisher():
    p = collection("npc_fisher")
    humanoid(p, 0.58, 0.94, 1.42)
    slab(p, "coat", (0, 0, 0.92), (0.66, 0.42, 0.70), "cloth")
    rod(p, "hat_brim", (0, 0, 1.60), 0.29, 0.04, "sand", 6)
    tip(p, "hat", (0, 0, 1.72), 0.17, 0.14, 0.22, "sand", 6)
    tip(p, "rod", (0.46, -0.06, 1.20), 0.028, 0.012, 1.60, "wood_light", 5,
        (0.20, 0, 0.12))
    rod(p, "reel", (0.42, -0.10, 0.98), 0.09, 0.06, "iron", 6,
        (math.pi / 2, 0, 0))
    rod(p, "bucket", (-0.44, 0, 0.22), 0.16, 0.32, "wood", 6)
    lump(p, "catch", (-0.44, 0, 0.40), 0.11, "steel", (1.3, 0.6, 0.5))
    return p


def build_npc_farmer():
    p = collection("npc_farmer")
    humanoid(p, 0.62, 0.96, 1.44)
    slab(p, "vest", (0, -0.21, 1.00), (0.60, 0.06, 0.62), "sand")
    rod(p, "hat_brim", (0, 0, 1.62), 0.30, 0.04, "sand", 6)
    tip(p, "hat", (0, 0, 1.76), 0.17, 0.15, 0.26, "sand_dark", 6)
    rod(p, "hoe", (0.46, -0.04, 0.94), 0.03, 1.60, "wood_light", 5,
        (0.10, 0, 0.14))
    slab(p, "hoe_blade", (0.56, -0.04, 1.70), (0.24, 0.05, 0.10), "iron")
    lump(p, "seed_sack", (-0.44, 0.14, 0.58), 0.18, "sand",
         (1.0, 0.9, 1.1))
    slab(p, "belt", (0, 0, 0.68), (0.64, 0.40, 0.09), "wood_dark")
    return p


def build_npc_captain():
    p = collection("npc_captain")
    humanoid(p, 0.66, 1.00, 1.48)
    slab(p, "coat", (0, -0.02, 0.96), (0.70, 0.44, 0.86), "enemy")
    slab(p, "hat", (0, 0, 1.68), (0.58, 0.30, 0.10), "charcoal" if "charcoal" in P else "stone_dark")
    slab(p, "hat_up", (0, 0, 1.78), (0.30, 0.14, 0.18),
         "charcoal" if "charcoal" in P else "stone_dark")
    for i, x in enumerate((-0.42, 0.42)):
        lump(p, "epaulet_%d" % i, (x, 0, 1.30), 0.12, "gold")
    slab(p, "sword", (0.52, -0.08, 0.92), (0.06, 0.06, 0.94), "steel")
    slab(p, "guard", (0.52, -0.08, 0.48), (0.24, 0.06, 0.05), "gold")
    slab(p, "map", (-0.36, -0.14, 0.92), (0.28, 0.03, 0.22), "bone")
    return p


def build_npc_apprentice():
    p = collection("npc_apprentice")
    humanoid(p, 0.56, 0.90, 1.38)
    tip(p, "cap", (0, 0, 1.54), 0.20, 0.14, 0.28, "cloth", 6)
    slab(p, "satchel", (-0.40, 0.14, 0.76), (0.24, 0.18, 0.28), "wood")
    for i, x in enumerate((-0.20, 0.20)):
        rod(p, "scroll_%d" % i, (x, 0.18, 1.02), 0.05, 0.30, "bone", 6,
            (math.pi / 2, 0, 0))
    for i, x in enumerate((-0.08, 0.08)):
        lump(p, "lens_%d" % i, (x, -0.19, 1.40), 0.07, "steel")
    slab(p, "wrench", (0.44, -0.10, 0.86), (0.06, 0.04, 0.44), "iron")
    lump(p, "gear", (0, -0.23, 1.00), 0.09, "gold")
    return p


def build_npc_sailor():
    p = collection("npc_sailor")
    humanoid(p, 0.62, 0.96, 1.44)
    slab(p, "striped_shirt", (0, 0, 1.00), (0.64, 0.42, 0.70), "cloth")
    lump(p, "bandana", (0, 0.02, 1.56), 0.20, "enemy", (1.0, 1.0, 0.55))
    lump(p, "rope_coil", (-0.42, 0.16, 0.74), 0.16, "sand", (1.1, 1.0, 0.4))
    slab(p, "cutlass", (0.48, -0.08, 0.90), (0.06, 0.05, 0.76), "steel")
    lump(p, "anchor_pin", (0, -0.23, 1.02), 0.09, "gold")
    slab(p, "belt", (0, 0, 0.66), (0.66, 0.42, 0.10), "wood_dark")
    return p


def build_npc_mermaid():
    p = collection("npc_mermaid")
    tip(p, "tail", (0, 0, 0.52), 0.24, 0.10, 0.92, "water", 8)
    for side in (-1, 1):
        slab(p, "fin_%d" % side, (side * 0.20, -0.02, 0.10),
             (0.34, 0.06, 0.12), "water")
    lump(p, "torso", (0, 0, 1.12), 0.24, "skin", (0.95, 0.75, 1.25))
    for i, x in enumerate((-0.30, 0.30)):
        rod(p, "arm_%d" % i, (x, 0, 1.18), 0.055, 0.50, "skin", 5,
            (0, -0.26 if i == 0 else 0.26, 0))
    lump(p, "head", (0, -0.02, 1.52), 0.19, "skin")
    lump(p, "hair", (0, 0.08, 1.60), 0.22, "arcane", (1.0, 1.0, 0.85))
    for i, x in enumerate((-0.11, 0.11)):
        lump(p, "shell_%d" % i, (x, -0.18, 1.16), 0.09, "coral")
    rod(p, "trident", (0.46, -0.04, 1.00), 0.03, 1.60, "gold", 5)
    for i, x in enumerate((0.38, 0.46, 0.54)):
        tip(p, "tine_%d" % i, (x, -0.04, 1.86), 0.04, 0.0, 0.24, "steel", 4)
    for i, (x, y, z) in enumerate(((-0.44, 0.12, 1.40), (0.30, 0.20, 1.72))):
        lump(p, "bubble_%d" % i, (x, y, z), 0.06, "ice")
    return p


NPCS = [
    ("npc_mayor", build_npc_mayor, (1.05, 0.55, 1.90), "town", "tricorn + cane"),
    ("npc_priest", build_npc_priest, (0.90, 0.60, 1.75), "temple", "hood + cross"),
    ("npc_bard", build_npc_bard, (1.15, 0.70, 1.80), "tavern", "lute + plume"),
    ("npc_hunter", build_npc_hunter, (1.15, 0.65, 1.85), "forest", "bow + quiver"),
    ("npc_fisher", build_npc_fisher, (1.10, 0.60, 1.85), "coast", "rod + bucket"),
    ("npc_farmer", build_npc_farmer, (1.10, 0.60, 1.90), "field", "hoe + seed sack"),
    ("npc_captain", build_npc_captain, (1.10, 0.55, 1.90), "ship", "bicorne + sword"),
    ("npc_apprentice", build_npc_apprentice, (1.00, 0.60, 1.70), "town", "goggles + scrolls"),
    ("npc_sailor", build_npc_sailor, (1.05, 0.60, 1.75), "ship", "bandana + cutlass"),
    ("npc_mermaid", build_npc_mermaid, (1.05, 0.60, 2.00), "ocean", "tail + trident"),
]


# ===========================================================================
# E - new families: ruins, vehicles, creatures (10)
# ===========================================================================
def build_ruin_colonnade():
    p = collection("ruin_colonnade")
    for i, x in enumerate((-0.68, -0.22, 0.24, 0.70)):
        slab(p, "base_%d" % i, (x, 0, 0.08), (0.30, 0.30, 0.16), "stone_dark")
        rod(p, "shaft_%d" % i, (x, 0, 0.72), 0.11, 1.32,
            "stone_light" if i % 2 else "stone", 6)
        tip(p, "break_%d" % i, (x, 0, 1.42), 0.11, 0.05, 0.16, "stone", 6)
    slab(p, "architrave", (0, 0, 1.62), (1.72, 0.24, 0.14), "stone_dark")
    for i, (x, y, r) in enumerate(((-0.86, 0.22, 0.13), (0.58, -0.24, 0.11),
                                   (0.92, 0.12, 0.09))):
        lump(p, "rubble_%d" % i, (x, y, r * 0.6), r, "stone")
    return p


def build_ruin_wall():
    p = collection("ruin_wall")
    for i, (z, h, key) in enumerate(((0.24, 0.48, "stone_dark"),
                                     (0.68, 0.48, "stone"),
                                     (1.12, 0.48, "stone_light"))):
        slab(p, "course_%d" % i, (0, 0, z), (1.60, 0.30, h), key)
    for i, x in enumerate((-0.48, 0.06, 0.54)):
        slab(p, "crack_%d" % i, (x, -0.16, 0.74), (0.05, 0.02, 0.66),
             "stone_dark")
    for i, (x, y, r) in enumerate(((-0.72, 0.30, 0.14), (0.60, -0.26, 0.12))):
        lump(p, "rubble_%d" % i, (x, y, r * 0.7), r, "stone")
    for i, x in enumerate((-0.36, 0.40)):
        lump(p, "ivy_%d" % i, (x, -0.18, 1.28), 0.15, "leaf",
             (1.0, 0.4, 0.8))
    return p


def build_broken_obelisk():
    p = collection("broken_obelisk")
    slab(p, "plinth", (0, 0, 0.10), (0.66, 0.66, 0.20), "stone_dark")
    tip(p, "shaft", (0, 0, 1.02), 0.28, 0.17, 1.64, "sand", 4)
    tip(p, "break", (0.04, 0, 1.92), 0.17, 0.10, 0.24, "stone_light", 4,
        (0.08, 0, 0.06))
    for i, z in enumerate((0.64, 1.04, 1.44)):
        slab(p, "glyph_%d" % i, (0, -0.18, z), (0.22, 0.03, 0.05), "stone_dark")
    for i, (x, y, r) in enumerate(((-0.40, 0.12, 0.11), (0.34, -0.20, 0.09))):
        lump(p, "rubble_%d" % i, (x, y, r * 0.6), r, "sand_dark")
    return p


def build_ruin_steps():
    p = collection("ruin_steps")
    for i in range(5):
        h = (i + 1) * 0.20
        slab(p, "step_%d" % i, (0, -0.60 + i * 0.30, h / 2),
             (1.20, 0.30, h), "stone" if i % 2 else "stone_light")
    for side in (-1, 1):
        slab(p, "cheek_%d" % side, (side * 0.66, 0, 0.32),
             (0.12, 1.50, 0.64), "stone_dark")
    for i, (x, y, r) in enumerate(((-0.80, 0.34, 0.12), (0.78, -0.42, 0.10))):
        lump(p, "rubble_%d" % i, (x, y, r * 0.6), r, "stone")
    return p


def build_farm_cart():
    p = collection("farm_cart")
    slab(p, "body", (0, 0, 0.46), (1.50, 0.90, 0.36), "wood")
    for i, y in enumerate((-0.46, 0.46)):
        slab(p, "side_%d" % i, (0, y, 0.72), (1.50, 0.08, 0.40), "wood_light")
    flat(p, "axle", (0, -0.10, 0.24), 0.055, 1.20, "wood_dark", 6)
    for i, x in enumerate((-0.60, 0.60)):
        rod(p, "wheel_%d" % i, (x, -0.10, 0.24), 0.32, 0.10,
            "wood_dark", 8, (math.pi / 2, 0, 0))
        rod(p, "hub_%d" % i, (x, -0.16, 0.24), 0.09, 0.06,
            "iron", 6, (math.pi / 2, 0, 0))
        for j in range(4):
            a = j * math.pi / 4
            spoke = slab(p, "spoke_%d_%d" % (i, j),
                         (x, -0.16, 0.24), (0.05, 0.05, 0.58), "wood")
            spoke.rotation_euler.x = a
    for i, side in enumerate((-1, 1)):
        slab(p, "shaft_%d" % i, (side * 0.40, -0.80, 0.42),
             (0.08, 0.90, 0.08), "wood")
    lump(p, "cargo", (-0.28, 0, 0.72), 0.20, "sand", (1.2, 1.0, 0.8))
    return p


def build_rowboat():
    p = collection("rowboat")
    lump(p, "hull", (0, 0, 0.20), 0.50, "wood", (1.55, 0.75, 0.55))
    slab(p, "inside", (0, 0, 0.34), (1.24, 0.40, 0.06), "wood_dark")
    tip(p, "prow", (0.78, 0, 0.32), 0.16, 0.0, 0.40, "wood_light", 6,
        (0, -math.pi / 2, 0))
    for i, x in enumerate((-0.26, 0.26)):
        slab(p, "seat_%d" % i, (x, 0, 0.42), (0.16, 0.52, 0.06), "wood_light")
        rod(p, "oar_%d" % i, (x, 0.44, 0.48), 0.025, 0.92, "wood_light", 5,
            (0, math.pi / 2, 0.30 if i else -0.30))
        slab(p, "blade_%d" % i, (x, 0.84, 0.36), (0.22, 0.28, 0.04), "wood")
    lump(p, "rope", (-0.42, 0.16, 0.40), 0.07, "sand")
    return p


def build_handcart():
    p = collection("handcart")
    slab(p, "tray", (0, 0, 0.46), (1.00, 0.72, 0.18), "wood")
    for i, side in enumerate((-1, 1)):
        slab(p, "wall_%d" % i, (side * 0.48, 0, 0.62), (0.06, 0.72, 0.32),
             "wood_light")
        slab(p, "handle_%d" % i, (side * 0.34, -0.66, 0.56),
             (0.07, 0.64, 0.07), "wood")
    flat(p, "axle", (0, 0.14, 0.22), 0.045, 0.92, "wood_dark", 6)
    for i, x in enumerate((-0.44, 0.44)):
        rod(p, "wheel_%d" % i, (x, 0.14, 0.22), 0.24, 0.08,
            "wood_dark", 8, (math.pi / 2, 0, 0))
    lump(p, "load_a", (-0.20, 0, 0.64), 0.18, "sand")
    lump(p, "load_b", (0.20, 0.06, 0.62), 0.16, "stone")
    return p


def build_forest_deer():
    p = collection("forest_deer")
    lump(p, "body", (0, 0.10, 0.86), 0.40, "wood", (1.45, 0.72, 0.85))
    lump(p, "neck", (0, -0.42, 1.10), 0.18, "wood", (0.85, 0.9, 1.4))
    lump(p, "head", (0, -0.60, 1.30), 0.20, "wood_light", (1.0, 1.1, 0.9))
    for i, x in enumerate((-0.24, 0.24)):
        for j, y in enumerate((-0.16, 0.34)):
            rod(p, "leg_%d_%d" % (i, j), (x, y, 0.36), 0.055, 0.72,
                "wood_dark", 4)
        tip(p, "ear_%d" % i, (x * 0.9, -0.66, 1.46), 0.08, 0.0, 0.22,
            "wood", 4, (0, x * 1.4, 0))
        for j, z in enumerate((1.54, 1.70)):
            rod(p, "antler_%d_%d" % (i, j), (x * 0.7, -0.56, z),
                0.026, 0.34, "sand", 4, (0, 0, x * 1.2))
        lump(p, "eye_%d" % i, (x * 0.85, -0.75, 1.34), 0.035, "stone_dark")
    lump(p, "tail", (0, 0.48, 1.02), 0.10, "bone")
    return p


def build_river_otter():
    p = collection("river_otter")
    lump(p, "body", (0, 0.06, 0.34), 0.30, "wood", (1.35, 0.9, 0.85))
    lump(p, "head", (0, -0.38, 0.56), 0.20, "wood_light")
    for i, x in enumerate((-0.18, 0.18)):
        rod(p, "leg_%d" % i, (x, 0.24, 0.14), 0.055, 0.28, "wood_dark", 4)
        rod(p, "front_%d" % i, (x, -0.22, 0.16), 0.05, 0.30, "wood_dark", 4)
        lump(p, "ear_%d" % i, (x * 0.75, -0.42, 0.72), 0.07, "wood")
        lump(p, "eye_%d" % i, (x * 0.7, -0.54, 0.60), 0.035, "stone_dark")
    tip(p, "tail", (0, 0.62, 0.26), 0.15, 0.02, 0.76, "wood", 6,
        (0.65, 0, 0))
    lump(p, "fish", (-0.24, -0.56, 0.22), 0.13, "steel", (1.3, 0.6, 0.5))
    return p


def build_sea_turtle():
    p = collection("sea_turtle")
    lump(p, "shell", (0, 0, 0.32), 0.46, "leaf_dark", (1.15, 1.0, 0.55))
    for i, (x, y) in enumerate(((-0.20, 0.10), (0.18, 0.12), (0, -0.18),
                                (-0.06, 0.28), (0.30, -0.10))):
        lump(p, "plate_%d" % i, (x, y, 0.52), 0.17,
             "leaf" if i % 2 else "sand", (1.0, 1.0, 0.25))
    lump(p, "head", (0, -0.50, 0.30), 0.18, "leaf", (1.0, 1.1, 0.85))
    for i, (x, y) in enumerate(((-0.34, -0.30), (0.34, -0.30),
                                (-0.34, 0.32), (0.34, 0.32))):
        tip(p, "flipper_%d" % i, (x, y, 0.22), 0.13, 0.03, 0.46,
            "leaf", 5, (0, 0, x * 1.6))
    tip(p, "tail", (0, 0.52, 0.22), 0.07, 0.0, 0.20, "leaf", 4,
        (-math.pi / 2, 0, 0))
    for i, x in enumerate((-0.09, 0.09)):
        lump(p, "eye_%d" % i, (x, -0.64, 0.34), 0.03, "stone_dark")
    return p


NEW_FAMILIES = [
    ("ruin_colonnade", build_ruin_colonnade, (1.85, 0.45, 1.75),
     "ruin", "four broken columns"),
    ("ruin_wall", build_ruin_wall, (1.70, 0.45, 1.45),
     "ruin", "cracked courses + ivy"),
    ("broken_obelisk", build_broken_obelisk, (0.70, 0.70, 2.15),
     "ruin", "snapped marker"),
    ("ruin_steps", build_ruin_steps, (1.50, 1.50, 1.05),
     "ruin", "five worn treads"),
    ("farm_cart", build_farm_cart, (1.60, 1.30, 0.95),
     "vehicle", "two-wheel cart"),
    ("rowboat", build_rowboat, (1.80, 1.05, 0.60),
     "vehicle", "oared skiff"),
    ("handcart", build_handcart, (1.10, 1.50, 0.75),
     "vehicle", "pushed barrow"),
    ("forest_deer", build_forest_deer, (1.05, 1.35, 1.80),
     "creature", "antlered stag"),
    ("river_otter", build_river_otter, (0.85, 1.35, 0.85),
     "creature", "low mustelid"),
    ("sea_turtle", build_sea_turtle, (1.35, 1.35, 0.70),
     "creature", "plated swimmer"),
]


# ---------------------------------------------------------------------------
ASSETS = DUNGEON + FARM_TOWN + OCEAN + NPCS + NEW_FAMILIES
EXPORTS = [name for name, _builder, _target, _family, _notes in ASSETS]

for name, builder, target, family, notes in ASSETS:
    col = builder()
    is_hero = name == "dungeon_treasury_interior"
    finalize(name, col, "models_v4", target, "%s; %s" % (family, notes),
             hero=is_hero, bevel_width=0.005,
             bevel_parts=("floor_slab", "chest_lid") if is_hero else None,
             records=RECORDS)
    SETS.append((name, col))

write_records("models_v4", RECORDS)
print("WAVE4: %d assets, %.2f u player reference" % (len(RECORDS), PLAYER))
try:
    contact_sheet(SETS, "models_v4.png", width=2800, height=600)
except Exception as exc:  # previews are optional; never fail the build for them
    print("PREVIEW SKIPPED", exc)
