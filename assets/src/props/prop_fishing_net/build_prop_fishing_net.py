"""Rebuild fishing_net: a clear hoop, faceted net bag and angled wooden handle.

The old model read as a tiny tent/shrine. Base z=0, height ~0.70 u.
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import gam
import math
import _family_hero as F
import _family_fix as X

ctx = gam.Ctx.from_argv()
gam.reset()
gam.load_material_lib(ctx)

M = {
    "wood": gam.material("m_net_handle", "#592F1A", roughness=0.9),
    "bone": gam.material("m_net_hoop", "#E8E0CC", roughness=0.85),
    "water": gam.material("m_net_mesh", "#3A7CA5", roughness=0.9),
    "sand": gam.material("m_net_knot", "#D9B46A", roughness=0.9),
}


def hoop(bm):
    X.hoop(bm, 0, 0, 0.66, 0.30, 0.022, n=8, mat=0)
    X.hoop(bm, 0, 0, 0.42, 0.20, 0.012, n=8, mat=1)
    X.hoop(bm, 0, 0, 0.20, 0.10, 0.010, n=6, mat=1)


def ribs(bm):
    for k in range(6):
        a = 2 * math.pi * k / 6
        rings = []
        for z, r in ((0.66, 0.30), (0.42, 0.20), (0.20, 0.10), (0.08, 0.025)):
            x, y = math.cos(a) * r, math.sin(a) * r
            rings.append([(x - 0.008, y - 0.008, z), (x + 0.008, y - 0.008, z),
                          (x + 0.008, y + 0.008, z), (x - 0.008, y + 0.008, z)])
        F.loft(bm, rings, mat=0)


def handle(bm):
    rings = []
    for z, x, r in ((0.03, 0.40, 0.034), (0.22, 0.50, 0.032),
                    (0.45, 0.47, 0.028), (0.66, 0.30, 0.024)):
        rings.append(F.ring(x, 0, z, r, n=6))
    F.loft(bm, rings, mat=0)


parts = [
    F.part("hoop", [M["bone"], M["sand"]], hoop),
    F.part("ribs", [M["bone"]], ribs),
    F.part("handle", [M["wood"]], handle, bevel=0.004, segments=1),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=35.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
