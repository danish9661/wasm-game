"""Rebuild market_umbrella: eight saturated alternating panels, scalloped edge
and a weighted stone foot. Base z=0, height 2.25 u, footprint 1.5 u.
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import gam
import math
from mathutils import Vector
import _family_hero as F

ctx = gam.Ctx.from_argv()
gam.reset()
gam.load_material_lib(ctx)

M = {
    "pole": gam.material("m_umbrella_pole", "#592F1A", roughness=0.9),
    "fire": gam.material("m_umbrella_fire", "#FF6A2A", roughness=0.82),
    "red": gam.material("m_umbrella_red", "#C0392B", roughness=0.82),
    "stone": gam.material("m_umbrella_foot", "#B8C0C8", roughness=0.92),
    "gold": gam.material("m_umbrella_finial", "#D9A62E", roughness=0.55, metallic=0.6),
}


def foot(bm):
    gam.bm_cylinder(bm, 0.32, 0.0, 0.09, n=8, mat=0)
    gam.bm_cylinder(bm, 0.24, 0.09, 0.17, n=8, mat=0)
    gam.bm_cylinder(bm, 0.12, 0.17, 0.22, n=8, mat=1)


def pole(bm):
    gam.bm_cylinder(bm, 0.045, 0.20, 2.20, n=8, r1=0.026, mat=0)
    gam.bm_cylinder(bm, 0.055, 1.38, 1.45, n=8, mat=1)
    F.ico(bm, (0, 0, 2.25), 0.055, mat=1, scale=(1.0, 1.0, 1.3))


def canopy(bm):
    n = 8
    apex_z, rim_z = 2.16, 1.47
    inner_r, outer_r = 0.055, 0.74
    apex = [bm.verts.new(Vector((inner_r * math.cos(2 * math.pi * k / n),
                                 inner_r * math.sin(2 * math.pi * k / n), apex_z))) for k in range(n)]
    rim = [bm.verts.new(Vector((outer_r * math.cos(2 * math.pi * k / n),
                                outer_r * math.sin(2 * math.pi * k / n),
                                rim_z + (0.035 if k % 2 == 0 else 0.0)))) for k in range(n)]
    for k in range(n):
        j = (k + 1) % n
        mat = 0 if k % 2 == 0 else 1
        top = [apex[k], apex[j], rim[j], rim[k]]
        bot = [bm.verts.new((p.co.x, p.co.y, p.co.z - 0.025)) for p in top]
        uvs = [(k * 0.12, 0.0), ((k + 1) * 0.12, 0.0),
               ((k + 1) * 0.12, 0.75), (k * 0.12, 0.75)]
        gam._face(bm, top, (0, 0, 1), uvs, mat)
        gam._face(bm, list(reversed(bot)), (0, 0, -1), uvs, mat)
        gam._face(bm, [top[2], top[3], bot[3], bot[2]], (0, 1, 0), uvs, mat)
        # hanging scallop on the panel edge
        if k % 2 == 0:
            mid = (rim[k].co + rim[j].co) * 0.5
            gam._face(bm, [rim[k], rim[j], bm.verts.new((mid.x, mid.y, rim_z - 0.085))],
                      (0, 0, -1), uvs, mat)


parts = [
    F.part("foot", [M["stone"], M["gold"]], foot, bevel=0.008, segments=1),
    F.part("pole", [M["pole"], M["gold"]], pole),
    F.part("canopy", [M["fire"], M["red"]], canopy),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=30.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
