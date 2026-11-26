"""Build script for char_player — Starfall's hero, built with the game-asset-modeling
workflow: bmesh parts, real bevels, weighted normals, packed UVs, game-material palette.

Run: gam_cli.py build assets/src/char/char_player/brief.json
"""
import os, sys
sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
import bpy, bmesh, math
from mathutils import Vector, Matrix
import gam

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "tunic":   gam.material("m_tunic_orange", "#FF8C26", roughness=0.9),
    "skin":    gam.material("m_skin", "#EAB88C", roughness=0.9),
    "hair":    gam.material("m_hair", "#4A2E1A", roughness=0.9),
    "leather": gam.material("m_leather", "#592F1A", roughness=0.9),
    "pants":   gam.material("m_pants", "#2E3440", roughness=0.9),
    "steel":   gam.material("m_steel", "#C8D0D8", roughness=0.85, metallic=0.4),
    "eye":     gam.material("m_eye", "#1A1A1A", roughness=0.9),
    "gold":    gam.material("m_gold", "#D9A62E", roughness=0.5, metallic=0.4),
}


def part(name, mat_key, fn):
    bm = bmesh.new()
    fn(bm)
    return gam.obj_from_bm(name, bm, [M[mat_key]])


def box(lo, hi, skip=()):
    return lambda bm: gam.bm_box(bm, lo, hi, skip=skip)


def cyl(r0, z0, z1, n=8, center=(0.0, 0.0), r1=None, matrix=None, skip_bot=False):
    return lambda bm: gam.bm_cylinder(bm, r0, z0, z1, n=n, center=center, r1=r1,
                                      top=True, bottom=not skip_bot, matrix=matrix)


def build_form():
    if ctx.placeholder:
        d = B["dims"]
        ob = gam.box(ctx.id + "_lod0", (d["x"], d["y"], d["z"]), base_z=0)
        ob.data.materials.append(M["tunic"])
        gam.uv_world_box(ob, meters_per_tile=1.0)
        return ob

    parts = []
    # legs: tapered octagonal shins, boots as beveled blocks
    for side in (-1, 1):
        x = side * 0.09
        parts.append(part("leg_%s" % side, "pants",
                          cyl(0.075, 0.06, 0.68, 8, (x, 0.0), r1=0.055)))
        parts.append(part("boot_%s" % side, "leather",
                          box((x - 0.085, -0.15, 0.0), (x + 0.085, 0.10, 0.10))))
    # hips + tapered torso (octagonal, wider at shoulders)
    parts.append(part("hips", "pants", cyl(0.145, 0.62, 0.78, 8, r1=0.13)))
    parts.append(part("torso", "tunic", cyl(0.135, 0.76, 1.27, 8, r1=0.175)))
    parts.append(part("belt", "leather", cyl(0.15, 0.83, 0.90, 8)))
    parts.append(part("buckle", "gold", box((-0.035, -0.16, 0.84), (0.035, -0.145, 0.905))))
    # shoulders / arms
    for side in (-1, 1):
        x = side * 0.205
        parts.append(part("arm_%s" % side, "tunic",
                          cyl(0.065, 0.90, 1.31, 8, (x, 0.0), r1=0.05)))
        parts.append(part("hand_%s" % side, "skin",
                          cyl(0.048, 1.29, 1.37, 8, (x, 0.0))))
    # neck + head (beveled block) + hair cap + fringe
    parts.append(part("neck", "skin", cyl(0.055, 1.26, 1.34, 8)))
    parts.append(part("head", "skin",
                      box((-0.115, -0.10, 1.34), (0.115, 0.10, 1.50))))
    parts.append(part("hair_cap", "hair",
                      box((-0.125, -0.105, 1.475), (0.125, 0.105, 1.535))))
    parts.append(part("fringe", "hair",
                      box((-0.12, -0.115, 1.44), (0.12, -0.095, 1.49))))
    # eyes on the front (-Y)
    for side in (-1, 1):
        x = side * 0.05
        parts.append(part("eye_%s" % side, "eye",
                          box((x - 0.014, -0.106, 1.425), (x + 0.014, -0.098, 1.443))))
    # sword at the right hand: blade + guard + grip (kept inside the 1.55u silhouette)
    parts.append(part("blade", "steel",
                      box((0.265, -0.012, 1.31), (0.295, 0.012, 1.53))))
    parts.append(part("guard", "gold",
                      box((0.245, -0.035, 1.29), (0.315, 0.035, 1.31))))
    parts.append(part("grip", "leather",
                      box((0.272, -0.014, 1.22), (0.288, 0.014, 1.29))))

    obj = gam.join(parts, ctx.id + "_lod0")
    # Real bevels + weighted normals: avoids the "stack of flat boxes" look.
    gam.bevel(obj, 0.006, segments=1, angle_deg=35.0)
    gam.shade_hard_surface(obj, angle_deg=35.0, weighted=True)
    gam.uv_pack(obj, margin_frac=0.01)
    return obj


def placeholder_extras():
    return []


obj = build_form()
extras = placeholder_extras() if ctx.placeholder else []
root = gam.finish(ctx, obj, extra_children=extras)
gam.export(ctx, root)
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
