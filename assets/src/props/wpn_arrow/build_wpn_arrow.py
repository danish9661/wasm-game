"""Rebuild arrow (second pass): thicker shaft, broad gold flights and a
large leaf point so the weapon survives the 60 px read. Base z=0.
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
    "wood": gam.material("m_arrow_wood", "#592F1A", roughness=0.9),
    "gold": gam.material("m_arrow_flight", "#D9A62E", roughness=0.7, metallic=0.3),
    "bone": gam.material("m_arrow_nock", "#E8E0CC", roughness=0.85),
    "steel": gam.material("m_arrow_head", "#C8D0D8", roughness=0.5, metallic=0.6),
}


def shaft(bm):
    gam.bm_cylinder(bm, 0.023, 0.02, 0.64, n=8, mat=0)
    gam.bm_cylinder(bm, 0.029, 0.02, 0.11, n=8, mat=0)


def flights(bm):
    for s in (-1, 1):
        def ring(w, off, z):
            return [(0.0, s * 0.016, z), (w, s * off, z + 0.02),
                    (w * 0.7, s * off * 0.9, z + 0.17), (0.0, s * 0.016, z + 0.19)]
        F.loft(bm, [ring(0.012, 0.020, 0.12), ring(0.085, 0.105, 0.15),
                    ring(0.070, 0.088, 0.29), ring(0.006, 0.020, 0.33)], mat=0)


def nock(bm):
    gam.bm_cylinder(bm, 0.030, 0.0, 0.05, n=6, mat=0)
    gam.bm_box(bm, (-0.030, -0.010, 0.0), (0.030, 0.010, 0.024), mat=0)


def head(bm):
    def diamond(w, t, z):
        return [(-w, 0, z), (0, -t, z), (w, 0, z), (0, t, z)]
    F.loft(bm, [diamond(0.050, 0.019, 0.64), diamond(0.068, 0.015, 0.70),
                diamond(0.048, 0.009, 0.80), diamond(0.004, 0.001, 0.88)], mat=0)


parts = [
    F.part("shaft", [M["wood"]], shaft),
    F.part("flights", [M["gold"]], flights),
    F.part("nock", [M["bone"]], nock),
    F.part("head", [M["steel"]], head),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=28.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
