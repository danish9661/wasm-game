"""Shared bmesh construction helpers for the Starfall hero-asset family.

Every build script imports this module. All geometry is authored in world
metres, Z up, model front facing -Y. Helpers write outward normals and
meter-scale UVs so a whole assembly packs at one density with gam.uv_pack.
"""
import math
import bmesh
from mathutils import Vector
import gam


def _cap_uvs(ring, axis):
    """Planar UVs (metres) for a ring whose loft axis is `axis`."""
    a = abs(axis.x), abs(axis.y), abs(axis.z)
    if a[2] >= a[0] and a[2] >= a[1]:
        return [(p.x, p.y) for p in ring]
    if a[1] >= a[0]:
        return [(p.x, p.z) for p in ring]
    return [(p.y, p.z) for p in ring]


def loft(bm, rings, mat=0, cap0=True, cap1=True):
    """Loft equal-length rings of 3D points into a closed faceted solid.

    `rings` is a list of rings, each a list of points in consistent order.
    Side UVs run around the perimeter (u, metres) and along the path (v, metres).
    Used for tapered limbs, bodies, trunks, blades and canopies.
    """
    rings = [[Vector(p) for p in r] for r in rings]
    n = len(rings[0])
    assert n >= 3 and all(len(r) == n for r in rings), "loft rings must match"
    centers = [sum(r, Vector()) / n for r in rings]
    solid_center = sum(centers, Vector()) / len(centers)
    vs = [[bm.verts.new(p) for p in r] for r in rings]
    per = [0.0]
    for k in range(1, n):
        per.append(per[-1] + (rings[0][k] - rings[0][k - 1]).length)
    dist = [0.0]
    for i in range(1, len(rings)):
        dist.append(dist[-1] + (centers[i] - centers[i - 1]).length)
    for i in range(len(rings) - 1):
        ax = (centers[i + 1] - centers[i]).normalized()
        for k in range(n):
            k2 = (k + 1) % n
            seg = (rings[i][k2] - rings[i][k]).length
            seg2 = (rings[i + 1][k2] - rings[i + 1][k]).length
            quad = [vs[i][k], vs[i][k2], vs[i + 1][k2], vs[i + 1][k]]
            uvs = [(per[k], dist[i]), (per[k] + seg, dist[i]),
                   (per[k] + seg2, dist[i + 1]), (per[k], dist[i + 1])]
            gam._face(bm, quad, ("from", solid_center), uvs, mat)
    if cap0:
        ax = (centers[1] - centers[0]).normalized()
        gam._face(bm, vs[0], -ax, _cap_uvs(rings[0], ax), mat)
    if cap1:
        ax = (centers[-1] - centers[-2]).normalized()
        gam._face(bm, vs[-1], ax, _cap_uvs(rings[-1], ax), mat)
    return vs


def ring(cx, cy, z, rx, ry=None, n=8, phase=math.pi / 8, squash=1.0):
    """Concentric ring of n points in a horizontal (XY) plane."""
    ry = rx if ry is None else ry
    return [(cx + rx * math.cos(phase + 2 * math.pi * k / n),
             cy + ry * math.sin(phase + 2 * math.pi * k / n),
             z + squash * (k % 2) * 0.0) for k in range(n)]


def ring_y(x, y, z, rx, rz, n=8, phase=math.pi / 8):
    """Ring of n points in a vertical (XZ) plane, lofted along Y."""
    return [(x + rx * math.cos(phase + 2 * math.pi * k / n), y,
             z + rz * math.sin(phase + 2 * math.pi * k / n)) for k in range(n)]


def oriented_box(bm, lo, hi, mat=0, rot=(0.0, 0.0, 0.0), center=None, skip=()):
    """bm_box whose absolute lo..hi box is rotated about `center` (radians)."""
    from mathutils import Matrix
    loc = Vector(((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2))
    pivot = Vector(center) if center is not None else loc
    rotm = Matrix.Rotation(rot[0], 4, 'X') @ Matrix.Rotation(rot[1], 4, 'Y') @ Matrix.Rotation(rot[2], 4, 'Z')
    M = Matrix.Translation(pivot) @ rotm @ Matrix.Translation(-pivot)
    bm2 = bmesh.new()
    faces = gam.bm_box(bm2, lo, hi, mat=mat, skip=skip)
    vs = sorted({v for f in faces for v in f.verts}, key=lambda v: (v.co.x, v.co.y, v.co.z))
    bmesh.ops.transform(bm2, matrix=M, verts=vs)
    lay = bm2.loops.layers.uv.get("UVMap")
    for f in faces:
        poly = [bm.verts.new(v.co) for v in f.verts]
        gam._face(bm, poly, ("from", M @ loc),
                  [tuple(l[lay].uv) for l in f.loops], mat)
    bm2.free()


def part(name, mats, fn, bevel=0.0, segments=1, angle=35.0):
    """One bmesh part -> object, optionally bevelled on its sharp edges."""
    bm = bmesh.new()
    fn(bm)
    ob = gam.obj_from_bm(name, bm, mats)
    if bevel:
        gam.bevel(ob, bevel, segments=segments, angle_deg=angle)
    return ob


def assemble(parts, name, pack_margin=0.01, shade_angle=35.0):
    """Join parts into the render mesh, pack UV0 at one density, apply weighted normals."""
    obj = gam.join(parts, name)
    gam.uv_pack(obj, margin_frac=pack_margin)
    gam.shade_hard_surface(obj, angle_deg=shade_angle, weighted=True)
    return obj


def ico(bm, center, radius, mat=0, subdiv=0, scale=(1.0, 1.0, 1.0)):
    """Faceted icosphere primitive (scaled) baked straight into `bm`."""
    tmp = bmesh.new()
    bmesh.ops.create_icosphere(tmp, subdivisions=subdiv, radius=radius)
    vs = [bm.verts.new(Vector(center) + Vector((v.co.x * scale[0], v.co.y * scale[1], v.co.z * scale[2]))) for v in tmp.verts]
    for f in tmp.faces:
        corners = [vs[v.index] for v in f.verts]
        cen = sum((c.co for c in corners), Vector()) / len(corners)
        gam._face(bm, corners, ("from", Vector(center)), [(c.co.x, c.co.y) for c in corners], mat)
    tmp.free()
