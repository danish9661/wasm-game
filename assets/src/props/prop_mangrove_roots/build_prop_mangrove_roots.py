"""Build prop_mangrove_roots: arched prop roots, wood stump, swamp crown.

Run: gam_cli.py all assets/src/props/prop_mangrove_roots/brief.json
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
    "wood": gam.material("m_wood", "#6B4423", roughness=0.9),
    "dark": gam.material("m_wood_dark", "#592F1A", roughness=0.9),
    "swamp": gam.material("m_swamp", "#4A6741", roughness=0.9),
    "leaf": gam.material("m_leaf", "#3A7A33", roughness=0.9),
    "leaf_light": gam.material("m_leaf_light", "#55953F", roughness=0.9),
}


def stump(bm):
    N.faceted_stump(bm, (0, 0, 0.10), 0.27, 1.20, mat=0, n=8, seed=3)


def root(bm, i):
    a = i * math.tau / 5.0 + 0.3
    c, s = math.cos(a), math.sin(a)
    N.branch(bm, (c * 0.20, s * 0.20, 1.12), (c * 0.50, s * 0.48, 0.60),
             0.075, 0.055, mat=0, n=5)
    N.branch(bm, (c * 0.50, s * 0.48, 0.60), (c * 0.74, s * 0.68, 0.015),
             0.055, 0.025, mat=0, n=5)


def foliage(bm, i):
    specs = [(-0.42, 0.05, 1.32, 0.46, 0.42, 0.52, 2),
             (0.43, -0.08, 1.30, 0.45, 0.41, 0.50, 5),
             (0.02, 0.30, 1.42, 0.40, 0.38, 0.52, 8)]
    x, y, z, rx, ry, h, seed = specs[i]
    N.leaf_lobe(bm, x, y, z, rx, ry, h, mat=0, n=6, seed=seed)


def pneumatophore(bm, i):
    a = i * math.tau / 3.0 + 0.9
    x, y = math.cos(a) * 0.42, math.sin(a) * 0.38
    N.branch(bm, (x, y, 0.01), (x + 0.03, y, 0.30 + 0.08 * (i % 2)),
             0.055, 0.035, mat=0, n=5)


def build_form():
    parts = [F.part("stump", [M["dark"]], stump, bevel=0.008, segments=1)]
    for i in range(5):
        parts.append(F.part("root_%d" % i, [M["wood"]],
                            lambda bm, i=i: root(bm, i)))
    for i in range(3):
        parts.append(F.part("foliage_%d" % i,
                            [M["leaf"] if i != 1 else M["leaf_light"]],
                            lambda bm, i=i: foliage(bm, i)))
    for i in range(3):
        parts.append(F.part("pneumatophore_%d" % i, [M["swamp"]],
                            lambda bm, i=i: pneumatophore(bm, i)))
    return parts


N.finish(ctx, build_form(), name="mangrove_roots")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
