"""Build prop_volcanic_bomb: angular dark stone with glowing lava cracks.

Run: gam_cli.py all assets/src/props/prop_volcanic_bomb/brief.json
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
    "stone": gam.material("m_stone", "#7A7F87", roughness=0.9),
    "dark": gam.material("m_stone_dark", "#5E646B", roughness=0.9),
    "lava": gam.material("m_lava", "#FF6A2A", roughness=0.6,
                         emissive="#FF6A2A", emissive_strength=1.8),
}


def body(bm):
    N.angular_rock(bm, (0, 0, 0.42), 0.42, mat=0, seed=11,
                   scale=(1.0, 0.92, 0.88))


def crack(bm, i):
    a = i * math.tau / 3.0 + 0.4
    x, y = math.cos(a) * 0.30, math.sin(a) * 0.28
    N.tapered_blade(bm, (x * 0.4, y * 0.4, 0.72),
                    (x, y, 0.16), 0.028, mat=0, n=3)


def ash(bm, i):
    a = i * math.tau / 3.0 + 0.9
    N.angular_rock(bm, (math.cos(a) * 0.40, math.sin(a) * 0.36, 0.05),
                   0.10, mat=0, seed=30 + i, scale=(1.0, 0.9, 0.6))


def build_form():
    parts = [F.part("body", [M["dark"]], body, bevel=0.006, segments=1)]
    for i in range(3):
        parts.append(F.part("crack_%d" % i, [M["lava"]],
                            lambda bm, i=i: crack(bm, i)))
    for i in range(3):
        parts.append(F.part("ash_%d" % i, [M["stone"]],
                            lambda bm, i=i: ash(bm, i)))
    return parts


N.finish(ctx, build_form(), name="volcanic_bomb")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
