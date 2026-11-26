"""Starfall — low-poly buildings & structures generator.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_structures.py

All assets are flat-shaded, palette-only, based at z=0 and centred on (0,0).
One collection per asset; each collection is exported as its own GLB.
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import *

new_scene()

# ---------------------------------------------------------------------------
# Shared palette materials (slight lightness variations of the contract hues)
# ---------------------------------------------------------------------------
M = {
    "wood":      mat("wood", C["wood"]),
    "wood_light": mat("wood_light", "#8A5C31"),
    "leather":   mat("leather", C["leather"]),
    "shingle":   mat("shingle", "#845A30"),
    "stone":     mat("stone", C["stone"]),
    "stone_dark": mat("stone_dark", "#5C626A"),
    "stone_light": mat("stone_light", "#99A0A8"),
    "iron":      mat("iron", C["iron"], metallic=0.35),
    "steel":     mat("steel", C["steel"], metallic=0.6, rough=0.5),
    "gold":      mat("gold", C["gold"], metallic=0.55, rough=0.45),
    "thatch":    mat("thatch", "#6E9A46"),
    "leaf":      mat("leaf", C["leaf"]),
    "cloth":     mat("cloth", C["cloth"]),
    "water":     mat("water", C["water"], rough=0.35),
    "bone":      mat("bone", C["bone"]),
    "enemy":     mat("enemy", C["enemy"]),
    "fire":      mat("fire", C["fire"], emission=C["fire"], strength=3.0),
    "ember":     mat("ember", C["gold"], emission=C["fire"], strength=2.0),
    "arcane":    mat("arcane", C["arcane"], emission=C["arcane"], strength=2.5),
    "arcane_dim": mat("arcane_dim", C["arcane"], emission=C["arcane"], strength=0.6),
    "void":      mat("void", "#382A4E", emission=C["arcane"], strength=0.4),
}


# ---------------------------------------------------------------------------
# Small local helpers
# ---------------------------------------------------------------------------
def slant(name, p1, p2, y, depth, thick, material, col):
    """A roof panel spanning two (x, z) points, lying in the XZ plane."""
    dx, dz = p2[0] - p1[0], p2[1] - p1[1]
    length = math.hypot(dx, dz)
    o = box(name, ((p1[0] + p2[0]) / 2, y, (p1[1] + p2[1]) / 2),
            (length, depth, thick), material, col)
    o.rotation_euler = (0, -math.atan2(dz, dx), 0)
    return o


def gable(name, col, half_w, eave_z, rise, y, depth, material, overhang=0.0):
    """Two identical roof panels meeting at a central ridge."""
    w = half_w + overhang
    slant(name + "_r", (0, eave_z + rise), (w, eave_z), y, depth, 0.11, material, col)
    slant(name + "_l", (-w, eave_z), (0, eave_z + rise), y, depth, 0.11, material, col)


def window(col, x, y, z, axis="y", w=0.3, h=0.3):
    """Framed glowing window; axis is the face normal's axis."""
    if axis == "y":
        box("glass", (x, y, z), (w, 0.05, h), M["water"], col)
        for dz in (-h / 2 - 0.03, h / 2 + 0.03):
            box("frame", (x, y - 0.03, z + dz), (w + 0.1, 0.05, 0.06), M["wood_light"], col)
    else:
        box("glass", (x, y, z), (0.05, w, h), M["water"], col)
        for dz in (-h / 2 - 0.03, h / 2 + 0.03):
            box("frame", (x - 0.03, y, z + dz), (0.05, w + 0.1, 0.06), M["wood_light"], col)


def tris(col):
    n = 0
    for o in col.objects:
        if o.type == "MESH":
            o.data.calc_loop_triangles()
            n += len(o.data.loop_triangles)
    return n


# ---------------------------------------------------------------------------
# Buildings
# ---------------------------------------------------------------------------
def build_house():
    col = collection("house")
    box("foundation", (0, 0, 0.1), (1.7, 1.5, 0.2), M["stone"], col)
    box("walls", (0, 0, 0.95), (1.6, 1.4, 1.5), M["wood"], col)
    for x in (-0.78, 0.78):
        for y in (-0.68, 0.68):
            box("corner_post", (x, y, 0.95), (0.12, 0.12, 1.55), M["leather"], col)
    gable("roof", col, 0.8, 1.7, 0.9, 0, 1.75, M["shingle"], 0.18)
    box("ridge", (0, 0, 2.61), (0.14, 1.82, 0.12), M["leather"], col)
    box("chimney", (0.52, 0.34, 2.45), (0.26, 0.26, 1.0), M["stone"], col)
    box("chimney_cap", (0.52, 0.34, 2.98), (0.34, 0.34, 0.1), M["iron"], col)
    box("door", (0, -0.73, 0.62), (0.44, 0.07, 0.84), M["leather"], col)
    ico("knob", (0.14, -0.79, 0.62), 0.045, M["gold"], col, 0)
    for x in (-0.5, 0.5):
        window(col, x, -0.735, 1.25)
    return col


def build_cabin():
    col = collection("cabin")
    box("base", (0, 0, 0.08), (1.6, 1.4, 0.16), M["stone"], col)
    for z in (0.28, 0.62, 0.96, 1.30):
        for y in (-0.62, 0.62):
            o = cyl("log", (0, y, z), 0.18, 1.55, M["wood"], col, 4)
            o.rotation_euler = (0, math.pi / 2, 0)
        for x in (-0.72, 0.72):
            o = cyl("log", (x, 0, z), 0.18, 1.34, M["wood"], col, 4)
            o.rotation_euler = (math.pi / 2, 0, 0)
    cone("hip_roof", (0, 0, 1.86), 1.16, 0, 0.94, M["shingle"], col, 4,
         (0, math.pi / 4, 0))
    box("chimney", (0.42, 0.38, 2.02), (0.2, 0.2, 0.62), M["stone"], col)
    box("door", (0, -0.74, 0.55), (0.42, 0.06, 0.78), M["leather"], col)
    ico("latch", (0.13, -0.79, 0.55), 0.04, M["gold"], col, 0)
    window(col, -0.43, -0.745, 1.05, w=0.26, h=0.26)
    # small porch
    for x in (-0.54, 0.54):
        box("porch_post", (x, -0.95, 0.66), (0.1, 0.1, 1.15), M["leather"], col)
    slant("porch_roof", (0, 1.38), (0.82, 1.12), -0.95, 0.52, 0.09, M["shingle"], col)
    return col


def build_hut():
    col = collection("hut")
    cyl("stone_base", (0, 0, 0.09), 0.74, 0.18, M["stone"], col, 8)
    cyl("round_wall", (0, 0, 0.76), 0.56, 1.24, M["wood"], col, 8)
    for z in (0.42, 0.76, 1.1):
        cyl("wall_band", (0, 0, z), 0.58, 0.06, M["leather"], col, 8)
    cone("thatch_roof", (0, 0, 1.88), 0.99, 0, 1.2, M["thatch"], col, 8)
    ico("roof_knob", (0, 0, 2.52), 0.11, M["bone"], col, 0)
    box("door", (0, -0.57, 0.52), (0.36, 0.06, 0.72), M["leather"], col)
    window(col, -0.32, -0.5, 0.95, w=0.24, h=0.24)
    return col


def build_inn():
    col = collection("inn")
    box("foundation", (0, 0, 0.1), (2.0, 1.6, 0.2), M["stone"], col)
    box("walls", (0, 0, 1.15), (1.9, 1.5, 2.0), M["wood_light"], col)
    for x in (-0.93, 0.93):
        for y in (-0.73, 0.73):
            box("beam", (x, y, 1.15), (0.13, 0.13, 2.1), M["leather"], col)
    box("floor_band", (0, 0, 1.18), (1.98, 1.58, 0.09), M["leather"], col)
    gable("roof", col, 0.95, 2.15, 0.82, 0, 1.85, M["shingle"], 0.18)
    box("ridge", (0, 0, 2.98), (0.14, 1.9, 0.12), M["leather"], col)
    box("chimney", (-0.62, 0.36, 2.72), (0.24, 0.24, 0.85), M["stone"], col)
    box("chimney_cap", (-0.62, 0.36, 3.17), (0.32, 0.32, 0.09), M["iron"], col)
    # front porch and balcony
    for x in (-0.78, 0.78):
        box("porch_post", (x, -0.98, 0.85), (0.11, 0.11, 1.6), M["leather"], col)
    slant("porch_awning", (0, 1.75), (1.05, 1.48), -0.95, 0.56, 0.09, M["shingle"], col)
    box("balcony_floor", (0, -0.88, 1.25), (1.9, 0.45, 0.1), M["wood_light"], col)
    for x in (-0.78, -0.26, 0.26, 0.78):
        box("baluster", (x, -1.09, 1.48), (0.07, 0.07, 0.4), M["leather"], col)
    box("balcony_rail", (0, -1.09, 1.68), (1.9, 0.09, 0.08), M["leather"], col)
    box("door", (0, -0.77, 0.62), (0.46, 0.07, 0.86), M["leather"], col)
    ico("knob", (0.15, -0.83, 0.62), 0.045, M["gold"], col, 0)
    for x in (-0.62, 0.62):
        window(col, x, -0.77, 0.85, w=0.32, h=0.3)
        window(col, x, -0.77, 1.75, w=0.32, h=0.3)
    # hanging inn sign
    box("sign_board", (0.72, -1.14, 1.95), (0.5, 0.08, 0.34), M["wood"], col)
    box("sign_emblem", (0.72, -1.2, 1.95), (0.16, 0.04, 0.16), M["gold"], col)
    return col


def build_barn():
    col = collection("barn")
    box("foundation", (0, 0, 0.08), (2.25, 1.75, 0.16), M["stone"], col)
    box("walls", (0, 0, 0.88), (2.15, 1.65, 1.5), M["leather"], col)
    for x in (-1.05, 1.05):
        box("corner", (x, 0, 0.88), (0.12, 1.68, 1.55), M["wood"], col)
    # gambrel roof: steep lower slopes, shallow upper slopes
    slant("roof_low_r", (1.22, 1.63), (0.78, 2.38), 0, 1.9, 0.1, M["shingle"], col)
    slant("roof_low_l", (-0.78, 2.38), (-1.22, 1.63), 0, 1.9, 0.1, M["shingle"], col)
    slant("roof_high_r", (0, 2.7), (0.78, 2.38), 0, 1.9, 0.1, M["shingle"], col)
    slant("roof_high_l", (-0.78, 2.38), (0, 2.7), 0, 1.9, 0.1, M["shingle"], col)
    box("ridge", (0, 0, 2.72), (0.14, 1.94, 0.1), M["wood"], col)
    # big double doors with X bracing
    for x in (-0.45, 0.45):
        box("barn_door", (x, -0.84, 0.72), (0.84, 0.07, 1.3), M["wood"], col)
        for dx in (-0.3, 0.3):
            o = box("brace", (x + dx, -0.89, 0.72), (0.08, 0.05, 1.2), M["leather"], col)
            o.rotation_euler = (0, 0, 0.55 * (1 if dx < 0 else -1))
    box("hayloft", (0, -0.85, 1.37), (0.5, 0.06, 0.38), M["leather"], col)
    box("hayloft_frame", (0, -0.89, 1.58), (0.64, 0.05, 0.07), M["gold"], col)
    return col


def build_watchtower():
    col = collection("watchtower")
    for x in (-0.48, 0.48):
        for y in (-0.48, 0.48):
            box("leg", (x, y, 1.0), (0.14, 0.14, 2.0), M["wood"], col)
    for z in (0.7, 1.45):
        for y in (-0.48, 0.48):
            box("cross_brace", (0, y, z), (1.05, 0.08, 0.09), M["leather"], col)
        for x in (-0.48, 0.48):
            box("cross_brace", (x, 0, z), (0.08, 1.05, 0.09), M["leather"], col)
    box("platform", (0, 0, 2.04), (1.45, 1.45, 0.14), M["wood_light"], col)
    for x in (-0.62, 0.62):
        box("rail_x", (x, 0, 2.45), (0.08, 1.35, 0.08), M["leather"], col)
    for y in (-0.62, 0.62):
        box("rail_y", (0, y, 2.45), (1.35, 0.08, 0.08), M["leather"], col)
    cone("roof", (0, 0, 2.95), 1.12, 0, 0.75, M["shingle"], col, 4,
         (0, math.pi / 4, 0))
    ico("finial", (0, 0, 3.38), 0.08, M["gold"], col, 0)
    # ladder
    for x in (-0.18, 0.18):
        box("ladder_rail", (x, -0.74, 1.0), (0.06, 0.06, 2.0), M["leather"], col)
    for z in (0.3, 0.75, 1.2, 1.65):
        box("rung", (0, -0.74, z), (0.42, 0.06, 0.06), M["wood_light"], col)
    return col


# ---------------------------------------------------------------------------
# Props, furniture & magic
# ---------------------------------------------------------------------------
def build_chest():
    col = collection("chest")
    box("body", (0, 0, 0.21), (0.8, 0.56, 0.42), M["wood"], col)
    box("lid", (0, 0, 0.53), (0.85, 0.6, 0.22), M["leather"], col)
    for x in (-0.27, 0.27):
        box("band", (x, 0, 0.32), (0.07, 0.62, 0.66), M["iron"], col)
    box("lock", (0, -0.31, 0.44), (0.16, 0.06, 0.18), M["gold"], col)
    ico("keyhole", (0, -0.35, 0.44), 0.035, M["leather"], col, 0)
    return col


def build_campfire():
    col = collection("campfire")
    for i in range(7):
        a = 2 * math.pi * i / 7
        ico("stone", (0.44 * math.cos(a), 0.44 * math.sin(a), 0.18),
            0.17, M["stone"], col, 0)
    for i in range(4):
        a = math.pi * i / 4
        o = cyl("log", (0, 0, 0.36), 0.08, 0.72, M["wood"], col, 5)
        o.rotation_euler = (math.cos(a) * 0.7, math.sin(a) * 0.7, a)
    cone("flame_outer", (0, 0, 0.53), 0.24, 0, 0.62, M["fire"], col, 6)
    cone("flame_inner", (0, 0, 0.44), 0.12, 0, 0.38, M["ember"], col, 5)
    ico("ember", (0.1, -0.08, 0.28), 0.07, M["ember"], col, 0)
    return col


def build_altar():
    col = collection("altar")
    box("platform", (0, 0, 0.1), (1.65, 1.65, 0.2), M["stone"], col)
    box("step", (0, 0, 0.28), (1.3, 1.3, 0.16), M["stone_light"], col)
    cyl("plinth", (0, 0, 0.46), 0.52, 0.2, M["stone_dark"], col, 8)
    for x in (-0.65, 0.65):
        for y in (-0.65, 0.65):
            cyl("pillar", (x, y, 0.88), 0.11, 0.95, M["stone"], col, 6)
            ico("pillar_cap", (x, y, 1.4), 0.13, M["stone_light"], col, 0)
    # hovering arcane crown: octagonal band + alternating spikes
    for i in range(8):
        a = 2 * math.pi * i / 8
        x, y = 0.58 * math.cos(a), 0.58 * math.sin(a)
        o = box("crown_band", (x, y, 1.55), (0.46, 0.1, 0.09), M["arcane"], col)
        o.rotation_euler = (0, 0, a + math.pi / 2)
        cone("crown_spike", (x, y, 1.76), 0.1, 0,
             0.36 if i % 2 == 0 else 0.24, M["arcane"], col, 4)
    ico("heart", (0, 0, 1.6), 0.18, M["arcane"], col, 0)
    box("rune", (0, 0, 0.58), (0.38, 0.38, 0.03), M["arcane_dim"], col)
    return col


def build_anvil():
    col = collection("anvil")
    cyl("stump", (0, 0, 0.23), 0.33, 0.46, M["wood"], col, 8)
    box("foot", (0, 0, 0.51), (0.52, 0.36, 0.1), M["iron"], col)
    box("waist", (0, 0, 0.65), (0.28, 0.24, 0.24), M["iron"], col)
    box("face", (0, 0, 0.82), (0.62, 0.34, 0.16), M["steel"], col)
    o = cone("horn", (0.48, 0, 0.82), 0.15, 0, 0.4, M["steel"], col, 6)
    o.rotation_euler = (0, -math.pi / 2, 0)
    return col


def build_well():
    col = collection("well")
    cyl("stone_ring", (0, 0, 0.34), 0.62, 0.68, M["stone"], col, 8)
    cyl("water", (0, 0, 0.66), 0.48, 0.05, M["water"], col, 8)
    for x in (-0.52, 0.52):
        box("post", (x, 0, 1.3), (0.11, 0.11, 1.35), M["wood"], col)
    box("beam", (0, 0, 1.93), (1.22, 0.12, 0.12), M["leather"], col)
    cone("roof", (0, 0, 2.24), 0.86, 0, 0.56, M["shingle"], col, 4,
         (0, math.pi / 4, 0))
    cyl("rope", (0, 0, 1.55), 0.02, 0.65, M["bone"], col, 5)
    cyl("bucket", (0, 0, 1.1), 0.16, 0.26, M["wood"], col, 6)
    cyl("bucket_hoop", (0, 0, 1.18), 0.17, 0.05, M["iron"], col, 6)
    return col


def build_bed():
    col = collection("bed")
    box("frame", (0, 0, 0.09), (0.98, 1.9, 0.14), M["wood"], col)
    box("mattress", (0, 0.05, 0.25), (0.88, 1.7, 0.18), M["bone"], col)
    box("blanket", (0, 0.5, 0.37), (0.9, 0.85, 0.09), M["cloth"], col)
    box("pillow", (0, -0.6, 0.39), (0.58, 0.32, 0.13), M["bone"], col)
    box("headboard", (0, -0.93, 0.38), (0.98, 0.1, 0.62), M["leather"], col)
    box("footboard", (0, 0.93, 0.28), (0.98, 0.1, 0.42), M["leather"], col)
    for x in (-0.42, 0.42):
        for y in (-0.82, 0.82):
            box("leg", (x, y, 0.04), (0.1, 0.1, 0.08), M["leather"], col)
    return col


def build_fence():
    col = collection("fence")
    for x in (-0.96, 0.96):
        box("post", (x, 0, 0.45), (0.14, 0.14, 0.9), M["wood"], col)
        cone("post_cap", (x, 0, 0.94), 0.11, 0, 0.16, M["leather"], col, 4)
    for z in (0.32, 0.67):
        box("rail", (0, 0, z), (1.96, 0.09, 0.1), M["wood_light"], col)
    return col


def build_wall():
    col = collection("wall")
    box("body", (0, 0, 0.6), (2.0, 0.4, 1.2), M["stone"], col)
    box("base", (0, 0, 0.09), (2.1, 0.48, 0.18), M["stone_dark"], col)
    box("cap", (0, 0, 1.27), (2.1, 0.5, 0.14), M["stone_light"], col)
    for x in (-0.76, -0.26, 0.26, 0.76):
        box("crenel", (x, 0, 1.46), (0.32, 0.48, 0.24), M["stone"], col)
    return col


def build_turret():
    col = collection("turret")
    cyl("shaft", (0, 0, 0.95), 0.5, 1.9, M["stone"], col, 8)
    cyl("band", (0, 0, 1.83), 0.55, 0.12, M["stone_dark"], col, 8)
    cyl("deck", (0, 0, 2.0), 0.66, 0.16, M["stone_light"], col, 8)
    for i in range(6):
        a = 2 * math.pi * i / 6
        o = box("merlon", (0.53 * math.cos(a), 0.53 * math.sin(a), 2.23),
                (0.28, 0.22, 0.34), M["stone"], col)
        o.rotation_euler = (0, 0, a)
    box("door", (0, -0.51, 0.42), (0.34, 0.06, 0.7), M["leather"], col)
    box("slit", (0, -0.52, 1.35), (0.1, 0.04, 0.42), M["void"], col)
    return col


def build_enchanting_table():
    col = collection("enchanting_table")
    for x in (-0.4, 0.4):
        for y in (-0.3, 0.3):
            box("leg", (x, y, 0.3), (0.1, 0.1, 0.6), M["wood"], col)
    box("top", (0, 0, 0.66), (1.05, 0.85, 0.12), M["wood_light"], col)
    box("rune_ring", (0, 0, 0.73), (0.8, 0.6, 0.02), M["arcane_dim"], col)
    box("book", (0, 0, 0.78), (0.36, 0.28, 0.08), M["cloth"], col)
    box("book_spine", (-0.17, 0, 0.79), (0.05, 0.3, 0.1), M["gold"], col)
    for i in range(4):
        a = 2 * math.pi * i / 4 + math.pi / 4
        ico("crystal", (0.44 * math.cos(a), 0.4 * math.sin(a), 0.93),
            0.12, M["arcane"], col, 0, (0.7, 0.7, 1.6))
    cone("focus", (0, 0, 1.12), 0.15, 0, 0.44, M["arcane"], col, 5)
    return col


def build_portal():
    col = collection("portal")
    box("step", (0, 0, 0.07), (2.0, 0.85, 0.14), M["stone_dark"], col)
    box("threshold", (0, 0, 0.15), (1.7, 0.65, 0.1), M["stone_light"], col)
    for x in (-0.74, 0.74):
        box("pillar", (x, 0, 1.0), (0.32, 0.42, 1.9), M["stone"], col)
        box("pillar_foot", (x, 0, 0.22), (0.42, 0.5, 0.16), M["stone_dark"], col)
    for i in range(7):
        a = math.pi - math.pi * i / 6
        x, z = 0.74 * math.cos(a), 1.95 + 0.74 * math.sin(a)
        o = box("arch_stone", (x, 0, z), (0.3, 0.44, 0.26), M["stone"], col)
        o.rotation_euler = (0, -(a + math.pi / 2), 0)
    box("core", (0, 0, 1.0), (1.18, 0.05, 1.75), M["arcane"], col)
    ico("core_heart", (0, 0, 1.08), 0.2, M["arcane"], col, 0, (0.7, 0.4, 1.3))
    for x in (-0.38, 0.38):
        box("rune", (x, -0.12, 0.2), (0.12, 0.03, 0.12), M["arcane_dim"], col)
    return col


def build_dungeon():
    col = collection("dungeon")
    box("base", (0, 0, 0.07), (2.0, 0.9, 0.14), M["stone_dark"], col)
    box("left_pillar", (-0.72, 0, 0.85), (0.34, 0.44, 1.7), M["stone"], col)
    box("right_pillar", (0.72, 0, 0.62), (0.34, 0.44, 1.24), M["stone"], col)
    for i, (x, z, a) in enumerate(((-0.48, 1.93, 0.7), (0.0, 2.09, 0.0),
                                   (0.45, 1.93, -0.7))):
        o = box("arch_stone", (x, 0, z), (0.44, 0.44, 0.3), M["stone"], col)
        o.rotation_euler = (0, a, 0)
    box("void_glow", (0, 0, 0.92), (1.08, 0.05, 1.72), M["void"], col)
    ico("rune", (0, -0.14, 1.35), 0.1, M["arcane_dim"], col, 0)
    for x, y, s in ((-1.02, 0.42, 0.2), (1.0, -0.4, 0.16), (0.9, 0.45, 0.13)):
        ico("rubble", (x, y, s), s, M["stone"], col, 0)
    return col


def build_banner():
    col = collection("banner")
    cyl("pole", (0, 0, 1.1), 0.055, 2.2, M["wood"], col, 6)
    box("crossbar", (0, 0, 2.08), (0.95, 0.07, 0.07), M["leather"], col)
    box("cloth", (0, -0.04, 1.5), (0.72, 0.05, 1.05), M["cloth"], col)
    box("cloth_tip", (0, -0.04, 0.91), (0.36, 0.05, 0.24), M["cloth"], col)
    box("emblem_h", (0, -0.08, 1.62), (0.32, 0.03, 0.08), M["gold"], col)
    box("emblem_v", (0, -0.08, 1.62), (0.08, 0.03, 0.32), M["gold"], col)
    ico("finial", (0, 0, 2.25), 0.1, M["gold"], col, 0)
    box("foot", (0, 0, 0.06), (0.38, 0.38, 0.12), M["stone"], col)
    return col


def build_lantern():
    col = collection("lantern")
    cyl("base", (0, 0, 0.05), 0.2, 0.1, M["iron"], col, 8)
    cyl("glass", (0, 0, 0.32), 0.16, 0.42, M["fire"], col, 8)
    for i in range(4):
        a = math.pi * i / 2 + math.pi / 4
        box("frame", (0.16 * math.cos(a), 0.16 * math.sin(a), 0.32),
            (0.05, 0.05, 0.48), M["iron"], col)
    cone("hood", (0, 0, 0.6), 0.24, 0, 0.18, M["iron"], col, 8)
    for x in (-0.13, 0.13):
        box("handle", (x, 0, 0.77), (0.04, 0.04, 0.24), M["iron"], col)
    box("handle_top", (0, 0, 0.9), (0.3, 0.04, 0.04), M["iron"], col)
    return col


def build_torch():
    col = collection("torch")
    cyl("handle", (0, 0, 0.35), 0.055, 0.7, M["wood"], col, 6)
    cyl("wrap", (0, 0, 0.68), 0.1, 0.16, M["leather"], col, 6)
    cone("flame", (0, 0, 0.96), 0.15, 0, 0.44, M["fire"], col, 6)
    cone("core", (0, 0, 0.88), 0.08, 0, 0.26, M["ember"], col, 5)
    return col


# ---------------------------------------------------------------------------
# Optional extras
# ---------------------------------------------------------------------------
def build_spike():
    col = collection("spike")
    cone("stake", (0, 0, 0.68), 0.14, 0, 1.28, M["wood"], col, 6)
    cone("tip", (0, 0, 1.34), 0.07, 0, 0.24, M["steel"], col, 6)
    cyl("base", (0, 0, 0.08), 0.24, 0.16, M["stone"], col, 6)
    box("brace", (0, 0, 0.38), (0.42, 0.08, 0.08), M["leather"], col)
    return col


def build_farm_plot():
    col = collection("farm_plot")
    box("soil", (0, 0, 0.06), (1.9, 1.9, 0.12), M["leather"], col)
    for y in (-0.6, 0, 0.6):
        box("furrow", (0, y, 0.13), (1.7, 0.22, 0.06), M["wood"], col)
    for x in (-0.6, 0, 0.6):
        for y in (-0.6, 0.6):
            cyl("stem", (x, y, 0.28), 0.03, 0.28, M["leaf"], col, 5)
            cone("crop", (x, y, 0.44), 0.13, 0, 0.26, M["leaf"], col, 5)
    return col


def build_healing_totem():
    col = collection("healing_totem")
    cyl("post", (0, 0, 0.6), 0.18, 1.2, M["wood"], col, 6)
    cyl("base", (0, 0, 0.07), 0.3, 0.14, M["stone"], col, 6)
    ico("crystal", (0, 0, 1.42), 0.24, M["leaf"], col, 0, (0.8, 0.8, 1.5))
    for i in range(4):
        a = math.pi * i / 2
        box("leaf", (0.3 * math.cos(a), 0.3 * math.sin(a), 1.1),
            (0.26, 0.08, 0.1), M["thatch"], col)
    box("band", (0, 0, 0.85), (0.4, 0.4, 0.07), M["bone"], col)
    return col


def build_statue():
    col = collection("statue")
    box("pedestal", (0, 0, 0.14), (0.9, 0.9, 0.28), M["stone_dark"], col)
    box("plinth", (0, 0, 0.36), (0.68, 0.68, 0.18), M["stone_light"], col)
    box("body", (0, 0, 0.95), (0.38, 0.3, 0.9), M["stone"], col)
    ico("head", (0, 0, 1.56), 0.22, M["stone_light"], col, 0)
    for x in (-0.3, 0.3):
        box("arm", (x, 0, 0.98), (0.12, 0.22, 0.68), M["stone"], col)
    cone("crown", (0, 0, 1.83), 0.19, 0, 0.26, M["gold"], col, 6)
    box("sword", (0.36, 0, 0.5), (0.07, 0.07, 0.9), M["steel"], col)
    return col


def build_totem():
    col = collection("totem")
    cyl("post", (0, 0, 0.95), 0.22, 1.9, M["wood"], col, 6)
    for z, c in ((0.5, M["enemy"]), (1.05, M["cloth"]), (1.6, M["wood_light"])):
        box("mask", (0, 0, z), (0.56, 0.52, 0.44), c, col)
        for x in (-0.14, 0.14):
            ico("eye", (x, -0.28, z + 0.08), 0.06, M["gold"], col, 0)
        box("beak", (0, -0.31, z - 0.1), (0.16, 0.1, 0.16), M["bone"], col)
    for x in (-0.38, 0.38):
        box("wing", (x, 0, 1.5), (0.3, 0.08, 0.4), M["thatch"], col)
    cone("cap", (0, 0, 2.08), 0.3, 0, 0.36, M["leaf"], col, 6)
    return col


def build_ruin_tower():
    col = collection("ruin_tower")
    cyl("shaft", (0, 0, 0.95), 0.55, 1.9, M["stone"], col, 8)
    cyl("dark_band", (0, 0, 0.5), 0.57, 0.16, M["stone_dark"], col, 8)
    for i, (x, y, h) in enumerate(((0.35, 0.25, 0.42), (-0.3, 0.32, 0.26),
                                   (0.1, -0.38, 0.34), (-0.35, -0.25, 0.2))):
        box("broken_top", (x, y, 1.9 + h / 2), (0.28, 0.3, h), M["stone_light"], col)
    box("crack", (0.42, -0.34, 0.9), (0.05, 0.05, 0.7), M["stone_dark"], col)
    box("window", (0, -0.56, 1.3), (0.2, 0.05, 0.45), M["void"], col)
    ico("rubble", (0.85, 0.2, 0.18), 0.18, M["stone"], col, 0)
    return col


def build_brazier():
    col = collection("brazier")
    for i in range(3):
        a = 2 * math.pi * i / 3
        cyl("leg", (0.3 * math.cos(a), 0.3 * math.sin(a), 0.3),
            0.06, 0.6, M["iron"], col, 5)
    cone("bowl", (0, 0, 0.69), 0.08, 0.38, 0.28, M["iron"], col, 8)
    cone("coal", (0, 0, 0.85), 0.3, 0, 0.22, M["ember"], col, 8)
    cone("flame", (0, 0, 1.1), 0.22, 0, 0.5, M["fire"], col, 6)
    return col


def build_sign():
    col = collection("sign")
    box("post", (0, 0, 0.7), (0.12, 0.12, 1.4), M["wood"], col)
    box("board", (0, 0, 1.24), (1.15, 0.1, 0.42), M["wood_light"], col)
    box("trim", (0, -0.06, 1.24), (1.2, 0.04, 0.48), M["leather"], col)
    for x in (-0.32, 0, 0.32):
        box("glyph", (x, -0.09, 1.24), (0.1, 0.03, 0.14), M["gold"], col)
    cone("cap", (0, 0, 1.52), 0.14, 0, 0.2, M["shingle"], col, 4)
    return col


# ---------------------------------------------------------------------------
# Build, export, preview
# ---------------------------------------------------------------------------
ASSETS = [
    ("house", build_house),
    ("cabin", build_cabin),
    ("hut", build_hut),
    ("inn", build_inn),
    ("barn", build_barn),
    ("watchtower", build_watchtower),
    ("chest", build_chest),
    ("campfire", build_campfire),
    ("altar", build_altar),
    ("anvil", build_anvil),
    ("well", build_well),
    ("bed", build_bed),
    ("fence", build_fence),
    ("wall", build_wall),
    ("turret", build_turret),
    ("enchanting_table", build_enchanting_table),
    ("portal", build_portal),
    ("dungeon", build_dungeon),
    ("banner", build_banner),
    ("lantern", build_lantern),
    ("torch", build_torch),
    ("spike", build_spike),
    ("farm_plot", build_farm_plot),
    ("healing_totem", build_healing_totem),
    ("statue", build_statue),
    ("totem", build_totem),
    ("ruin_tower", build_ruin_tower),
    ("brazier", build_brazier),
    ("sign", build_sign),
]

built = [(name, build()) for name, build in ASSETS]
for name, col in built:
    export_asset(name, col)
    print("TRIS", name, tris(col))

# contact_sheet moves the objects for the preview, so it must come last
contact_sheet(built, "structures.png", width=3200, height=880)
print("DONE structures", len(built), "assets")
