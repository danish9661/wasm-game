"""Starfall asset expansion — wave 5: remaining gaps and hero interiors.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_models_v5.py

Thirty-one new low-poly, flat-shaded, palette-only assets: biome variety
(swamp, tundra, desert, ocean wreck, volcanic), enemy variants, town props,
dungeon props and three complete modular interiors (tavern, smithy, shrine).
Every name was checked against the 223 GLBs already in ``assets/models``;
nothing duplicates an earlier asset.

Scale contract: 1u = 1 tile = 1 m, player 1.55 u, storey 2.5 u,
doors 0.9 x 2.0 u, 1 u grid, base exactly z = 0, footprint centred on (0, 0).
The three interiors are heroes: bmesh forms, selective bevel, weighted normals,
flat shading and a UV pack (``expansion_common.hero_finish``).

Measurements are written to ``assets/models/expansion_models_v5.json``.
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import new_scene, box, cyl, cone, ico, contact_sheet
from expansion_common import collection, finalize, write_records, tris
from v5_common import V, bm_box, bm_cone

new_scene()
RECORDS = []
SETS = []
PLAYER = 1.55


def slab(p, name, loc, scale, key):
    return box(name, loc, scale, V[key], p)


def rod(p, name, loc, radius, depth, key, verts=6, tilt=(0, 0, 0)):
    o = cyl(name, loc, radius, depth, V[key], p, verts)
    o.rotation_euler = tilt
    return o


def tip(p, name, loc, r1, r2, depth, key, verts=5, tilt=(0, 0, 0)):
    o = cone(name, loc, r1, r2, depth, V[key], p, verts)
    o.rotation_euler = tilt
    return o


def lump(p, name, loc, radius, key, scale=None):
    return ico(name, loc, radius, V[key], p, 0, scale)


def flat(p, name, loc, radius, depth, key, verts=5, angle=0.0):
    return rod(p, name, loc, radius, depth, key, verts, (0, math.pi / 2, angle))


# ===========================================================================
# A - biome variety (12)
# ===========================================================================
def build_swamp_reeds():
    p = collection("swamp_reeds")
    lump(p, "mud_mound", (0, 0, 0.09), 0.46, "swamp_dark", (1.7, 1.0, 0.32))
    for i, (x, y, h) in enumerate(((-0.42, -0.12, 1.34), (-0.18, 0.22, 1.55),
                                    (0.06, -0.26, 1.48), (0.30, 0.14, 1.62),
                                    (0.50, -0.08, 1.25), (-0.02, 0.04, 1.40),
                                    (0.24, 0.30, 1.15))):
        rod(p, "stalk_%d" % i, (x, y, h / 2), 0.028, h, "leaf", 5,
            (0.05 * (i % 3 - 1), 0, 0.04 * (i % 2 - 0.5)))
        if i % 2 == 0:
            tip(p, "cattail_%d" % i, (x, y, h + 0.10), 0.055, 0.02, 0.22,
                "wood_dark", 5)
        else:
            tip(p, "blade_%d" % i, (x, y, h + 0.04), 0.07, 0.0, 0.14,
                "leaf_light", 5)
    for i, (x, y) in enumerate(((-0.34, 0.26), (0.40, -0.30))):
        flat(p, "water_%d" % i, (x, y, 0.035), 0.22, 0.06, "water", 6)
    return p


def build_mangrove_roots():
    p = collection("mangrove_roots")
    rod(p, "trunk", (0, 0.10, 0.72), 0.18, 1.34, "wood_dark", 8)
    for i, (x, y, a) in enumerate(((-0.55, -0.30, 0.75), (-0.52, 0.34, -0.65),
                                    (0.55, -0.32, -0.70), (0.56, 0.30, 0.70),
                                    (0.05, -0.50, 0.15))):
        rod(p, "root_%d" % i, (x * 0.6, y * 0.6, 0.32), 0.055, 0.96, "wood", 6,
            (a, 0, math.atan2(y, x) * 0.35))
        tip(p, "fork_%d" % i, (x, y, 0.10), 0.075, 0.02, 0.18, "wood_dark", 5)
    for i, (x, y, r) in enumerate(((-0.60, 0.10, 0.20), (0.62, -0.04, 0.18),
                                    (0.02, 0.52, 0.16))):
        lump(p, "mud_%d" % i, (x, y, r * 0.42), r, "swamp_dark", (1.4, 1.0, 0.5))
    flat(p, "pool", (0, -0.06, 0.025), 0.62, 0.05, "water", 8)
    return p


def build_ice_spire():
    p = collection("ice_spire")
    slab(p, "snow_base", (0, 0, 0.05), (1.05, 0.95, 0.10), "snow")
    tip(p, "spire_main", (0, 0, 1.02), 0.30, 0.0, 1.94, "ice", 6,
        (0.04, 0, -0.05))
    for i, (x, y, h, r) in enumerate(((-0.38, 0.16, 1.20, 0.17),
                                       (0.36, 0.24, 0.94, 0.14),
                                       (0.16, -0.38, 1.44, 0.15))):
        tip(p, "spire_%d" % i, (x, y, h / 2), r, 0.0, h, "frost", 5,
            (0.10 * (i - 1), 0.06 * (i % 2), 0.08))
    for i, (x, y, r) in enumerate(((-0.35, -0.34, 0.16), (0.44, -0.10, 0.13),
                                    (0.04, 0.46, 0.15))):
        lump(p, "chunk_%d" % i, (x, y, r * 0.6), r, "ice", (1.0, 0.8, 0.7))
    return p


def build_frozen_log():
    p = collection("frozen_log")
    flat(p, "log", (0, 0, 0.34), 0.30, 1.62, "wood_dark", 8, 0.18)
    for i, (x, y, r) in enumerate(((-0.56, -0.04, 0.26), (0.08, 0.14, 0.30),
                                    (0.62, -0.10, 0.23))):
        lump(p, "ice_coat_%d" % i, (x, y, 0.40), r, "ice", (1.0, 1.0, 0.7))
    for i, (x, y) in enumerate(((-0.36, 0.28), (0.40, -0.28))):
        rod(p, "branch_%d" % i, (x, y, 0.52), 0.055, 0.52, "wood", 5,
            (0.55, 0, 0.7 if i else -0.6))
        tip(p, "frost_%d" % i, (x, y, 0.78), 0.10, 0.0, 0.16, "frost", 5)
    slab(p, "snow_patch", (0, 0.34, 0.045), (1.0, 0.46, 0.09), "snow")
    return p


def build_desert_ruin_wall():
    p = collection("desert_ruin_wall")
    for row in range(3):
        for col in range(3):
            if row == 2 and col == 2:
                continue
            o = slab(p, "block_%d_%d" % (row, col),
                     (-0.58 + col * 0.58, 0, 0.23 + row * 0.44),
                     (0.54, 0.42, 0.42),
                     "sand_dark" if (row + col) % 2 else "sand")
            o.rotation_euler.z = 0.02 * ((row + col) % 3 - 1)
    tip(p, "broken_top", (0.58, 0.02, 1.48), 0.20, 0.0, 0.24, "sand", 5)
    lump(p, "dune", (-0.42, 0.30, 0.10), 0.30, "sand", (1.8, 1.0, 0.5))
    for i, (x, y, r) in enumerate(((0.78, -0.28, 0.15), (-0.70, -0.30, 0.12))):
        lump(p, "rubble_%d" % i, (x, y, r * 0.55), r, "sand_dark")
    for i, x in enumerate((-0.28, 0.30)):
        slab(p, "crack_%d" % i, (x, -0.215, 0.78), (0.04, 0.02, 0.52),
             "stone_dark")
    return p


def build_sandstone_pillar():
    p = collection("sandstone_pillar")
    slab(p, "plinth", (0, 0, 0.10), (0.62, 0.62, 0.20), "sand_dark")
    rod(p, "shaft", (0, 0, 1.16), 0.22, 1.94, "sand", 8)
    for i, x in enumerate((-0.16, 0.0, 0.16)):
        slab(p, "flute_%d" % i, (x, -0.215, 1.12), (0.055, 0.03, 1.80),
             "sand_dark")
    slab(p, "capital", (0, 0, 2.20), (0.52, 0.52, 0.14), "sand_dark")
    tip(p, "chip", (0.06, 0.04, 2.34), 0.16, 0.0, 0.16, "sand", 5)
    for i, (x, y, r) in enumerate(((-0.42, 0.22, 0.14), (0.40, -0.18, 0.12))):
        lump(p, "rubble_%d" % i, (x, y, r * 0.6), r, "sand_dark")
    slab(p, "dune", (0, 0.34, 0.03), (0.90, 0.24, 0.06), "sand")
    return p


def build_wreck_hull_section():
    p = collection("wreck_hull_section")
    for i, (z, w) in enumerate(((0.22, 1.60), (0.52, 1.74), (0.82, 1.86))):
        o = slab(p, "plank_%d" % i, (0, 0, z), (w, 0.12, 0.22), "wood_dark")
        o.rotation_euler.y = 0.07 * (i - 1)
    slab(p, "keel", (0, 0, 0.12), (1.90, 0.16, 0.22), "wood")
    for i, x in enumerate((-0.70, -0.38, 0.0, 0.38, 0.70)):
        rod(p, "rib_%d" % i, (x, 0, 0.58), 0.05, 0.92, "wood", 5,
            (0.16, 0, 0.10 * (i - 2)))
    for i, (x, y) in enumerate(((-0.52, 0.34), (0.46, -0.30))):
        lump(p, "barnacle_%d" % i, (x, y, 0.92), 0.13, "bone_dark")
    tip(p, "splinter", (0.82, 0, 1.02), 0.14, 0.0, 0.34, "wood", 5,
        (0, 0.5, 0))
    flat(p, "rope", (-0.20, 0.26, 0.90), 0.035, 0.62, "swamp_dark", 5, 0.4)
    return p


def build_wreck_rudder():
    p = collection("wreck_rudder")
    slab(p, "blade", (0, 0, 0.72), (0.72, 0.12, 1.16), "wood_dark")
    for i, z in enumerate((0.34, 0.72, 1.10)):
        slab(p, "plank_%d" % i, (0, 0.075, z), (0.70, 0.06, 0.32), "wood")
    for i, z in enumerate((0.34, 1.08)):
        rod(p, "pintle_%d" % i, (-0.36, 0.02, z), 0.05, 0.30, "iron", 6,
            (math.pi / 2, 0, 0))
    rod(p, "tiller", (0.46, 0, 1.34), 0.055, 0.70, "wood", 6, (0, 0, 0.45))
    for i, z in enumerate((0.40, 1.04)):
        slab(p, "band_%d" % i, (0, 0.11, z), (0.76, 0.05, 0.07), "iron")
    lump(p, "barnacle", (0.22, 0.16, 0.28), 0.13, "bone_dark")
    flat(p, "rope", (-0.30, 0.14, 1.28), 0.03, 0.44, "swamp_dark", 5, 0.5)
    return p


def build_wreck_debris():
    p = collection("wreck_debris")
    for i, (x, y, a) in enumerate(((-0.42, -0.18, 0.25), (0.10, 0.22, -0.35),
                                    (0.52, -0.10, 0.55))):
        slab(p, "plank_%d" % i, (x, y, 0.10), (0.90, 0.20, 0.10), "wood_dark")
    slab(p, "sail_scrap", (-0.20, 0.32, 0.22), (0.62, 0.44, 0.04), "cloth")
    rod(p, "mast_stub", (0.18, -0.10, 0.24), 0.12, 0.66, "wood", 6,
        (0, 0, 0.30))
    slab(p, "crate", (0.60, 0.28, 0.20), (0.36, 0.36, 0.36), "wood")
    lump(p, "barrel_half", (-0.62, 0.22, 0.14), 0.22, "wood", (1.0, 1.0, 0.6))
    for i, (x, y) in enumerate(((-0.24, -0.34), (0.34, 0.06), (0.70, -0.30))):
        lump(p, "barnacle_%d" % i, (x, y, 0.10), 0.10, "bone_dark")
    flat(p, "rope", (-0.05, -0.02, 0.26), 0.035, 0.72, "swamp_dark", 5, 0.2)
    return p


def build_obsidian_spire():
    p = collection("obsidian_spire")
    lump(p, "basalt_base", (0, 0, 0.12), 0.38, "char", (1.4, 1.2, 0.55))
    tip(p, "spire_main", (0, 0, 0.96), 0.26, 0.0, 1.72, "obsidian", 6)
    for i, (x, y, h) in enumerate(((-0.38, 0.14, 0.78), (0.34, 0.30, 0.58),
                                    (0.22, -0.36, 0.92))):
        tip(p, "shard_%d" % i, (x, y, h / 2), 0.12, 0.0, h, "obsidian", 5,
            (0.10 * (i - 1), 0, 0.10))
    for i, (x, y, r) in enumerate(((-0.44, -0.28, 0.14), (0.46, 0.02, 0.12))):
        lump(p, "chunk_%d" % i, (x, y, r * 0.7), r, "obsidian")
    for i, x in enumerate((-0.10, 0.16)):
        slab(p, "lava_crack_%d" % i, (x, 0.24, 0.09), (0.05, 0.16, 0.04),
             "fire")
    lump(p, "ash", (0, -0.42, 0.04), 0.18, "ash", (1.5, 1.0, 0.4))
    return p


def build_volcanic_bomb():
    p = collection("volcanic_bomb")
    lump(p, "core", (0, 0, 0.28), 0.38, "fire")
    for i, (x, y, z, r) in enumerate(((-0.20, 0.08, 0.30, 0.22),
                                       (0.20, 0.12, 0.32, 0.20),
                                       (0.02, -0.20, 0.26, 0.21),
                                       (0.04, 0.06, 0.52, 0.18))):
        lump(p, "crust_%d" % i, (x, y, z), r, "char", (1.0, 1.0, 0.8))
    for i, (x, y) in enumerate(((-0.12, 0.30), (0.26, -0.10))):
        tip(p, "ember_%d" % i, (x, y, 0.62), 0.09, 0.0, 0.18, "fire", 5)
    for i, (x, y, r) in enumerate(((-0.38, -0.22, 0.10), (0.34, 0.30, 0.09))):
        lump(p, "ash_%d" % i, (x, y, r * 0.5), r, "ash")
    return p


def build_basalt_columns():
    p = collection("basalt_columns")
    lump(p, "ash_base", (0, 0, 0.05), 0.52, "ash", (1.6, 1.3, 0.25))
    for i, (x, y, h) in enumerate(((-0.42, -0.18, 1.22), (-0.14, 0.28, 1.58),
                                    (0.20, -0.30, 1.36), (0.46, 0.14, 1.05),
                                    (0.02, 0.02, 0.84))):
        rod(p, "column_%d" % i, (x, y, h / 2), 0.17, h, "char", 6)
        tip(p, "break_%d" % i, (x, y, h + 0.06), 0.17, 0.0, 0.14,
            "obsidian", 6)
    for i, x in enumerate((-0.26, 0.30)):
        slab(p, "glow_%d" % i, (x, -0.44, 0.04), (0.06, 0.14, 0.03), "fire")
    return p


BIOME = [
    ("swamp_reeds", build_swamp_reeds, (1.2, 0.9, 1.6),
     "biome swamp", "cattail reed bed + pools"),
    ("mangrove_roots", build_mangrove_roots, (1.7, 1.4, 1.4),
     "biome swamp", "arching root tangle + mud"),
    ("ice_spire", build_ice_spire, (1.1, 1.0, 2.0),
     "biome tundra", "frozen spike cluster"),
    ("frozen_log", build_frozen_log, (1.8, 1.0, 0.8),
     "biome tundra", "ice-coated fallen log"),
    ("desert_ruin_wall", build_desert_ruin_wall, (1.8, 0.6, 1.5),
     "biome desert", "crumbled sandstone courses"),
    ("sandstone_pillar", build_sandstone_pillar, (0.7, 0.7, 2.4),
     "biome desert", "fluted column + capital"),
    ("wreck_hull_section", build_wreck_hull_section, (2.0, 1.0, 1.2),
     "biome ocean", "broken hull ribs + planking"),
    ("wreck_rudder", build_wreck_rudder, (1.0, 0.5, 1.6),
     "biome ocean", "detached rudder + tiller"),
    ("wreck_debris", build_wreck_debris, (1.7, 1.2, 0.6),
     "biome ocean", "salted wreckage scatter"),
    ("obsidian_spire", build_obsidian_spire, (1.0, 1.0, 2.0),
     "biome volcanic", "black glass spire + lava seams"),
    ("volcanic_bomb", build_volcanic_bomb, (1.0, 1.0, 0.9),
     "biome volcanic", "cracked molten ejecta"),
    ("basalt_columns", build_basalt_columns, (1.3, 1.2, 1.7),
     "biome volcanic", "hexagonal column cluster"),
]


# ===========================================================================
# B - enemy variants (4)
# ===========================================================================
def build_armored_goblin():
    p = collection("armored_goblin")
    lump(p, "body", (0, 0, 0.70), 0.30, "enemy", (1.0, 0.8, 1.15))
    lump(p, "head", (0, -0.06, 1.10), 0.21, "leaf", (1.0, 0.9, 0.95))
    for i, x in enumerate((-0.24, 0.24)):
        tip(p, "ear_%d" % i, (x, -0.02, 1.16), 0.11, 0.0, 0.24, "leaf", 4,
            (0, 0, 1.35 if i else -1.35))
        lump(p, "eye_%d" % i, (x * 0.45, -0.20, 1.13), 0.045, "fire")
        rod(p, "arm_%d" % i, (x * 1.15, 0, 0.68), 0.075, 0.52, "skin", 5,
            (0, 0, 0.18 if i else -0.18))
        lump(p, "pauldron_%d" % i, (x * 1.05, 0, 0.94), 0.16, "iron",
             (1.0, 1.0, 0.7))
        rod(p, "leg_%d" % i, (x * 0.42, 0, 0.26), 0.09, 0.50, "wood_dark", 6)
    slab(p, "breastplate", (0, -0.19, 0.78), (0.42, 0.10, 0.44), "iron")
    tip(p, "helmet", (0, -0.04, 1.28), 0.20, 0.03, 0.18, "steel", 6)
    slab(p, "shield", (-0.44, -0.20, 0.66), (0.30, 0.08, 0.52), "wood_dark")
    lump(p, "boss", (-0.44, -0.26, 0.66), 0.10, "gold")
    slab(p, "sword", (0.46, -0.04, 0.96), (0.07, 0.05, 0.66), "steel")
    slab(p, "guard", (0.46, -0.04, 0.68), (0.24, 0.06, 0.06), "gold")
    return p


def build_wraith_knight():
    p = collection("wraith_knight")
    tip(p, "cloak", (0, 0.04, 0.66), 0.42, 0.16, 1.28, "arcane", 8)
    lump(p, "torso", (0, 0, 1.14), 0.26, "char", (1.0, 0.7, 1.1))
    tip(p, "hood", (0, -0.02, 1.52), 0.23, 0.10, 0.34, "arcane", 8)
    lump(p, "face", (0, -0.14, 1.44), 0.14, "char")
    for i, x in enumerate((-0.08, 0.08)):
        lump(p, "eye_%d" % i, (x, -0.24, 1.45), 0.035, "fire")
    for i, x in enumerate((-0.30, 0.30)):
        lump(p, "pauldron_%d" % i, (x, 0, 1.28), 0.17, "iron")
        rod(p, "arm_%d" % i, (x * 1.2, -0.02, 0.96), 0.065, 0.58, "arcane_light", 5)
        tip(p, "crown_%d" % i, (x * 0.6, 0, 1.72), 0.08, 0.0, 0.22,
            "arcane", 5)
    slab(p, "sword", (0.44, -0.10, 0.92), (0.07, 0.05, 0.90), "steel")
    slab(p, "guard", (0.44, -0.10, 0.52), (0.26, 0.06, 0.06), "gold")
    for i, (x, y, z) in enumerate(((-0.44, 0.16, 0.42), (0.34, 0.20, 0.28),
                                    (0.06, -0.34, 0.18))):
        lump(p, "wisp_%d" % i, (x, y, z), 0.12, "arcane_light")
    return p


def build_fire_elemental():
    p = collection("fire_elemental")
    lump(p, "core", (0, 0, 1.00), 0.40, "fire", (1.0, 0.85, 1.2))
    lump(p, "inner", (0, -0.10, 1.02), 0.24, "gold")
    for i, x in enumerate((-0.09, 0.09)):
        lump(p, "eye_%d" % i, (x, -0.36, 1.14), 0.04, "char")
    for i, x in enumerate((-0.44, 0.44)):
        rod(p, "arm_%d" % i, (x, 0, 0.98), 0.09, 0.66, "fire", 6,
            (0, 0, 0.35 if i else -0.35))
        lump(p, "fist_%d" % i, (x * 1.25, -0.02, 0.72), 0.16, "fire")
        tip(p, "leg_%d" % i, (x * 0.45, 0, 0.22), 0.14, 0.0, 0.44, "char", 6)
    for i, (x, y, h) in enumerate(((-0.18, 0.04, 0.50), (0.02, -0.06, 0.66),
                                    (0.22, 0.08, 0.46), (0.12, 0.22, 0.34),
                                    (-0.28, -0.10, 0.30))):
        tip(p, "flame_%d" % i, (x, y, 1.30 + h / 2), 0.13, 0.0, h, "fire", 5)
    for i, (x, y, z) in enumerate(((-0.40, -0.22, 0.50), (0.42, 0.18, 0.62))):
        lump(p, "ember_%d" % i, (x, y, z), 0.09, "gold")
    return p


def build_stone_elemental():
    p = collection("stone_elemental")
    lump(p, "torso", (0, 0, 1.04), 0.42, "stone", (1.0, 0.75, 1.1))
    lump(p, "head", (0, -0.04, 1.52), 0.22, "stone_dark", (1.0, 0.9, 0.9))
    for i, x in enumerate((-0.09, 0.09)):
        lump(p, "eye_%d" % i, (x, -0.21, 1.55), 0.04, "arcane")
    for i, x in enumerate((-0.46, 0.46)):
        lump(p, "shoulder_%d" % i, (x, 0, 1.28), 0.22, "stone_dark")
        rod(p, "arm_%d" % i, (x * 1.15, 0, 0.86), 0.13, 0.68, "stone", 6,
            (0, 0, 0.12 if i else -0.12))
        lump(p, "fist_%d" % i, (x * 1.30, -0.02, 0.56), 0.17, "stone_dark")
        rod(p, "leg_%d" % i, (x * 0.45, 0, 0.30), 0.15, 0.60, "stone", 6)
    for i, (x, y, z) in enumerate(((-0.20, -0.32, 1.10), (0.22, -0.30, 0.92),
                                    (0.06, -0.34, 1.32))):
        lump(p, "crystal_%d" % i, (x, y, z), 0.10, "arcane")
    lump(p, "rubble", (0, 0.30, 0.06), 0.30, "stone_dark", (1.6, 1.0, 0.4))
    return p


ENEMY = [
    ("armored_goblin", build_armored_goblin, (0.95, 0.7, 1.35),
     "enemy variant", "goblin in plate + shield"),
    ("wraith_knight", build_wraith_knight, (1.05, 0.7, 1.9),
     "enemy variant", "crowned wraith + sword"),
    ("fire_elemental", build_fire_elemental, (1.2, 0.9, 2.0),
     "enemy variant", "molten elemental + flame crown"),
    ("stone_elemental", build_stone_elemental, (1.4, 1.0, 1.85),
     "enemy variant", "rock elemental + arcane veins"),
]


# ===========================================================================
# C - town props (6)
# ===========================================================================
def build_market_crates():
    p = collection("market_crates")
    slab(p, "stand", (0, 0, 0.16), (1.48, 0.86, 0.12), "wood_dark")
    for i, (x, y, z, s) in enumerate(((-0.44, -0.10, 0.52, 0.62),
                                       (0.28, 0.10, 0.52, 0.68),
                                       (-0.08, -0.06, 1.00, 0.54))):
        slab(p, "crate_%d" % i, (x, y, z), (s, s, s * 0.72), "wood")
        for j, dx in enumerate((-s / 2 + 0.04, s / 2 - 0.04)):
            slab(p, "slat_%d_%d" % (i, j), (x + dx, y - s / 2 - 0.01, z),
                 (0.06, 0.03, s * 0.72), "wood_dark")
    for i, (x, y, z) in enumerate(((-0.46, -0.06, 0.88), (0.22, 0.06, 0.88),
                                    (0.46, 0.16, 0.56), (-0.10, -0.04, 1.32))):
        lump(p, "produce_%d" % i, (x, y, z), 0.13,
             "enemy" if i % 2 else "leaf")
    flat(p, "rope", (0.50, -0.34, 0.24), 0.03, 0.48, "swamp_dark", 5, 0.3)
    return p


def build_shop_sign():
    p = collection("shop_sign")
    rod(p, "post", (0, 0, 0.75), 0.075, 1.48, "wood_dark", 6)
    slab(p, "arm", (0.30, 0, 1.38), (0.72, 0.10, 0.10), "wood")
    slab(p, "board", (0.56, 0, 1.14), (0.74, 0.12, 0.52), "wood")
    for i, (z, w) in enumerate(((1.38, 0.78), (0.90, 0.78))):
        slab(p, "trim_%d" % i, (0.56, -0.01, z), (w, 0.14, 0.06), "gold")
    for i, x in enumerate((0.32, 0.80)):
        slab(p, "edge_%d" % i, (x, 0, 1.14), (0.06, 0.14, 0.52), "wood_dark")
    rod(p, "mug", (0.56, -0.09, 1.15), 0.10, 0.24, "bone", 6,
        (math.pi / 2, 0, 0))
    lump(p, "foam", (0.56, -0.16, 1.24), 0.08, "snow")
    for i, x in enumerate((0.38, 0.74)):
        rod(p, "chain_%d" % i, (x, 0, 1.46), 0.02, 0.20, "iron", 5)
    slab(p, "foot", (0, 0, 0.05), (0.30, 0.30, 0.10), "stone_dark")
    return p


def build_bench_ornate():
    p = collection("bench_ornate")
    for i, y in enumerate((-0.18, 0.12)):
        slab(p, "seat_%d" % i, (0, y, 0.52), (1.50, 0.26, 0.08), "wood")
    for i, y in enumerate((0.22, 0.36)):
        slab(p, "back_%d" % i, (0, y, 0.80 + i * 0.22),
             (1.50, 0.08, 0.18), "wood")
    for i, x in enumerate((-0.62, 0.62)):
        for j, y in enumerate((-0.18, 0.24)):
            slab(p, "leg_%d_%d" % (i, j), (x, y, 0.25),
                 (0.12, 0.12, 0.50), "wood_dark")
        slab(p, "arm_%d" % i, (x, 0.02, 0.74), (0.12, 0.60, 0.10), "wood_dark")
        lump(p, "finial_%d" % i, (x, 0.30, 0.82), 0.11, "gold")
        slab(p, "brace_%d" % i, (x * 0.72, 0.06, 0.36),
             (0.08, 0.44, 0.08), "wood_dark")
    return p


def build_vendor_cart():
    p = collection("vendor_cart")
    slab(p, "bed", (0, 0, 0.62), (1.54, 0.90, 0.16), "wood")
    for i, x in enumerate((-0.62, 0.62)):
        rod(p, "wheel_%d" % i, (x, -0.50, 0.42), 0.36, 0.12, "wood_dark", 8,
            (math.pi / 2, 0, 0))
        lump(p, "hub_%d" % i, (x, -0.58, 0.42), 0.11, "iron")
        rod(p, "canopy_post_%d" % i, (x, 0.34, 1.30), 0.05, 1.34, "wood", 5)
    slab(p, "awning", (0, 0, 1.98), (1.66, 1.04, 0.06), "cloth")
    for i, x in enumerate((-0.80, 0.80)):
        slab(p, "valance_%d" % i, (x, 0.10, 1.88), (0.06, 0.86, 0.16), "crimson")
    for i, (x, y) in enumerate(((-0.34, -0.12), (0.30, 0.14), (0.04, -0.24))):
        slab(p, "crate_%d" % i, (x, y, 0.82), (0.38, 0.34, 0.32), "wood")
    for i, x in enumerate((-0.32, 0.34)):
        lump(p, "fruit_%d" % i, (x, -0.10, 1.06), 0.12, "enemy")
    rod(p, "shaft", (0.86, -0.30, 0.60), 0.05, 0.72, "wood", 6,
        (0, 0, math.pi / 2))
    return p


def build_street_planter():
    p = collection("street_planter")
    slab(p, "trough", (0, 0, 0.30), (1.10, 0.54, 0.56), "stone_dark")
    slab(p, "soil", (0, 0, 0.60), (0.98, 0.44, 0.08), "wood_dark")
    for i, (x, z, w) in enumerate(((-0.60, 0.42, 0.10), (0.60, 0.42, 0.10),
                                    (0, -0.30, 1.10), (0, 0.30, 1.10))):
        slab(p, "rim_%d" % i, (x, 0 if abs(x) < 0.5 else 0, z),
             (w if abs(x) < 0.5 else 0.10, 0.56 if abs(x) < 0.5 else 0.10,
              0.10), "stone")
    for i, x in enumerate((-0.30, 0.30)):
        lump(p, "shrub_%d" % i, (x, 0, 0.78), 0.24, "leaf")
    for i, (x, y) in enumerate(((-0.44, -0.12), (0.12, 0.18), (0.44, -0.10))):
        rod(p, "stem_%d" % i, (x, y, 0.82), 0.025, 0.34, "leaf_dark", 5)
        lump(p, "flower_%d" % i, (x, y, 1.02), 0.09, "gold")
    slab(p, "feet", (0, 0, 0.04), (0.86, 0.40, 0.08), "stone")
    return p


def build_hitching_post():
    p = collection("hitching_post")
    for i, x in enumerate((-0.68, 0.68)):
        rod(p, "post_%d" % i, (x, 0, 0.51), 0.10, 1.02, "wood_dark", 6)
        tip(p, "cap_%d" % i, (x, 0, 1.08), 0.12, 0.0, 0.16, "iron", 6)
    slab(p, "rail", (0, 0, 0.86), (1.42, 0.14, 0.12), "wood")
    for i, x in enumerate((-0.42, 0.0, 0.42)):
        rod(p, "ring_%d" % i, (x, -0.12, 0.72), 0.07, 0.05, "iron", 6,
            (math.pi / 2, 0, 0))
    slab(p, "foot", (0, 0, 0.05), (1.56, 0.30, 0.10), "stone_dark")
    return p


TOWN = [
    ("market_crates", build_market_crates, (1.6, 1.0, 1.4),
     "town prop", "stacked stall crates + produce"),
    ("shop_sign", build_shop_sign, (1.0, 0.4, 1.7),
     "town prop", "hanging painted mug board"),
    ("bench_ornate", build_bench_ornate, (1.6, 0.7, 1.1),
     "town prop", "carved two-seat bench"),
    ("vendor_cart", build_vendor_cart, (1.8, 1.1, 2.0),
     "town prop", "canopied market cart"),
    ("street_planter", build_street_planter, (1.2, 0.7, 1.1),
     "town prop", "flowering trough"),
    ("hitching_post", build_hitching_post, (1.8, 0.4, 1.2),
     "town prop", "rail with three rings"),
]


# ===========================================================================
# D - dungeon props (6)
# ===========================================================================
def build_blood_altar():
    p = collection("blood_altar")
    slab(p, "base", (0, 0, 0.10), (1.36, 0.82, 0.20), "stone_dark")
    slab(p, "step", (0, 0, 0.30), (1.12, 0.68, 0.20), "stone")
    slab(p, "table", (0, 0, 0.62), (0.90, 0.54, 0.44), "stone_dark")
    for i, (x, w) in enumerate(((-0.52, 0.08), (0.52, 0.08))):
        slab(p, "rim_%d" % i, (x, 0, 0.86), (w, 0.60, 0.10), "stone_light")
    for i, y in enumerate((-0.32, 0.32)):
        slab(p, "rim_%d" % i, (0, y, 0.86), (1.08, 0.08, 0.10), "stone_light")
    rod(p, "chalice", (0, 0, 1.02), 0.11, 0.20, "gold", 6)
    tip(p, "bowl", (0, 0, 1.16), 0.14, 0.06, 0.12, "gold", 6)
    for i, x in enumerate((-0.32, 0.32)):
        rod(p, "candle_%d" % i, (x, 0.12, 1.04), 0.045, 0.30, "bone", 5)
        tip(p, "flame_%d" % i, (x, 0.12, 1.23), 0.07, 0.0, 0.14, "fire", 5)
    for i, x in enumerate((-0.30, 0.0, 0.30)):
        slab(p, "rune_%d" % i, (x, -0.30, 0.62), (0.10, 0.03, 0.24),
             "crimson")
    for i, x in enumerate((-0.40, 0.42)):
        slab(p, "drip_%d" % i, (x, 0.20, 0.52), (0.05, 0.05, 0.52), "crimson")
    return p


def build_chain_hoist():
    p = collection("chain_hoist")
    slab(p, "mount", (0, 0, 1.90), (0.76, 0.30, 0.18), "stone_dark")
    rod(p, "wheel", (0, -0.14, 1.76), 0.28, 0.10, "iron", 8,
        (math.pi / 2, 0, 0))
    rod(p, "axle", (0, -0.14, 1.76), 0.07, 0.44, "steel", 6,
        (math.pi / 2, 0, 0))
    for i in range(8):
        z = 0.48 + i * 0.16
        o = slab(p, "link_%d" % i, (0.06, 0, z), (0.14, 0.09, 0.14), "iron")
        o.rotation_euler.z = math.pi / 2 if i % 2 else 0
    tip(p, "hook", (0.06, 0, 0.34), 0.10, 0.02, 0.24, "steel", 6,
        (math.pi, 0, 0))
    slab(p, "cage_floor", (0.06, 0, 0.10), (0.48, 0.40, 0.08), "iron")
    for i, (x, y) in enumerate(((-0.16, -0.14), (0.28, -0.14),
                                 (0.28, 0.14), (-0.16, 0.14))):
        rod(p, "cage_bar_%d" % i, (x, y, 0.34), 0.025, 0.48, "iron", 5)
    slab(p, "cage_top", (0.06, 0, 0.58), (0.48, 0.40, 0.06), "iron")
    return p


def build_open_sarcophagus():
    p = collection("open_sarcophagus")
    slab(p, "floor", (0, 0, 0.08), (1.86, 0.88, 0.16), "stone_dark")
    for i, y in enumerate((-0.40, 0.40)):
        slab(p, "side_%d" % i, (0, y, 0.42), (1.86, 0.10, 0.60), "stone")
    for i, x in enumerate((-0.88, 0.88)):
        slab(p, "end_%d" % i, (x, 0, 0.42), (0.10, 0.80, 0.60), "stone")
    slab(p, "inner", (0, 0, 0.20), (1.60, 0.66, 0.10), "char")
    o = slab(p, "lid", (0, 0.62, 0.56), (1.92, 0.62, 0.14), "stone_light")
    o.rotation_euler.x = 0.42
    lump(p, "skull", (-0.44, 0, 0.32), 0.15, "bone")
    lump(p, "mask", (-0.44, -0.10, 0.34), 0.09, "gold")
    for i, (x, y, a) in enumerate(((0.18, -0.14, 0.2), (0.52, 0.12, -0.4),
                                    (0.02, 0.20, 0.6))):
        flat(p, "bone_%d" % i, (x, y, 0.30), 0.035, 0.44, "bone", 4, a)
    for i, x in enumerate((-0.60, 0.60)):
        slab(p, "trim_%d" % i, (x, 0, 0.76), (0.10, 0.86, 0.06), "gold")
    return p


def build_portcullis_gate():
    p = collection("portcullis_gate")
    for i, x in enumerate((-0.78, 0.78)):
        slab(p, "jamb_%d" % i, (x, 0, 1.20), (0.14, 0.34, 2.40), "stone_dark")
    slab(p, "lintel", (0, 0, 2.32), (1.70, 0.40, 0.16), "stone")
    for i in range(7):
        x = -0.60 + i * 0.20
        slab(p, "bar_%d" % i, (x, 0, 1.14), (0.07, 0.16, 2.28), "iron")
        tip(p, "spike_%d" % i, (x, 0, 0.08), 0.09, 0.0, 0.20, "steel", 5,
            (math.pi, 0, 0))
    for i, z in enumerate((0.52, 1.72)):
        slab(p, "brace_%d" % i, (0, 0.10, z), (1.48, 0.08, 0.10), "iron")
    for i, x in enumerate((-0.66, 0.66)):
        lump(p, "hinge_%d" % i, (x, 0.12, 2.10), 0.09, "steel")
    for i, x in enumerate((-0.40, 0.40)):
        rod(p, "chain_%d" % i, (x, 0, 2.44), 0.03, 0.24, "iron", 5)
    return p


def build_crypt_brazier():
    p = collection("crypt_brazier")
    lump(p, "base", (0, 0, 0.10), 0.26, "stone_dark", (1.3, 1.3, 0.5))
    for i, a in enumerate((0.0, 2.09, 4.19)):
        rod(p, "leg_%d" % i, (0.18 * math.cos(a), 0.18 * math.sin(a), 0.34),
            0.045, 0.52, "iron", 5, (0.14 * math.sin(a), -0.14 * math.cos(a), 0))
    tip(p, "bowl", (0, 0, 0.72), 0.30, 0.22, 0.24, "iron", 8)
    lump(p, "coal", (0, 0, 0.82), 0.16, "char")
    for i, (x, y, h) in enumerate(((-0.10, 0.02, 0.48), (0.08, -0.06, 0.62),
                                    (0.02, 0.12, 0.36))):
        tip(p, "flame_%d" % i, (x, y, 0.90 + h / 2), 0.11, 0.0, h, "fire", 5)
    for i, (x, y) in enumerate(((-0.26, 0.20), (0.28, -0.16))):
        lump(p, "ember_%d" % i, (x, y, 0.08), 0.08, "fire")
    return p


def build_bone_chandelier():
    p = collection("bone_chandelier")
    rod(p, "ring", (0, 0, 0.62), 0.50, 0.08, "bone", 8, (math.pi / 2, 0, 0))
    for i, a in enumerate((0.0, 1.05, 2.09, 3.14, 4.19, 5.24)):
        x, y = 0.44 * math.cos(a), 0.44 * math.sin(a)
        rod(p, "candle_%d" % i, (x, y, 0.78), 0.05, 0.28, "bone", 5)
        tip(p, "flame_%d" % i, (x, y, 0.96), 0.07, 0.0, 0.14, "fire", 5)
    for i, a in enumerate((0.5, 2.6, 4.7)):
        x, y = 0.48 * math.cos(a), 0.48 * math.sin(a)
        rod(p, "chain_%d" % i, (x, y, 1.06), 0.025, 0.78, "iron", 5)
    lump(p, "skull", (0, 0, 0.56), 0.15, "bone")
    for i, (x, y, ang) in enumerate(((-0.28, 0.26, 0.4), (0.30, -0.20, -0.5),
                                      (0.06, -0.38, 0.2), (-0.34, -0.10, 1.1))):
        flat(p, "bone_%d" % i, (x, y, 0.66), 0.035, 0.40, "bone_dark", 4, ang)
    return p


DUNGEON = [
    ("blood_altar", build_blood_altar, (1.4, 0.9, 1.3),
     "dungeon prop", "chalice altar + runes"),
    ("chain_hoist", build_chain_hoist, (0.9, 0.7, 2.1),
     "dungeon prop", "wheel, chain + hanging cage"),
    ("open_sarcophagus", build_open_sarcophagus, (2.0, 1.0, 0.9),
     "dungeon prop", "open lid + remains"),
    ("portcullis_gate", build_portcullis_gate, (1.7, 0.4, 2.5),
     "dungeon prop", "spiked iron gate"),
    ("crypt_brazier", build_crypt_brazier, (0.8, 0.8, 1.3),
     "dungeon prop", "tripod fire bowl"),
    ("bone_chandelier", build_bone_chandelier, (1.2, 1.2, 1.4),
     "dungeon prop", "six-candle ring + skull"),
]


# ===========================================================================
# E - hero interiors (3): 1u grid, 2.5u storey, 0.9x2.0 doors
# ===========================================================================
def _room_floor(p, half, key="stone_dark", planks=False):
    bm_box("floor_slab", (0, 0, 0.05), (half * 2, half * 2, 0.10), key, p)
    if planks:
        for j in range(-half * 2 + 1, half * 2):
            bm_box("plank_seam_%d" % j, (0, j * 0.5, 0.105),
                   (half * 2, 0.02, 0.01), "wood_dark", p)
        return
    for i in range(-half + 1, half):
        bm_box("grout_x_%d" % i, (i, 0, 0.105), (0.02, half * 2, 0.01),
               "stone", p)
        bm_box("grout_y_%d" % i, (0, i, 0.105), (half * 2, 0.02, 0.01),
               "stone", p)


def _room_walls(p, half, courses=3, door_on_south=True, mat_key="stone",
                post_size=1.0):
    """Walls on the ±half grid lines; a true 0.9x2.0 door in the south wall.

    The two camera-facing walls (south and east) are a 1.1 u diorama cutaway
    except for the door cell, so the hero props stay readable from the 45°
    isometric camera. North and west keep all three 2.5 u courses.
    """
    idx = 0
    # side="x" is the wall at constant Y (south/north); side="y" is the wall
    # at constant X (west/east). The iso camera looks from the south-east:
    # south (y=-half) and east (x=+half) are the cutaway walls.
    for side, fixed in (("x", -half), ("x", half), ("y", -half), ("y", half)):
        for i in range(-half + 1, half):
            door_cell = side == "x" and fixed < 0 and i == 0
            if door_cell:
                continue  # a real opening; jambs, leaf and lintel fill it
            x, y = (i, fixed) if side == "x" else (fixed, i)
            rot = (0.0, 0.0, 0.0) if side == "x" else (0.0, 0.0, math.pi / 2)
            cutaway = ((side == "x" and fixed < 0)
                       or (side == "y" and fixed > 0))
            if cutaway:
                bm_box("wall_%d" % idx, (x, y, 0.55), (1.0, 0.16, 1.10),
                       mat_key, p, rot)
                idx += 1
                continue
            for course in range(courses):
                h = 2.5 / courses
                bm_box("wall_%d" % idx, (x, y, h * (course + 0.5)),
                       (1.0, 0.16, h - 0.02),
                       mat_key if course != 0 else "stone_dark", p, rot)
                idx += 1
    if door_on_south:
        for i, x in enumerate((-0.50, 0.50)):
            bm_box("door_jamb_%d" % i, (x, -half, 1.0),
                   (0.10, 0.20, 2.0), "wood_dark", p)
        bm_box("door_leaf", (0, -half - 0.05, 1.0),
               (0.90, 0.10, 2.0), "wood", p)
        bm_box("door_lintel", (0, -half, 2.08),
               (1.10, 0.24, 0.16), "stone_dark", p)
        for i, z in enumerate((0.50, 1.52)):
            bm_box("door_band_%d" % i, (0, -half - 0.11, z),
                   (0.90, 0.04, 0.08), "iron", p)
        bm_cone("door_knob", (0.28, -half - 0.14, 1.0), 0.07, 0.0, 0.12,
                "gold", p, 5, (math.pi / 2, 0, 0))
    # Corner posts close the 0.5 u gaps between the wall runs. Only the
    # south-east corner is low; the other three meet full-height far walls.
    for i, (x, y) in enumerate(((half, -half), (-half, -half),
                                (-half, half), (half, half))):
        h = 1.1 if (x > 0 and y < 0) else 2.5
        bm_box("corner_post_%d" % i, (x, y, h / 2),
               (post_size, post_size, h), mat_key, p)


def _beam(p, name, x, y, width=4.0):
    """A ceiling beam the hanging lanterns/censer actually attach to."""
    bm_box(name, (x, y, 2.40), (width, 0.16, 0.16), "wood_dark", p)


def _window(p, x, y, z=1.1):
    bm_box("window_frame", (x, y, z + 0.45), (0.90, 0.10, 0.90),
           "wood_dark", p)
    bm_box("window_glass", (x, y - 0.06, z + 0.45), (0.72, 0.02, 0.72),
           "ice", p)
    bm_box("window_sill", (x, y - 0.08, z), (1.02, 0.18, 0.08), "stone_light", p)
    bm_box("window_bar", (x, y - 0.08, z + 0.45), (0.05, 0.04, 0.72), "iron", p)
    # A straight stone lintel seated exactly on the frame head (z + 0.90).
    # No angled voussoirs: repeated critiques showed they read as detached
    # shards at this scale, and a plain lintel is cleaner in the diorama style.
    bm_box("window_lintel", (x, y - 0.03, z + 0.97),
           (1.02, 0.14, 0.14), "stone_dark", p)


def build_tavern_interior():
    p = collection("tavern_interior")
    _room_floor(p, 2, "wood", planks=True)
    _room_walls(p, 2, 3, True, "wood")
    for x in (-1.0, 1.0):
        _window(p, x, 1.94)
    # A sand rug and a pale stone bar top break up the all-wood read at 60 px.
    bm_box("rug", (0.10, 0.20, 0.11), (0.90, 0.60, 0.02), "sand", p)
    bm_box("rug_border", (0.10, 0.20, 0.125), (0.70, 0.42, 0.01), "gold", p)
    # Long bar counter on the east cutaway wall with three stools.
    bm_box("counter_base", (1.30, -0.40, 0.46), (0.62, 2.20, 0.90), "wood_dark", p)
    bm_box("counter_top", (1.28, -0.40, 0.96), (0.80, 2.40, 0.10), "stone_light", p)
    bm_box("counter_inset", (1.28, -0.40, 1.02), (0.30, 2.20, 0.03), "wood", p)
    bm_box("counter_rail", (0.90, -0.40, 0.98), (0.12, 2.40, 0.08), "gold", p)
    for i, (y, key) in enumerate(((-1.00, "swamp"), (-0.40, "gold"),
                                  (0.20, "enemy"))):
        bm_cone("bottle_%d" % i, (1.28, y, 1.16), 0.09, 0.06, 0.24,
                key, p, 6)
    for i, y in enumerate((-1.20, -0.40, 0.40)):
        # True bar stools on the room side, tall enough to clear the counter
        # lip from the 30-degree isometric camera.
        bm_cone("stool_seat_%d" % i, (0.38, y, 1.12), 0.22, 0.22, 0.10,
                "wood_dark", p, 6)
        for j, a in enumerate((0.0, 2.09, 4.19)):
            bm_box("stool_leg_%d_%d" % (i, j),
                   (0.38 + 0.15 * math.cos(a), y + 0.15 * math.sin(a), 0.59),
                   (0.06, 0.06, 0.98), "wood", p)
    # Two round tables with benches and mugs.
    for i, (x, y) in enumerate(((-1.10, -0.90), (-0.90, 1.25))):
        bm_cone("table_top_%d" % i, (x, y, 0.76), 0.50, 0.50, 0.08, "wood", p, 8)
        bm_box("table_post_%d" % i, (x, y, 0.38), (0.12, 0.12, 0.72),
               "wood_dark", p)
        bm_cone("table_foot_%d" % i, (x, y, 0.06), 0.28, 0.20, 0.10,
                "wood_dark", p, 6)
        for j, dy in enumerate((-0.62, 0.62)):
            bm_box("bench_%d_%d" % (i, j), (x, y + dy, 0.44),
                   (0.92, 0.26, 0.08), "wood", p)
            for k, sx in enumerate((-0.35, 0.35)):
                bm_box("bench_leg_%d_%d_%d" % (i, j, k),
                       (x + sx, y + dy, 0.25), (0.08, 0.08, 0.30),
                       "wood_dark", p)
        bm_cone("mug_%d" % i, (x, y, 0.90), 0.13, 0.13, 0.20, "sand", p, 6)
    # Fireplace on the far west wall, with a narrow flame and one hot core.
    bm_box("hearth", (-1.88, 0.20, 0.48), (0.28, 1.30, 0.96), "stone", p)
    bm_box("firebox", (-1.70, 0.20, 0.42), (0.12, 0.86, 0.66), "stone_dark", p)
    for i, y in enumerate((0.0, 0.20, 0.40)):
        bm_cone("flame_%d" % i, (-1.60, y, 0.68), 0.15, 0.0, 0.76, "fire", p, 5)
        if i == 1:
            bm_cone("flame_core", (-1.55, y, 0.66), 0.07, 0.0, 0.46,
                    "gold", p, 5)
    bm_box("mantel", (-1.84, 0.20, 1.02), (0.42, 1.50, 0.12), "stone", p)
    # Lanterns hang from short brackets on the full-height west/north walls;
    # the bar sign rises from the counter. No unsupported ceiling rails.
    bm_box("lantern_bracket_0", (-1.80, -0.40, 2.34), (0.24, 0.08, 0.08),
           "iron", p)
    bm_box("lantern_bracket_1", (0.50, 1.80, 2.34), (0.08, 0.24, 0.08),
           "iron", p)
    for i, (x, y) in enumerate(((-1.68, -0.40), (0.50, 1.68))):
        bm_box("lantern_chain_%d" % i, (x, y, 2.24), (0.04, 0.04, 0.40),
               "iron", p)
        bm_cone("lantern_%d" % i, (x, y, 1.90), 0.20, 0.13, 0.32, "gold", p, 6)
    bm_box("sign_post", (0.95, -0.40, 1.26), (0.08, 0.08, 0.50), "iron", p)
    bm_box("bar_sign", (0.95, -0.40, 1.34), (0.10, 0.68, 0.36), "gold", p)
    return p


def build_smithy_interior():
    p = collection("smithy_interior")
    _room_floor(p, 2, "stone_dark")
    _room_walls(p, 2, 3, True, "stone")
    _window(p, 0.8, 1.94, 1.1)
    # Forge: dark stone hearth, exposed iron coal bed, a large warm flame.
    bm_box("forge_base", (-1.55, -0.60, 0.42), (0.72, 1.30, 0.84), "stone_dark", p)
    bm_box("coal_bed", (-1.30, -0.60, 0.86), (0.30, 0.96, 0.14), "char", p)
    for i, y in enumerate((-0.88, -0.60, -0.32)):
        bm_cone("flame_%d" % i, (-1.24, y, 1.20), 0.18, 0.0, 0.82, "fire", p, 5)
        bm_cone("flame_core_%d" % i, (-1.18, y, 1.12), 0.08, 0.0, 0.52,
                "gold", p, 5)
    bm_box("flue", (-1.86, -0.60, 1.23), (0.32, 1.00, 0.80), "stone_dark", p)
    bm_box("chimney", (-1.86, -0.60, 2.06), (0.32, 1.00, 0.88), "stone_dark", p)
    bm_box("chimney_cap", (-1.86, -0.60, 2.42), (0.42, 1.10, 0.10), "stone", p)
    bm_box("forge_lip", (-1.16, -0.60, 0.88), (0.06, 0.90, 0.10), "char", p)
    # Anvil on a stump. The horn is a cone about local Y, so it lies horizontal.
    bm_cone("stump", (0.24, -0.30, 0.34), 0.40, 0.36, 0.66, "wood_dark", p, 8)
    bm_box("anvil_base", (0.24, -0.30, 0.78), (0.86, 0.52, 0.28), "iron", p)
    bm_box("anvil_body", (0.24, -0.30, 1.00), (0.60, 0.40, 0.30), "steel", p)
    bm_cone("anvil_horn", (0.72, -0.30, 1.00), 0.22, 0.0, 0.60, "steel", p, 6,
            (0, math.pi / 2, 0))
    bm_box("tool_rack", (-1.00, 1.86, 1.72), (1.60, 0.14, 0.14), "wood_dark", p)
    for i, x in enumerate((-1.40, -1.00, -0.60)):
        bm_box("tool_%d" % i, (x, 1.74, 1.44), (0.10, 0.10, 0.52), "iron", p)
        bm_box("tool_head_%d" % i, (x, 1.74, 1.18), (0.24, 0.24, 0.14),
               "steel", p)
    bm_box("trough", (0.90, 1.30, 0.28), (1.10, 0.54, 0.56), "wood_dark", p)
    bm_box("trough_water", (0.90, 1.30, 0.58), (0.96, 0.42, 0.04), "water", p)
    # Torches have dark backplates and short arms on the full-height far walls.
    bm_box("sconce_back_0", (-1.92, 1.00, 1.70), (0.12, 0.32, 0.34), "char", p)
    bm_box("sconce_arm_0", (-1.76, 1.00, 1.70), (0.32, 0.08, 0.08), "iron", p)
    bm_box("sconce_back_1", (-0.30, 1.92, 1.70), (0.32, 0.12, 0.34), "char", p)
    bm_box("sconce_arm_1", (-0.30, 1.76, 1.70), (0.08, 0.32, 0.08), "iron", p)
    for i, (x, y) in enumerate(((-1.66, 1.00), (-0.30, 1.66))):
        bm_cone("torch_flame_%d" % i, (x, y, 1.92), 0.16, 0.0, 0.40,
                "fire", p, 5)
    return p


def build_shrine_interior():
    p = collection("shrine_interior")
    _room_floor(p, 2, "stone_light")
    # Slimmer corner posts and inboard windows keep the north glazing and its
    # dark lintels visible from the south-east isometric camera.
    _room_walls(p, 2, 3, True, "stone_light", post_size=0.75)
    # 0.125 u infills close the gap between each 1 u wall run and its post.
    for x in (-1.5625, 1.5625):
        for y in (-2.0, 2.0):
            h = 1.1 if (x > 0 and y < 0) else 2.5
            bm_box("wall_infill_x_%s_%s" % (x, y), (x, y, h / 2),
                   (0.125, 0.16, h), "stone_light", p)
    for x in (-2.0, 2.0):
        for y in (-1.5625, 1.5625):
            h = 1.1 if (x > 0 and y < 0) else 2.5
            bm_box("wall_infill_y_%s_%s" % (x, y), (x, y, h / 2),
                   (0.16, 0.125, h), "stone_light", p)
    for x in (-1.2, 1.2):
        _window(p, x, 1.94, 1.1)
    # Raised dais and altar at the north wall.
    bm_box("dais", (0, 1.10, 0.16), (2.20, 1.20, 0.32), "stone_dark", p)
    bm_box("altar_step", (0, 1.02, 0.42), (1.50, 0.90, 0.20), "stone", p)
    bm_box("altar_body", (0, 1.02, 0.78), (1.06, 0.62, 0.54), "stone_dark", p)
    bm_box("altar_top", (0, 1.02, 1.08), (1.24, 0.76, 0.12), "stone", p)
    bm_box("altar_rim", (0, 1.02, 1.15), (1.10, 0.62, 0.08), "gold", p)
    # Faceted bipyramid crystal: two six-sided cones meet at their equator.
    bm_cone("crystal_top", (0, 1.02, 1.72), 0.32, 0.0, 0.64, "arcane", p, 6)
    bm_cone("crystal_bottom", (0, 1.02, 1.25), 0.32, 0.0, 0.34,
            "arcane", p, 6, (math.pi, 0, 0))
    for i, x in enumerate((-0.48, 0.48)):
        bm_cone("candle_%d" % i, (x, 0.92, 1.32), 0.09, 0.09, 0.40,
                "bone", p, 6)
        bm_cone("flame_%d" % i, (x, 0.92, 1.58), 0.08, 0.0, 0.18,
                "fire", p, 5)
    # Two short paired columns flank the crystal; they stay clear of the
    # window bays in the 45-degree iso projection (screen x ~= x + y).
    for i, x in enumerate((-0.20, 0.20)):
        bm_cone("column_%d" % i, (x, 1.45, 0.90), 0.12, 0.12, 1.80,
                "stone_dark", p, 8)
        bm_box("capital_%d" % i, (x, 1.45, 1.80), (0.30, 0.30, 0.12),
               "stone_light", p)
    bm_box("banner_0", (-1.92, 0.20, 1.72), (0.05, 0.66, 1.10), "arcane", p)
    bm_box("banner_rod_0", (-1.86, 0.20, 2.30), (0.18, 0.72, 0.06), "gold", p)
    bm_box("banner_1", (-1.92, -0.60, 1.72), (0.05, 0.66, 1.10), "arcane", p)
    bm_box("banner_rod_1", (-1.86, -0.60, 2.30), (0.18, 0.72, 0.06), "gold", p)
    # Kneeling cushions and a censer hung from the beam between the columns.
    for i, x in enumerate((-0.44, 0.44)):
        bm_box("cushion_%d" % i, (x, -0.30, 0.17), (0.52, 0.42, 0.14),
               "cloth", p)
        bm_box("cushion_top_%d" % i, (x, -0.30, 0.25), (0.42, 0.32, 0.02),
               "sand", p)
    bm_box("censer_beam", (0, 1.90, 2.40), (4.00, 0.16, 0.16), "wood_dark", p)
    bm_box("censer_chain", (0, 1.90, 2.31), (0.04, 0.04, 0.34), "iron", p)
    bm_cone("censer", (0, 1.90, 2.02), 0.16, 0.10, 0.24, "gold", p, 6)
    return p


INTERIORS = [
    ("tavern_interior", build_tavern_interior, (4.2, 4.2, 2.5),
     "interior hero", "4x4 tavern; bar, tables, hearth, 0.9x2.0 door"),
    ("smithy_interior", build_smithy_interior, (4.2, 4.2, 2.5),
     "interior hero", "4x4 smithy; forge, anvil, trough, 0.9x2.0 door"),
    ("shrine_interior", build_shrine_interior, (4.2, 4.2, 2.5),
     "interior hero", "4x4 shrine; altar, crystal, banners, 0.9x2.0 door"),
]


# ---------------------------------------------------------------------------
ASSETS = BIOME + ENEMY + TOWN + DUNGEON + INTERIORS
EXPORTS = [name for name, _builder, _target, _family, _notes in ASSETS]
HEROES = {name for name, *_ in INTERIORS}

for name, builder, target, family, notes in ASSETS:
    col = builder()
    is_hero = name in HEROES
    rec = finalize(name, col, "models_v5", target, "%s; %s" % (family, notes),
                   hero=is_hero, bevel_width=0.006,
                   bevel_parts=("floor_slab", "counter_top", "altar_top",
                                "anvil_base") if is_hero else None,
                   records=RECORDS)
    budget = 1500 if is_hero else 500
    if rec["triangles"] > budget:
        print("BUDGET WARNING %s: %d > %d" % (name, rec["triangles"], budget))
    SETS.append((name, col))

write_records("models_v5", RECORDS)
print("WAVE5: %d assets, %.2f u player reference" % (len(RECORDS), PLAYER))
try:
    contact_sheet(SETS, "models_v5.png", width=3200, height=720)
except Exception as exc:  # previews are optional; never fail the build for them
    print("PREVIEW SKIPPED", exc)
