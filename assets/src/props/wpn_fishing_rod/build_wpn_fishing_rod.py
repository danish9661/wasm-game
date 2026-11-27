"""Rebuild fishing_rod: tapered bent rod, reel, taut line and a large float.

The old rod was a hairline stick with a cube reel; this silhouette reads in hand.
Base z=0, height ~2.15 u.
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
    "wood": gam.material("m_rod_wood", "#592F1A", roughness=0.9),
    "sand": gam.material("m_rod_tip", "#D9B46A", roughness=0.85),
    "leather": gam.material("m_rod_grip", "#592F1A", roughness=0.95),
    "gold": gam.material("m_rod_reel", "#D9A62E", roughness=0.55, metallic=0.6),
    "iron": gam.material("m_rod_hub", "#B8C0C8", roughness=0.6, metallic=0.5),
    "bone": gam.material("m_rod_line", "#E8E0CC", roughness=0.85),
    "red": gam.material("m_rod_float", "#C0392B", roughness=0.75),
}


def rod(bm):
    rings = []
    for z, x, r in ((0.012, 0.0, 0.036), (0.40, 0.0, 0.028),
                    (1.10, 0.015, 0.019), (1.70, 0.055, 0.012),
                    (2.15, 0.100, 0.005)):
        rings.append(F.ring(x, 0, z, r, n=6))
    F.loft(bm, rings, mat=0)


def grip(bm):
    gam.bm_cylinder(bm, 0.046, 0.0, 0.36, n=8, mat=0)
    for z in (0.08, 0.18, 0.28):
        gam.bm_cylinder(bm, 0.049, z, z + 0.022, n=8, mat=0)


def reel(bm):
    F.ico(bm, (0.0, 0.075, 0.52), 0.085, mat=0, scale=(1.0, 0.55, 1.0))
    F.ico(bm, (0.0, 0.145, 0.52), 0.045, mat=1, scale=(1.0, 0.5, 1.0))
    gam.bm_box(bm, (-0.014, 0.02, 0.44), (0.014, 0.17, 0.48), mat=0)


def line_and_float(bm):
    # Thin but visible 4-sided line from the rod tip down to the float.
    gam.bm_cylinder(bm, 0.004, 0.20, 2.12, n=4, center=(0.16, 0.0), mat=0)
    F.ico(bm, (0.16, 0.0, 0.19), 0.055, mat=1, scale=(1.0, 1.0, 1.35))
    F.ico(bm, (0.16, 0.0, 0.155), 0.042, mat=0, scale=(1.0, 1.0, 0.8))


parts = [
    F.part("rod", [M["wood"]], rod),
    F.part("grip", [M["leather"]], grip),
    F.part("reel", [M["gold"], M["iron"]], reel),
    F.part("line", [M["bone"], M["red"]], line_and_float),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=30.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
