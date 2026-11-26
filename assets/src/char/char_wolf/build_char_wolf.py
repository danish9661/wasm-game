"""Build char_wolf: a chunky faceted wild wolf - barrel body, merged
head/neck, charcoal lower muzzle, haunched dark legs, broad low brush tail.
Front faces -Y, feet at z=0.
Run: gam_cli.py build assets/src/char/char_wolf/brief.json
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
    "body": gam.material("m_wolf_body", "#592F1A", roughness=0.9),
    "dark": gam.material("m_wolf_dark", "#1A1A1A", roughness=0.85),
    "eye": gam.material("m_wolf_eye", "#D9A62E", roughness=0.6, metallic=0.6),
    "snow": gam.material("m_snow", "#E4ECF6", roughness=0.9),
}


def body(bm):
    F.loft(bm, [F.ring_y(0, -0.36, 0.68, 0.19, 0.20),
                F.ring_y(0, -0.05, 0.69, 0.18, 0.19),
                F.ring_y(0, 0.30, 0.66, 0.16, 0.17),
                F.ring_y(0, 0.48, 0.65, 0.12, 0.13)], mat=0)


def head(bm):
    """Neck -> skull; starts inside the body so the cheek has no open seam."""
    F.loft(bm, [F.ring_y(0, -0.28, 0.70, 0.145, 0.150, n=6),
                F.ring_y(0, -0.52, 0.84, 0.125, 0.125, n=6),
                F.ring_y(0, -0.70, 0.85, 0.100, 0.100, n=6)], mat=0)


def muzzle(bm):
    F.loft(bm, [F.ring_y(0, -0.68, 0.84, 0.082, 0.080, n=6),
                F.ring_y(0, -0.78, 0.80, 0.042, 0.050, n=6)], mat=0)


def chest(bm):
    F.loft(bm, [F.ring_y(0, -0.30, 0.60, 0.160, 0.160, n=6),
                F.ring_y(0, -0.56, 0.72, 0.130, 0.130, n=6)], mat=0)


def legs(bm, sx, sy):
    """Dark haunched leg: compact paw -> hock -> thigh, roots inside the body."""
    x, y = sx * 0.135, sy * 0.29
    rings = [F.ring(x, y, 0.0, 0.070, 0.080, n=5),
             F.ring(x, y, 0.10, 0.050, 0.055, n=5),
             F.ring(x, y, 0.32, 0.085, 0.090, n=5),
             F.ring(x, y, 0.55, 0.100, 0.105, n=5)]
    F.loft(bm, rings, mat=0)


def tail(bm):
    """Broad, deep, low-set brush with a blunt tip, rooted inside the rump."""
    F.loft(bm, [F.ring_y(0, 0.34, 0.66, 0.150, 0.140, n=8),
                F.ring_y(0, 0.54, 0.68, 0.155, 0.145, n=8),
                F.ring_y(0, 0.72, 0.73, 0.120, 0.115, n=8),
                F.ring_y(0, 0.86, 0.77, 0.055, 0.055, n=8)], mat=0)


def tail_tip(bm):
    F.loft(bm, [F.ring_y(0, 0.84, 0.76, 0.058, 0.058, n=6),
                F.ring_y(0, 0.94, 0.78, 0.012, 0.012, n=6)], mat=0)


def ears(bm, side):
    x = side * 0.078
    F.loft(bm, [F.ring(x, -0.58, 0.93, 0.072, 0.045, n=4),
                F.ring(x, -0.60, 1.06, 0.008, 0.008, n=4)], mat=0)
    F.loft(bm, [F.ring(x, -0.635, 0.945, 0.040, 0.012, n=4),
                F.ring(x, -0.645, 1.02, 0.006, 0.006, n=4)], mat=1)


def eye(bm, side):
    x = side * 0.090
    gam.bm_box(bm, (x - 0.010, -0.743, 0.865), (x + 0.010, -0.736, 0.890))


def build_form():
    if ctx.placeholder:
        d = B["dims"]
        ob = gam.box(ctx.id + "_lod0", (d["x"], d["y"], d["z"]), base_z=0)
        ob.data.materials.append(M["body"])
        gam.uv_world_box(ob, meters_per_tile=1.0)
        return ob
    parts = [
        F.part("body", [M["body"]], body),
        F.part("head", [M["body"]], head, bevel=0.004, segments=1),
        F.part("muzzle", [M["dark"]], muzzle),
        F.part("chest", [M["snow"]], chest),
        F.part("tail", [M["body"]], tail),
        F.part("tail_tip", [M["dark"]], tail_tip),
    ]
    for s in (-1, 1):
        parts.append(F.part("ear_%s" % s, [M["body"], M["dark"]], lambda bm, s=s: ears(bm, s)))
        parts.append(F.part("eye_%s" % s, [M["eye"]], lambda bm, s=s: eye(bm, s)))
        for sy in (-1, 1):
            parts.append(F.part("leg_%s_%s" % (s, sy), [M["dark"]],
                                lambda bm, s=s, sy=sy: legs(bm, s, sy)))
    return F.assemble(parts, ctx.id + "_lod0", pack_margin=0.01)


obj = build_form()
root = gam.finish(ctx, obj, shade=False)
gam.export(ctx, root)
print("GAM_REPORT", gam.mesh_report(bpy.data.objects[ctx.id + "_lod0"]))
