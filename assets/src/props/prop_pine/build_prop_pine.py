"""Build prop_pine: overlapping offset needle tiers, never three stacked cones.

Run: gam_cli.py all assets/src/props/prop_pine/brief.json
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
    "dark": gam.material("m_leaf_dark", "#2C6327", roughness=0.9),
    "leaf": gam.material("m_leaf", "#3A7A33", roughness=0.9),
    "light": gam.material("m_leaf_light", "#55953F", roughness=0.9),
}

TIERS = [(0.68, 0.60, 1.04), (1.48, 0.48, 0.98), (2.18, 0.34, 0.86)]


def trunk(bm):
    N.faceted_stump(bm, (0, 0, 0.0), 0.125, 1.10, mat=0, n=6, seed=7)


def needle_lobe(bm, tier, side):
    z, r, h = TIERS[tier]
    a = (0.0 if side == 0 else math.pi) + tier * 0.55
    dx, dy = math.cos(a) * r * 0.28, math.sin(a) * r * 0.28
    # Four rings with an asymmetric waist produce an irregular needle fan,
    # not the clean straight profile of a primitive cone.
    F.loft(bm, [
        F.ring(dx * 0.4, dy * 0.4, z - 0.10, r * 0.34, r * 0.34, n=7),
        F.ring(dx, dy, z + 0.16, r, r * 0.94, n=7),
        F.ring(dx * 1.2, dy * 1.2, z + h * 0.66, r * 0.62, r * 0.58, n=7),
        F.ring(dx * 1.35, dy * 1.35, z + h, r * 0.05, r * 0.05, n=7),
    ], mat=0)


def branch_stub(bm, i):
    a = i * math.tau / 6.0 + 0.2
    z = 0.95 + 0.52 * (i % 3)
    r = 0.36 - 0.07 * (i % 3)
    N.branch(bm, (0, 0, z),
             (math.cos(a) * r, math.sin(a) * r, z + 0.16),
             0.035, 0.012, mat=0, n=4)


def build_form():
    parts = [F.part("trunk", [M["wood"]], trunk)]
    for t in range(3):
        for s in range(2):
            parts.append(F.part("needles_%d_%d" % (t, s),
                                [M["dark"] if t == 0 else
                                 M["leaf"] if t == 1 else M["light"]],
                                lambda bm, t=t, s=s: needle_lobe(bm, t, s)))
    for i in range(6):
        parts.append(F.part("stub_%d" % i, [M["wood"]],
                            lambda bm, i=i: branch_stub(bm, i)))
    return parts


N.finish(ctx, build_form(), name="pine")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
