"""Build prop_cactus: ribbed tapered column, rounded arms, spine nubs.

Run: gam_cli.py all assets/src/props/prop_cactus/brief.json
"""
import math
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import bpy
import gam
import _family_hero as F
import _family_nature as N

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "body": gam.material("m_leaf", "#3A7A33", roughness=0.9),
    "dark": gam.material("m_leaf_dark", "#2C6327", roughness=0.9),
    "light": gam.material("m_leaf_light", "#55953F", roughness=0.9),
    "spine": gam.material("m_sand", "#D9B46A", roughness=0.9),
}


def column(bm):
    F.loft(bm, [F.ring(0, 0, 0.0, 0.20, 0.18, n=8),
                F.ring(0, 0, 0.18, 0.215, 0.19, n=8),
                F.ring(0, 0, 0.95, 0.20, 0.18, n=8),
                F.ring(0, 0, 1.20, 0.15, 0.14, n=8)], mat=0)


def rib(bm, i):
    a = i * math.tau / 5.0
    x, y = math.cos(a) * 0.205, math.sin(a) * 0.185
    N.tapered_blade(bm, (x, y, 0.02), (x * 0.80, y * 0.80, 1.14),
                    0.016, mat=0, n=3)


def arm(bm, side):
    sx = -1 if side == 0 else 1
    z = 0.68 if side == 0 else 0.86
    N.branch(bm, (sx * 0.10, 0, z), (sx * 0.37, 0, z + 0.02),
             0.11, 0.095, mat=0, n=6)
    N.branch(bm, (sx * 0.37, 0, z + 0.02), (sx * 0.40, 0, z + 0.34),
             0.095, 0.065, mat=0, n=6)


def spine(bm, i):
    a = i * math.tau / 8.0 + 0.2
    z = 0.20 + 0.11 * (i % 8)
    x, y = math.cos(a) * 0.215, math.sin(a) * 0.19
    N.tapered_blade(bm, (x, y, z),
                    (x + math.cos(a) * 0.09, y + math.sin(a) * 0.08, z + 0.04),
                    0.016, mat=0, n=3)


def build_form():
    parts = [F.part("column", [M["body"]], column, bevel=0.008, segments=1)]
    for i in range(5):
        parts.append(F.part("rib_%d" % i, [M["dark"] if i % 2 else M["light"]],
                            lambda bm, i=i: rib(bm, i)))
    for s in range(2):
        parts.append(F.part("arm_%d" % s, [M["body"]],
                            lambda bm, s=s: arm(bm, s), bevel=0.006, segments=1))
    for i in range(2):
        parts.append(F.part("spine_%d" % i, [M["spine"]],
                            lambda bm, i=i: spine(bm, i)))
    return parts


N.finish(ctx, build_form(), name="cactus")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
