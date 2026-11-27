"""Rebuild coral_brain: pale rounded lobes with concentric brain-coral ridges.

The old model was a pink heap of angular lumps. Base z=0, height ~0.65 u.
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
    "stone": gam.material("m_coral_stone", "#B8C0C8", roughness=0.94),
    "sand": gam.material("m_coral_sand", "#D9B46A", roughness=0.94),
}


def body(bm):
    F.ico(bm, (0, 0, 0.30), 0.30, mat=0, scale=(1.42, 1.24, 1.0))
    for x, y, z, r in ((-0.33, 0.06, 0.13, 0.15), (0.31, 0.09, 0.12, 0.14),
                       (0.02, -0.27, 0.11, 0.13), (-0.17, 0.26, 0.10, 0.12),
                       (0.20, -0.24, 0.10, 0.11)):
        F.ico(bm, (x, y, z), r, mat=0 if x < 0 else 1, scale=(1.0, 1.0, 0.8))


def ridges(bm):
    X.torus(bm, (0, 0, 0.49), 0.34, 0.028, n=14, mat_for=lambda k: k % 3 == 0)
    X.torus(bm, (0, 0, 0.555), 0.235, 0.027, n=12, mat_for=lambda k: k % 2)
    X.torus(bm, (-0.02, 0.01, 0.605), 0.135, 0.024, n=10, mat_for=lambda k: 0 if k % 2 else 1)
    X.torus(bm, (-0.02, 0.01, 0.64), 0.055, 0.018, n=8, mat_for=lambda k: 0)


parts = [
    F.part("body", [M["stone"], M["sand"]], body),
    F.part("ridges", [M["sand"], M["stone"]], ridges),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=32.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
