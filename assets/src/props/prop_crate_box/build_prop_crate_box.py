"""Rebuild crate_box: a slatted wooden shipping crate with iron straps and a latch.

Base at z=0, centred on (0,0), front faces -Y. Run with gam_cli.py all.
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import gam
import bmesh
import _family_hero as F
import _family_fix as X

ctx = gam.Ctx.from_argv()
gam.reset()
gam.load_material_lib(ctx)

M = {
    "wood": gam.material("m_crate_wood", "#592F1A", roughness=0.9),
    "plank": gam.material("m_crate_plank", "#D9B46A", roughness=0.9),
    "iron": gam.material("m_crate_iron", "#B8C0C8", roughness=0.6, metallic=0.5),
    "gold": gam.material("m_crate_gold", "#D9A62E", roughness=0.55, metallic=0.6),
    "void": gam.material("m_crate_void", "#1A1A1A", roughness=1.0),
}


def core(bm):
    gam.bm_box(bm, (-0.31, -0.38, 0.06), (0.31, 0.38, 0.60), mat=0)


def slats(bm):
    for sign in (-1, 1):
        X.plank_wall(bm, -0.31, 0.31, sign * 0.395, 0.065, 0.60,
                     planks=2, mat=1)
        # Side slats (rotate the plank helper's role by building them along Y).
        for i in range(2):
            z = 0.095 + i * 0.27
            gam.bm_box(bm, (sign * 0.315, -0.38, z),
                       (sign * 0.345, 0.38, z + 0.13), mat=1)


def corners(bm):
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = sx * 0.325, sy * 0.395
            gam.bm_box(bm, (x - 0.032, y - 0.038, 0.04),
                       (x + 0.032, y + 0.038, 0.66), mat=0)


def straps(bm):
    for x in (-0.20, 0.20):
        gam.bm_box(bm, (x - 0.026, -0.418, 0.05),
                   (x + 0.026, -0.398, 0.64), mat=0)
        gam.bm_box(bm, (x - 0.026, 0.398, 0.05),
                   (x + 0.026, 0.418, 0.64), mat=0)
    gam.bm_box(bm, (-0.34, -0.42, 0.60), (0.34, 0.42, 0.635), mat=0)


def lid(bm):
    gam.bm_box(bm, (-0.35, -0.42, 0.635), (0.35, 0.42, 0.72), mat=0)
    for i in range(3):
        x0 = -0.35 + i * 0.235
        gam.bm_box(bm, (x0 + 0.012, -0.43, 0.72),
                   (x0 + 0.222, 0.43, 0.735), mat=1)


def latch(bm):
    gam.bm_box(bm, (-0.075, -0.432, 0.36), (0.075, -0.412, 0.50), mat=0)
    F.ico(bm, (0.0, -0.436, 0.43), 0.028, mat=1, scale=(1.0, 0.5, 1.0))


parts = [
    F.part("core", [M["wood"]], core),
    F.part("slats", [M["wood"], M["plank"]], slats),
    F.part("corners", [M["wood"]], corners),
    F.part("straps", [M["iron"]], straps),
    F.part("lid", [M["wood"], M["plank"]], lid),
    F.part("latch", [M["gold"], M["void"]], latch, bevel=0.004, segments=1),
]

obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=35.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
