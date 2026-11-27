"""Build prop_altar: stepped stone plinth, tapered pedestal, gold-capped
corner posts and a hovering faceted arcane crystal. Front faces -Y, base z=0.
Run: gam_cli.py build assets/src/props/prop_altar/brief.json
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import bmesh
import bpy
import gam
import _family_hero as F

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "stone": gam.material("m_stone", "#7A7F87", roughness=0.9),
    "gold": gam.material("m_gold", "#D9A62E", roughness=0.6, metallic=0.6),
    "arcane": gam.material("m_arcane", "#8A50D0", roughness=0.55,
                           emissive="#8A50D0", emissive_strength=2.2),
    "dim": gam.material("m_arcane_dim", "#6B3FA0", roughness=0.7,
                        emissive="#8A50D0", emissive_strength=1.2),
}


def plinth(bm):
    gam.bm_box(bm, (-0.50, -0.36, 0.0), (0.50, 0.36, 0.18))
    gam.bm_box(bm, (-0.43, -0.30, 0.18), (0.43, 0.30, 0.34))
    gam.bm_box(bm, (-0.36, -0.25, 0.34), (0.36, 0.25, 0.48))


def pedestal(bm):
    F.loft(bm, [F.ring(0, 0, 0.47, 0.25, 0.20),
                F.ring(0, 0, 1.24, 0.22, 0.17)], mat=0)


def table_slab(bm):
    gam.bm_box(bm, (-0.42, -0.30, 1.24), (0.42, 0.30, 1.37))


def post(bm, sx, sy):
    x, y = sx * 0.33, sy * 0.22
    gam.bm_cylinder(bm, 0.065, 1.36, 1.66, n=6, center=(x, y), r1=0.055)


def post_cap(bm, sx, sy):
    x, y = sx * 0.33, sy * 0.22
    gam.bm_box(bm, (x - 0.045, y - 0.045, 1.65), (x + 0.045, y + 0.045, 1.69))


def ring(bm):
    """Closed gold annulus (an open tube is half-culled from above -> C-arc)."""
    import math
    from mathutils import Vector
    n = 8
    z0, z1, ro, ri = 1.40, 1.445, 0.32, 0.285
    a = [2 * math.pi * k / n for k in range(n)]
    olo = [bm.verts.new((ro * math.cos(t), ro * math.sin(t), z0)) for t in a]
    ohi = [bm.verts.new((ro * math.cos(t), ro * math.sin(t), z1)) for t in a]
    ilo = [bm.verts.new((ri * math.cos(t), ri * math.sin(t), z0)) for t in a]
    ihi = [bm.verts.new((ri * math.cos(t), ri * math.sin(t), z1)) for t in a]
    for k in range(n):
        j = (k + 1) % n
        u0, u1 = 2 * math.pi * ro * k / n, 2 * math.pi * ro * (k + 1) / n
        mid = (a[k] + a[j]) / 2
        radial = Vector((math.cos(mid), math.sin(mid), 0.0))
        gam._face(bm, [olo[k], olo[j], ohi[j], ohi[k]], radial,
                  [(u0, z0), (u1, z0), (u1, z1), (u0, z1)])
        gam._face(bm, [ihi[k], ihi[j], ilo[j], ilo[k]], -radial,
                  [(u0, z1), (u1, z1), (u1, z0), (u0, z0)])
        gam._face(bm, [ohi[k], ohi[j], ihi[j], ihi[k]], (0, 0, 1),
                  [(ohi[k].co.x, ohi[k].co.y), (ohi[j].co.x, ohi[j].co.y),
                   (ihi[j].co.x, ihi[j].co.y), (ihi[k].co.x, ihi[k].co.y)])
        gam._face(bm, [ilo[k], ilo[j], olo[j], olo[k]], (0, 0, -1),
                  [(ilo[k].co.x, ilo[k].co.y), (ilo[j].co.x, ilo[j].co.y),
                   (olo[j].co.x, olo[j].co.y), (olo[k].co.x, olo[k].co.y)])


def crystal(bm):
    F.ico(bm, (0, 0, 1.75), 0.17, scale=(0.75, 0.75, 1.4))


def shard(bm, sx):
    F.ico(bm, (sx * 0.20, 0.02, 1.61), 0.055, scale=(0.8, 0.8, 1.5))


def build_form():
    if ctx.placeholder:
        d = B["dims"]
        ob = gam.box(ctx.id + "_lod0", (d["x"], d["y"], d["z"]), base_z=0)
        ob.data.materials.append(M["stone"])
        gam.uv_world_box(ob, meters_per_tile=1.0)
        return ob
    parts = [
        F.part("plinth", [M["stone"]], plinth, bevel=0.025, segments=1),
        F.part("pedestal", [M["stone"]], pedestal),
        F.part("table_slab", [M["stone"]], table_slab, bevel=0.015, segments=1),
        F.part("ring", [M["gold"]], ring),
        F.part("crystal", [M["arcane"]], crystal),
    ]
    for s in (-1, 1):
        parts.append(F.part("shard_%s" % s, [M["dim"]], lambda bm, s=s: shard(bm, s)))
        for sy in (-1, 1):
            parts.append(F.part("post_%s_%s" % (s, sy), [M["stone"]],
                                lambda bm, s=s, sy=sy: post(bm, s, sy)))
            parts.append(F.part("cap_%s_%s" % (s, sy), [M["gold"]],
                                lambda bm, s=s, sy=sy: post_cap(bm, s, sy)))
    return F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01)


obj = build_form()
root = gam.finish(ctx, obj)
gam.export(ctx, root)
# Runtime GLB fix: collider nodes must not be drawn (see strip_collider_meshes).
sys.path.insert(0, os.path.abspath(os.path.join(
    os.path.dirname(__file__), "..", "..", "..", "..", "scripts")))
from strip_collider_meshes import strip_file
strip_file(ctx.glb)
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
