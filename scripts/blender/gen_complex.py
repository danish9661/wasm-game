"""Starfall — complex low-poly structures (towns, ruins, dungeons).

Twelve deliberately elaborate, multi-part structures: keeps with corner towers
and spiral stairs, colonnaded town halls, an arched stone bridge, a ruined
triumph arch, a full dungeon room, etc.  Everything is flat-shaded, palette-only,
based at z = 0 and centred on (0, 0).  One collection per asset; every collection
is exported as its own GLB.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_complex.py
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import *

new_scene()

# ---------------------------------------------------------------------------
# Shared palette materials (only contract hues + slight lightness variations)
# ---------------------------------------------------------------------------
M = {
    "stone":      mat("stone", C["stone"]),
    "stone_dark": mat("stone_dark", "#5C626A"),
    "stone_light":mat("stone_light", "#99A0A8"),
    "sand_stone": mat("sand_stone", C["sand"]),
    "wood":       mat("wood", C["wood"]),
    "wood_light": mat("wood_light", "#8A5C31"),
    "leather":    mat("leather", C["leather"]),
    "shingle":    mat("shingle", "#845A30"),
    "thatch":     mat("thatch", "#6E9A46"),
    "iron":       mat("iron", C["iron"], metallic=0.35),
    "steel":      mat("steel", C["steel"], metallic=0.6, rough=0.5),
    "gold":       mat("gold", C["gold"], metallic=0.55, rough=0.45),
    "cloth":      mat("cloth", C["cloth"]),
    "cloth_red":  mat("cloth_red", C["enemy"]),
    "leaf":       mat("leaf", C["leaf"]),
    "bone":       mat("bone", C["bone"]),
    "water":      mat("water", C["water"], rough=0.35),
    "void":       mat("void", "#382A4E", emission=C["arcane"], strength=0.4),
    "fire":       mat("fire", C["fire"], emission=C["fire"], strength=3.0),
    "ember":      mat("ember", C["gold"], emission=C["fire"], strength=2.0),
    "arcane":     mat("arcane", C["arcane"], emission=C["arcane"], strength=2.5),
    "arcane_dim": mat("arcane_dim", C["arcane"], emission=C["arcane"], strength=0.6),
}

METRICS = {}


# ---------------------------------------------------------------------------
# Small local helpers.  B/CYL/CONE take a *bottom* z so "base at z=0" is easy.
# ---------------------------------------------------------------------------
def B(name, x, y, z0, sx, sy, h, material, col, rot=None):
    o = box(name, (x, y, z0 + h / 2), (sx, sy, h), material, col)
    if rot:
        o.rotation_euler = rot
    return o


def CYL(name, x, y, z0, r, h, material, col, vertices=8):
    return cyl(name, (x, y, z0 + h / 2), r, h, material, col, vertices)


def CONE(name, x, y, z0, r1, r2, h, material, col, vertices=8, rot=None):
    return cone(name, (x, y, z0 + h / 2), r1, r2, h, material, col,
                vertices, rot)


def slant(name, p1, p2, y, depth, thick, material, col):
    """Roof panel between two (x, z) points, extruded along the y axis."""
    dx, dz = p2[0] - p1[0], p2[1] - p1[1]
    length = math.hypot(dx, dz)
    o = box(name, ((p1[0] + p2[0]) / 2, y, (p1[1] + p2[1]) / 2),
            (length, depth, thick), material, col)
    o.rotation_euler = (0, -math.atan2(dz, dx), 0)
    return o


def voussoirs(cx, y, cz, radius, count, material, col,
              a0=0.0, a1=math.pi, w=0.26, depth=0.5, prefix="arch_stone"):
    """Ring of rotated blocks forming an arch in the XZ plane."""
    for i in range(count):
        a = a0 + (a1 - a0) * (i + 0.5) / count
        x = cx + radius * math.cos(a)
        z = cz + radius * math.sin(a)
        o = B(prefix, x, y, z - w / 2, w, depth, w, material, col)
        o.rotation_euler = (0, math.pi / 2 - a, 0)
    return None


def window(col, x, y, z0, axis="y", w=0.3, h=0.45):
    """Small inset window with a lintel and sill (3 boxes)."""
    if axis == "y":
        B("window_glass", x, y, z0, w, 0.05, h, M["void"], col)
        B("window_lintel", x, y - 0.04, z0 + h, w + 0.14, 0.09, 0.08,
          M["stone_light"], col)
        B("window_sill", x, y - 0.04, z0 - 0.08, w + 0.14, 0.09, 0.08,
          M["stone_light"], col)
    else:
        B("window_glass", x, y, z0, 0.05, w, h, M["void"], col)
        B("window_lintel", x - 0.04, y, z0 + h, 0.09, w + 0.14, 0.08,
          M["stone_light"], col)
        B("window_sill", x - 0.04, y, z0 - 0.08, 0.09, w + 0.14, 0.08,
          M["stone_light"], col)


def door(col, x, y, z0=0.0, w=0.9, h=2.0, facing="y"):
    """Contract-size doorway: 0.9 wide x 2.0 tall, with jambs and lintel."""
    if facing == "y":
        B("door_leaf", x, y, z0, w, 0.08, h, M["leather"], col)
        B("door_lintel", x, y - 0.02, z0 + h + 0.06, w + 0.26, 0.14, 0.16,
          M["stone_light"], col)
        for s in (-1, 1):
            B("door_jamb", x + s * (w / 2 + 0.06), y - 0.02, z0,
              0.12, 0.14, h, M["stone_light"], col)
    else:
        B("door_leaf", x, y, z0, 0.08, w, h, M["leather"], col)
        B("door_lintel", x - 0.02, y, z0 + h + 0.06, 0.14, w + 0.26, 0.16,
          M["stone_light"], col)
        for s in (-1, 1):
            B("door_jamb", x - 0.02, y + s * (w / 2 + 0.06), z0,
              0.14, 0.12, h, M["stone_light"], col)


def tri_count(col):
    n = 0
    for o in col.objects:
        if o.type == "MESH":
            o.data.calc_loop_triangles()
            n += len(o.data.loop_triangles)
    return n


def finalize(name, col):
    """Exact vertex AABB -> shift so base is z=0 and (x,y) is centred."""
    bpy.context.view_layer.update()
    pts = []
    for o in col.objects:
        if o.type != "MESH":
            continue
        m = o.matrix_world
        for v in o.data.vertices:
            pts.append(m @ v.co)
    if not pts:
        return col
    minx, maxx = min(p.x for p in pts), max(p.x for p in pts)
    miny, maxy = min(p.y for p in pts), max(p.y for p in pts)
    minz, maxz = min(p.z for p in pts), max(p.z for p in pts)
    dx, dy, dz = -(minx + maxx) / 2, -(miny + maxy) / 2, -minz
    if dx or dy or dz:
        for o in col.objects:
            o.location.x += dx
            o.location.y += dy
            o.location.z += dz
    bpy.context.view_layer.update()
    fx, fy, h = maxx - minx, maxy - miny, maxz - minz
    METRICS[name] = (fx, fy, h, tri_count(col))
    print("METRIC %-18s footprint %.2f x %.2f  height %.2f  tris %d"
          % (name, fx, fy, h, METRICS[name][3]))
    return col


# ---------------------------------------------------------------------------
# 1. castle_keep — 2-storey keep, four corner towers, great tower & spire
# ---------------------------------------------------------------------------
def build_castle_keep():
    col = collection("castle_keep")
    B("plinth", 0, 0, 0, 4.8, 4.8, 0.3, M["stone_dark"], col)
    B("main_walls", 0, 0, 0.3, 3.2, 3.2, 5.0, M["stone"], col)
    for z in (2.65, 5.15):
        B("string_course", 0, 0, z, 3.4, 3.4, 0.16, M["stone_light"], col)
    for x in (-1.8, 1.8):
        for y in (-1.8, 1.8):
            CYL("corner_tower", x, y, 0.3, 0.52, 5.4, M["stone"], col)
            CYL("tower_band", x, y, 5.45, 0.58, 0.16, M["stone_dark"], col)
            for i in range(3):
                a = 2 * math.pi * i / 3 + math.pi / 6
                B("merlon", x + 0.42 * math.cos(a), y + 0.42 * math.sin(a),
                  5.7, 0.24, 0.24, 0.4, M["stone"], col)
    CYL("great_tower", 0, 0, 0.3, 0.86, 6.0, M["stone_light"], col)
    CYL("great_band", 0, 0, 6.05, 0.93, 0.18, M["stone_dark"], col)
    for i in range(6):
        a = 2 * math.pi * i / 6
        B("great_merlon", 0.78 * math.cos(a), 0.78 * math.sin(a),
          6.3, 0.26, 0.26, 0.4, M["stone"], col)
    CONE("spire", 0, 0, 6.7, 0.62, 0, 0.8, M["shingle"], col)
    ico("finial", (0, 0, 7.58), 0.12, M["gold"], col, 0)
    door(col, 0, -1.63, 0.3)
    window(col, -0.9, -1.63, 3.1)
    window(col, 0.9, -1.63, 3.1)
    window(col, 1.63, 0, 3.6, axis="x")
    window(col, -1.63, 0, 3.6, axis="x")
    for i in range(3):
        B("step", 0, -2.42 + i * 0.26, 0, 1.5, 0.28, 0.1 * (i + 1),
          M["stone_light"], col)
    return finalize("castle_keep", col)


# ---------------------------------------------------------------------------
# 2. town_hall — colonnade, balcony, gabled roof, bell turret
# ---------------------------------------------------------------------------
def build_town_hall():
    col = collection("town_hall")
    B("plinth", 0, 0, 0, 5.2, 3.8, 0.25, M["stone_dark"], col)
    B("ground_floor", 0, 0, 0.25, 4.6, 3.2, 2.5, M["stone"], col)
    B("upper_floor", 0, 0, 2.75, 4.6, 3.2, 2.25, M["sand_stone"], col)
    B("floor_band", 0, 0, 2.68, 4.8, 3.4, 0.14, M["stone_dark"], col)
    slant("roof_r", (0, 6.0), (2.55, 5.0), 0, 3.5, 0.12, M["shingle"], col)
    slant("roof_l", (-2.55, 5.0), (0, 6.0), 0, 3.5, 0.12, M["shingle"], col)
    B("ridge", 0, 0, 5.96, 0.14, 3.6, 0.12, M["leather"], col)
    for x in (-1.7, -0.57, 0.57, 1.7):
        CYL("colonnade_col", x, -1.72, 0.25, 0.15, 2.5,
            M["stone_light"], col, 8)
        B("col_cap", x, -1.72, 2.65, 0.38, 0.38, 0.12, M["stone_dark"], col)
    for x in (-1.14, 0, 1.14):
        B("arch_lintel", x, -1.72, 2.42, 1.13, 0.34, 0.18, M["stone"], col)
    B("balcony_floor", 0, -1.9, 2.72, 4.4, 0.55, 0.14, M["wood_light"], col)
    for x in (-1.9, -1.15, -0.38, 0.38, 1.15, 1.9):
        B("baluster", x, -2.14, 2.86, 0.08, 0.08, 0.42, M["leather"], col)
    B("balcony_rail", 0, -2.14, 3.28, 4.4, 0.1, 0.09, M["leather"], col)
    door(col, 0, -1.63, 0.25)
    for x in (-1.6, 0, 1.6):
        window(col, x, -1.63, 3.5)
    CYL("bell_tower", 1.65, 0.85, 5.0, 0.46, 1.6, M["stone"], col)
    CYL("bell_band", 1.65, 0.85, 6.35, 0.52, 0.12, M["stone_dark"], col)
    CONE("bell_roof", 1.65, 0.85, 6.47, 0.6, 0, 0.7, M["shingle"], col)
    ico("bell", (1.65, 0.85, 5.85), 0.18, M["gold"], col, 0)
    for i in range(3):
        B("step", 0, -2.15 + i * 0.24, 0, 1.5, 0.26, 0.09 * (i + 1),
          M["stone_light"], col)
    B("banner_pole", -2.1, -1.9, 2.8, 0.06, 0.06, 1.6, M["wood"], col)
    B("banner_cloth", -2.1, -1.9, 3.5, 0.06, 0.5, 0.8, M["cloth_red"], col)
    return finalize("town_hall", col)


# ---------------------------------------------------------------------------
# 3. market_stall — frame, striped canopy, counter, goods, sign & lantern
# ---------------------------------------------------------------------------
def build_market_stall():
    col = collection("market_stall")
    B("platform", 0, 0, 0, 3.2, 2.2, 0.12, M["wood_light"], col)
    for x in (-1.45, 1.45):
        for y in (-0.95, 0.95):
            B("post", x, y, 0.12, 0.12, 0.12, 2.2, M["wood"], col)
    B("counter", 0, 0.35, 0.12, 2.5, 0.5, 0.72, M["wood"], col)
    B("counter_top", 0, 0.35, 0.84, 2.7, 0.62, 0.1, M["wood_light"], col)
    slant("canopy_r", (0, 2.9), (1.65, 2.42), 0, 2.2, 0.1, M["cloth"], col)
    slant("canopy_l", (-1.65, 2.42), (0, 2.9), 0, 2.2, 0.1,
          M["sand_stone"], col)
    B("ridge", 0, 0, 2.86, 0.1, 2.3, 0.1, M["leather"], col)
    for i in range(6):
        x = -1.35 + i * 0.54
        B("valance", x, -1.12, 2.36, 0.42, 0.08, 0.32,
          M["cloth"] if i % 2 else M["sand_stone"], col)
    for x in (-0.9, 0, 0.9):
        B("bolt", x, 0.35, 0.94, 0.32, 0.3, 0.22,
          M["cloth"] if x != 0 else M["cloth_red"], col)
    B("crate", -1.0, 0.9, 0.12, 0.5, 0.5, 0.5, M["wood"], col)
    B("crate_lid", -1.0, 0.9, 0.62, 0.54, 0.54, 0.08, M["leather"], col)
    B("crate2", 1.05, 0.9, 0.12, 0.42, 0.42, 0.42, M["wood_light"], col)
    CYL("barrel", 1.1, -0.5, 0.12, 0.26, 0.6, M["wood"], col)
    CYL("barrel_hoop", 1.1, -0.5, 0.34, 0.28, 0.08, M["iron"], col)
    B("sign_post", -1.55, -1.05, 0.12, 0.08, 0.08, 2.0, M["wood"], col)
    B("sign_board", -1.55, -1.05, 1.95, 0.62, 0.08, 0.36,
      M["wood_light"], col)
    B("sign_emblem", -1.55, -1.1, 1.95, 0.18, 0.04, 0.18, M["gold"], col)
    CONE("lantern", 0, -1.0, 2.1, 0.14, 0, 0.26, M["fire"], col, 6)
    return finalize("market_stall", col)


# ---------------------------------------------------------------------------
# 4. bridge — three stone arches, piers, parapets, keystones & lamps
# ---------------------------------------------------------------------------
def build_bridge():
    col = collection("bridge")
    B("deck", 0, 0, 1.45, 6.6, 2.0, 0.28, M["stone_light"], col)
    B("underdeck", 0, 0, 1.3, 6.4, 1.85, 0.16, M["stone_dark"], col)
    for x in (-3.0, -1.0, 1.0, 3.0):
        B("pier", x, 0, 0, 0.5, 1.75, 1.3, M["stone"], col)
        B("pier_foot", x, 0, 0, 0.66, 1.95, 0.18, M["stone_dark"], col)
    for cx in (-2.0, 0, 2.0):
        voussoirs(cx, 0, 0.55, 0.75, 7, M["stone"], col,
                  0, math.pi, 0.24, 1.8, "bridge_arch")
        ico("keystone", (cx, 0, 1.32), 0.16, M["stone_light"], col, 0)
        B("spandrel", cx, 0, 0.95, 1.9, 1.8, 0.5, M["stone"], col)
    for y in (-0.93, 0.93):
        B("parapet", 0, y, 1.73, 6.6, 0.16, 0.46, M["stone_light"], col)
        for x in (-2.7, -1.35, 0, 1.35, 2.7):
            B("coping", x, y, 2.19, 0.3, 0.26, 0.14, M["stone"], col)
    for x in (-3.05, 3.05):
        CYL("lamp_post", x, -0.93, 2.19, 0.06, 0.7, M["iron"], col, 6)
        CONE("lamp", x, -0.93, 2.89, 0.16, 0, 0.3, M["fire"], col, 6)
    return finalize("bridge", col)


# ---------------------------------------------------------------------------
# 5. ruins_arch — broken collonade, collapsed lintel, rubble, ivy & runes
# ---------------------------------------------------------------------------
def build_ruins_arch():
    col = collection("ruins_arch")
    B("plinth", 0, 0, 0, 4.2, 2.0, 0.18, M["stone_dark"], col)
    for x, h in ((-1.45, 2.5), (1.45, 1.55)):
        CYL("pillar_base", x, 0, 0.18, 0.52, 0.2, M["stone_dark"], col)
        CYL("pillar", x, 0, 0.38, 0.4, h, M["stone"], col)
        CYL("pillar_cap", x, 0, 0.38 + h, 0.5, 0.16,
            M["stone_light"], col)
    voussoirs(0, 0, 1.5, 1.5, 6, M["stone"], col,
              0, math.pi * 0.66, 0.3, 1.0, "arch_voussoir")
    B("fallen_lintel", 0.55, -1.0, 0.18, 0.85, 0.5, 0.42,
      M["stone_light"], col, (0, 0, 0.22))
    B("fallen_column", -0.45, 0.75, 0.18, 0.34, 0.34, 1.2,
      M["stone"], col, (0, 0, -0.3))
    for x, y, s in ((1.4, -0.8, 0.2), (-1.7, 0.7, 0.17),
                    (0.9, 1.0, 0.15), (-0.3, -1.0, 0.14), (1.9, 0.4, 0.12)):
        ico("rubble", (x, y, s), s, M["stone"], col, 0)
    for x, y, s in ((-1.45, -0.25, 0.5), (-1.45, 0.15, 0.35),
                    (-1.05, 0.3, 0.4), (1.45, -0.3, 0.3)):
        B("ivy", x, y, s, 0.05, 0.34, 0.42, M["leaf"], col)
    for x, y in ((-0.4, -0.6), (1.0, 0.5)):
        ico("rune", (x, y, 0.22), 0.1, M["arcane"], col, 0)
    return finalize("ruins_arch", col)


# ---------------------------------------------------------------------------
# 6. dungeon_room — walled cell, archway, bars, torches, chains & loot
# ---------------------------------------------------------------------------
def build_dungeon_room():
    col = collection("dungeon_room")
    B("floor", 0, 0, 0, 4.8, 4.2, 0.2, M["stone_dark"], col)
    for x in (-1.2, 0, 1.2):
        B("floor_joint", x, 0, 0.2, 0.06, 4.0, 0.02, M["stone"], col)
    B("back_wall", 0, 2.0, 0.2, 4.8, 0.3, 3.4, M["stone"], col)
    for x in (-2.3, 2.3):
        B("side_wall", x, 0, 0.2, 0.3, 4.0, 3.0, M["stone"], col)
        B("buttress", x, 1.6, 0.2, 0.4, 0.6, 3.2, M["stone_dark"], col)
    for x in (-0.85, 0.85):
        B("arch_pier", x, 1.8, 0.2, 0.3, 0.4, 2.0, M["stone_light"], col)
    voussoirs(0, 1.8, 1.95, 0.85, 5, M["stone_light"], col,
              0, math.pi, 0.22, 0.5, "door_arch")
    B("dark_void", 0, 1.72, 0.2, 1.5, 0.05, 2.0, M["void"], col)
    B("cornice_back", 0, 2.0, 3.5, 4.8, 0.5, 0.16, M["stone_dark"], col)
    for x in (-2.3, 2.3):
        B("cornice_side", x, 0, 3.1, 0.5, 4.0, 0.16, M["stone_dark"], col)
    for y in (-1.2, -0.7, -0.2, 0.3):
        CYL("cell_bar", -2.15, y, 0.2, 0.04, 2.4, M["iron"], col, 6)
    B("bar_top", -2.15, -0.45, 2.55, 0.08, 2.2, 0.08, M["iron"], col)
    for x in (-1.7, 1.7):
        B("sconce", x, 1.75, 1.7, 0.16, 0.3, 0.16, M["iron"], col)
        CONE("torch_flame", x, 1.65, 1.78, 0.14, 0, 0.4, M["fire"], col, 6)
    for x in (-0.6, 0.6):
        CYL("chain", x, -0.4, 1.1, 0.03, 2.3, M["iron"], col, 5)
        ico("hook", (x, -0.4, 0.95), 0.1, M["iron"], col, 0)
    B("chest_body", 1.3, -1.2, 0.2, 0.8, 0.5, 0.42, M["wood"], col)
    B("chest_lid", 1.3, -1.2, 0.62, 0.86, 0.54, 0.18, M["leather"], col)
    B("chest_lock", 1.3, -1.48, 0.5, 0.16, 0.05, 0.16, M["gold"], col)
    for x, y, s in ((0.2, -1.5, 0.18), (-1.2, -1.4, 0.14),
                    (1.7, 0.9, 0.16)):
        ico("bones", (x, y, s), s, M["bone"], col, 0)
    for x, y, s in ((-1.8, 0.4, 0.2), (1.9, -0.5, 0.16),
                    (0.6, 1.2, 0.14)):
        ico("rubble", (x, y, s), s, M["stone"], col, 0)
    return finalize("dungeon_room", col)


# ---------------------------------------------------------------------------
# 7. watchtower_large — stone shaft, spiral stair, battlement deck & flag
# ---------------------------------------------------------------------------
def build_watchtower_large():
    col = collection("watchtower_large")
    B("plinth", 0, 0, 0, 3.4, 3.4, 0.3, M["stone_dark"], col)
    CYL("shaft", 0, 0, 0.3, 1.05, 4.5, M["stone"], col)
    for z in (1.8, 3.6, 4.4):
        CYL("band", 0, 0, z, 1.11, 0.14, M["stone_dark"], col)
    door(col, 0, -1.06, 0.3)
    window(col, 0, -1.06, 2.6)
    window(col, 1.06, 0, 2.6, axis="x")
    window(col, -1.06, 0, 2.6, axis="x")
    B("deck", 0, 0, 4.8, 2.9, 2.9, 0.2, M["stone_light"], col)
    CYL("cabin", 0, 0, 5.0, 0.95, 1.5, M["stone_light"], col)
    for i in range(4):
        a = math.pi * i / 2
        B("cabin_window", 0.9 * math.cos(a), 0.9 * math.sin(a), 5.5,
          0.06, 0.3, 0.45, M["void"], col)
    for i in range(8):
        a = 2 * math.pi * i / 8
        B("merlon", 1.25 * math.cos(a), 1.25 * math.sin(a), 5.0,
          0.28, 0.28, 0.5, M["stone"], col)
    CONE("roof", 0, 0, 6.4, 1.45, 0, 1.0, M["shingle"], col)
    ico("finial", (0, 0, 7.45), 0.12, M["gold"], col, 0)
    for i in range(7):
        a = -math.pi / 2 + i * 0.5
        r = 1.38
        B("stair", r * math.cos(a), r * math.sin(a), 0.3 + i * 0.5,
          0.5, 0.42, 0.12, M["wood_light"], col, (0, -a, 0))
    for i in range(0, 7, 2):
        a = -math.pi / 2 + i * 0.5
        CYL("rail_post", 1.62 * math.cos(a), 1.62 * math.sin(a),
            0.42 + i * 0.5, 0.04, 0.7, M["iron"], col, 5)
    CYL("flagpole", 0, 0, 7.4, 0.04, 0.6, M["wood"], col, 6)
    B("flag", 0.28, 0, 7.85, 0.5, 0.05, 0.28, M["cloth_red"], col)
    B("lantern_arm", 0, -1.2, 3.2, 0.1, 0.6, 0.08, M["iron"], col)
    CONE("lantern", 0, -1.45, 2.95, 0.14, 0, 0.26, M["fire"], col, 6)
    return finalize("watchtower_large", col)


# ---------------------------------------------------------------------------
# 8. blacksmith_shop — forge, tall chimney, anvil, bellows & tools
# ---------------------------------------------------------------------------
def build_blacksmith_shop():
    col = collection("blacksmith_shop")
    B("foundation", 0, 0, 0, 4.4, 3.2, 0.25, M["stone_dark"], col)
    B("walls", 0, 0, 0.25, 4.2, 3.0, 2.6, M["stone"], col)
    B("upper_walls", 0, 0, 2.85, 4.2, 3.0, 1.2, M["wood"], col)
    slant("roof_r", (0, 4.55), (2.3, 4.05), 0, 3.2, 0.12,
          M["shingle"], col)
    slant("roof_l", (-2.3, 4.05), (0, 4.55), 0, 3.2, 0.12,
          M["shingle"], col)
    B("ridge", 0, 0, 4.51, 0.14, 3.3, 0.12, M["leather"], col)
    B("chimney", 1.35, 0.9, 3.9, 0.72, 0.72, 1.5, M["stone_dark"], col)
    B("chimney_cap", 1.35, 0.9, 5.4, 0.86, 0.86, 0.14, M["stone"], col)
    B("forge_mouth", -1.15, -1.54, 0.25, 1.1, 0.08, 1.1, M["void"], col)
    CONE("forge_fire", -1.15, -1.44, 0.3, 0.42, 0, 0.75,
         M["fire"], col, 6)
    ico("forge_ember", (-1.15, -1.44, 0.42), 0.16, M["ember"], col, 0)
    door(col, 1.05, -1.53, 0.25)
    B("anvil_stump", 1.15, -0.8, 0.25, 0.6, 0.6, 0.5, M["wood"], col)
    B("anvil_base", 1.15, -0.8, 0.75, 0.5, 0.36, 0.12, M["iron"], col)
    B("anvil_horn", 1.15, -0.8, 0.87, 0.62, 0.3, 0.16, M["steel"], col)
    B("bellows_body", -0.15, -0.9, 0.25, 0.7, 0.5, 0.3, M["leather"], col)
    B("bellows_top", -0.15, -0.9, 0.55, 0.62, 0.44, 0.1, M["wood"], col)
    for i in range(3):
        B("tool", 0.4 + i * 0.28, -1.52, 2.0, 0.07, 0.07, 0.55,
          M["steel"], col)
    B("tool_rack", 0.78, -1.52, 2.5, 1.0, 0.1, 0.1, M["wood"], col)
    B("trough_base", -1.55, 0.7, 0.25, 0.55, 1.0, 0.4, M["wood"], col)
    B("trough_water", -1.55, 0.7, 0.65, 0.45, 0.9, 0.05, M["water"], col)
    for x, y, s in ((0.9, 1.1, 0.2), (1.5, 1.0, 0.16), (0.4, 1.3, 0.14)):
        ico("coal", (x, y, s + 0.25), s, M["stone_dark"], col, 0)
    B("sign_board", -1.85, -1.55, 2.2, 0.6, 0.08, 0.4, M["wood_light"], col)
    B("sign_emblem", -1.85, -1.6, 2.2, 0.2, 0.04, 0.2, M["gold"], col)
    for i in range(2):
        B("step", 1.05, -1.75 + i * 0.22, 0, 1.3, 0.24,
          0.09 * (i + 1), M["stone_light"], col)
    return finalize("blacksmith_shop", col)


# ---------------------------------------------------------------------------
# 9. temple — stepped platform, six-column colonnade, pediment & idol
# ---------------------------------------------------------------------------
def build_temple():
    col = collection("temple")
    for i in range(3):
        B("platform_step", 0, 0, 0, 5.6 - i * 0.4, 4.4 - i * 0.4,
          0.12, M["stone_dark"] if i == 0 else M["stone_light"], col)
    B("stylobate", 0, 0, 0.36, 4.8, 3.6, 0.24, M["sand_stone"], col)
    for x in (-1.9, -1.14, -0.38, 0.38, 1.14, 1.9):
        CYL("column", x, -1.25, 0.6, 0.2, 2.8, M["sand_stone"], col, 8)
        B("column_cap", x, -1.25, 3.4, 0.52, 0.52, 0.16,
          M["stone_light"], col)
    B("entablature", 0, 0, 3.56, 5.0, 3.8, 0.24, M["stone_light"], col)
    B("frieze", 0, 0, 3.8, 5.0, 3.8, 0.18, M["stone_dark"], col)
    slant("pediment_r", (0, 4.9), (2.5, 3.98), 0, 3.8, 0.12,
          M["sand_stone"], col)
    slant("pediment_l", (-2.5, 3.98), (0, 4.9), 0, 3.8, 0.12,
          M["sand_stone"], col)
    B("pediment_ridge", 0, 0, 4.86, 0.14, 3.9, 0.12, M["gold"], col)
    for x in (-2.1, 2.1):
        B("cella_wall", x, 0.9, 0.6, 0.3, 1.8, 2.9, M["sand_stone"], col)
    B("cella_back", 0, 1.65, 0.6, 4.2, 0.3, 2.9, M["sand_stone"], col)
    B("idol_plinth", 0, 0.5, 0.6, 0.9, 0.9, 0.5, M["stone_dark"], col)
    ico("idol", (0, 0.5, 1.45), 0.3, M["gold"], col, 0, (0.8, 0.8, 1.4))
    CONE("idol_flame", 0, 0.5, 1.75, 0.18, 0, 0.55, M["fire"], col, 6)
    for x in (-2.55, 2.55):
        CYL("brazier_stem", x, -1.3, 0.36, 0.1, 0.9, M["stone_dark"], col, 6)
        CONE("brazier_bowl", x, -1.3, 1.26, 0.1, 0.32, 0.26,
             M["iron"], col, 8)
        CONE("brazier_flame", x, -1.3, 1.52, 0.2, 0, 0.45,
             M["fire"], col, 6)
    for x in (-1.55, 0, 1.55):
        B("front_step", x, -2.0, 0, 0.9, 0.3, 0.12, M["stone_light"], col)
    return finalize("temple", col)


# ---------------------------------------------------------------------------
# 10. wizard_tower — tapering shaft, spiral stair, balcony & crystal spire
# ---------------------------------------------------------------------------
def build_wizard_tower():
    col = collection("wizard_tower")
    CYL("base", 0, 0, 0, 1.2, 0.3, M["stone_dark"], col)
    CYL("shaft", 0, 0, 0.3, 0.82, 4.8, M["stone"], col)
    for z in (1.5, 3.0, 4.7):
        CYL("arcane_band", 0, 0, z, 0.88, 0.14, M["arcane_dim"], col)
    door(col, 0, -0.83, 0.3)
    window(col, 0, -0.83, 2.4)
    window(col, 0.83, 0, 2.4, axis="x")
    for i in range(8):
        a = -math.pi / 2 + i * 0.55
        r = 1.1
        B("spiral_step", r * math.cos(a), r * math.sin(a), 0.3 + i * 0.5,
          0.55, 0.4, 0.12, M["stone_light"], col, (0, -a, 0))
    CYL("balcony", 0, 0, 4.0, 1.3, 0.14, M["stone_light"], col)
    for i in range(6):
        a = 2 * math.pi * i / 6
        B("baluster", 1.18 * math.cos(a), 1.18 * math.sin(a), 4.14,
          0.1, 0.1, 0.42, M["iron"], col)
    CONE("roof", 0, 0, 5.1, 1.05, 0, 1.2, M["shingle"], col)
    ico("crystal", (0, 0, 6.5), 0.3, M["arcane"], col, 0, (0.7, 0.7, 1.5))
    CONE("crystal_tip", 0, 0, 6.85, 0.16, 0, 0.5, M["arcane"], col, 6)
    for i in range(4):
        a = 2 * math.pi * i / 4 + 0.4
        ico("rune", (0.95 * math.cos(a), 0.95 * math.sin(a), 5.4),
            0.1, M["arcane"], col, 0)
    CYL("antenna", 0, 0, 7.15, 0.03, 0.7, M["iron"], col, 6)
    ico("orb", (0, 0, 7.9), 0.12, M["arcane"], col, 0)
    return finalize("wizard_tower", col)


# ---------------------------------------------------------------------------
# 11. village_gate — twin towers, portcullis, drawbridge & banners
# ---------------------------------------------------------------------------
def build_village_gate():
    col = collection("village_gate")
    for x in (-1.65, 1.65):
        B("tower_foot", x, 0, 0, 1.45, 1.45, 0.25, M["stone_dark"], col)
        B("tower", x, 0, 0.25, 1.2, 1.2, 4.0, M["stone"], col)
        for z in (2.4, 4.1):
            B("tower_band", x, 0, z, 1.32, 1.32, 0.14, M["stone_dark"], col)
        for i in range(4):
            a = math.pi * i / 2 + math.pi / 4
            B("merlon", x + 0.5 * math.cos(a), 0.5 * math.sin(a), 4.24,
              0.28, 0.28, 0.45, M["stone"], col)
    B("gate_wall", 0, 0, 2.2, 2.1, 0.65, 1.7, M["stone"], col)
    for x in (-0.82, 0.82):
        B("arch_pier", x, -0.35, 0.25, 0.3, 0.5, 2.2,
          M["stone_light"], col)
    voussoirs(0, -0.35, 1.85, 0.82, 5, M["stone_light"], col,
              0, math.pi, 0.24, 0.6, "gate_arch")
    B("gate_dark", 0, -0.68, 0.25, 1.5, 0.05, 2.1, M["void"], col)
    for x in (-0.54, -0.18, 0.18, 0.54):
        B("portcullis_bar", x, -0.72, 0.25, 0.08, 0.08, 2.0, M["iron"], col)
    for z in (0.8, 1.6):
        B("portcullis_cross", 0, -0.72, z, 1.5, 0.08, 0.08, M["iron"], col)
    B("drawbridge", 0, -1.45, 0.12, 1.5, 1.5, 0.14, M["wood"], col)
    for x in (-0.65, 0.65):
        CYL("bridge_chain", x, -1.9, 0.26, 0.03, 2.0, M["iron"], col, 5)
    for x in (-1.65, 1.65):
        B("banner_pole", x, -0.66, 2.0, 0.06, 0.06, 1.6, M["wood"], col)
        B("banner", x, -0.66, 2.75, 0.06, 0.45, 0.7,
          M["cloth_red"] if x < 0 else M["cloth"], col)
        CONE("wall_lantern", x, -0.72, 1.7, 0.14, 0, 0.3,
             M["fire"], col, 6)
    for i in range(2):
        B("step", 0, -2.4 + i * 0.24, 0, 1.7, 0.26,
          0.09 * (i + 1), M["stone_light"], col)
    return finalize("village_gate", col)


# ---------------------------------------------------------------------------
# 12. stone_wall_corner — L-shaped rampart, turret, merlons & arrow slits
# ---------------------------------------------------------------------------
def build_stone_wall_corner():
    col = collection("stone_wall_corner")
    B("base_x", -1.5, 0, 0, 3.6, 0.8, 0.22, M["stone_dark"], col)
    B("base_y", 0, -1.5, 0, 0.8, 3.6, 0.22, M["stone_dark"], col)
    B("wall_x", -1.6, 0, 0.22, 3.2, 0.6, 2.4, M["stone"], col)
    B("wall_y", 0, -1.6, 0.22, 0.6, 3.2, 2.4, M["stone"], col)
    B("coping_x", -1.6, 0, 2.62, 3.3, 0.72, 0.16, M["stone_light"], col)
    B("coping_y", 0, -1.6, 2.62, 0.72, 3.3, 0.16, M["stone_light"], col)
    CYL("corner_turret", 0, 0, 0.22, 0.78, 3.4, M["stone"], col)
    CYL("turret_band", 0, 0, 3.25, 0.84, 0.14, M["stone_dark"], col)
    CYL("turret_deck", 0, 0, 3.39, 0.9, 0.16, M["stone_light"], col)
    for i in range(5):
        x = -2.8 + i * 0.63
        B("wall_merlon", x, 0, 2.78, 0.32, 0.6, 0.44, M["stone"], col)
        B("wall_merlon", 0, x, 2.78, 0.6, 0.32, 0.44, M["stone"], col)
    for i in range(4):
        a = math.pi * i / 2 + math.pi / 4
        B("turret_merlon", 0.68 * math.cos(a), 0.68 * math.sin(a),
          3.55, 0.26, 0.26, 0.42, M["stone"], col)
    for x in (-2.4, -0.9):
        B("arrow_slit", x, 0.32, 1.35, 0.12, 0.06, 0.55, M["void"], col)
        B("arrow_slit", 0.32, x, 1.35, 0.06, 0.12, 0.55, M["void"], col)
    for x, y in ((-2.9, 0.55), (0.55, -2.9)):
        B("buttress", x, y, 0.22, 0.4, 0.5, 2.2, M["stone_dark"], col)
    for x, y, s in ((-2.5, 0.9, 0.2), (0.9, -2.4, 0.17),
                    (-1.9, -1.9, 0.14)):
        ico("rubble", (x, y, s), s, M["stone"], col, 0)
    B("banner_pole", -2.55, -0.35, 2.78, 0.05, 0.05, 1.3, M["wood"], col)
    B("banner", -2.55, -0.35, 3.4, 0.05, 0.4, 0.6, M["cloth_red"], col)
    B("gate_post", 0.95, 0, 0.22, 0.34, 0.7, 2.0, M["stone_light"], col)
    return finalize("stone_wall_corner", col)


# ---------------------------------------------------------------------------
# Build, export, preview
# ---------------------------------------------------------------------------
ASSETS = [
    ("castle_keep", build_castle_keep),
    ("town_hall", build_town_hall),
    ("market_stall", build_market_stall),
    ("bridge", build_bridge),
    ("ruins_arch", build_ruins_arch),
    ("dungeon_room", build_dungeon_room),
    ("watchtower_large", build_watchtower_large),
    ("blacksmith_shop", build_blacksmith_shop),
    ("temple", build_temple),
    ("wizard_tower", build_wizard_tower),
    ("village_gate", build_village_gate),
    ("stone_wall_corner", build_stone_wall_corner),
]

built = []
for name, build in ASSETS:
    col = build()
    export_asset(name, col)
    built.append((name, col))

# The shared contact_sheet hardcodes 2.4 u spacing; these structures are up to
# 6.7 u wide, so shrink the (already-exported) objects about the world origin
# for a readable preview only.  Scaling location and scale together keeps the
# base on z = 0 and the layout centred.
PREVIEW_SCALE = 0.34
for _, col in built:
    for o in col.objects:
        o.location *= PREVIEW_SCALE
        o.scale *= PREVIEW_SCALE

# contact_sheet moves the objects for the preview, so it must come last
contact_sheet(built, "complex.png", width=4800, height=700)
print("DONE complex", len(built), "assets")
