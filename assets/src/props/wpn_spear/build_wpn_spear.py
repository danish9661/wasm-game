"""Rebuild spear: a thicker, readable shaft with a leaf blade and grip wrap.

The old hairline shaft + pinhead disappeared at 60 px. Base z=0, height 1.27 u.
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import gam
import _family_hero as F

ctx = gam.Ctx.from_argv()
gam.reset()
gam.load_material_lib(ctx)

M = {
    "wood": gam.material("m_spear_wood", "#592F1A", roughness=0.9),
    "leather": gam.material("m_spear_grip", "#592F1A", roughness=0.92),
    "gold": gam.material("m_spear_collar", "#D9A62E", roughness=0.55, metallic=0.6),
    "steel": gam.material("m_spear_blade", "#C8D0D8", roughness=0.5, metallic=0.65),
    "iron": gam.material("m_spear_butt", "#B8C0C8", roughness=0.6, metallic=0.5),
}


def shaft(bm):
    gam.bm_cylinder(bm, 0.032, 0.04, 1.06, n=8, mat=0)
    gam.bm_cylinder(bm, 0.038, 0.12, 0.40, n=8, mat=1)
    for z in (0.16, 0.26, 0.36):
        gam.bm_cylinder(bm, 0.042, z, z + 0.018, n=8, mat=1)


def collar(bm):
    gam.bm_cylinder(bm, 0.052, 1.02, 1.08, n=8, mat=0)
    gam.bm_cylinder(bm, 0.044, 1.08, 1.11, n=8, mat=0)


def blade(bm):
    def diamond(w, t, z):
        return [(-w, 0, z), (0, -t, z), (w, 0, z), (0, t, z)]
    F.loft(bm, [diamond(0.075, 0.020, 1.10), diamond(0.088, 0.016, 1.16),
                diamond(0.070, 0.010, 1.22), diamond(0.030, 0.004, 1.26),
                diamond(0.002, 0.001, 1.27)], mat=0)


def butt(bm):
    gam.bm_cylinder(bm, 0.042, 0.01, 0.05, n=6, r1=0.030, mat=0)


parts = [
    F.part("shaft", [M["wood"], M["leather"]], shaft),
    F.part("collar", [M["gold"]], collar),
    F.part("blade", [M["steel"]], blade, bevel=0.002, segments=1),
    F.part("butt", [M["iron"]], butt),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=28.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
