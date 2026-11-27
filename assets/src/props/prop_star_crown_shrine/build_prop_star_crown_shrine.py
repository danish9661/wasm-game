"""Build prop_star_crown_shrine: a recessed stele with an arched niche,
two buttresses, a low gold halo and a five-point Star Crown floating above it.

Run: gam_cli.py build assets/src/props/prop_star_crown_shrine/brief.json
Front faces -Y, base z=0, centred at the world origin. Palette-only, no textures.
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import bmesh
import bpy
import math
from mathutils import Vector
import gam
import _family_hero as F

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "stone": gam.material("m_stone", "#7A7F87", roughness=0.9),
    "niche": gam.material("m_niche", "#5E646B", roughness=0.95),
    "gold": gam.material("m_gold", "#D9A62E", roughness=0.6, metallic=0.6),
    # Low emissive on the halo and star keeps their silhouette at night.
    "glow": gam.material("m_gold_glow", "#D9A62E", roughness=0.6, metallic=0.45,
                         emissive="#D9A62E", emissive_strength=0.45),
    "gem": gam.material("m_star_gem", "#6B3FA0", roughness=0.55,
                        emissive="#6B3FA0", emissive_strength=1.25),
}

CY = -0.06    # crown plane, in front of the stele's recessed panel
LIFT = 0.25   # raises the crown above the stele top so it reads from the back


def steps(bm):
    """Wide, low, three-step base — the opposite of an altar's tall pedestal."""
    gam.bm_box(bm, (-0.62, -0.30, 0.00), (0.62, 0.30, 0.10))
    gam.bm_box(bm, (-0.54, -0.26, 0.10), (0.54, 0.26, 0.20))
    gam.bm_box(bm, (-0.46, -0.22, 0.20), (0.46, 0.22, 0.28))


def stele(bm):
    gam.bm_box(bm, (-0.44, 0.16, 0.28), (0.44, 0.26, 1.70))


def niche(bm):
    gam.bm_box(bm, (-0.30, 0.10, 0.55), (0.30, 0.15, 1.55))


def buttress(bm, sx):
    x = sx * 0.52
    gam.bm_box(bm, (x - 0.08, 0.14, 0.28), (x + 0.08, 0.26, 1.30))


def arch(bm):
    """Stepped corbelled crown above the recessed niche (closed boxes only)."""
    gam.bm_box(bm, (-0.44, 0.14, 1.70), (0.44, 0.28, 1.78))
    gam.bm_box(bm, (-0.34, 0.14, 1.78), (0.34, 0.28, 1.90))
    gam.bm_box(bm, (-0.20, 0.14, 1.90), (0.20, 0.28, 2.02))


def bracket(bm, sx):
    x = sx * 0.30
    gam.bm_box(bm, (x - 0.05, -0.06, 1.22 + LIFT),
               (x + 0.05, 0.16, 1.34 + LIFT))


def halo(bm):
    """Low six-sided gold annulus, seated on the two brackets."""
    n = 6
    z0, z1, ro, ri = 1.34 + LIFT, 1.40 + LIFT, 0.34, 0.29
    a = [2 * math.pi * k / n for k in range(n)]
    olo = [bm.verts.new((ro * math.cos(t), CY + ro * math.sin(t), z0)) for t in a]
    ohi = [bm.verts.new((ro * math.cos(t), CY + ro * math.sin(t), z1)) for t in a]
    ilo = [bm.verts.new((ri * math.cos(t), CY + ri * math.sin(t), z0)) for t in a]
    ihi = [bm.verts.new((ri * math.cos(t), CY + ri * math.sin(t), z1)) for t in a]
    for k in range(n):
        j = (k + 1) % n
        mid = (a[k] + a[j]) / 2
        radial = Vector((math.cos(mid), math.sin(mid), 0.0))
        gam._face(bm, [olo[k], olo[j], ohi[j], ohi[k]], radial,
                  [(ro * a[k], z0), (ro * a[j], z0),
                   (ro * a[j], z1), (ro * a[k], z1)])
        gam._face(bm, [ihi[k], ihi[j], ilo[j], ilo[k]], -radial,
                  [(ri * a[k], z1), (ri * a[j], z1),
                   (ri * a[j], z0), (ri * a[k], z0)])
        gam._face(bm, [ohi[k], ohi[j], ihi[j], ihi[k]], (0, 0, 1),
                  [(v.co.x, v.co.y) for v in (ohi[k], ohi[j], ihi[j], ihi[k])])
        gam._face(bm, [ilo[k], ilo[j], olo[j], olo[k]], (0, 0, -1),
                  [(v.co.x, v.co.y) for v in (ilo[k], ilo[j], olo[j], olo[k])])


def crown_band(bm):
    """Gold annulus that floats 0.12 u above the halo."""
    F.loft(bm, [F.ring(0, CY, 1.52 + LIFT, 0.22, 0.19, n=8),
                F.ring(0, CY, 1.60 + LIFT, 0.20, 0.17, n=8)], mat=0)


def star_point(bm, angle):
    """A square-based, outward-leaning pyramid: reads from every yaw."""
    d = Vector((math.cos(angle), math.sin(angle), 0.0))
    p = Vector((-math.sin(angle), math.cos(angle), 0.0))
    c = Vector((0.0, CY, 1.58 + LIFT))
    r0, half = 0.14, 0.055
    base = [bm.verts.new(c + d * r0 + p * s * half + Vector((0, 0, 0)))
            for s in (-1, 1)]
    base += [bm.verts.new(c + d * r0 + p * s * half + Vector((0, 0, 0.08)))
             for s in (1, -1)]
    apex = bm.verts.new(c + d * 0.36 + Vector((0, 0, 0.26)))
    for k in range(4):
        gam._face(bm, [base[k], base[(k + 1) % 4], apex],
                  ("from", apex.co.copy()), [(0, 0), (1, 0), (0.5, 1)])
    gam._face(bm, list(reversed(base)), (0, 0, -1),
              [(v.co.x, v.co.y) for v in reversed(base)])


def gem(bm):
    F.ico(bm, (0, CY, 1.76 + LIFT), 0.12, scale=(0.9, 0.9, 1.5))


def build_form():
    if ctx.placeholder:
        d = B["dims"]
        ob = gam.box(ctx.id + "_lod0", (d["x"], d["y"], d["z"]), base_z=0)
        ob.data.materials.append(gam.material("m_placeholder_grey", "#8A8B86",
                                              roughness=0.9))
        gam.uv_world_box(ob, meters_per_tile=1.0)
        return ob

    parts = [
        F.part("steps", [M["stone"]], steps, bevel=0.015, segments=1),
        F.part("stele", [M["stone"]], stele, bevel=0.012, segments=1),
        F.part("niche", [M["niche"]], niche),
        F.part("arch", [M["stone"]], arch),
        F.part("halo", [M["glow"]], halo),
        F.part("crown_band", [M["glow"]], crown_band),
        F.part("gem", [M["gem"]], gem),
    ]
    for sx in (-1, 1):
        parts.append(F.part("buttress_%s" % sx, [M["stone"]],
                            lambda bm, sx=sx: buttress(bm, sx)))
        parts.append(F.part("bracket_%s" % sx, [M["stone"]],
                            lambda bm, sx=sx: bracket(bm, sx)))
    for i in range(5):
        a = math.tau * i / 5.0 - math.pi / 2
        parts.append(F.part("star_%d" % i, [M["glow"]],
                            lambda bm, a=a: star_point(bm, a)))
    return F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01)


obj = build_form()
root = gam.finish(ctx, obj)
gam.export(ctx, root)
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
