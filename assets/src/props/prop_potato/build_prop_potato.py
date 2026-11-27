"""Build prop_potato: visible half-buried tubers in soil, not a dirt slab.

Run: gam_cli.py all assets/src/props/prop_potato/brief.json
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
    "soil": gam.material("m_soil", "#592F1A", roughness=0.95),
    "tuber": gam.material("m_tuber", "#8C6239", roughness=0.95),
    "eye": gam.material("m_eye", "#6B4423", roughness=0.95),
    "shoot": gam.material("m_leaf", "#3A7A33", roughness=0.9),
}

TUBERS = [(-0.27, 0.05, 0.24, 0.19), (0.10, -0.16, 0.22, 0.17),
          (0.30, 0.12, 0.21, 0.16), (-0.05, 0.25, 0.20, 0.15)]


def soil(bm):
    N.angular_rock(bm, (0, 0, 0.12), 0.46, mat=0, seed=6,
                   scale=(1.05, 0.88, 0.30))


def tuber(bm, i):
    x, y, z, r = TUBERS[i]
    N.angular_rock(bm, (x, y, z), r, mat=0, seed=40 + i,
                   scale=(1.25, 0.95, 0.70))


def eye(bm, i):
    x, y, z, r = TUBERS[i // 2]
    a = (i % 2) * math.pi + 0.6
    N.angular_rock(bm, (x + math.cos(a) * r * 0.7, y + math.sin(a) * r * 0.6,
                        z + r * 0.52), 0.028, mat=0, seed=60 + i,
                   scale=(1.0, 1.0, 0.5))


def shoot(bm, i):
    x, y = (-0.14, 0.10) if i == 0 else (0.20, -0.02)
    N.tapered_blade(bm, (x, y, 0.34), (x + 0.04, y + 0.02, 0.60),
                    0.045, mat=0, n=3)
    N.tapered_blade(bm, (x, y, 0.40),
                    (x + (0.18 if i == 0 else -0.18), y + 0.06, 0.52),
                    0.055, mat=0, n=3)


def build_form():
    parts = [F.part("soil", [M["soil"]], soil)]
    for i in range(4):
        parts.append(F.part("tuber_%d" % i, [M["tuber"]],
                            lambda bm, i=i: tuber(bm, i)))
    for i in range(8):
        parts.append(F.part("eye_%d" % i, [M["eye"]],
                            lambda bm, i=i: eye(bm, i)))
    for i in range(2):
        parts.append(F.part("shoot_%d" % i, [M["shoot"]],
                            lambda bm, i=i: shoot(bm, i)))
    return parts


N.finish(ctx, build_form(), name="potato")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
