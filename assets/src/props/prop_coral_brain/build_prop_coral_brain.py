"""Build prop_coral_brain: sand-coloured dome with winding raised ridges.

Run: gam_cli.py all assets/src/props/prop_coral_brain/brief.json
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
    "coral": gam.material("m_sand", "#D9B46A", roughness=0.9),
    "ridge": gam.material("m_sand_dark", "#B8944E", roughness=0.9),
    "groove": gam.material("m_swamp", "#4A6741", roughness=0.9),
}


def dome(bm):
    F.loft(bm, [F.ring(0, 0, 0.0, 0.46, 0.44, n=12),
                F.ring(0, 0, 0.26, 0.47, 0.45, n=12),
                F.ring(0, 0, 0.56, 0.35, 0.34, n=12),
                F.ring(0, 0, 0.82, 0.05, 0.05, n=12)], mat=0)


def ridge(bm, i):
    a = i * math.tau / 6.0 + 0.3
    x0, y0 = math.cos(a) * 0.40, math.sin(a) * 0.38
    x1, y1 = math.cos(a + 0.5) * 0.22, math.sin(a + 0.5) * 0.21
    N.tapered_blade(bm, (x0, y0, 0.10), (x1, y1, 0.72),
                    0.034, mat=0, n=3)


def groove(bm, i):
    a = i * math.tau / 4.0 + 0.8
    x0, y0 = math.cos(a) * 0.30, math.sin(a) * 0.28
    x1, y1 = math.cos(a - 0.4) * 0.14, math.sin(a - 0.4) * 0.13
    N.tapered_blade(bm, (x0, y0, 0.32), (x1, y1, 0.70),
                    0.016, mat=0, n=3)


def build_form():
    parts = [F.part("dome", [M["coral"]], dome, bevel=0.010, segments=1)]
    for i in range(6):
        parts.append(F.part("ridge_%d" % i, [M["ridge"]],
                            lambda bm, i=i: ridge(bm, i)))
    for i in range(4):
        parts.append(F.part("groove_%d" % i, [M["groove"]],
                            lambda bm, i=i: groove(bm, i)))
    return parts


N.finish(ctx, build_form(), name="coral_brain")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
