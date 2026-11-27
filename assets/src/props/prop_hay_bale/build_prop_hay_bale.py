"""Build prop_hay_bale: a round, banded straw bale (never a wrapped cube).

Run: gam_cli.py all assets/src/props/prop_hay_bale/brief.json
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
    "straw": gam.material("m_straw", "#D9B46A", roughness=0.95),
    "band": gam.material("m_straw_band", "#B8944E", roughness=0.95),
    "straw_light": gam.material("m_straw_light", "#E3C888", roughness=0.95),
}


def barrel(bm):
    F.loft(bm, [F.ring(0, 0, 0.0, 0.49, 0.35, n=10),
                F.ring(0, 0, 0.16, 0.55, 0.39, n=10),
                F.ring(0, 0, 0.56, 0.55, 0.39, n=10),
                F.ring(0, 0, 0.72, 0.49, 0.35, n=10)], mat=0)


def band(bm, z):
    F.loft(bm, [F.ring(0, 0, z - 0.028, 0.558, 0.398, n=10),
                F.ring(0, 0, z + 0.028, 0.558, 0.398, n=10)], mat=0)


def straw_nub(bm, i):
    a = i * math.tau / 7.0 + 0.2
    r = 0.24 + 0.08 * (i % 2)
    x, y = math.cos(a) * r, math.sin(a) * r * 0.72
    N.tapered_blade(bm, (x, y, 0.70), (x + math.cos(a) * 0.13,
                                       y + math.sin(a) * 0.09, 0.76),
                    0.022, mat=0, n=3)


def build_form():
    parts = [F.part("barrel", [M["straw"]], barrel, bevel=0.008, segments=1)]
    for z in (0.17, 0.36, 0.55):
        parts.append(F.part("band_%.2f" % z, [M["band"]],
                            lambda bm, z=z: band(bm, z)))
    for i in range(7):
        parts.append(F.part("straw_%d" % i, [M["straw_light"]],
                            lambda bm, i=i: straw_nub(bm, i)))
    return parts


N.finish(ctx, build_form(), name="hay_bale")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
