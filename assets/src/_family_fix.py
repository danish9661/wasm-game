"""Shared bmesh helpers for the visual-QA prop rebuilds.

All geometry is authored in world metres, Z up, front facing -Y, base at z=0.
Parts are built with gam primitives (UVs in metres), then assembled with
_family_hero.assemble() which packs UV0 and applies weighted normals.
"""
import math

from mathutils import Vector

import gam


def torus(bm, center, R, r, n=12, plane="xy", mat_for=None, phase=0.0):
    """Closed square-section torus, n segments.

    plane='xy' = flat ring on the ground; plane='xz' = upright wheel (Y axis).
    mat_for(k) picks the material index of segment k (e.g. alternating sectors).
    """
    center = Vector(center)
    axial = Vector((0, 0, 1)) if plane == "xy" else Vector((0, 1, 0))

    def radial(a):
        return Vector((math.cos(a), math.sin(a), 0)) if plane == "xy" \
            else Vector((math.cos(a), 0, math.sin(a)))

    vs = []
    for k in range(n):
        a = phase + 2 * math.pi * k / n
        rad = radial(a)
        c = center + rad * R
        vs.append([bm.verts.new(c + rad * r + axial * r),
                   bm.verts.new(c + rad * r - axial * r),
                   bm.verts.new(c - rad * r - axial * r),
                   bm.verts.new(c - rad * r + axial * r)])
    for k in range(n):
        j = (k + 1) % n
        rad = radial(phase + 2 * math.pi * (k + 0.5) / n)
        outward = [rad, -axial, -rad, axial]
        m = mat_for(k) if mat_for else 0
        u0, u1 = k * 0.12, (k + 1) * 0.12
        for s in range(4):
            s2 = (s + 1) % 4
            gam._face(bm, [vs[k][s], vs[j][s], vs[j][s2], vs[k][s2]],
                      outward[s], [(u0, 0.0), (u1, 0.0), (u1, 0.08), (u0, 0.08)], m)


def hoop(bm, cx, cy, z, radius, thickness, n=10, mat=0):
    """Flat octagonal hoop (annulus) in the XY plane — net frames, bucket rims."""
    ro, ri = radius + thickness, radius - thickness
    angles = [2 * math.pi * k / n for k in range(n)]
    ot = [bm.verts.new((cx + ro * math.cos(a), cy + ro * math.sin(a), z)) for a in angles]
    it = [bm.verts.new((cx + ri * math.cos(a), cy + ri * math.sin(a), z)) for a in angles]
    ob = [bm.verts.new((cx + ro * math.cos(a), cy + ro * math.sin(a), z - 0.018)) for a in angles]
    ib = [bm.verts.new((cx + ri * math.cos(a), cy + ri * math.sin(a), z - 0.018)) for a in angles]
    for k in range(n):
        j = (k + 1) % n
        u0, u1 = k * 0.1, (k + 1) * 0.1
        uvs = [(u0, 0.0), (u1, 0.0), (u1, thickness), (u0, thickness)]
        gam._face(bm, [it[k], it[j], ot[j], ot[k]], (0, 0, 1), uvs, mat)
        gam._face(bm, [ob[k], ob[j], ib[j], ib[k]], (0, 0, -1), uvs, mat)
        gam._face(bm, [ot[k], ot[j], ob[j], ob[k]], (math.cos(angles[k]), math.sin(angles[k]), 0), uvs, mat)
        gam._face(bm, [ib[k], ib[j], it[j], it[k]], (-math.cos(angles[k]), -math.sin(angles[k]), 0), uvs, mat)


def spoked_wheel(bm, center, radius, width, spokes=8, mat_rim=0, mat_spoke=1, mat_hub=2):
    """Upright low-poly cart wheel in the XZ plane, axle along Y."""
    cx, cy, cz = center
    torus(bm, (cx, cy, cz), radius - 0.035, 0.035, n=spokes + 2, plane="xz", mat_for=lambda k: mat_rim)
    hub = gam.bm_cylinder(bm, 0.065, -width / 2, width / 2, n=6,
                          center=(cx, cz), mat=mat_hub)
    # bm_cylinder is Z-up: rotate its vertices into the XZ wheel plane.
    import bmesh
    from mathutils import Matrix
    rot = Matrix.Translation((0, cy, 0)) @ Matrix.Rotation(math.pi / 2, 4, "X")
    bmesh.ops.transform(bm, matrix=rot,
                        verts=list({v for f in hub for v in f.verts}))
    for k in range(spokes):
        theta = 2 * math.pi * k / spokes
        # Canonical spoke points along +Z, rotated about the wheel axle (Y).
        p0 = (cx - 0.026, cy - width * 0.30, cz + 0.05)
        p1 = (cx + 0.026, cy + width * 0.30, cz + radius - 0.03)
        import _family_hero as F
        F.oriented_box(bm, p0, p1, mat=mat_spoke, center=(cx, cy, cz),
                       rot=(0, math.pi / 2 - theta, 0))


def flame_tongue(bm, x, y, z0, height, radius, mat, lean=0.0):
    """Faceted tapered flame with a pronounced point and a slight lean."""
    import _family_hero as F

    def diamond(w, z, ox):
        return [(x + ox - w, y, z), (x + ox, y - w * 0.75, z),
                (x + ox + w, y, z), (x + ox, y + w * 0.75, z)]

    F.loft(bm, [diamond(radius, z0, 0.0),
                diamond(radius * 0.78, z0 + height * 0.55, lean * 0.55),
                diamond(radius * 0.20, z0 + height * 0.86, lean * 0.9),
                diamond(0.006, z0 + height, lean)], mat=mat)


def plank_wall(bm, x0, x1, y, z0, z1, planks=4, mat=0, gap=0.006):
    """Horizontal slats on a crate/cart side, with readable gaps between them."""
    h = (z1 - z0 - gap * (planks - 1)) / planks
    for i in range(planks):
        lo = (x0, y - 0.018, z0 + i * (h + gap))
        hi = (x1, y + 0.018, z0 + i * (h + gap) + h)
        gam.bm_box(bm, lo, hi, mat=mat)
