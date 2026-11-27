"""Build prop_tree: a broadleaf diorama tree - thick tapered trunk, inset
boughs and three distinct faceted canopy lobes (one lighter). Base at z=0.
Run: gam_cli.py build assets/src/props/prop_tree/brief.json
"""
import math
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import bmesh
import bpy
import gam
from mathutils import Matrix, Vector
import _family_hero as F

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "wood": gam.material("m_wood", "#6B4423", roughness=0.9),
    "leaf": gam.material("m_leaf", "#3A7A33", roughness=0.9),
    "leaf_light": gam.material("m_leaf_light", "#548F42", roughness=0.9),
}


def trunk(bm):
    F.loft(bm, [F.ring(0, 0, 0.0, 0.165, 0.155),
                F.ring(0, 0, 0.18, 0.140, 0.130),
                F.ring(0, 0, 1.55, 0.118, 0.110),
                F.ring(0, 0, 2.05, 0.085, 0.080)], mat=0)


def branch(bm, start, end, r0, r1):
    a, b = Vector(start), Vector(end)
    axis = (b - a).normalized()
    rot = Vector((0, 0, 1)).rotation_difference(axis).to_matrix().to_4x4()
    Mtx = Matrix.Translation(a) @ rot
    gam.bm_cylinder(bm, r0, 0.0, (b - a).length, n=6, r1=r1, matrix=Mtx)


def branches(bm):
    # tips tucked well inside the canopy so no bare fork shows at eye level
    branch(bm, (0, 0, 1.72), (-0.32, 0.06, 2.26), 0.052, 0.018)
    branch(bm, (0, 0, 1.82), (0.34, -0.10, 2.32), 0.050, 0.018)
    branch(bm, (0, 0, 1.95), (0.02, 0.24, 2.48), 0.044, 0.016)


def canopy_main(bm):
    F.loft(bm, [F.ring(0, 0, 1.88, 0.36, 0.34),
                F.ring(0, 0, 2.34, 0.46, 0.44),
                F.ring(0, 0, 2.72, 0.37, 0.35),
                F.ring(0, 0, 2.97, 0.09, 0.09)], mat=0)


def canopy_lobe(bm, cx, cy, z, rx, ry, h):
    F.loft(bm, [F.ring(cx, cy, z - 0.32, rx * 0.52, ry * 0.52, n=6),
                F.ring(cx, cy, z, rx, ry, n=6),
                F.ring(cx, cy, z + h, rx * 0.55, ry * 0.55, n=6)], mat=0)


def build_form():
    if ctx.placeholder:
        d = B["dims"]
        ob = gam.box(ctx.id + "_lod0", (d["x"], d["y"], d["z"]), base_z=0)
        ob.data.materials.append(M["wood"])
        gam.uv_world_box(ob, meters_per_tile=1.0)
        return ob, None
    parts = [
        F.part("trunk", [M["wood"]], trunk, bevel=0.004, segments=1),
        F.part("branches", [M["wood"]], branches),
        F.part("canopy_main", [M["leaf"]], canopy_main),
        F.part("canopy_lobe_light", [M["leaf_light"]],
               lambda bm: canopy_lobe(bm, -0.38, 0.10, 2.18, 0.28, 0.27, 0.38)),
        F.part("canopy_lobe_r", [M["leaf"]],
               lambda bm: canopy_lobe(bm, 0.39, -0.13, 2.32, 0.28, 0.27, 0.32)),
    ]
    obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01)
    # Trunk-sized collider: do not block the whole 1.3 m canopy footprint.
    col = gam.box(ctx.id + "_col_src", (0.34, 0.34, 1.95), base_z=0)
    return obj, col


obj, col_src = build_form()
root = gam.finish(ctx, obj, collider_src=col_src)
gam.export(ctx, root)
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
