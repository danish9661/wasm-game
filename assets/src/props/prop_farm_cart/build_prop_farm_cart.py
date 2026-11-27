"""Rebuild farm_cart: slatted bed, spoked wheels, axle, shafts and a small load.

Replaces the open trough with invisible wheels. Base z=0, 1.8 x 1.7 u footprint.
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
    "wood": gam.material("m_cart_wood", "#592F1A", roughness=0.9),
    "plank": gam.material("m_cart_plank", "#D9B46A", roughness=0.9),
    "iron": gam.material("m_cart_iron", "#B8C0C8", roughness=0.6, metallic=0.5),
    "gold": gam.material("m_cart_hub", "#D9A62E", roughness=0.55, metallic=0.55),
}


def bed(bm):
    gam.bm_box(bm, (-0.68, -0.44, 0.40), (0.68, 0.44, 0.58), mat=0)
    for sign in (-1, 1):
        X.plank_wall(bm, -0.68, 0.68, sign * 0.46, 0.58, 0.92, planks=1, mat=1)


def running_gear(bm):
    for side in (-1, 1):
        X.spoked_wheel(bm, (0.0, side * 0.79, 0.34), 0.34, 0.11,
                       spokes=4, mat_rim=0, mat_spoke=1, mat_hub=3)
    gam.bm_box(bm, (-0.04, -0.79, 0.30), (0.04, 0.79, 0.38), mat=2)


def shafts(bm):
    for side in (-1, 1):
        gam.bm_box(bm, (0.68, side * 0.30 - 0.04, 0.42),
                   (0.94, side * 0.30 + 0.04, 0.50), mat=0)


def load(bm):
    F.ico(bm, (-0.24, 0.0, 0.70), 0.15, mat=0, scale=(1.2, 1.0, 0.8))


parts = [
    F.part("bed", [M["wood"], M["plank"]], bed, bevel=0.008, segments=1),
    F.part("gear", [M["wood"], M["plank"], M["iron"], M["gold"]], running_gear),
    F.part("shafts", [M["wood"]], shafts),
    F.part("load", [M["plank"], M["gold"]], load),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=32.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
