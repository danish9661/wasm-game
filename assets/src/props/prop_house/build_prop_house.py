"""Build prop_house: a timber-framed cottage with pitched, shingled roof,
gables, chimney, door and framed windows. Front faces -Y, base at z=0.
Run: gam_cli.py build assets/src/props/prop_house/brief.json
"""
import math
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import bmesh
import bpy
import gam
import _family_hero as F

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "stone": gam.material("m_stone", "#7A7F87", roughness=0.9),
    "plaster": gam.material("m_plaster", "#D9D2C6", roughness=0.9),
    "wood": gam.material("m_wood", "#6B4423", roughness=0.9),
    "roof": gam.material("m_roof_slate", "#666E78", roughness=0.9),
    "glass": gam.material("m_glass", "#3A7CA5", roughness=0.8),
    "gold": gam.material("m_gold", "#D9A62E", roughness=0.6, metallic=0.6),
}

SLOPE = math.atan2(1.18, 1.15)
RIDGE_Z = 3.01
EAVE_Z = 1.83
SLOPE_CENTER_Z = (RIDGE_Z + EAVE_Z) / 2


def plinth(bm):
    gam.bm_box(bm, (-1.10, -0.90, 0.0), (1.10, 0.90, 0.16))


def walls(bm):
    gam.bm_box(bm, (-1.04, -0.84, 0.16), (1.04, 0.84, 1.86), skip=("bottom",))


def timbers(bm):
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = sx * 1.01, sy * 0.81
            gam.bm_box(bm, (x - 0.055, y - 0.055, 0.16), (x + 0.055, y + 0.055, 1.86))
    # horizontal band under the eaves
    gam.bm_box(bm, (-1.06, -0.86, 1.76), (1.06, 0.86, 1.86))


def gable(bm, y):
    tri = [(-1.04, y, 1.85), (1.04, y, 1.85), (0.0, y, RIDGE_Z - 0.02)]
    gam.bm_prism(bm, tri, (0.0, 0.06 if y < 0 else -0.06, 0.0))


def roof(bm):
    for side in (-1, 1):
        cx = 0.575 * side
        F.oriented_box(bm, (cx - 0.823, -0.95, SLOPE_CENTER_Z - 0.020),
                       (cx + 0.823, 0.95, SLOPE_CENTER_Z + 0.020),
                       rot=(0.0, SLOPE * side, 0.0), center=(cx, 0.0, SLOPE_CENTER_Z))
    gam.bm_box(bm, (-0.085, -0.95, RIDGE_Z - 0.04), (0.085, 0.95, RIDGE_Z + 0.03))
    # fascia bands along both eaves read at CCTV distance
    for sy in (-1, 1):
        gam.bm_box(bm, (-1.18, sy * 0.95 - 0.035, EAVE_Z - 0.08),
                   (1.18, sy * 0.95 + 0.035, EAVE_Z))


def shingles(bm):
    """Staggered tiles, kept inside the roof plane (no staircase overhang)."""
    for side in (-1, 1):
        for course, t in enumerate((-0.45, -0.15, 0.15, 0.45)):
            x = 0.575 * side + t * math.cos(SLOPE) * side
            z = SLOPE_CENTER_Z - t * math.sin(SLOPE) + 0.018
            for k in range(4):
                yc = -0.51 + k * 0.34 + (0.17 if course % 2 else 0.0)
                if abs(yc) + 0.155 > 0.90:
                    continue
                F.oriented_box(bm, (x - 0.105, yc - 0.155, z - 0.009),
                               (x + 0.105, yc + 0.155, z + 0.009),
                               rot=(0.0, SLOPE * side, 0.0), center=(x, yc, z))
        # continuous rake boards mask the tile ends along both sloped edges
        for sy in (-1, 1):
            cx = 0.575 * side
            F.oriented_box(bm, (cx - 0.823, sy * 0.95 - 0.03, SLOPE_CENTER_Z - 0.028),
                           (cx + 0.823, sy * 0.95 + 0.03, SLOPE_CENTER_Z + 0.028),
                           rot=(0.0, SLOPE * side, 0.0), center=(cx, sy * 0.95, SLOPE_CENTER_Z))


def chimney(bm):
    gam.bm_box(bm, (0.52, 0.18, 2.30), (0.82, 0.48, 3.06))
    gam.bm_box(bm, (0.48, 0.14, 3.00), (0.86, 0.52, 3.10))


def door(bm):
    gam.bm_box(bm, (-0.28, -0.86, 0.16), (0.28, -0.80, 1.22))
    # frame
    gam.bm_box(bm, (-0.33, -0.89, 0.16), (-0.28, -0.82, 1.27))
    gam.bm_box(bm, (0.28, -0.89, 0.16), (0.33, -0.82, 1.27))
    gam.bm_box(bm, (-0.33, -0.89, 1.22), (0.33, -0.82, 1.27))
    # steps
    gam.bm_box(bm, (-0.34, -1.02, 0.0), (0.34, -0.86, 0.08))
    gam.bm_box(bm, (-0.40, -1.14, 0.0), (0.40, -1.02, 0.035))


def window_glass(bm, cx):
    gam.bm_box(bm, (cx - 0.19, -0.865, 0.93), (cx + 0.19, -0.845, 1.37))


def window_frame(bm, cx):
    """Timber surround, sill and mullion - the glass itself is a flat pane."""
    gam.bm_box(bm, (cx - 0.22, -0.90, 0.90), (cx - 0.17, -0.82, 1.41))
    gam.bm_box(bm, (cx + 0.17, -0.90, 0.90), (cx + 0.22, -0.82, 1.41))
    gam.bm_box(bm, (cx - 0.22, -0.90, 1.36), (cx + 0.22, -0.82, 1.41))
    gam.bm_box(bm, (cx - 0.22, -0.90, 0.90), (cx + 0.22, -0.82, 0.95))
    gam.bm_box(bm, (cx - 0.015, -0.89, 0.94), (cx + 0.015, -0.83, 1.37))
    # projecting sill
    gam.bm_box(bm, (cx - 0.24, -0.96, 0.87), (cx + 0.24, -0.82, 0.92))


def handle(bm):
    F.ico(bm, (0.17, -0.91, 0.72), 0.035, scale=(1.0, 0.7, 1.0))


def build_form():
    if ctx.placeholder:
        d = B["dims"]
        ob = gam.box(ctx.id + "_lod0", (d["x"], d["y"], d["z"]), base_z=0)
        ob.data.materials.append(M["stone"])
        gam.uv_world_box(ob, meters_per_tile=1.0)
        return ob
    parts = [
        F.part("plinth", [M["stone"]], plinth, bevel=0.006, segments=1),
        F.part("walls", [M["plaster"]], walls),
        F.part("timbers", [M["wood"]], timbers, bevel=0.004, segments=1),
        F.part("gable_front", [M["plaster"]], lambda bm: gable(bm, -0.84)),
        F.part("gable_back", [M["plaster"]], lambda bm: gable(bm, 0.84)),
        F.part("roof", [M["roof"]], roof, bevel=0.008, segments=1),
        F.part("shingles", [M["roof"]], shingles),
        F.part("chimney", [M["stone"]], chimney, bevel=0.008, segments=1),
        F.part("door", [M["wood"]], door),
        F.part("window_glass_l", [M["glass"]], lambda bm: window_glass(bm, -0.62)),
        F.part("window_glass_r", [M["glass"]], lambda bm: window_glass(bm, 0.62)),
        F.part("window_frame_l", [M["wood"]], lambda bm: window_frame(bm, -0.62)),
        F.part("window_frame_r", [M["wood"]], lambda bm: window_frame(bm, 0.62)),
        F.part("handle", [M["gold"]], handle),
    ]
    return F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01)


obj = build_form()
root = gam.finish(ctx, obj)
gam.export(ctx, root)
# Runtime GLB fix: the web 3D renderer walks every node and draws the box
# collider as an untextured white cube, hiding the detailed house. Keep the
# collider node, drop only its renderable mesh reference.
sys.path.insert(0, os.path.abspath(os.path.join(
    os.path.dirname(__file__), "..", "..", "..", "..", "scripts")))
from strip_collider_meshes import strip_file
strip_file(ctx.glb)
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
