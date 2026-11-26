"""Build wpn_sword: a faceted broadsword with a crisp ridge, chamfered guard and grip.

Author at the grip (pivot_point in the brief); front faces -Y.
Run: gam_cli.py build assets/src/props/wpn_sword/brief.json
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import bmesh
import bpy
import gam
from mathutils import Matrix
import _family_hero as F

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "steel": gam.material("m_steel_blade", "#C8D0D8", roughness=0.65, metallic=0.6),
    "gold": gam.material("m_gold", "#D9A62E", roughness=0.6, metallic=0.6),
    "leather": gam.material("m_leather_grip", "#592F1A", roughness=0.9),
}


def blade(bm):
    """Diamond cross-section loft: pronounced ridge on both sides, broadsword point."""
    def diamond(w, t, z):
        return [(-w, 0.0, z), (0.0, -t, z), (w, 0.0, z), (0.0, t, z)]
    F.loft(bm, [diamond(0.038, 0.014, 0.035),
                diamond(0.040, 0.013, 0.10),
                diamond(0.034, 0.011, 0.88),
                diamond(0.026, 0.008, 1.04),
                diamond(0.012, 0.004, 1.10)], mat=0)


def guard(bm):
    F.oriented_box(bm, (-0.034, -0.040, 0.0), (0.034, 0.040, 0.045))
    # chunky, near-square quillons: start inside the block so bevels weld cleanly
    for side in (-1, 1):
        x0, x1 = 0.020 * side, 0.125 * side
        r0 = [(x0, -0.038, 0.004), (x0, 0.038, 0.004),
              (x0, 0.038, 0.042), (x0, -0.038, 0.042)]
        r1 = [(x1, -0.028, 0.012), (x1, 0.028, 0.012),
              (x1, 0.028, 0.038), (x1, -0.028, 0.038)]
        F.loft(bm, [r0, r1], mat=0)


def grip(bm):
    """One continuous waisted grip with two raised wraps — no loose rings or tabs."""
    def r(z, rad):
        return F.ring(0.0, 0.0, z, rad, rad, n=8)
    F.loft(bm, [r(-0.13, 0.016), r(-0.113, 0.019), r(-0.100, 0.016),
                r(-0.062, 0.016), r(-0.049, 0.019), r(-0.036, 0.016),
                r(0.005, 0.018)], mat=0)


def pommel(bm):
    F.ico(bm, (0.0, 0.0, -0.150), 0.028, scale=(1.0, 0.75, 1.0))


def build_form():
    if ctx.placeholder:
        d = B["dims"]
        ob = gam.box(ctx.id + "_lod0", (d["x"], d["y"], d["z"]), base_z=0)
        ob.data.materials.append(M["steel"])
        gam.uv_world_box(ob, meters_per_tile=1.0)
        return ob
    parts = [
        F.part("blade", [M["steel"]], blade),
        F.part("guard", [M["gold"]], guard, bevel=0.002, segments=1),
        F.part("grip", [M["leather"]], grip),
        F.part("pommel", [M["gold"]], pommel),
    ]
    # Sharp-by-angle at 20deg keeps the ridge facets separate; bevels stay weighted.
    return F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=20.0)


obj = build_form()
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
