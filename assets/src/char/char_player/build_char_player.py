"""Build char_player: Starfall's orange-tunic hero, faceted and readable at iso scale.

Replaces the old stacked-box version. All forms are bmesh lofts/cylinders with a
few selective bevels, weighted normals and a packed UV0. Front faces -Y.
Run: gam_cli.py build assets/src/char/char_player/brief.json
"""
import os
import sys

sys.path.insert(0, os.path.expanduser("~/.claude/skills/game-asset-modeling/scripts"))
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

import bpy
import gam
import _family_hero as F

ctx = gam.Ctx.from_argv()
B = ctx.brief
gam.reset()
lib = gam.load_material_lib(ctx)

M = {
    "tunic": gam.material("m_tunic_orange", "#FF8C26", roughness=0.9),
    "skin": gam.material("m_skin", "#EAB88C", roughness=0.9),
    "hair": gam.material("m_hair", "#4A2E1A", roughness=0.9),
    "leather": gam.material("m_leather", "#592F1A", roughness=0.9),
    "pants": gam.material("m_pants", "#3C4453", roughness=0.9),
    "steel": gam.material("m_steel", "#C8D0D8", roughness=0.7, metallic=0.6),
    "eye": gam.material("m_eye", "#1A1A1A", roughness=0.9),
    "gold": gam.material("m_gold", "#D9A62E", roughness=0.6, metallic=0.6),
}


def ring_at(x, y, z, rx, ry, n=8, phase=0.0):
    return F.ring(x, y, z, rx, ry, n=n, phase=phase)


def legs(bm, side):
    x = side * 0.100
    F.loft(bm, [ring_at(x, 0.0, 0.085, 0.062, 0.060, n=5),
                ring_at(x, 0.0, 0.68, 0.050, 0.048, n=5)], mat=0)


def boot(bm, side):
    """A distinct ankle + forward toe, not a slab under both feet."""
    x = side * 0.100
    F.loft(bm, [ring_at(x, -0.035, 0.0, 0.064, 0.088, n=6),
                ring_at(x, 0.0, 0.095, 0.058, 0.052, n=6)], mat=0)


def hips(bm):
    gam.bm_cylinder(bm, 0.135, 0.66, 0.82, n=6, r1=0.125)


def torso(bm):
    F.loft(bm, [ring_at(0, 0, 0.80, 0.130, 0.105),
                ring_at(0, 0, 1.05, 0.158, 0.115),
                ring_at(0, 0, 1.30, 0.168, 0.100)], mat=0)


def belt(bm):
    gam.bm_cylinder(bm, 0.148, 0.80, 0.88, n=6)


def buckle(bm):
    gam.bm_box(bm, (-0.032, -0.150, 0.84), (0.032, -0.138, 0.90))


def arm(bm, side):
    x = side * 0.200
    F.loft(bm, [ring_at(x, 0, 0.90, 0.062, 0.058, n=5),
                ring_at(x, 0, 1.28, 0.048, 0.046, n=5)], mat=0)


def hand(bm, side):
    x = side * 0.208
    gam.bm_cylinder(bm, 0.050, 0.82, 0.91, n=5, center=(x, -0.025))


def held_grip(bm):
    gam.bm_box(bm, (0.280, -0.115, 0.83), (0.320, -0.075, 0.91))


def neck(bm):
    gam.bm_cylinder(bm, 0.050, 1.31, 1.36, n=6)


def head(bm):
    F.loft(bm, [ring_at(0, 0, 1.355, 0.088, 0.078),
                ring_at(0, 0, 1.435, 0.114, 0.097),
                ring_at(0, 0, 1.495, 0.105, 0.086)], mat=0)


def hair(bm):
    F.loft(bm, [ring_at(0, 0, 1.482, 0.110, 0.094, n=6),
                ring_at(0, 0, 1.550, 0.082, 0.073, n=6)], mat=0)
    # short fringe: keeps the eyes visible from the isometric camera
    gam.bm_box(bm, (-0.105, -0.099, 1.480), (0.105, -0.090, 1.500))


def eye(bm, side):
    x = side * 0.050
    gam.bm_box(bm, (x - 0.013, -0.106, 1.428), (x + 0.013, -0.098, 1.447))


def brow(bm, side):
    x = side * 0.050
    gam.bm_box(bm, (x - 0.020, -0.105, 1.450), (x + 0.020, -0.095, 1.458))


def held_blade(bm):
    # Held OUTSIDE the right arm (arm reach x<=0.25): a thin diamond blade
    # embedded in the sleeve is invisible from the isometric camera.
    x, y = 0.300, -0.095
    def d(w, t, z):
        return [(x - w, y, z), (x, y - t, z), (x + w, y, z), (x, y + t, z)]
    F.loft(bm, [d(0.020, 0.008, 0.92), d(0.021, 0.008, 1.16),
                d(0.006, 0.003, 1.24)], mat=0)


def held_guard(bm):
    gam.bm_box(bm, (0.265, -0.130, 0.90), (0.335, -0.060, 0.925))


def build_form():
    if ctx.placeholder:
        d = B["dims"]
        ob = gam.box(ctx.id + "_lod0", (d["x"], d["y"], d["z"]), base_z=0)
        ob.data.materials.append(M["tunic"])
        gam.uv_world_box(ob, meters_per_tile=1.0)
        return ob
    parts = []
    for s in (-1, 1):
        parts += [F.part("boot_%s" % s, [M["leather"]], lambda bm, s=s: boot(bm, s)),
                  F.part("leg_%s" % s, [M["pants"]], lambda bm, s=s: legs(bm, s))]
    parts += [F.part("hips", [M["pants"]], hips),
              F.part("torso", [M["tunic"]], torso),
              F.part("belt", [M["leather"]], belt),
              F.part("buckle", [M["gold"]], buckle)]
    for s in (-1, 1):
        parts += [F.part("arm_%s" % s, [M["tunic"]], lambda bm, s=s: arm(bm, s)),
                  F.part("hand_%s" % s, [M["skin"]], lambda bm, s=s: hand(bm, s))]
    parts += [F.part("neck", [M["skin"]], neck),
              F.part("head", [M["skin"]], head, bevel=0.004, segments=1),
              F.part("hair", [M["hair"]], hair)]
    for s in (-1, 1):
        parts.append(F.part("eye_%s" % s, [M["eye"]], lambda bm, s=s: eye(bm, s)))
        parts.append(F.part("brow_%s" % s, [M["skin"]], lambda bm, s=s: brow(bm, s)))
    parts += [F.part("held_grip", [M["leather"]], held_grip),
              F.part("held_blade", [M["steel"]], held_blade),
              F.part("held_guard", [M["gold"]], held_guard)]
    return F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01, shade_angle=1.0)


obj = build_form()
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
