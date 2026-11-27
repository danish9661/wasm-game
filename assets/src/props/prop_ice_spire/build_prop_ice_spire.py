"""Build prop_ice_spire: irregular icy prisms instead of cones on a slab.

Run: gam_cli.py all assets/src/props/prop_ice_spire/brief.json
"""
import math
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import bpy
import gam
import _family_hero as F
import _family_nature as N

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "snow": gam.material("m_snow", "#BFD9EE", roughness=0.55),
    "ice": gam.material("m_ice", "#A9C8E0", roughness=0.42),
    "glow": gam.material("m_ice_glow", "#CFE4F5", roughness=0.4,
                         emissive="#CFE4F5", emissive_strength=0.9),
}

SHARDS = [
    (0.0, 0.0, 2.05, 0.21, (0.06, -0.04), 1),
    (-0.30, 0.10, 1.38, 0.15, (-0.10, 0.03), 2),
    (0.30, -0.08, 1.18, 0.14, (0.09, -0.05), 3),
    (0.04, 0.30, 0.86, 0.11, (-0.03, 0.08), 4),
]


def shard(bm, i):
    x, y, h, r, tilt, seed = SHARDS[i]
    N.crystal_shard(bm, (x, y, 0.02), r, h, mat=0, n=5, tilt=tilt, seed=seed)


def chip(bm, i):
    a = i * math.tau / 3.0 + 0.4
    N.angular_rock(bm, (math.cos(a) * 0.44, math.sin(a) * 0.42, 0.06),
                   0.13, mat=0, seed=10 + i, scale=(1.0, 0.85, 0.65))


def glow(bm, i):
    a = i * math.tau / 3.0 + 0.8
    N.crystal_shard(bm, (math.cos(a) * 0.20, math.sin(a) * 0.18, 0.03),
                    0.055, 0.42 + 0.08 * i, mat=0, n=5, seed=20 + i)


def build_form():
    parts = [F.part("shard_%d" % i, [M["snow"] if i in (0, 2) else M["ice"]],
                    lambda bm, i=i: shard(bm, i), bevel=0.005, segments=1)
             for i in range(4)]
    for i in range(3):
        parts.append(F.part("chip_%d" % i, [M["ice"]],
                            lambda bm, i=i: chip(bm, i)))
    for i in range(2):
        parts.append(F.part("glow_%d" % i, [M["glow"]],
                            lambda bm, i=i: glow(bm, i)))
    return parts


N.finish(ctx, build_form(), name="ice_spire")
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
