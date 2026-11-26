"""Starfall character & enemy generator.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_characters.py

Every asset is a flat-shaded, texture-free low-poly collection exported to
assets/models/<name>.glb.  The final contact sheet is rendered only after all
exports, because it shifts the objects to lay them out for the preview.
"""
import sys
import os

sys.path.insert(0, os.path.dirname(__file__))
from common import *  # noqa: F401,F403  (new_scene, mat, collection, box, cyl, cone, ico, export_asset, contact_sheet, C)

import math

PI = math.pi

new_scene()

# ---------------------------------------------------------------------------
# Palette materials (all hues come from docs/ART_DIRECTION.md)
# ---------------------------------------------------------------------------
M = {
    "tunic":   mat("player_tunic", C["player"]),
    "skin":    mat("skin", C["skin"]),
    "leather": mat("leather", C["leather"]),
    "wood":    mat("wood", C["wood"]),
    "cloth":   mat("cloth", C["cloth"]),
    "bone":    mat("bone", C["bone"]),
    "stone":   mat("stone", C["stone"]),
    "iron":    mat("iron", C["iron"]),
    "steel":   mat("steel", C["steel"], rough=0.4, metallic=0.6),
    "gold":    mat("gold", C["gold"], rough=0.45, metallic=0.5),
    "enemy":   mat("enemy", C["enemy"]),
    "fire":    mat("fire", C["fire"], emission=C["fire"], strength=1.4),
    "leaf":    mat("leaf", C["leaf"]),
    "grass":   mat("grass", C["grass"]),
    "swamp":   mat("swamp", C["swamp"]),
    "water":   mat("water", C["water"]),
    "arcane":  mat("arcane", C["arcane"]),
    "sand":    mat("sand", C["sand"]),
    "snow":    mat("snow", C["snow"]),
    "spark":   mat("spark", C["gold"], emission=C["fire"], strength=2.0),
}


# ---------------------------------------------------------------------------
# Thin wrappers: identical primitives plus optional rotation (flat via common)
# ---------------------------------------------------------------------------
def bx(name, loc, scale, material, col, rot=(0, 0, 0)):
    o = box(name, loc, scale, material, col)
    o.rotation_euler = rot
    return o


def cy(name, loc, radius, depth, material, col, vertices=8, rot=(0, 0, 0)):
    o = cyl(name, loc, radius, depth, material, col, vertices)
    o.rotation_euler = rot
    return o


def cn(name, loc, r1, r2, depth, material, col, vertices=8, rot=(0, 0, 0)):
    return cone(name, loc, r1, r2, depth, material, col, vertices, rot)


def ic(name, loc, radius, material, col, subdivisions=0, scale=None):
    return ico(name, loc, radius, material, col, subdivisions, scale)


def scale_collection(b, factor):
    """Uniformly resize a finished collection about the origin (keeps z=0)."""
    for o in b.objects:
        o.location *= factor
        o.scale *= factor


def face(b, z, y=-0.23, dx=0.09, colour=None, glow=False):
    """Two simple eyes on the -Y (front) side of a head."""
    m = M["fire"] if glow else (colour or M["leather"])
    for s, tag in ((-1, "l"), (1, "r")):
        ic("eye_" + tag, (s * dx, y, z), 0.04 if glow else 0.035, m, b)


# ===========================================================================
# HERO
# ===========================================================================
def build_player(b):
    # boots / legs
    bx("boot_l", (-0.14, 0, 0.09), (0.20, 0.28, 0.18), M["leather"], b)
    bx("boot_r", (0.14, 0, 0.09), (0.20, 0.28, 0.18), M["leather"], b)
    bx("leg_l", (-0.13, 0, 0.33), (0.18, 0.20, 0.30), M["cloth"], b)
    bx("leg_r", (0.13, 0, 0.33), (0.18, 0.20, 0.30), M["cloth"], b)
    # signature orange tunic + dark belt
    bx("tunic", (0, 0, 0.79), (0.46, 0.30, 0.54), M["tunic"], b)
    bx("belt", (0, 0, 0.55), (0.48, 0.32, 0.09), M["leather"], b)
    bx("buckle", (0, -0.17, 0.55), (0.10, 0.03, 0.09), M["gold"], b)
    # arms and hands
    bx("arm_l", (-0.31, 0, 0.84), (0.14, 0.16, 0.42), M["tunic"], b, (0, 0, 0.18))
    bx("arm_r", (0.31, 0, 0.84), (0.14, 0.16, 0.42), M["tunic"], b, (0, 0, -0.18))
    ic("hand_l", (-0.35, 0, 0.60), 0.07, M["skin"], b)
    ic("hand_r", (0.35, 0, 0.60), 0.07, M["skin"], b)
    # short hero cape
    bx("cape", (0, 0.19, 0.80), (0.42, 0.06, 0.50), M["cloth"], b)
    # head, hair, face
    bx("neck", (0, 0, 1.08), (0.12, 0.12, 0.10), M["skin"], b)
    ic("head", (0, 0, 1.32), 0.22, M["skin"], b, scale=(1.0, 0.92, 1.05))
    bx("hair", (0, 0.02, 1.47), (0.44, 0.40, 0.12), M["leather"], b)
    face(b, 1.33, -0.20)
    bx("mouth", (0, -0.21, 1.23), (0.08, 0.02, 0.025), M["leather"], b)
    # gold circlet = hero marker
    bx("circlet", (0, 0, 1.46), (0.46, 0.42, 0.045), M["gold"], b)


# ===========================================================================
# REGULAR ENEMIES
# ===========================================================================
def build_goblin(b):
    # short legs & big feet
    bx("leg_l", (-0.12, 0, 0.16), (0.14, 0.16, 0.32), M["leaf"], b)
    bx("leg_r", (0.12, 0, 0.16), (0.14, 0.16, 0.32), M["leaf"], b)
    bx("foot_l", (-0.12, -0.05, 0.05), (0.16, 0.24, 0.10), M["leather"], b)
    bx("foot_r", (0.12, -0.05, 0.05), (0.16, 0.24, 0.10), M["leather"], b)
    ic("body", (0, 0, 0.50), 0.25, M["grass"], b, scale=(1.0, 0.85, 1.1))
    bx("loincloth", (0, 0, 0.36), (0.34, 0.28, 0.16), M["leather"], b)
    # oversized head and huge flat ears = readable goblin silhouette
    ic("head", (0, -0.02, 0.86), 0.26, M["grass"], b, scale=(1.1, 0.95, 1.0))
    cn("ear_l", (-0.32, -0.02, 0.91), 0.11, 0, 0.40, M["grass"], b, 4, (0, -PI / 2, 0.25))
    cn("ear_r", (0.32, -0.02, 0.91), 0.11, 0, 0.40, M["grass"], b, 4, (0, PI / 2, -0.25))
    face(b, 0.90, -0.25, 0.10, glow=True)
    cn("nose", (0, -0.28, 0.82), 0.06, 0, 0.16, M["grass"], b, 4, (PI / 2, 0, 0))
    bx("tooth", (0, -0.28, 0.73), (0.10, 0.02, 0.04), M["bone"], b)
    bx("arm_l", (-0.30, 0, 0.55), (0.11, 0.12, 0.34), M["grass"], b, (0, 0, 0.30))
    bx("arm_r", (0.30, 0, 0.55), (0.11, 0.12, 0.34), M["grass"], b, (0, 0, -0.30))


def build_skeleton(b):
    for s, tag in ((-1, "l"), (1, "r")):
        x = s * 0.13
        bx("foot_" + tag, (x, -0.04, 0.05), (0.15, 0.26, 0.10), M["bone"], b)
        bx("shin_" + tag, (x, 0, 0.28), (0.09, 0.09, 0.42), M["bone"], b)
        bx("thigh_" + tag, (x, 0, 0.60), (0.11, 0.11, 0.42), M["bone"], b)
        # ribs fan out from the spine
        for i, z in enumerate((0.98, 1.10, 1.22)):
            bx("rib_%s%d" % (tag, i), (s * 0.19, 0, z), (0.24, 0.10, 0.055),
               M["bone"], b, (0, 0, -s * 0.18))
        ic("shoulder_" + tag, (s * 0.30, 0, 1.29), 0.12, M["bone"], b)
        bx("upperarm_" + tag, (s * 0.36, 0, 1.07), (0.08, 0.08, 0.42),
           M["bone"], b, (0, 0, -s * 0.15))
        bx("forearm_" + tag, (s * 0.40, -0.10, 0.75), (0.07, 0.07, 0.38), M["bone"], b)
    bx("pelvis", (0, 0, 0.85), (0.32, 0.18, 0.14), M["bone"], b)
    bx("spine", (0, 0.02, 1.10), (0.10, 0.10, 0.52), M["bone"], b)
    ic("skull", (0, 0, 1.45), 0.22, M["bone"], b, scale=(1.0, 0.9, 1.0))
    bx("jaw", (0, -0.06, 1.34), (0.26, 0.16, 0.09), M["bone"], b)
    bx("eye_l", (-0.09, -0.18, 1.47), (0.07, 0.02, 0.07), M["leather"], b)
    bx("eye_r", (0.09, -0.18, 1.47), (0.07, 0.02, 0.07), M["leather"], b)


def build_slime(b):
    cy("puddle", (0, 0, 0.025), 0.48, 0.05, M["leaf"], b, 8)
    ic("body", (0, 0, 0.38), 0.44, M["grass"], b, subdivisions=1, scale=(1.0, 1.0, 0.85))
    ic("core", (0, -0.05, 0.38), 0.20, M["water"], b, scale=(1.0, 0.8, 1.0))
    ic("eye_l", (-0.15, -0.37, 0.44), 0.07, M["bone"], b)
    ic("eye_r", (0.15, -0.37, 0.44), 0.07, M["bone"], b)
    ic("pupil_l", (-0.15, -0.42, 0.44), 0.035, M["leather"], b)
    ic("pupil_r", (0.15, -0.42, 0.44), 0.035, M["leather"], b)
    bx("mouth", (0, -0.42, 0.28), (0.16, 0.02, 0.03), M["leather"], b)


def build_wolf(b):
    ic("body", (0, 0.05, 0.58), 0.33, M["wood"], b, scale=(1.5, 0.9, 0.85))
    for s, tag in ((-1, "l"), (1, "r")):
        x = s * 0.24
        for y, part in ((-0.28, "front"), (0.32, "back")):
            bx("leg_%s_%s" % (tag, part), (x, y, 0.20), (0.11, 0.12, 0.40), M["wood"], b)
            bx("paw_%s_%s" % (tag, part), (x, y - 0.03, 0.045), (0.13, 0.17, 0.09), M["leather"], b)
    bx("neck", (0, -0.30, 0.75), (0.20, 0.20, 0.28), M["wood"], b, (0.35, 0, 0))
    ic("head", (0, -0.52, 0.82), 0.23, M["wood"], b, scale=(1.0, 1.05, 1.0))
    bx("snout", (0, -0.76, 0.74), (0.16, 0.22, 0.13), M["leather"], b)
    bx("nose", (0, -0.88, 0.76), (0.07, 0.05, 0.06), M["leather"], b)
    cn("ear_l", (-0.14, -0.50, 1.02), 0.09, 0, 0.24, M["wood"], b, 4)
    cn("ear_r", (0.14, -0.50, 1.02), 0.09, 0, 0.24, M["wood"], b, 4)
    face(b, 0.88, -0.70, 0.12, glow=True)
    cn("tail", (0, 0.62, 0.70), 0.13, 0, 0.55, M["wood"], b, 6, (-0.9, 0, 0))
    cn("tuft_l", (-0.10, 0.05, 0.95), 0.09, 0, 0.20, M["leather"], b, 4)
    cn("tuft_r", (0.10, 0.20, 0.95), 0.09, 0, 0.20, M["leather"], b, 4)


def build_bat(b):
    bx("foot_l", (-0.08, 0, 0.05), (0.08, 0.10, 0.10), M["leather"], b)
    bx("foot_r", (0.08, 0, 0.05), (0.08, 0.10, 0.10), M["leather"], b)
    ic("body", (0, 0, 0.40), 0.20, M["arcane"], b, scale=(0.8, 0.9, 1.2))
    ic("head", (0, -0.06, 0.62), 0.17, M["arcane"], b)
    cn("ear_l", (-0.10, -0.06, 0.80), 0.07, 0, 0.26, M["arcane"], b, 4)
    cn("ear_r", (0.10, -0.06, 0.80), 0.07, 0, 0.26, M["arcane"], b, 4)
    face(b, 0.64, -0.20, 0.07, glow=True)
    # flat 3-sided cones give crisp triangular wing membranes facing the camera
    cn("wing_l", (-0.44, 0, 0.60), 0.44, 0, 0.03, M["leather"], b, 3, (PI / 2, PI, 0))
    cn("wing_r", (0.44, 0, 0.60), 0.44, 0, 0.03, M["leather"], b, 3, (PI / 2, 0, 0))
    bx("strut_l", (-0.30, -0.03, 0.56), (0.34, 0.025, 0.035), M["arcane"], b, (0, 0.45, 0))
    bx("strut_r", (0.30, -0.03, 0.56), (0.34, 0.025, 0.035), M["arcane"], b, (0, -0.45, 0))


def build_spider(b):
    ic("abdomen", (0, 0.22, 0.42), 0.30, M["leather"], b, scale=(1.0, 1.2, 0.85))
    ic("thorax", (0, -0.18, 0.40), 0.19, M["stone"], b)
    # eight jointed legs, two segments each
    for s, tag in ((-1, "l"), (1, "r")):
        for i, y in enumerate((-0.28, -0.08, 0.12, 0.32)):
            bx("upper_%s%d" % (tag, i), (s * 0.33, y, 0.34), (0.36, 0.05, 0.05),
               M["stone"], b, (0, -s * 0.25, 0))
            bx("lower_%s%d" % (tag, i), (s * 0.60, y, 0.15), (0.34, 0.05, 0.05),
               M["stone"], b, (0, s * 0.90, 0))
    face(b, 0.45, -0.35, 0.09, glow=True)
    cn("fang_1", (-0.08, -0.34, 0.28), 0.045, 0, 0.14, M["bone"], b, 4, (PI, 0, 0))
    cn("fang_2", (0.08, -0.34, 0.28), 0.045, 0, 0.14, M["bone"], b, 4, (PI, 0, 0))


def build_archer(b):
    bx("boot_l", (-0.12, 0, 0.06), (0.16, 0.24, 0.12), M["leather"], b)
    bx("boot_r", (0.12, 0, 0.06), (0.16, 0.24, 0.12), M["leather"], b)
    bx("leg_l", (-0.12, 0, 0.30), (0.14, 0.16, 0.38), M["cloth"], b)
    bx("leg_r", (0.12, 0, 0.30), (0.14, 0.16, 0.38), M["cloth"], b)
    bx("torso", (0, 0, 0.82), (0.42, 0.28, 0.54), M["cloth"], b)
    bx("belt", (0, 0, 0.57), (0.44, 0.30, 0.08), M["leather"], b)
    # pointed hood with a dark face inside
    cn("hood", (0, 0, 1.39), 0.28, 0, 0.46, M["cloth"], b, 6)
    bx("shade", (0, -0.21, 1.32), (0.24, 0.02, 0.20), M["leather"], b)
    bx("eye_l", (-0.07, -0.225, 1.34), (0.035, 0.015, 0.035), M["bone"], b)
    bx("eye_r", (0.07, -0.225, 1.34), (0.035, 0.015, 0.035), M["bone"], b)
    bx("arm_l", (-0.28, 0, 0.85), (0.12, 0.14, 0.40), M["cloth"], b, (0, 0, 0.2))
    bx("arm_r", (0.28, 0, 0.85), (0.12, 0.14, 0.40), M["cloth"], b, (0, 0, -0.2))
    ic("hand_l", (-0.31, 0, 0.63), 0.065, M["skin"], b)
    ic("hand_r", (0.31, 0, 0.63), 0.065, M["skin"], b)
    # quiver on the back with three gold-tipped arrows
    cy("quiver", (-0.23, 0.22, 1.02), 0.09, 0.45, M["wood"], b, 6, (0.25, 0, 0.2))
    for i, x in enumerate((-0.30, -0.23, -0.16)):
        bx("shaft_%d" % i, (x, 0.27, 1.32), (0.02, 0.02, 0.32), M["wood"], b)
        cn("tip_%d" % i, (x, 0.27, 1.50), 0.04, 0, 0.10, M["gold"], b, 4)
    # bow: grip, two angled limbs and a string
    bx("bow_grip", (0.37, -0.05, 0.96), (0.05, 0.05, 0.28), M["wood"], b)
    bx("bow_top", (0.44, -0.05, 1.25), (0.05, 0.05, 0.36), M["wood"], b, (0, 0.45, 0))
    bx("bow_low", (0.44, -0.05, 0.67), (0.05, 0.05, 0.36), M["wood"], b, (0, -0.45, 0))
    bx("bow_string", (0.34, -0.05, 0.96), (0.012, 0.012, 0.86), M["bone"], b)


def build_raider(b):
    bx("boot_l", (-0.13, 0, 0.07), (0.17, 0.24, 0.14), M["leather"], b)
    bx("boot_r", (0.13, 0, 0.07), (0.17, 0.24, 0.14), M["leather"], b)
    bx("leg_l", (-0.13, 0, 0.32), (0.15, 0.16, 0.38), M["leather"], b)
    bx("leg_r", (0.13, 0, 0.32), (0.15, 0.16, 0.38), M["leather"], b)
    # flared cloak is the defining silhouette
    cn("cloak", (0, 0.05, 0.72), 0.44, 0.18, 0.96, M["enemy"], b, 6)
    bx("torso", (0, 0, 0.90), (0.38, 0.28, 0.46), M["leather"], b)
    bx("belt", (0, 0, 0.65), (0.42, 0.32, 0.09), M["gold"], b)
    bx("hood", (0, 0, 1.40), (0.36, 0.34, 0.32), M["leather"], b)
    bx("mask", (0, -0.19, 1.33), (0.24, 0.03, 0.18), M["stone"], b)
    face(b, 1.35, -0.21, 0.075, glow=True)
    bx("pauldron_l", (-0.28, 0, 1.06), (0.18, 0.24, 0.12), M["iron"], b)
    bx("pauldron_r", (0.28, 0, 1.06), (0.18, 0.24, 0.12), M["iron"], b)
    # dagger held forward
    bx("blade", (0.38, -0.18, 0.78), (0.05, 0.04, 0.36), M["steel"], b, (0.4, 0, 0.25))
    bx("hilt", (0.32, -0.10, 0.56), (0.05, 0.05, 0.14), M["wood"], b)
    bx("guard", (0.35, -0.13, 0.64), (0.16, 0.05, 0.04), M["gold"], b)


def build_brute(b):
    bx("foot_l", (-0.30, 0, 0.09), (0.30, 0.38, 0.18), M["leather"], b)
    bx("foot_r", (0.30, 0, 0.09), (0.30, 0.38, 0.18), M["leather"], b)
    bx("leg_l", (-0.30, 0, 0.48), (0.24, 0.26, 0.78), M["leather"], b)
    bx("leg_r", (0.30, 0, 0.48), (0.24, 0.26, 0.78), M["leather"], b)
    bx("torso", (0, 0, 1.28), (0.86, 0.52, 0.76), M["enemy"], b)
    bx("belt", (0, 0, 0.94), (0.90, 0.56, 0.12), M["leather"], b)
    for s, tag in ((-1, "l"), (1, "r")):
        ic("shoulder_" + tag, (s * 0.52, 0, 1.55), 0.24, M["iron"], b)
        cn("spike_" + tag, (s * 0.52, 0, 1.82), 0.12, 0, 0.28, M["steel"], b, 4)
        bx("arm_" + tag, (s * 0.64, 0, 1.15), (0.28, 0.30, 0.70), M["skin"], b, (0, 0, -s * 0.1))
        ic("fist_" + tag, (s * 0.68, -0.05, 0.72), 0.19, M["skin"], b)
    ic("head", (0, -0.03, 1.86), 0.25, M["skin"], b)
    bx("jaw", (0, -0.16, 1.72), (0.30, 0.16, 0.09), M["skin"], b)
    face(b, 1.90, -0.25, 0.10, glow=True)
    cn("horn_l", (-0.18, 0, 2.08), 0.08, 0, 0.30, M["bone"], b, 4, (0, -0.5, 0))
    cn("horn_r", (0.18, 0, 2.08), 0.08, 0, 0.30, M["bone"], b, 4, (0, 0.5, 0))
    # keep regular brutes clearly smaller than the bosses
    scale_collection(b, 0.84)


# ===========================================================================
# BOSSES — visibly larger and more elaborate
# ===========================================================================
def build_scorpion_queen(b):
    # six splayed legs
    for s, tag in ((-1, "l"), (1, "r")):
        for i, y in enumerate((-0.30, 0.05, 0.40)):
            bx("leg_%s%d" % (tag, i), (s * 0.52, y, 0.16), (0.55, 0.08, 0.08),
               M["stone"], b, (0, -s * 0.45, 0))
    ic("body", (0, 0.20, 0.66), 0.45, M["arcane"], b, scale=(1.35, 1.0, 0.8))
    ic("head", (0, -0.44, 0.62), 0.24, M["enemy"], b)
    bx("eye_l", (-0.12, -0.64, 0.70), (0.06, 0.03, 0.06), M["fire"], b)
    bx("eye_r", (0.12, -0.64, 0.70), (0.06, 0.03, 0.06), M["fire"], b)
    # two big pincers on articulated arms
    for s, tag in ((-1, "l"), (1, "r")):
        bx("arm_" + tag, (s * 0.70, -0.42, 0.52), (0.36, 0.12, 0.12),
           M["stone"], b, (0, -s * 0.25, 0))
        cn("pincer_%s_up" % tag, (s * 0.94, -0.72, 0.58), 0.13, 0, 0.34,
           M["steel"], b, 4, (0, -s * 0.35, 0))
        cn("pincer_%s_lo" % tag, (s * 0.94, -0.72, 0.44), 0.13, 0, 0.34,
           M["steel"], b, 4, (0, s * 0.35, PI))
    # arched tail with a glowing stinger
    cy("tail_1", (0, 0.62, 0.95), 0.18, 0.50, M["arcane"], b, 8, (-0.9, 0, 0))
    cy("tail_2", (0, 0.80, 1.40), 0.15, 0.48, M["arcane"], b, 8, (-2.3, 0, 0))
    cy("tail_3", (0, 0.45, 1.70), 0.12, 0.42, M["enemy"], b, 8, (-4.0, 0, 0))
    cn("stinger", (0, 0.05, 1.72), 0.10, 0, 0.34, M["fire"], b, 4, (PI / 2, 0, 0))
    cn("crown_l", (-0.14, -0.44, 0.86), 0.07, 0, 0.26, M["gold"], b, 4)
    cn("crown_r", (0.14, -0.44, 0.86), 0.07, 0, 0.26, M["gold"], b, 4)
    scale_collection(b, 1.15)


def build_toad_king(b):
    for s, tag in ((-1, "l"), (1, "r")):
        ic("leg_" + tag, (s * 0.46, 0.12, 0.20), 0.25, M["swamp"], b, scale=(1.2, 1.2, 0.8))
        ic("eye_" + tag, (s * 0.28, -0.28, 1.38), 0.15, M["bone"], b)
        bx("pupil_" + tag, (s * 0.28, -0.41, 1.38), (0.05, 0.02, 0.07), M["leather"], b)
    ic("body", (0, 0.10, 0.62), 0.55, M["swamp"], b, scale=(1.3, 1.0, 0.85))
    ic("belly", (0, -0.35, 0.55), 0.35, M["grass"], b, scale=(1.1, 0.7, 1.0))
    ic("head", (0, -0.10, 1.10), 0.42, M["swamp"], b)
    bx("mouth", (0, -0.50, 0.98), (0.54, 0.03, 0.07), M["leather"], b)
    # golden crown with three points
    cn("crown", (0, -0.08, 1.53), 0.38, 0.30, 0.30, M["gold"], b, 6)
    for i, dx in enumerate((-0.26, 0, 0.26)):
        cn("crown_spike_%d" % i, (dx, -0.08, 1.74), 0.07, 0, 0.24, M["gold"], b, 4)
    for i, (x, y) in enumerate(((-0.45, 0.45), (0.35, 0.50), (0.05, 0.62))):
        cn("wart_%d" % i, (x, y, 1.02), 0.08, 0, 0.16, M["leaf"], b, 4)
    scale_collection(b, 1.20)


def build_stormcaller(b):
    # long robe tapering to the ground
    cn("robe", (0, 0, 0.68), 0.50, 0, 1.36, M["arcane"], b, 8)
    cy("hem", (0, 0, 0.06), 0.42, 0.10, M["gold"], b, 8)
    bx("torso", (0, 0, 1.48), (0.44, 0.30, 0.50), M["arcane"], b)
    cn("hood", (0, 0, 1.80), 0.30, 0, 0.44, M["arcane"], b, 6)
    bx("shade", (0, -0.22, 1.72), (0.22, 0.02, 0.18), M["leather"], b)
    face(b, 1.74, -0.235, 0.07, glow=True)
    bx("arm_l", (-0.30, 0, 1.40), (0.12, 0.14, 0.54), M["cloth"], b, (0, 0, 0.55))
    bx("arm_r", (0.30, 0, 1.40), (0.12, 0.14, 0.54), M["cloth"], b, (0, 0, -0.55))
    # tall staff with a lightning orb
    cy("staff", (0.48, 0, 1.16), 0.045, 2.10, M["wood"], b, 6)
    ic("orb", (0.48, 0, 2.24), 0.16, M["spark"], b)
    for i, rot in enumerate(((0, 0, 0.4), (0, 0, -0.4), (0, 0, 1.2), (0, 0, -1.2))):
        bx("bolt_%d" % i, (0.48, 0, 2.50), (0.025, 0.025, 0.30), M["spark"], b, rot)
    for i, (x, y, z) in enumerate(((-0.52, 0.10, 1.30), (0.60, 0.16, 1.86), (-0.42, -0.20, 1.95))):
        bx("rune_%d" % i, (x, y, z), (0.10, 0.02, 0.10), M["spark"], b, (0, 0, 0.6))


def build_ocean_leviathan(b):
    # rising serpent body
    for i, (y, z, r) in enumerate(((0.25, 0.50, 0.46), (0.48, 1.00, 0.38),
                                   (0.34, 1.46, 0.30), (-0.02, 1.80, 0.22))):
        ic("coil_%d" % i, (0, y, z), r, M["water"], b, scale=(1.0, 1.0, 1.0))
    ic("head", (0, -0.30, 1.96), 0.28, M["water"], b, scale=(1.0, 1.15, 0.9))
    bx("jaw", (0, -0.55, 1.80), (0.34, 0.22, 0.10), M["leather"], b)
    bx("eye_l", (-0.15, -0.53, 2.02), (0.055, 0.03, 0.055), M["fire"], b)
    bx("eye_r", (0.15, -0.53, 2.02), (0.055, 0.03, 0.055), M["fire"], b)
    for i, (y, z) in enumerate(((0.20, 0.95), (0.48, 1.32), (0.30, 1.72), (-0.05, 2.02))):
        cn("fin_%d" % i, (0, y, z), 0.16, 0, 0.30, M["snow"], b, 4, (-0.7, 0, 0))
    cn("tentacle_l", (-0.42, 0.15, 0.18), 0.13, 0, 0.55, M["water"], b, 6, (0, PI / 2, 0))
    cn("tentacle_r", (0.42, 0.15, 0.18), 0.13, 0, 0.55, M["water"], b, 6, (0, -PI / 2, 0))
    cn("crest_l", (-0.12, -0.25, 2.22), 0.07, 0, 0.24, M["gold"], b, 4)
    cn("crest_r", (0.12, -0.25, 2.22), 0.07, 0, 0.24, M["gold"], b, 4)
    ic("foam", (0, 0.10, 0.16), 0.16, M["snow"], b)


def build_colossus(b):
    bx("foot_l", (-0.35, 0, 0.13), (0.48, 0.60, 0.26), M["stone"], b)
    bx("foot_r", (0.35, 0, 0.13), (0.48, 0.60, 0.26), M["stone"], b)
    bx("leg_l", (-0.35, 0, 0.68), (0.42, 0.45, 1.08), M["stone"], b)
    bx("leg_r", (0.35, 0, 0.68), (0.42, 0.45, 1.08), M["stone"], b)
    bx("pelvis", (0, 0, 1.28), (0.92, 0.55, 0.36), M["stone"], b)
    bx("torso", (0, 0, 1.86), (1.18, 0.70, 1.00), M["stone"], b)
    for s, tag in ((-1, "l"), (1, "r")):
        ic("shoulder_" + tag, (s * 0.74, 0, 2.25), 0.36, M["stone"], b)
        bx("arm_" + tag, (s * 0.86, 0, 1.58), (0.40, 0.42, 1.02), M["stone"], b, (0, 0, -s * 0.08))
        ic("fist_" + tag, (s * 0.88, -0.05, 0.96), 0.30, M["stone"], b)
    bx("head", (0, 0, 2.62), (0.50, 0.50, 0.46), M["stone"], b)
    bx("eye_l", (-0.14, -0.26, 2.66), (0.08, 0.03, 0.08), M["fire"], b)
    bx("eye_r", (0.14, -0.26, 2.66), (0.08, 0.03, 0.08), M["fire"], b)
    for i, x in enumerate((-0.16, 0.0, 0.16)):
        cn("crown_%d" % i, (x, 0, 2.90), 0.08, 0, 0.30, M["gold"], b, 4)
    # glowing cracks in the stone chest
    bx("crack_l", (-0.22, -0.36, 1.94), (0.05, 0.02, 0.42), M["fire"], b, (0, 0, 0.35))
    bx("crack_r", (0.20, -0.36, 1.86), (0.05, 0.02, 0.34), M["fire"], b, (0, 0, -0.4))


# ===========================================================================
# EXTRAS (if time permits — all included)
# ===========================================================================
def build_guard(b):
    bx("boot_l", (-0.13, 0, 0.08), (0.18, 0.26, 0.16), M["leather"], b)
    bx("boot_r", (0.13, 0, 0.08), (0.18, 0.26, 0.16), M["leather"], b)
    bx("leg_l", (-0.13, 0, 0.32), (0.16, 0.18, 0.32), M["cloth"], b)
    bx("leg_r", (0.13, 0, 0.32), (0.16, 0.18, 0.32), M["cloth"], b)
    bx("tunic", (0, 0, 0.78), (0.44, 0.30, 0.54), M["cloth"], b)
    bx("belt", (0, 0, 0.55), (0.46, 0.32, 0.09), M["leather"], b)
    ic("head", (0, 0, 1.30), 0.21, M["skin"], b)
    cn("helm", (0, 0, 1.38), 0.25, 0, 0.34, M["iron"], b, 6)
    face(b, 1.31, -0.20, 0.08)
    bx("arm_l", (-0.30, 0, 0.84), (0.13, 0.14, 0.40), M["iron"], b, (0, 0, 0.18))
    bx("arm_r", (0.30, 0, 0.84), (0.13, 0.14, 0.40), M["iron"], b, (0, 0, -0.18))
    bx("shield", (-0.42, -0.12, 0.80), (0.10, 0.36, 0.52), M["steel"], b)
    bx("shield_boss", (-0.48, -0.12, 0.80), (0.03, 0.14, 0.14), M["gold"], b)


def build_golem(b):
    bx("foot_l", (-0.20, 0, 0.10), (0.26, 0.34, 0.20), M["stone"], b)
    bx("foot_r", (0.20, 0, 0.10), (0.26, 0.34, 0.20), M["stone"], b)
    bx("leg_l", (-0.20, 0, 0.42), (0.24, 0.24, 0.60), M["stone"], b)
    bx("leg_r", (0.20, 0, 0.42), (0.24, 0.24, 0.60), M["stone"], b)
    ic("body", (0, 0, 1.05), 0.42, M["stone"], b, scale=(1.2, 0.9, 1.1))
    for s, tag in ((-1, "l"), (1, "r")):
        ic("shoulder_" + tag, (s * 0.46, 0, 1.24), 0.20, M["iron"], b)
        bx("arm_" + tag, (s * 0.52, 0, 0.90), (0.22, 0.24, 0.58), M["stone"], b)
        ic("fist_" + tag, (s * 0.54, -0.04, 0.58), 0.17, M["stone"], b)
    bx("head", (0, 0, 1.50), (0.34, 0.32, 0.32), M["stone"], b)
    bx("eye_l", (-0.08, -0.17, 1.53), (0.06, 0.03, 0.06), M["fire"], b)
    bx("eye_r", (0.08, -0.17, 1.53), (0.06, 0.03, 0.06), M["fire"], b)
    cn("crystal", (0, 0, 1.78), 0.11, 0, 0.28, M["spark"], b, 4)


def build_ogre(b):
    bx("foot_l", (-0.24, 0, 0.09), (0.28, 0.36, 0.18), M["leather"], b)
    bx("foot_r", (0.24, 0, 0.09), (0.28, 0.36, 0.18), M["leather"], b)
    bx("leg_l", (-0.24, 0, 0.42), (0.22, 0.24, 0.70), M["grass"], b)
    bx("leg_r", (0.24, 0, 0.42), (0.22, 0.24, 0.70), M["grass"], b)
    ic("body", (0, 0, 1.15), 0.46, M["grass"], b, scale=(1.2, 0.9, 1.0))
    bx("loincloth", (0, 0, 0.76), (0.62, 0.52, 0.22), M["leather"], b)
    for s, tag in ((-1, "l"), (1, "r")):
        bx("arm_" + tag, (s * 0.58, 0, 1.08), (0.24, 0.26, 0.72), M["grass"], b, (0, 0, -s * 0.2))
        ic("fist_" + tag, (s * 0.66, -0.05, 0.66), 0.17, M["grass"], b)
    ic("head", (0, -0.02, 1.70), 0.28, M["grass"], b)
    bx("jaw", (0, -0.18, 1.56), (0.34, 0.18, 0.10), M["grass"], b)
    face(b, 1.74, -0.27, 0.11, glow=True)
    cn("tusk_l", (-0.15, -0.28, 1.54), 0.06, 0, 0.20, M["bone"], b, 4, (PI, 0, 0))
    cn("tusk_r", (0.15, -0.28, 1.54), 0.06, 0, 0.20, M["bone"], b, 4, (PI, 0, 0))


def build_wraith(b):
    # inverted cone tapers to a point at z=0: floating ghost silhouette
    cn("shroud", (0, 0, 0.75), 0, 0.42, 1.50, M["arcane"], b, 8)
    cy("hem", (0, 0, 0.95), 0.34, 0.08, M["cloth"], b, 8)
    ic("head", (0, -0.05, 1.46), 0.22, M["bone"], b)
    cn("hood", (0, 0.02, 1.58), 0.28, 0, 0.40, M["arcane"], b, 6)
    face(b, 1.46, -0.23, 0.08, glow=True)
    bx("arm_l", (-0.34, 0, 1.20), (0.10, 0.12, 0.46), M["cloth"], b, (0, 0, 0.55))
    bx("arm_r", (0.34, 0, 1.20), (0.10, 0.12, 0.46), M["cloth"], b, (0, 0, -0.55))
    cn("wisp_l", (-0.46, 0, 0.95), 0.09, 0, 0.24, M["arcane"], b, 4, (PI, 0, 0))
    cn("wisp_r", (0.46, 0, 0.95), 0.09, 0, 0.24, M["arcane"], b, 4, (PI, 0, 0))


def build_imp(b):
    bx("foot_l", (-0.09, 0, 0.05), (0.12, 0.16, 0.10), M["leather"], b)
    bx("foot_r", (0.09, 0, 0.05), (0.12, 0.16, 0.10), M["leather"], b)
    ic("body", (0, 0, 0.42), 0.24, M["enemy"], b, scale=(1.0, 0.9, 1.1))
    ic("head", (0, -0.03, 0.76), 0.21, M["enemy"], b)
    cn("horn_l", (-0.12, 0, 0.94), 0.07, 0, 0.26, M["bone"], b, 4, (0, -0.45, 0))
    cn("horn_r", (0.12, 0, 0.94), 0.07, 0, 0.26, M["bone"], b, 4, (0, 0.45, 0))
    face(b, 0.78, -0.21, 0.08, glow=True)
    cn("wing_l", (-0.30, 0.06, 0.58), 0.22, 0, 0.03, M["leather"], b, 3, (PI / 2, PI, 0))
    cn("wing_r", (0.30, 0.06, 0.58), 0.22, 0, 0.03, M["leather"], b, 3, (PI / 2, 0, 0))
    bx("arm_l", (-0.26, -0.02, 0.48), (0.09, 0.10, 0.28), M["enemy"], b, (0, 0, 0.3))
    bx("arm_r", (0.26, -0.02, 0.48), (0.09, 0.10, 0.28), M["enemy"], b, (0, 0, -0.3))
    cn("tail", (0, 0.26, 0.38), 0.07, 0, 0.44, M["enemy"], b, 4, (-1.3, 0, 0))
    cn("tail_tip", (0, 0.56, 0.44), 0.07, 0, 0.18, M["fire"], b, 4, (PI / 2, 0, 0))


# ---------------------------------------------------------------------------
# Build, export, then render the contact sheet (last — it moves the objects)
# ---------------------------------------------------------------------------
ASSETS = [
    ("player",            build_player),
    ("goblin",            build_goblin),
    ("skeleton",          build_skeleton),
    ("slime",             build_slime),
    ("wolf",              build_wolf),
    ("bat",               build_bat),
    ("spider",            build_spider),
    ("archer",            build_archer),
    ("raider",            build_raider),
    ("brute",             build_brute),
    ("scorpion_queen",    build_scorpion_queen),
    ("toad_king",         build_toad_king),
    ("stormcaller",       build_stormcaller),
    ("ocean_leviathan",   build_ocean_leviathan),
    ("colossus",          build_colossus),
    ("guard",             build_guard),
    ("golem",             build_golem),
    ("ogre",              build_ogre),
    ("wraith",            build_wraith),
    ("imp",               build_imp),
]

built = []
for name, builder in ASSETS:
    col = collection(name)
    builder(col)
    export_asset(name, col)
    built.append((name, col))

contact_sheet(built, "characters.png")
print("DONE: %d character assets" % len(built))
