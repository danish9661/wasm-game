"""Build prop_dead_tree: a forked bare snag with root flare and twigs.

Run: gam_cli.py all assets/src/props/prop_dead_tree/brief.json
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
    "light": gam.material("m_wood_light", "#7C5633", roughness=0.9),
}

PRIMARIES = [
    ((-0.55, 0.10, 2.42), 1.62),
    ((0.52, -0.12, 2.18), 1.88),
    ((0.04, 0.48, 2.56), 2.10),
]


def trunk(bm):
    F.loft(bm, [F.ring(0, 0, 0.0, 0.19, 0.17, n=6),
                F.ring(0, 0, 0.28, 0.155, 0.145, n=6),
                F.ring(0, 0, 1.52, 0.125, 0.115, n=6),
                F.ring(0, 0, 2.16, 0.098, 0.092, n=6),
                F.ring(0, 0, 2.60, 0.055, 0.055, n=6)], mat=0)


def root(bm, i):
    a = i * math.tau / 3.0 + 0.35
    N.branch(bm, (0, 0, 0.30), (math.cos(a) * 0.52, math.sin(a) * 0.46, 0.015),
             0.095, 0.028, mat=0, n=5)


def primary(bm, i):
    end, z = PRIMARIES[i]
    N.branch(bm, (0, 0, z), end, 0.072, 0.022, mat=0, n=5)


def twig(bm, i, side):
    end, z = PRIMARIES[i]
    sx = -1 if side == 0 else 1
    start = (end[0] * 0.62, end[1] * 0.62, z + (end[2] - z) * 0.62)
    target = (end[0] + sx * 0.20, end[1] + sx * 0.10,
              end[2] + 0.22 if side else end[2] - 0.10)
    N.branch(bm, start, target, 0.026, 0.008, mat=0, n=4)


def build_form():
    parts = [F.part("trunk", [M["wood"]], trunk, bevel=0.006, segments=1)]
    for i in range(3):
        parts.append(F.part("root_%d" % i, [M["dark"]],
                            lambda bm, i=i: root(bm, i)))
    for i in range(3):
        parts.append(F.part("branch_%d" % i, [M["light"]],
                            lambda bm, i=i: primary(bm, i)))
    for i in range(3):
        for s in range(2):
            parts.append(F.part("twig_%d_%d" % (i, s), [M["wood"]],
                                lambda bm, i=i, s=s: twig(bm, i, s)))
    return parts


N.finish(ctx, build_form(), name="dead_tree")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
