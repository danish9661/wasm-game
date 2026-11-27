"""Build prop_vines: one climbing twisted strand with broad leaves on a support.

Run: gam_cli.py all assets/src/props/prop_vines/brief.json
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
    "vine": gam.material("m_leaf", "#3A7A33", roughness=0.9),
    "light": gam.material("m_leaf_light", "#55953F", roughness=0.9),
    "wood": gam.material("m_wood", "#6B4423", roughness=0.9),
}


def support(bm):
    N.faceted_stump(bm, (0, 0, 0.0), 0.13, 0.52, mat=0, n=6, seed=4)


def strand(bm, i):
    z0 = 0.05 + i * 0.26
    a0 = i * 1.1
    a1 = a0 + 1.1
    r = 0.17
    N.branch(bm, (math.cos(a0) * r, math.sin(a0) * r, z0),
             (math.cos(a1) * r, math.sin(a1) * r, z0 + 0.28),
             0.038, 0.030, mat=0, n=5)


def leaf(bm, i):
    z = 0.20 + 0.12 * i
    a = i * math.tau / 6.0 + 0.3
    x, y = math.cos(a) * 0.17, math.sin(a) * 0.17
    N.tapered_blade(bm, (x, y, z),
                    (x + math.cos(a) * 0.28, y + math.sin(a) * 0.26, z + 0.10),
                    0.075, mat=0, n=3)


def build_form():
    parts = [F.part("support", [M["wood"]], support)]
    for i in range(3):
        parts.append(F.part("strand_%d" % i, [M["vine"]],
                            lambda bm, i=i: strand(bm, i)))
    for i in range(6):
        parts.append(F.part("leaf_%d" % i,
                            [M["vine"] if i % 2 else M["light"]],
                            lambda bm, i=i: leaf(bm, i)))
    return parts


N.finish(ctx, build_form(), name="vines")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
