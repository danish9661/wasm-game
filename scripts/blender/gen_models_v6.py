"""Starfall asset wave 6: visual-QA MINOR polish + 20 missing models.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_models_v6.py

Every asset in this file is authored with bmesh forms (boxes / frusta / ico
solids), a selective silhouette bevel, weighted normals, flat shading and a
non-overlapping UV pack (``expansion_common.hero_finish``). Colours are only
lightness variants of the ART_DIRECTION.md palette; no textures, no new hues.

Units: 1u = 1 tile, base z = 0, footprint centred, player = 1.55 u.
Measurements are written to assets/models/expansion_models_v6.json.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))

import bmesh
import bpy
from mathutils import Euler, Matrix, Vector

from common import new_scene, contact_sheet
from expansion_common import collection, finalize, write_records
from v5_common import V, bm_box, bm_cone

# The wave-5 palette calls dark leather "wood_dark"; keep the familiar name so
# the humanoid blocks read naturally without introducing a new hue.
V["leather"] = V["wood_dark"]

PLAYER = 1.55
new_scene()
RECORDS = []
SETS = []


# ---------------------------------------------------------------------------
# bmesh primitives. All UVs are planar-projected from the generating face so
# every polygon lands in its own atlas island; hero_finish packs them again.
# ---------------------------------------------------------------------------
def _matrix(loc, scale=(1.0, 1.0, 1.0), rot=(0.0, 0.0, 0.0)):
    return Matrix.LocRotScale(Vector(loc), Euler(rot, "XYZ").to_quaternion(),
                              Vector(scale))


def _new_object(name, bm, material, col):
    bm.normal_update()
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    uv = bm.loops.layers.uv.verify()
    for face in bm.faces:
        n = face.normal
        for loop in face.loops:
            co = loop.vert.co
            if abs(n.z) >= abs(n.x) and abs(n.z) >= abs(n.y):
                u, v = co.x, co.y
            elif abs(n.x) >= abs(n.y):
                u, v = co.y, co.z
            else:
                u, v = co.x, co.z
            loop[uv].uv = (u, v)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(material)
    for poly in me.polygons:
        poly.use_smooth = False
    obj = bpy.data.objects.new(name, me)
    col.objects.link(obj)
    return obj


def B(p, name, loc, scale, key, rot=(0.0, 0.0, 0.0)):
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0, matrix=_matrix(loc, scale, rot))
    return _new_object(name, bm, V[key], p)


def K(p, name, loc, r1, r2, depth, key, verts=6, rot=(0.0, 0.0, 0.0)):
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=verts,
                          radius1=r1, radius2=r2, depth=depth,
                          matrix=_matrix(loc, (1.0, 1.0, 1.0), rot))
    return _new_object(name, bm, V[key], p)


def O(p, name, loc, radius, key, scale=(1.0, 1.0, 1.0), sub=0):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=sub, radius=radius,
                               matrix=_matrix(loc, scale))
    return _new_object(name, bm, V[key], p)


def R(p, name, loc, radius, depth, key, verts=6, rot=(0.0, 0.0, 0.0)):
    return K(p, name, loc, radius, radius, depth, key, verts, rot)


def W(p, name, loc, width, depth, height, key, rot=(0.0, 0.0, 0.0)):
    """A four-sided tapered wedge (angular gable / blade / awning)."""
    return K(p, name, loc, width / 2.0, 0.02, height, key, 4,
             (rot[0] + math.pi / 4.0, rot[1], rot[2]))


# Bevel only the silhouette-defining group; small detail facets stay crisp and
# the triangle budget is not wasted on sub-10 px accents.
BEVEL_TAGS = ("torso", "body", "head", "shell", "roof", "dome", "shaft",
              "counter", "dais", "anvil", "tower", "bale", "core",
              "pedestal", "plinth", "cap", "horn")


def f(name, col, target, family, notes):
    rec = finalize(name, col, "models_v6", target, notes, hero=True,
                   bevel_width=0.005, bevel_parts=BEVEL_TAGS, records=RECORDS)
    SETS.append((name, col))
    return rec


# ===========================================================================
# A - interior variants (3), 1u grid, 2.5u storey, 0.9x2.0 doors
# ===========================================================================
def room_shell(p, half, floor_key, wall_key, window_keys=(), door=True):
    B(p, "floor_slab", (0, 0, 0.05), (half * 2, half * 2, 0.10), floor_key)
    for i, y in enumerate((-1.0, 0.0, 1.0)):
        B(p, "plank_%d" % i, (0, y, 0.106), (half * 2, 0.025, 0.012),
          "wood_dark")
    # Solid north / west walls; south / east are 1.1u cutaways for the camera.
    B(p, "wall_north", (0, half, 1.25), (half * 2, 0.16, 2.5), wall_key)
    B(p, "wall_west", (-half, 0, 1.25), (0.16, half * 2, 2.5), wall_key)
    if door:
        for i, x in enumerate((-1.45, 1.45)):
            B(p, "wall_south_%d" % i, (x, -half, 0.55),
              (1.10, 0.16, 1.10), wall_key)
    else:
        B(p, "wall_south", (0, -half, 0.55), (half * 2, 0.16, 1.10), wall_key)
    B(p, "wall_east", (half, 0, 0.55), (0.16, half * 2, 1.10), wall_key)
    for i, (x, y) in enumerate(((half, -half), (-half, -half),
                                (-half, half), (half, half))):
        h = 1.1 if (x > 0 and y < 0) else 2.5
        B(p, "corner_post_%d" % i, (x, y, h / 2), (0.7, 0.7, h), wall_key)
    if door:
        for i, x in enumerate((-0.50, 0.50)):
            B(p, "door_jamb_%d" % i, (x, -half, 1.0),
              (0.10, 0.20, 2.0), "wood_dark")
        B(p, "door_leaf", (0, -half - 0.05, 1.0), (0.90, 0.10, 2.0), "wood")
        B(p, "door_lintel", (0, -half, 2.08), (1.10, 0.24, 0.16), "stone_dark")
        B(p, "door_band", (0, -half - 0.11, 1.35), (0.90, 0.04, 0.08), "iron")
    for i, x in enumerate(window_keys):
        B(p, "window_frame_%d" % i, (x, half - 0.04, 1.55),
          (0.86, 0.10, 0.86), "wood_dark")
        B(p, "window_glass_%d" % i, (x, half - 0.10, 1.55),
          (0.68, 0.03, 0.68), "ice")
        B(p, "window_sill_%d" % i, (x, half - 0.12, 1.08),
          (1.00, 0.18, 0.08), "stone_light")
        pass  # window bar omitted: the frame and sill already read at 60 px


def build_library_interior():
    p = collection("library_interior")
    room_shell(p, 2, "wood", "stone", (-1.0, 1.0))
    for i, x in enumerate((-1.45, -0.55, 0.55, 1.45)):
        B(p, "shelf_%d" % i, (x, 1.62, 1.00), (0.72, 0.34, 1.92), "wood_dark")
        for j, z in enumerate((0.52, 1.06, 1.60)):
            B(p, "shelf_plank_%d_%d" % (i, j), (x, 1.42, z),
              (0.70, 0.06, 0.06), "wood")
            for k, key in enumerate(("enemy", "arcane", "cloth", "sand")):
                B(p, "book_%d_%d_%d" % (i, j, k),
                  (x - 0.24 + k * 0.16, 1.44, z + 0.15),
                  (0.11, 0.14, 0.24), key)
    B(p, "reading_desk", (-0.40, -0.55, 0.70), (1.50, 0.74, 0.10), "wood")
    for i, x in enumerate((-0.95, 0.15)):
        B(p, "desk_leg_%d" % i, (x, -0.55, 0.36), (0.12, 0.60, 0.70), "wood_dark")
    B(p, "open_book", (-0.40, -0.55, 0.80), (0.52, 0.40, 0.06), "bone")
    B(p, "book_spine", (-0.40, -0.55, 0.84), (0.04, 0.40, 0.03), "gold")
    K(p, "globe", (0.95, -1.10, 1.06), 0.28, 0.28, 0.56, "water", 8)
    B(p, "globe_stand", (0.95, -1.10, 0.62), (0.10, 0.10, 0.60), "gold")
    for i, (x, y) in enumerate(((-1.35, -1.05), (1.30, 0.45))):
        R(p, "scroll_%d" % i, (x, y, 0.93), 0.10, 0.56, "bone", 6,
          (0, math.pi / 2, 0.2))
    B(p, "desk_lamp", (-1.05, -0.55, 0.96), (0.14, 0.14, 0.24), "gold")
    K(p, "lamp_flame", (-1.05, -0.55, 1.16), 0.08, 0.0, 0.20, "fire", 5)
    return p


def build_guard_post_interior():
    p = collection("guard_post_interior")
    room_shell(p, 2, "stone_dark", "stone", (-1.0, 1.0))
    for i, x in enumerate((-1.30, 1.30)):
        B(p, "bunk_%d" % i, (x, 1.30, 0.36), (0.90, 1.00, 0.22), "wood")
        B(p, "bunk_blanket_%d" % i, (x, 1.30, 0.52),
          (0.86, 0.72, 0.14), "cloth" if i == 0 else "enemy")
        B(p, "pillow_%d" % i, (x, 0.98, 0.55), (0.60, 0.26, 0.12), "bone")
        for j, y in enumerate((0.94, 1.66)):
            B(p, "bunk_leg_%d_%d" % (i, j), (x, y, 0.18),
              (0.10, 0.10, 0.36), "wood_dark")
    B(p, "weapon_rack", (0, 1.82, 1.28), (1.70, 0.18, 0.14), "wood_dark")
    for i, x in enumerate((-0.55, -0.18, 0.18, 0.55)):
        B(p, "rack_blade_%d" % i, (x, 1.76, 0.96), (0.06, 0.05, 0.68),
          "steel")
        B(p, "rack_grip_%d" % i, (x, 1.76, 0.56), (0.08, 0.06, 0.22), "leather")
    B(p, "map_table", (0, -0.60, 0.74), (1.40, 0.90, 0.10), "wood")
    for i, x in enumerate((-0.55, 0.55)):
        B(p, "table_leg_%d" % i, (x, -0.60, 0.36),
          (0.12, 0.70, 0.68), "wood_dark")
    B(p, "tactical_map", (0, -0.60, 0.82), (0.94, 0.58, 0.03), "sand")
    B(p, "map_mark", (-0.14, -0.52, 0.85), (0.08, 0.08, 0.03), "enemy")
    B(p, "chest", (1.34, -1.08, 0.36), (0.84, 0.56, 0.68), "wood_dark")
    B(p, "chest_lid", (1.34, -1.08, 0.74), (0.90, 0.60, 0.12), "wood")
    B(p, "chest_band", (1.34, -1.08, 0.50), (0.10, 0.60, 0.70), "iron")
    B(p, "banner", (-1.86, 0.30, 1.60), (0.06, 0.72, 1.20), "enemy")
    B(p, "banner_rod", (-1.80, 0.30, 2.26), (0.20, 0.80, 0.06), "gold")
    K(p, "brazier", (-1.40, -1.28, 0.58), 0.28, 0.16, 0.28, "iron", 6)
    K(p, "brazier_flame", (-1.40, -1.28, 0.88), 0.16, 0.0, 0.40, "fire", 5)
    return p


def build_market_stall_interior():
    p = collection("market_stall_interior")
    room_shell(p, 2, "wood", "wood", (1.0,))
    B(p, "counter", (0.90, -0.20, 0.58), (0.94, 2.60, 1.12), "wood_dark")
    B(p, "counter_top", (0.90, -0.20, 1.18), (1.12, 2.80, 0.12), "stone_light")
    B(p, "counter_rail", (0.28, -0.20, 1.12), (0.10, 2.80, 0.10), "gold")
    for i, y in enumerate((-1.00, -0.34, 0.34, 1.00)):
        B(p, "counter_slat_%d" % i, (0.36, y, 0.70),
          (0.08, 0.50, 0.90), "wood")
    for i, (x, y, key) in enumerate(((-0.90, -0.90, "enemy"),
                                     (-0.28, -0.30, "gold"),
                                     (-1.10, 0.30, "leaf"),
                                     (-0.50, 1.00, "sand"))):
        B(p, "display_crate_%d" % i, (x, y, 0.34), (0.70, 0.64, 0.64), "wood")
        O(p, "produce_%d" % i, (x, y, 0.76), 0.17, key, (1.0, 1.0, 0.8))
    B(p, "awning_frame", (-0.20, -1.70, 2.02), (3.40, 0.12, 0.12), "wood_dark")
    for i, x in enumerate((-1.40, 1.00)):
        B(p, "awning_post_%d" % i, (x, -1.70, 1.04),
          (0.12, 0.12, 2.06), "wood")
    for i, x in enumerate((-1.0, -0.2, 0.6)):
        W(p, "awning_panel_%d" % i, (x, -1.72, 2.20),
          0.90, 0.90, 0.36, "enemy" if i % 2 else "gold")
    R(p, "scale_dial", (-1.30, 0.95, 1.18), 0.24, 0.06, "gold", 8,
      (math.pi / 2, 0, 0))
    B(p, "scale_post", (-1.30, 0.95, 0.90), (0.10, 0.10, 0.60), "iron")
    B(p, "hanging_lantern", (0.30, 1.40, 1.92), (0.26, 0.26, 0.34), "gold")
    K(p, "lantern_flame", (0.30, 1.40, 1.66), 0.08, 0.0, 0.16, "fire", 5)
    return p


INTERIORS = [
    ("library_interior", build_library_interior, (4.2, 4.2, 2.5),
     "interior hero", "4x4 library; shelves, reading desk, globe, scrolls"),
    ("guard_post_interior", build_guard_post_interior, (4.2, 4.2, 2.5),
     "interior hero", "4x4 guard post; bunks, weapon rack, map, chest"),
    ("market_stall_interior", build_market_stall_interior, (4.2, 4.2, 2.7),
     "interior hero", "4x4 market stall; counter, crates, awning, scales"),
]


# ===========================================================================
# B - biome-specific landmark pieces (5)
# ===========================================================================
def build_desert_obelisk_landmark():
    p = collection("desert_obelisk_landmark")
    B(p, "dune", (0, 0, 0.10), (1.90, 1.50, 0.20), "sand")
    B(p, "plinth", (0, 0, 0.30), (1.10, 1.10, 0.40), "sand_dark")
    K(p, "shaft", (0, 0, 1.62), 0.36, 0.22, 2.36, "sand", 4,
      (0, 0, math.pi / 4))
    for i, z in enumerate((0.92, 1.42, 1.92)):
        B(p, "glyph_band_%d" % i, (0, -0.31, z), (0.44, 0.06, 0.10), "gold")
    K(p, "cap", (0, 0, 2.90), 0.24, 0.0, 0.36, "gold", 4,
      (0, 0, math.pi / 4))
    for i, (x, y, r) in enumerate(((-0.72, 0.36, 0.24), (0.66, -0.30, 0.20))):
        O(p, "rock_%d" % i, (x, y, r * 0.6), r, "sand_dark", (1.3, 1.0, 0.7))
    return p


def build_tundra_totem_landmark():
    p = collection("tundra_totem_landmark")
    B(p, "snow_base", (0, 0, 0.10), (1.40, 1.30, 0.20), "snow")
    B(p, "pedestal", (0, 0, 0.34), (0.86, 0.86, 0.30), "stone_dark")
    K(p, "shaft", (0, 0, 1.24), 0.26, 0.20, 1.74, "ice", 6)
    O(p, "face", (0, -0.22, 1.72), 0.24, "snow", (1.0, 0.7, 1.0))
    for i, x in enumerate((-0.10, 0.10)):
        O(p, "eye_%d" % i, (x, -0.39, 1.76), 0.055, "arcane")
    for i, x in enumerate((-0.24, 0.24)):
        B(p, "antler_%d" % i, (x, 0, 2.02), (0.08, 0.08, 0.68), "frost",
          (0, 0, 0.34 if i else -0.34))
        K(p, "antler_tip_%d" % i, (x * 1.55, 0, 2.34), 0.07, 0.0, 0.22,
          "frost", 5)
    K(p, "rune", (0, -0.27, 1.10), 0.12, 0.0, 0.24, "arcane", 5,
      (math.pi / 2, 0, 0))
    return p


def build_swamp_shrine_landmark():
    p = collection("swamp_shrine_landmark")
    B(p, "mud_pool", (0, 0, 0.05), (1.70, 1.40, 0.10), "swamp_dark")
    for i, (x, y, a) in enumerate(((-0.55, -0.35, 0.5), (-0.50, 0.38, -0.5),
                                    (0.55, -0.38, -0.5), (0.55, 0.35, 0.5))):
        B(p, "root_%d" % i, (x, y, 0.34), (0.12, 0.12, 0.66), "wood_dark",
          (a, 0, 0))
    B(p, "plinth", (0, 0, 0.40), (1.10, 0.90, 0.32), "stone_dark")
    B(p, "frame", (0, 0, 1.18), (0.96, 0.34, 1.24), "wood")
    K(p, "arch", (0, 0, 1.94), 0.52, 0.52, 0.34, "wood_dark", 8,
      (math.pi / 2, 0, 0))
    O(p, "idol", (0, -0.20, 1.18), 0.30, "swamp", (0.8, 0.6, 1.2))
    for i, x in enumerate((-0.12, 0.12)):
        O(p, "idol_eye_%d" % i, (x, -0.42, 1.28), 0.06, "arcane")
    for i, x in enumerate((-0.42, 0.42)):
        B(p, "bone_charm_%d" % i, (x, -0.24, 0.90), (0.10, 0.08, 0.42), "bone")
    K(p, "offering", (0, -0.62, 0.60), 0.16, 0.20, 0.18, "gold", 6)
    return p


def build_volcanic_forge_landmark():
    p = collection("volcanic_forge_landmark")
    O(p, "basalt_base", (0, 0, 0.14), 0.62, "char", (1.5, 1.3, 0.5))
    B(p, "pedestal", (0, 0, 0.38), (1.10, 0.92, 0.48), "obsidian")
    B(p, "anvil", (0, 0, 0.78), (0.94, 0.56, 0.36), "iron")
    K(p, "horn", (0.70, 0, 0.78), 0.20, 0.0, 0.52, "iron", 6,
      (0, math.pi / 2, 0))
    for i, x in enumerate((-0.34, 0.34)):
        B(p, "lava_channel_%d" % i, (x, -0.48, 0.44), (0.14, 0.06, 0.46),
          "fire")
    B(p, "chimney", (-0.62, 0.30, 1.06), (0.40, 0.40, 1.28), "obsidian")
    B(p, "chimney_cap", (-0.62, 0.30, 1.74), (0.52, 0.52, 0.10), "char")
    for i, (x, y, h) in enumerate(((-0.30, 0.20, 0.50), (0.10, -0.10, 0.66),
                                    (0.34, 0.16, 0.40))):
        K(p, "flame_%d" % i, (x, y, 0.96 + h / 2), 0.13, 0.0, h, "fire", 5)
    return p


def build_ocean_lighthouse_landmark():
    p = collection("ocean_lighthouse_landmark")
    O(p, "rock_base", (0, 0, 0.18), 0.70, "stone_dark", (1.6, 1.3, 0.55))
    K(p, "tower", (0, 0, 1.28), 0.38, 0.28, 2.04, "stone", 8)
    for i, z in enumerate((0.72, 1.42, 2.12)):
        R(p, "band_%d" % i, (0, 0, z), 0.35 - 0.03 * i, 0.16, "enemy", 8)
    B(p, "gallery", (0, 0, 2.38), (1.06, 1.06, 0.12), "stone_dark")
    for i, a in enumerate((0.0, 1.57, 3.14, 4.71)):
        B(p, "rail_%d" % i, (0.44 * math.cos(a), 0.44 * math.sin(a), 2.62),
          (0.08, 0.08, 0.40), "iron")
    B(p, "lamp_room", (0, 0, 2.78), (0.60, 0.60, 0.54), "gold")
    O(p, "lamp", (0, 0, 2.82), 0.24, "fire", (1.0, 1.0, 1.0))
    K(p, "roof", (0, 0, 3.22), 0.44, 0.0, 0.38, "enemy", 8)
    B(p, "door", (0, -0.36, 0.48), (0.42, 0.08, 0.76), "wood_dark")
    return p


LANDMARKS = [
    ("desert_obelisk_landmark", build_desert_obelisk_landmark,
     (1.9, 1.5, 3.1), "biome landmark", "sandstone obelisk + gold cap"),
    ("tundra_totem_landmark", build_tundra_totem_landmark,
     (1.4, 1.3, 2.5), "biome landmark", "ice totem + frost antlers"),
    ("swamp_shrine_landmark", build_swamp_shrine_landmark,
     (1.7, 1.4, 2.2), "biome landmark", "rooted swamp idol + charms"),
    ("volcanic_forge_landmark", build_volcanic_forge_landmark,
     (1.5, 1.3, 1.9), "biome landmark", "basalt forge + lava channels"),
    ("ocean_lighthouse_landmark", build_ocean_lighthouse_landmark,
     (1.4, 1.3, 3.4), "biome landmark", "banded lighthouse + lamp"),
]


# ===========================================================================
# C - NPC profession variants (7). Shared humanoid base, distinct tool chain.
# ===========================================================================
def humanoid(p, name, cloth="cloth", accent="gold", hat="hair",
             skin="skin", height=1.0):
    s = height
    B(p, "torso", (0, 0, 1.06 * s), (0.44, 0.28, 0.62 * s), cloth)
    B(p, "belt", (0, 0, 0.78 * s), (0.46, 0.30, 0.10), "leather")
    K(p, "hips", (0, 0, 0.68 * s), 0.21, 0.20, 0.22, cloth, 6)
    for i, x in enumerate((-0.14, 0.14)):
        B(p, "leg_%d" % i, (x, 0, 0.36 * s),
          (0.16, 0.17, 0.66 * s), "wood_dark")
        B(p, "boot_%d" % i, (x, -0.04, 0.06), (0.20, 0.26, 0.12), "leather")
        B(p, "arm_%d" % i, (x * 2.05, 0, 1.06 * s),
          (0.14, 0.16, 0.54 * s), cloth,
          (0, 0, -0.12 if i else 0.12))
        O(p, "hand_%d" % i, (x * 2.55, -0.02, 0.80 * s),
          0.09, skin, (1.0, 0.9, 1.0))
    R(p, "neck", (0, 0, 1.42 * s), 0.10, 0.12, skin, 6)
    O(p, "head", (0, -0.02, 1.60 * s), 0.19, skin, (1.0, 0.92, 1.0))
    B(p, "hair", (0, 0.07, 1.68 * s), (0.38, 0.30, 0.16), hat)
    for i, x in enumerate((-0.075, 0.075)):
        B(p, "eye_%d" % i, (x, -0.185, 1.62 * s),
          (0.042, 0.02, 0.042), "char")
    B(p, "collar", (0, -0.14, 1.34 * s), (0.34, 0.08, 0.10), accent)


def build_npc_alchemist():
    p = collection("npc_alchemist")
    humanoid(p, "alchemist", "arcane", "gold", "arcane")
    K(p, "hat", (0, 0, 1.92), 0.28, 0.06, 0.44, "arcane", 6)
    B(p, "hat_brim", (0, 0, 1.72), (0.66, 0.52, 0.07), "arcane")
    for i, (x, key) in enumerate(((-0.46, "water"), (-0.20, "fire"))):
        B(p, "pouch_%d" % i, (x, -0.02, 0.86), (0.20, 0.18, 0.28), "leather")
        K(p, "vial_%d" % i, (x, -0.14, 1.00), 0.07, 0.05, 0.20, key, 6)
    R(p, "flask", (0.42, -0.16, 1.06), 0.13, 0.28, "steel", 6)
    K(p, "flask_neck", (0.42, -0.16, 1.26), 0.06, 0.06, 0.16, "steel", 6)
    O(p, "potion", (0.42, -0.16, 1.08), 0.09, "arcane")
    return p


def build_npc_cartographer():
    p = collection("npc_cartographer")
    humanoid(p, "cartographer", "cloth", "sand", "leather")
    B(p, "hat_brim", (0, 0, 1.74), (0.72, 0.64, 0.07), "leather")
    B(p, "hat_crown", (0, 0, 1.86), (0.40, 0.40, 0.20), "leather")
    R(p, "map_roll", (-0.42, -0.20, 1.06), 0.10, 0.62, "bone", 6,
      (0, 0, 0.42))
    B(p, "map_sheet", (0.36, -0.28, 1.10), (0.54, 0.04, 0.46), "sand",
      (0.2, 0.0, 0.1))
    B(p, "map_route", (0.36, -0.31, 1.12), (0.06, 0.02, 0.36), "enemy",
      (0.2, 0.0, 0.1))
    B(p, "quill", (0.46, -0.18, 1.36), (0.05, 0.05, 0.42), "gold",
      (0, 0, 0.28))
    B(p, "satchel", (-0.30, 0.16, 0.90), (0.32, 0.20, 0.36), "wood_dark")
    return p


def build_npc_chef():
    p = collection("npc_chef")
    humanoid(p, "chef", "snow", "enemy", "hair")
    B(p, "apron", (0, -0.17, 0.90), (0.44, 0.06, 0.74), "bone")
    B(p, "apron_bib", (0, -0.17, 1.24), (0.32, 0.06, 0.30), "bone")
    R(p, "chef_hat", (0, 0, 1.90), 0.26, 0.36, "snow", 8)
    B(p, "hat_band", (0, 0, 1.76), (0.56, 0.52, 0.10), "snow")
    B(p, "ladle_handle", (0.48, -0.20, 1.10), (0.06, 0.06, 0.78), "wood",
      (0, 0, 0.22))
    O(p, "ladle_bowl", (0.58, -0.22, 0.78), 0.14, "steel", (1.0, 0.8, 0.5))
    B(p, "towel", (-0.38, -0.18, 0.86), (0.14, 0.06, 0.36), "bone")
    return p


def build_npc_herbalist():
    p = collection("npc_herbalist")
    humanoid(p, "herbalist", "swamp", "leaf", "swamp_dark")
    K(p, "hood", (0, 0, 1.78), 0.24, 0.08, 0.40, "swamp", 6)
    B(p, "satchel", (-0.36, 0.14, 0.90), (0.38, 0.26, 0.46), "wood_dark")
    for i, (x, key) in enumerate(((-0.38, "leaf"), (0.38, "gold"),
                                   (0.50, "arcane"))):
        B(p, "herb_stem_%d" % i, (x, -0.12, 1.16),
          (0.05, 0.05, 0.42), "leaf_dark", (0, 0, 0.2 if i else -0.2))
        O(p, "herb_tip_%d" % i, (x + 0.05, -0.12, 1.38), 0.11, key)
    B(p, "belt_pouch", (0.36, 0.04, 0.84), (0.20, 0.16, 0.24), "leather")
    return p


def build_npc_jeweler():
    p = collection("npc_jeweler")
    humanoid(p, "jeweler", "cloth", "gold", "hair")
    B(p, "vest", (0, -0.16, 1.06), (0.38, 0.06, 0.56), "arcane")
    for i, z in enumerate((1.02, 1.16)):
        B(p, "vest_band_%d" % i, (0, -0.20, z), (0.38, 0.04, 0.05), "gold")
    R(p, "loupe", (0.42, -0.24, 1.48), 0.13, 0.05, "gold", 8,
      (math.pi / 2, 0, 0))
    B(p, "loupe_handle", (0.42, -0.12, 1.48), (0.05, 0.26, 0.05), "wood_dark")
    K(p, "gem", (-0.42, -0.18, 1.10), 0.12, 0.0, 0.20, "arcane", 5)
    B(p, "gem_tray", (-0.42, -0.18, 0.98), (0.30, 0.24, 0.06), "gold")
    B(p, "apron", (0, -0.17, 0.90), (0.42, 0.06, 0.56), "leather")
    return p


def build_npc_shepherd():
    p = collection("npc_shepherd")
    humanoid(p, "shepherd", "sand", "wood", "leather")
    B(p, "cloak", (0, 0.18, 1.10), (0.52, 0.10, 0.84), "wood_dark")
    B(p, "staff", (0.48, -0.10, 1.14), (0.07, 0.07, 1.92), "wood",
      (0, 0, 0.06))
    R(p, "crook", (0.60, -0.10, 2.08), 0.12, 0.06, "wood_dark", 6,
      (math.pi / 2, 0, 0))
    B(p, "crook_tip", (0.72, -0.10, 2.02), (0.14, 0.07, 0.12), "wood_dark")
    B(p, "wool_sack", (-0.44, 0.14, 0.84), (0.34, 0.24, 0.40), "bone")
    B(p, "hat_brim", (0, 0, 1.72), (0.62, 0.56, 0.06), "leather")
    R(p, "whistle", (-0.44, -0.18, 1.02), 0.05, 0.20, "gold", 6)
    return p


def build_npc_physician():
    p = collection("npc_physician")
    humanoid(p, "physician", "snow", "enemy", "hair")
    B(p, "coat", (0, -0.16, 0.96), (0.46, 0.06, 0.86), "bone")
    B(p, "cross_h", (0, -0.21, 1.02), (0.22, 0.03, 0.07), "enemy")
    B(p, "cross_v", (0, -0.21, 1.02), (0.07, 0.03, 0.22), "enemy")
    R(p, "spectacle_l", (-0.09, -0.20, 1.62), 0.07, 0.03, "steel", 8,
      (math.pi / 2, 0, 0))
    R(p, "spectacle_r", (0.09, -0.20, 1.62), 0.07, 0.03, "steel", 8,
      (math.pi / 2, 0, 0))
    B(p, "spectacle_bridge", (0, -0.21, 1.62), (0.08, 0.03, 0.03), "steel")
    B(p, "medicine_case", (0.42, 0.10, 0.86), (0.30, 0.20, 0.36), "wood_dark")
    K(p, "tonic", (-0.42, -0.18, 1.06), 0.08, 0.08, 0.26, "arcane", 6)
    return p


NPCS = [
    ("npc_alchemist", build_npc_alchemist, (0.9, 0.7, 2.1),
     "npc profession", "hooded alchemist with flask and vials"),
    ("npc_cartographer", build_npc_cartographer, (1.0, 0.7, 1.9),
     "npc profession", "map, quill and wide-brim hat"),
    ("npc_chef", build_npc_chef, (1.0, 0.7, 2.0),
     "npc profession", "white uniform, tall hat and ladle"),
    ("npc_herbalist", build_npc_herbalist, (1.0, 0.8, 1.9),
     "npc profession", "hood, herb bundle and satchel"),
    ("npc_jeweler", build_npc_jeweler, (0.9, 0.7, 1.8),
     "npc profession", "loupe, gem tray and gold-trimmed vest"),
    ("npc_shepherd", build_npc_shepherd, (1.0, 0.7, 2.2),
     "npc profession", "crook, cloak and wool sack"),
    ("npc_physician", build_npc_physician, (0.9, 0.7, 1.8),
     "npc profession", "coat with red cross, spectacles and tonic"),
]


# ===========================================================================
# D - five new building silhouettes
# ===========================================================================
def gable_roof(p, prefix, half_x, half_y, z, key, pitch=0.42):
    for i, sign in enumerate((-1, 1)):
        B(p, "%s_roof_%d" % (prefix, i),
          (sign * half_x * 0.52, 0, z + 0.22),
          (half_x * 1.14, half_y * 2, 0.10), key,
          (0, sign * pitch, 0))


def build_granary():
    p = collection("granary")
    B(p, "floor_slab", (0, 0, 0.58), (2.4, 1.8, 0.16), "wood")
    for i, x in enumerate((-0.95, 0.95)):
        for j, y in enumerate((-0.68, 0.68)):
            B(p, "stilt_%d_%d" % (i, j), (x, y, 0.28),
              (0.20, 0.20, 0.56), "wood_dark")
    for i, y in enumerate((-0.80, 0.80)):
        B(p, "wall_%d" % i, (0, y, 1.22), (2.42, 0.14, 1.18), "wood")
    for i, x in enumerate((-1.14, 1.14)):
        B(p, "wall_end_%d" % i, (x, 0, 1.22), (0.14, 1.66, 1.18), "wood_dark")
    gable_roof(p, "granary", 1.25, 0.95, 1.94, "stone_dark")
    B(p, "loft_door", (0, -0.89, 1.20), (0.62, 0.08, 0.86), "wood_dark")
    B(p, "loft_lintel", (0, -0.90, 1.70), (0.76, 0.10, 0.10), "gold")
    for i, z in enumerate((0.98, 1.44)):
        B(p, "vent_%d" % i, (0, 0.89, z), (0.48, 0.06, 0.10), "iron")
    B(p, "ladder_a", (-0.34, -1.02, 0.52), (0.08, 0.08, 1.04), "wood_dark",
      (0, 0, 0.10))
    B(p, "ladder_b", (0.34, -1.02, 0.52), (0.08, 0.08, 1.04), "wood_dark",
      (0, 0, -0.10))
    for i, z in enumerate((0.28, 0.62, 0.96)):
        B(p, "rung_%d" % i, (0, -1.02, z), (0.72, 0.07, 0.07), "wood")
    return p


def build_chapel():
    p = collection("chapel")
    B(p, "step", (0, -0.90, 0.08), (1.32, 0.60, 0.16), "stone_light")
    B(p, "base", (0, 0, 0.18), (1.96, 1.60, 0.36), "stone_dark")
    for i, y in enumerate((-0.70, 0.70)):
        B(p, "wall_%d" % i, (0, y, 1.26), (1.96, 0.18, 1.80), "stone")
    for i, x in enumerate((-0.89, 0.89)):
        B(p, "wall_end_%d" % i, (x, 0, 1.26), (0.18, 1.42, 1.80), "stone_light")
    gable_roof(p, "chapel", 1.08, 0.86, 2.26, "stone_dark")
    B(p, "door_frame", (0, -0.82, 0.86), (0.72, 0.10, 1.34), "wood_dark")
    B(p, "door_leaf", (0, -0.88, 0.84), (0.56, 0.08, 1.16), "wood")
    B(p, "door_arch", (0, -0.84, 1.52), (0.76, 0.12, 0.12), "gold")
    for i, x in enumerate((-0.58, 0.58)):
        B(p, "window_%d" % i, (x, -0.82, 1.40), (0.24, 0.06, 0.52), "arcane")
        B(p, "window_frame_%d" % i, (x, -0.84, 1.40),
          (0.32, 0.04, 0.62), "gold")
    B(p, "belfry", (0, 0, 2.64), (0.60, 0.46, 0.60), "stone")
    R(p, "bell", (0, -0.26, 2.62), 0.12, 0.28, "gold", 6)
    K(p, "belfry_cap", (0, 0, 3.02), 0.46, 0.0, 0.32, "enemy", 4,
      (0, 0, math.pi / 4))
    return p


def build_fisher_hut():
    p = collection("fisher_hut")
    B(p, "pier", (0, -0.92, 0.12), (2.60, 0.90, 0.20), "wood_dark")
    for i, x in enumerate((-1.06, 1.06)):
        for j, y in enumerate((-0.30, 0.30)):
            B(p, "stilt_%d_%d" % (i, j), (x, y, 0.44),
              (0.18, 0.18, 0.88), "wood_dark")
    B(p, "floor", (0, 0, 0.90), (2.20, 1.60, 0.16), "wood")
    for i, y in enumerate((-0.66, 0.66)):
        B(p, "wall_%d" % i, (0, y, 1.46), (2.22, 0.14, 1.02), "wood")
    for i, x in enumerate((-1.02, 1.02)):
        B(p, "wall_end_%d" % i, (x, 0, 1.46), (0.14, 1.38, 1.02), "wood_dark")
    gable_roof(p, "fisher", 1.14, 0.88, 2.06, "wood_dark")
    for i, x in enumerate((-0.66, 0.0, 0.66)):
        B(p, "shingle_%d" % i, (x, 0.02, 2.46), (0.56, 1.82, 0.08), "stone_dark")
    B(p, "door", (0, -0.74, 1.34), (0.58, 0.08, 0.88), "wood_dark")
    B(p, "window", (-0.66, -0.74, 1.50), (0.28, 0.06, 0.32), "ice")
    for i, z in enumerate((0.44, 1.12)):
        R(p, "net_knot_%d" % i, (0.72, -1.02, z), 0.10, 0.06, "bone", 6,
          (math.pi / 2, 0, 0))
    B(p, "lantern", (-1.06, -0.92, 1.06), (0.22, 0.22, 0.30), "gold")
    K(p, "lantern_flame", (-1.06, -0.92, 0.86), 0.08, 0.0, 0.16, "fire", 5)
    return p


def build_gatehouse():
    p = collection("gatehouse")
    B(p, "base", (0, 0, 0.12), (3.10, 1.20, 0.24), "stone_dark")
    for i, x in enumerate((-1.14, 1.14)):
        B(p, "tower_%d" % i, (x, 0, 1.40), (0.96, 1.04, 2.56), "stone")
        for j, z in enumerate((2.72, 2.94)):
            B(p, "battlement_%d_%d" % (i, j), (x, -0.36 + j * 0.72, z),
              (0.94, 0.28, 0.30), "stone_light")
    B(p, "lintel", (0, 0, 2.40), (1.44, 1.06, 0.40), "stone_dark")
    B(p, "gate_arch", (0, -0.60, 1.30), (1.32, 0.14, 0.30), "stone_light")
    for i in range(5):
        x = -0.48 + i * 0.24
        B(p, "portcullis_%d" % i, (x, -0.56, 0.92),
          (0.07, 0.08, 1.80), "iron")
        K(p, "spike_%d" % i, (x, -0.56, 0.06), 0.07, 0.0, 0.18, "steel", 5,
          (math.pi, 0, 0))
    for i, z in enumerate((0.52, 1.72)):
        B(p, "gate_band_%d" % i, (0, -0.64, z), (1.42, 0.08, 0.10), "iron")
    B(p, "bridge", (0, -1.06, 0.10), (1.60, 0.90, 0.16), "wood")
    for i, x in enumerate((-0.60, 0.60)):
        B(p, "chain_%d" % i, (x, -0.66, 2.16), (0.06, 0.06, 0.40), "iron")
    B(p, "banner", (1.14, -0.56, 2.12), (0.06, 0.10, 0.90), "enemy")
    return p


def build_observatory():
    p = collection("observatory")
    B(p, "step", (0, -0.86, 0.08), (1.60, 0.50, 0.16), "stone_light")
    K(p, "tower", (0, 0, 0.92), 0.68, 0.58, 1.68, "stone", 8)
    R(p, "band", (0, 0, 1.68), 0.62, 0.14, "stone_dark", 8)
    O(p, "dome", (0, 0, 2.04), 0.72, "steel", (1.0, 1.0, 0.72))
    B(p, "slit", (0, -0.70, 2.16), (0.14, 0.08, 0.62), "char",
      (0.18, 0, 0))
    B(p, "telescope_mount", (0, 0, 1.90), (0.16, 0.16, 0.36), "gold")
    R(p, "telescope", (0.18, -0.20, 2.16), 0.11, 0.92, "gold", 8,
      (0.42, 0.0, 0.34))
    B(p, "door", (0, -0.62, 0.62), (0.46, 0.08, 0.86), "wood_dark")
    for i, x in enumerate((-0.48, 0.48)):
        B(p, "window_%d" % i, (x, -0.56, 1.16), (0.18, 0.06, 0.30), "ice")
    for i, z in enumerate((0.28, 0.56, 0.84)):
        B(p, "stair_%d" % i, (0, -0.98 + i * 0.20, z),
          (0.90, 0.20, 0.14), "stone_dark")
    return p


BUILDINGS = [
    ("granary", build_granary, (2.6, 2.0, 2.7), "building silhouette",
     "raised timber granary with gable roof and ladder"),
    ("chapel", build_chapel, (2.1, 1.7, 3.2), "building silhouette",
     "stone chapel with arched door, windows and belfry"),
    ("fisher_hut", build_fisher_hut, (2.6, 1.9, 2.6), "building silhouette",
     "stilted hut on a pier with net and lantern"),
    ("gatehouse", build_gatehouse, (3.2, 1.5, 3.2), "building silhouette",
     "twin-tower gatehouse with portcullis and bridge"),
    ("observatory", build_observatory, (1.8, 1.8, 2.5), "building silhouette",
     "stone observatory with dome, telescope and stairs"),
]


# ===========================================================================
# E - MINOR polish: rebuilds of models the visual critics explicitly deferred
# None of these models is bound to a wave2/3/4 animation manifest.
# ===========================================================================
def build_bat_polish():
    p = collection("bat")
    O(p, "body", (0, 0, 0.72), 0.24, "wood", (1.0, 0.85, 1.15))
    O(p, "head", (0, -0.16, 0.92), 0.20, "sand", (1.05, 0.95, 0.95))
    for i, x in enumerate((-0.11, 0.11)):
        O(p, "eye_%d" % i, (x, -0.33, 0.96), 0.055, "gold")
        K(p, "ear_%d" % i, (x * 1.5, -0.10, 1.16), 0.09, 0.0, 0.26,
          "wood_dark", 4)
        K(p, "fang_%d" % i, (x * 0.5, -0.33, 0.82), 0.035, 0.0, 0.12,
          "bone", 4)
    for i, s in enumerate((-1, 1)):
        W(p, "wing_%d" % i, (s * 0.52, 0.06, 0.84), 0.74, 0.60, 0.52,
          "stone_light", (0, s * 0.30, 0))
        B(p, "wing_rib_%d" % i, (s * 0.48, 0.14, 0.86),
          (0.70, 0.05, 0.06), "wood_dark", (0, s * 0.20, 0))
        K(p, "foot_%d" % i, (s * 0.14, 0, 0.24), 0.07, 0.0, 0.30,
          "wood_dark", 5)
    return p


def build_fire_elemental_polish():
    p = collection("fire_elemental")
    K(p, "core", (0, 0, 0.96), 0.34, 0.26, 1.48, "fire", 8)
    O(p, "heart", (0, -0.06, 1.02), 0.22, "gold", (1.0, 0.9, 1.0))
    for i, x in enumerate((-0.09, 0.09)):
        O(p, "eye_%d" % i, (x, -0.32, 1.22), 0.045, "char")
    for i, s in enumerate((-1, 1)):
        B(p, "arm_%d" % i, (s * 0.46, 0, 1.06), (0.50, 0.18, 0.18), "fire",
          (0, 0, s * 0.32))
        O(p, "fist_%d" % i, (s * 0.72, -0.02, 0.90), 0.16, "fire")
        K(p, "leg_%d" % i, (s * 0.20, 0, 0.30), 0.17, 0.10, 0.60, "gold", 6)
        O(p, "foot_%d" % i, (s * 0.20, -0.08, 0.08), 0.16, "fire",
          (1.0, 1.2, 0.5))
    # Tongues are seated on the core shoulders instead of floating.
    for i, (x, y, h) in enumerate(((-0.18, -0.06, 0.46), (0.0, 0.08, 0.62),
                                    (0.18, -0.04, 0.40))):
        K(p, "flame_%d" % i, (x, y, 1.74 + h / 2), 0.15, 0.0, h, "fire", 5)
    return p


def build_stone_elemental_polish():
    p = collection("stone_elemental")
    # Tall, dark and crowned: deliberately unlike the broad grey golem.
    K(p, "torso", (0, 0, 1.14), 0.28, 0.34, 1.34, "obsidian", 6)
    O(p, "head", (0, -0.04, 1.94), 0.22, "char", (0.9, 0.85, 1.1))
    for i, x in enumerate((-0.09, 0.09)):
        O(p, "eye_%d" % i, (x, -0.22, 1.98), 0.045, "arcane")
    for i, s in enumerate((-1, 1)):
        B(p, "arm_%d" % i, (s * 0.40, 0, 1.18), (0.16, 0.16, 0.82),
          "obsidian", (0, 0, s * 0.12))
        O(p, "fist_%d" % i, (s * 0.48, -0.02, 0.74), 0.14, "char")
        K(p, "leg_%d" % i, (s * 0.16, 0, 0.36), 0.15, 0.11, 0.72,
          "obsidian", 6)
    for i, (x, a) in enumerate(((-0.20, -0.42), (0.0, 0.0), (0.20, 0.42))):
        K(p, "crown_%d" % i, (x, 0, 2.26), 0.09, 0.0, 0.42, "arcane", 5,
          (0, 0, a))
    for i, x in enumerate((-0.22, 0.22)):
        B(p, "band_%d" % i, (x, -0.20, 1.30), (0.08, 0.06, 0.46), "gold")
    return p


def build_crossbow_polish():
    p = collection("crossbow")
    B(p, "stock", (0, 0, 0.66), (0.14, 0.16, 0.96), "wood")
    B(p, "grip", (0, -0.04, 0.30), (0.12, 0.14, 0.32), "leather",
      (0.18, 0, 0))
    B(p, "bow_limbs", (0, 0.06, 1.06), (0.98, 0.10, 0.10), "steel",
      (0, 0.10, 0))
    for i, x in enumerate((-0.48, 0.48)):
        K(p, "limb_tip_%d" % i, (x, 0.14, 1.10), 0.06, 0.0, 0.16,
          "gold", 5)
    B(p, "string", (0, 0.14, 1.08), (0.94, 0.025, 0.025), "bone")
    B(p, "bolt", (0, 0.20, 0.94), (0.06, 0.72, 0.06), "wood",
      (math.pi / 2, 0, 0))
    K(p, "bolt_head", (0, 0.56, 0.94), 0.075, 0.0, 0.18, "steel", 5,
      (math.pi / 2, 0, 0))
    B(p, "sight", (0, -0.10, 0.96), (0.06, 0.06, 0.12), "gold")
    B(p, "trigger_guard", (0, -0.12, 0.52), (0.16, 0.06, 0.20), "iron")
    return p


def build_arrow_polish():
    p = collection("arrow")
    R(p, "shaft", (0, 0, 0.50), 0.045, 0.82, "wood", 6)
    R(p, "grip", (0, 0, 0.32), 0.055, 0.18, "leather", 6)
    R(p, "collar", (0, 0, 0.88), 0.065, 0.06, "gold", 6)
    K(p, "head", (0, 0, 0.98), 0.11, 0.015, 0.22, "steel", 5)
    for i, (a, key) in enumerate(((0.0, "gold"), (2.09, "gold"),
                                   (4.19, "sand"))):
        B(p, "flight_%d" % i, (0.10 * math.cos(a), 0.10 * math.sin(a), 0.16),
          (0.05, 0.20, 0.28), key, (0, 0, a))
    return p


def build_coral_brain_polish():
    p = collection("coral_brain")
    O(p, "body", (0, 0, 0.30), 0.42, "arcane", (1.25, 1.05, 0.72))
    for i, (x, y, r) in enumerate(((-0.26, 0.02, 0.18),
                                     (0.22, 0.14, 0.16),
                                     (0.06, -0.22, 0.15))):
        O(p, "lobe_%d" % i, (x, y, 0.44), r, "arcane_light")
    for i, (x, y, a) in enumerate(((-0.16, -0.12, 0.3), (0.12, 0.10, -0.4),
                                    (0.26, -0.14, 0.7))):
        B(p, "ridge_%d" % i, (x, y, 0.55), (0.34, 0.07, 0.06), "fire",
          (0, 0, a))
        B(p, "ridge_bone_%d" % i, (x, y, 0.59), (0.24, 0.05, 0.04), "bone",
          (0, 0, a))
    return p


def build_apple_tree_polish():
    p = collection("apple_tree")
    K(p, "trunk", (0, 0, 0.76), 0.22, 0.16, 1.42, "wood", 8)
    for i, (x, y, a) in enumerate(((-0.42, 0.14, 0.4), (0.38, -0.10, -0.5))):
        B(p, "bough_%d" % i, (x, y, 1.16), (0.68, 0.14, 0.14), "wood",
          (0, 0, a))
    for i, (x, y, z, r) in enumerate(((-0.48, 0.06, 2.04, 0.48),
                                       (0.32, 0.20, 2.16, 0.52),
                                       (-0.04, -0.34, 2.26, 0.46))):
        O(p, "crown_%d" % i, (x, y, z), r, "leaf", (1.2, 1.05, 0.85))
    for i, (x, y, z) in enumerate(((-0.62, -0.10, 1.94),
                                     (0.46, 0.02, 2.02),
                                     (0.06, -0.48, 2.10),
                                     (-0.18, 0.42, 2.08))):
        O(p, "apple_%d" % i, (x, y, z), 0.11, "enemy")
        B(p, "apple_leaf_%d" % i, (x + 0.08, y, z + 0.12),
          (0.12, 0.06, 0.03), "leaf_light")
    return p


def build_hay_bale_polish():
    p = collection("hay_bale")
    O(p, "bale", (0, 0, 0.38), 0.44, "sand", (1.35, 1.0, 0.86))
    for i, x in enumerate((-0.24, 0.24)):
        B(p, "band_%d" % i, (x, 0, 0.38), (0.08, 0.88, 0.72), "leather")
    for i, (x, y, z) in enumerate(((-0.40, 0.22, 0.60), (0.28, -0.20, 0.64),
                                     (0.12, 0.34, 0.30), (-0.22, -0.30, 0.20))):
        B(p, "straw_%d" % i, (x, y, z), (0.24, 0.06, 0.05), "gold",
          (0, 0.2 * (i - 1.5), 0.3))
    O(p, "straw_tuft", (0.02, 0.02, 0.72), 0.16, "sand", (1.5, 1.0, 0.4))
    return p


def build_volcanic_bomb_polish():
    p = collection("volcanic_bomb")
    O(p, "core", (0, 0, 0.32), 0.40, "fire", (1.0, 1.0, 0.85))
    for i, (x, y, z, r) in enumerate(((-0.20, 0.10, 0.32, 0.22),
                                        (0.22, 0.08, 0.34, 0.20),
                                        (0.02, -0.22, 0.28, 0.21),
                                        (0.04, 0.06, 0.54, 0.18))):
        O(p, "crust_%d" % i, (x, y, z), r, "char", (1.0, 1.0, 0.8))
    for i, (x, y, a) in enumerate(((-0.10, 0.32, 0.2), (0.28, -0.14, -0.3),
                                    (-0.30, -0.10, 0.7))):
        B(p, "crack_%d" % i, (x, y, 0.56), (0.30, 0.09, 0.06), "fire",
          (0, 0, a))
        B(p, "crack_glow_%d" % i, (x, y, 0.61), (0.20, 0.06, 0.04), "gold",
          (0, 0, a))
    for i, (x, y) in enumerate(((-0.14, 0.30), (0.26, -0.10))):
        K(p, "ember_%d" % i, (x, y, 0.74), 0.11, 0.0, 0.24, "fire", 5)
    return p


POLISH = [
    ("bat", build_bat_polish, (0.9, 0.8, 1.2), "minor polish",
     "brighter wing/bone palette and readable eyes at 60 px"),
    ("fire_elemental", build_fire_elemental_polish, (1.6, 0.9, 2.0),
     "minor polish", "continuous flame body, seated arms and feet"),
    ("stone_elemental", build_stone_elemental_polish, (1.0, 0.8, 2.4),
     "minor polish", "tall obsidian crowned silhouette, unlike the golem"),
    ("crossbow", build_crossbow_polish, (1.0, 0.8, 1.1), "minor polish",
     "raised from 0.62u to a readable 1.05u with a visible bolt"),
    ("arrow", build_arrow_polish, (0.3, 0.3, 1.0), "minor polish",
     "broad gold/sand flights readable at 60 px"),
    ("coral_brain", build_coral_brain_polish, (1.1, 0.9, 0.7),
     "minor polish", "saturated arcane body with fire/bone ridges"),
    ("apple_tree", build_apple_tree_polish, (1.9, 1.6, 2.6),
     "minor polish", "four larger apples, boughs, 2.6u height"),
    ("hay_bale", build_hay_bale_polish, (1.2, 0.9, 0.8), "minor polish",
     "rounded elliptical bale with leather bands and straw tufts"),
    ("volcanic_bomb", build_volcanic_bomb_polish, (1.0, 0.9, 0.9),
     "minor polish", "wide fire/gold lava cracks and seated embers"),
]

ASSETS = INTERIORS + LANDMARKS + NPCS + BUILDINGS + POLISH
# Literal list so scripts/verify_models.py learns the new generator category.
EXPORTS = [
    "library_interior", "guard_post_interior", "market_stall_interior",
    "desert_obelisk_landmark", "tundra_totem_landmark",
    "swamp_shrine_landmark", "volcanic_forge_landmark",
    "ocean_lighthouse_landmark", "npc_alchemist", "npc_cartographer",
    "npc_chef", "npc_herbalist", "npc_jeweler", "npc_shepherd",
    "npc_physician", "granary", "chapel", "fisher_hut", "gatehouse",
    "observatory", "bat", "fire_elemental", "stone_elemental", "crossbow",
    "arrow", "coral_brain", "apple_tree", "hay_bale", "volcanic_bomb",
]

for name, builder, target, family, notes in ASSETS:
    col = builder()
    rec = f(name, col, target, family, notes)
    budget = 1500 if "interior" in name else 500
    if rec["triangles"] > budget:
        print("BUDGET WARNING %s: %d > %d" %
              (name, rec["triangles"], budget))

write_records("models_v6", RECORDS)
print("WAVE6: %d assets (20 new + 9 minor polish)" % len(RECORDS))
try:
    contact_sheet(SETS[:20], "models_v6.png", width=3200, height=720)
except Exception as exc:  # previews are optional
    print("PREVIEW SKIPPED", exc)
