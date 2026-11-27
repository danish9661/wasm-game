"""Rebuild rope_coil: three stacked low-poly rope loops with a loose end.

Replaces the flat spoked "asterisk" with a readable coil. Base z=0.
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import gam
import _family_hero as F
import _family_fix as X

ctx = gam.Ctx.from_argv()
gam.reset()
gam.load_material_lib(ctx)

M = {
    "rope": gam.material("m_rope_coil_rope", "#D9B46A", roughness=0.95),
    "rope_dark": gam.material("m_rope_coil_dark", "#592F1A", roughness=0.95),
}


def loops(bm):
    for z, r in ((0.055, 0.50), (0.15, 0.41), (0.235, 0.30)):
        X.torus(bm, (0, 0, z), r, 0.052, n=12,
                mat_for=lambda k, z=z: 1 if (k + int(z * 10)) % 5 == 0 else 0)


def tail(bm):
    path = [(0.48, 0.05), (0.58, 0.04), (0.66, 0.012)]
    rings = []
    for x, z in path:
        rings.append([(x - 0.048, -0.035, z), (x + 0.048, -0.035, z),
                      (x + 0.048, 0.035, z), (x - 0.048, 0.035, z)])
    F.loft(bm, rings, mat=0)


parts = [
    F.part("loops", [M["rope"], M["rope_dark"]], loops),
    F.part("tail", [M["rope"]], tail),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=35.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
