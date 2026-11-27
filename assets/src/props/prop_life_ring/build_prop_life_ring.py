"""Rebuild life_ring: one thick ring with alternating orange/snow sectors.

Replaces four disconnected blocks. Base z=0, diameter ~1.15 u.
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import gam
import _family_fix as X
import _family_hero as F

ctx = gam.Ctx.from_argv()
gam.reset()
gam.load_material_lib(ctx)

M = {
    "orange": gam.material("m_lifering_orange", "#FF6A2A", roughness=0.8),
    "white": gam.material("m_lifering_white", "#E2E8F0", roughness=0.85),
    "gold": gam.material("m_lifering_band", "#D9A62E", roughness=0.55, metallic=0.5),
}


def ring(bm):
    X.torus(bm, (0, 0, 0.11), 0.46, 0.11, n=12,
            mat_for=lambda k: 0 if (k // 2) % 2 == 0 else 1)


def bands(bm):
    for k in range(4):
        import math
        a = math.pi / 4 + k * math.pi / 2
        x, y = math.cos(a) * 0.46, math.sin(a) * 0.46
        F.ico(bm, (x, y, 0.17), 0.045, mat=0, scale=(1.0, 1.0, 0.5))


parts = [
    F.part("ring", [M["orange"], M["white"]], ring),
    F.part("bands", [M["gold"]], bands),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=35.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
