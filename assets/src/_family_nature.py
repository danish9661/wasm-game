"""Shared bmesh form helpers for the Starfall nature/biome fix family.

Every builder imports this module and `gam`. Parts are authored in world
metres, Z up, front facing -Y, base at z = 0, footprint centred on (0, 0).
All primitives write outward normals and metre-scale UVs so `gam.uv_pack`
can pack the whole assembly at one density. Nothing here is smooth-shaded.
"""
import math
import random

import bmesh
from mathutils import Matrix, Vector

import gam
import _family_hero as F


def rng(seed):
    return random.Random(seed)


def angular_rock(bm, center, radius, mat=0, seed=1, scale=(1.0, 0.9, 0.7),
                 flatten_base=True, flat_top=False):
    """Angular, non-spherical faceted boulder (not an icosphere blob).

    A subdivision-1 icosphere with deterministic jitter, flattened underside
    and optional flattened top, so it reads as a chipped stone from 45/30.
    """
    r = rng(seed)
    tmp = bmesh.new()
    bmesh.ops.create_icosphere(tmp, subdivisions=1, radius=radius)
    cx, cy, cz = center
    vs = []
    for v in tmp.verts:
        x, y, z = v.co.x, v.co.y, v.co.z
        j = 0.86 + 0.28 * r.random()
        p = Vector((cx + x * scale[0] * j,
                    cy + y * scale[1] * j,
                    cz + z * scale[2] * j))
        if flatten_base and p.z < cz - radius * scale[2] * 0.45:
            p.z = cz - radius * scale[2] * 0.45
        if flat_top and p.z > cz + radius * scale[2] * 0.35:
            p.z = cz + radius * scale[2] * 0.35
        vs.append(bm.verts.new(p))
    for f in tmp.faces:
        corners = [vs[v.index] for v in f.verts]
        cen = sum((c.co for c in corners), Vector()) / len(corners)
        uvs = [(c.co.x - cx, c.co.y - cy) if abs(cen.z - cz) > abs(cen.x - cx)
               else (c.co.x - cx, c.co.z - cz) for c in corners]
        gam._face(bm, corners, ("from", cen), uvs, mat)
    tmp.free()


def leaf_lobe(bm, cx, cy, z, rx, ry, h, mat=0, n=6, seed=0):
    """Pointed, faceted canopy lobe built from lofted rings (never a sphere)."""
    r = rng(seed)
    phase = math.pi / n + 0.12 * r.random()
    rings = [
        F.ring(cx, cy, z - 0.22 * h, rx * 0.42, ry * 0.42, n=n, phase=phase),
        F.ring(cx, cy, z - 0.03 * h, rx, ry, n=n, phase=phase),
        F.ring(cx, cy, z + 0.45 * h, rx * 0.78, ry * 0.78, n=n, phase=phase),
        F.ring(cx, cy, z + h, rx * 0.08, ry * 0.08, n=n, phase=phase),
    ]
    F.loft(bm, rings, mat=mat)


def branch(bm, start, end, r0, r1, mat=0, n=5):
    """Tapered solid branch between two points."""
    a, b = Vector(start), Vector(end)
    axis = (b - a).normalized()
    rot = Vector((0, 0, 1)).rotation_difference(axis).to_matrix().to_4x4()
    mtx = Matrix.Translation(a) @ rot
    gam.bm_cylinder(bm, r0, 0.0, (b - a).length, n=n, r1=r1, matrix=mtx)


def tapered_blade(bm, base, tip, width, mat=0, n=3):
    """A thin tapered leaf/grass blade with a real tip (not a cone)."""
    a, b = Vector(base), Vector(tip)
    axis = (b - a).normalized()
    rot = Vector((0, 0, 1)).rotation_difference(axis).to_matrix().to_4x4()
    mtx = Matrix.Translation(a) @ rot
    gam.bm_cylinder(bm, width, 0.0, (b - a).length, n=n, r1=width * 0.05,
                    matrix=mtx)


def crystal_shard(bm, center, radius, height, mat=0, n=5, tilt=(0, 0),
                  seed=0):
    """Faceted crystal point with an asymmetric waist and a sharp tip."""
    r = rng(seed)
    cx, cy, cz = center
    phase = math.pi / n + 0.1 * r.random()
    rings = [
        F.ring(cx, cy, cz, radius * 0.9, radius * 0.9, n=n, phase=phase),
        F.ring(cx + tilt[0] * 0.3, cy + tilt[1] * 0.3, cz + height * 0.28,
               radius, radius * 0.82, n=n, phase=phase),
        F.ring(cx + tilt[0], cy + tilt[1], cz + height,
               radius * 0.05, radius * 0.05, n=n, phase=phase),
    ]
    F.loft(bm, rings, mat=mat)


def cap(bm, center, radius, mat=0, n=8, flat=False):
    """Flattened faceted mushroom/coral cap with a slight rim."""
    cx, cy, cz = center
    rings = [
        F.ring(cx, cy, cz - 0.02, radius * 0.72, radius * 0.72, n=n),
        F.ring(cx, cy, cz + 0.01, radius, radius, n=n),
        F.ring(cx, cy, cz + (0.06 if flat else 0.16), radius * 0.62,
               radius * 0.62, n=n),
    ]
    F.loft(bm, rings, mat=mat)


def faceted_stump(bm, center, radius, height, mat=0, n=7, seed=0):
    """Irregular tapered trunk with visible bark facets."""
    r = rng(seed)
    cx, cy, cz = center
    phase = math.pi / n
    rings = [
        F.ring(cx, cy, cz, radius * 1.18, radius * 1.18, n=n, phase=phase),
        F.ring(cx, cy, cz + height * 0.45, radius * (0.94 + 0.06 * r.random()),
               radius * 0.98, n=n, phase=phase),
        F.ring(cx, cy, cz + height, radius * 0.74, radius * 0.78, n=n,
               phase=phase),
    ]
    F.loft(bm, rings, mat=mat)


def finish(ctx, parts, name=None, pack_margin=0.01, collider=None):
    """Join parts, UV pack, weighted normals; finish + export via gam.

    `parts` is a list of objects made with `F.part`. Copies the exported GLB
    into assets/models/<name>.glb so the game loader keeps its filename.
    """
    import os
    import shutil

    render_name = name or ctx.id.replace("prop_", "", 1)
    obj = F.assemble(parts, ctx.id + "_lod0", pack_margin=pack_margin)
    kwargs = {}
    if collider is not None:
        col = gam.box(ctx.id + "_col_src", collider, base_z=0)
        kwargs["collider_src"] = col
    root = gam.finish(ctx, obj, **kwargs)
    src = gam.export(ctx, root)
    dst = os.path.join(ctx.assets_root, "models", render_name + ".glb")
    shutil.copyfile(src, dst)
    print("NATURE_GLB", dst)
    return root
