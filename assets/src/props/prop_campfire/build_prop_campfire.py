"""Rebuild campfire: stone ring, crossed logs, glowing embers and three
large orange flame tongues. Base z=0, footprint ~1.15 u, height 0.83 u.
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
    "stone": gam.material("m_campfire_stone", "#B8C0C8", roughness=0.94),
    "wood": gam.material("m_campfire_log", "#592F1A", roughness=0.92),
    "fire": gam.material("m_campfire_flame", "#FF6A2A", roughness=0.8,
                         emissive="#FF6A2A", emissive_strength=1.6),
    "ember": gam.material("m_campfire_ember", "#D9A62E", roughness=0.8,
                          emissive="#FF6A2A", emissive_strength=1.0),
}


def stones(bm):
    for k in range(8):
        a = 2 * math.pi * k / 8
        x, y = math.cos(a) * 0.44, math.sin(a) * 0.44
        r = 0.145 if k % 2 else 0.125
        F.ico(bm, (x, y, 0.13), r, mat=0, scale=(1.0, 1.0, 0.8))


def logs(bm):
    for k, angle in enumerate((0.0, math.pi / 2, math.pi / 4, -math.pi / 4)):
        F.oriented_box(bm, (-0.37, -0.055, 0.17), (0.37, 0.055, 0.27),
                       mat=0, rot=(0, 0, angle), center=(0, 0, 0.22))


def embers(bm):
    F.ico(bm, (0, 0, 0.26), 0.15, mat=0, scale=(1.1, 1.1, 0.5))
    for x, y in ((-0.16, 0.08), (0.14, -0.10), (0.04, 0.15)):
        F.ico(bm, (x, y, 0.27), 0.055, mat=0, scale=(1.0, 1.0, 0.7))


def flames(bm):
    X.flame_tongue(bm, 0.0, 0.0, 0.28, 0.55, 0.20, mat=0)
    X.flame_tongue(bm, -0.16, 0.06, 0.26, 0.38, 0.12, mat=0, lean=-0.06)
    X.flame_tongue(bm, 0.14, -0.07, 0.26, 0.34, 0.11, mat=1, lean=0.06)


parts = [
    F.part("stones", [M["stone"]], stones),
    F.part("logs", [M["wood"]], logs, bevel=0.008, segments=1),
    F.part("embers", [M["ember"]], embers),
    F.part("flames", [M["fire"], M["ember"]], flames),
]
obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=32.0)
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
