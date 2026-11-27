"""Rebuild haystack: golden stacked hay cone with straw tufts and binding
bands, replacing the off-white faceted heap. Base z=0, height 1.05 u.
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
    "hay": gam.material("m_haystack_hay", "#D9B46A", roughness=0.95),
    "straw": gam.material("m_haystack_straw", "#D9A62E", roughness=0.95),
    "band": gam.material("m_haystack_band", "#592F1A", roughness=0.95),
}


def body(bm):
    F.loft(bm, [F.ring(0, 0, 0.02, 0.50, n=8),
                F.ring(0, 0, 0.32, 0.49, n=8),
                F.ring(0, 0, 0.70, 0.30, n=8),
                F.ring(0, 0, 0.98, 0.10, n=8),
                F.ring(0, 0, 1.04, 0.02, n=8)], mat=0)


def straw(bm):
    for k in range(10):
        a = 2 * math.pi * k / 10
        for z, r in ((0.30, 0.50),):
            F.oriented_box(bm, (0.28, -0.018, z), (r + 0.07, 0.018, z + 0.035),
                           mat=0, rot=(0, 0, a), center=(0, 0, z))
    for k in range(3):
        a = 2 * math.pi * k / 3
        x, y = math.cos(a) * 0.07, math.sin(a) * 0.07
        F.ico(bm, (x, y, 1.04), 0.045, mat=0, scale=(1.0, 1.0, 1.7))


def bands(bm):
    X.torus(bm, (0, 0, 0.30), 0.49, 0.018, n=8, mat_for=lambda k: 0)
    X.torus(bm, (0, 0, 0.67), 0.33, 0.015, n=6, mat_for=lambda k: 0)


parts = [
    F.part("body", [M["hay"]], body),
    F.part("straw", [M["straw"]], straw),
    F.part("bands", [M["band"]], bands),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=35.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
