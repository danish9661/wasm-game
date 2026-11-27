"""Small bmesh toolbox for the visual-quality repairs.

All repair assets are built from bmesh parts (never stacked bpy.ops primitives),
bevelled, flat-shaded with weighted normals and given a non-overlapping packed
UV0 atlas. glTF-exportable materials only (flat Principled colours), so these
meshes survive the web renderer and the Khronos validator unchanged.
"""
import math

import bmesh
import bpy
from mathutils import Matrix, Vector


def bm_box(bm, lo, hi, mat=0, skip=(), rot=None, center=None):
    """Axis-aligned box with outward normals; optional face skipping/rotation."""
    x0, y0, z0 = lo
    x1, y1, z1 = hi
    verts = [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0),
             (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]
    if rot is not None:
        import mathutils
        c = Vector(center) if center else Vector(((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2))
        m = mathutils.Euler(rot).to_matrix().to_4x4()
        pts = []
        for v in verts:
            p = Vector(v) - c
            world = m @ p + c
            pts.append(tuple(world))
        verts = pts
    base = len(bm.verts)
    for v in verts:
        bm.verts.new(v)
    bm.verts.ensure_lookup_table()
    faces = [
        ("bottom", (3, 2, 1, 0)), ("top", (4, 5, 6, 7)),
        ("front", (0, 1, 5, 4)), ("right", (1, 2, 6, 5)),
        ("back", (2, 3, 7, 6)), ("left", (3, 0, 4, 7)),
    ]
    made = []
    for name, idx in faces:
        if name in skip:
            continue
        f = bm.faces.new([bm.verts[base + i] for i in idx])
        f.material_index = mat
        made.append(f)
    return made


def bm_cone(bm, center, radius, r2, depth, sides=6, mat=0,
            rot_z=0.0, skip_caps=False):
    """Cylinder/frustum with optional caps, centred at ``center``."""
    import mathutils
    matrix = mathutils.Matrix.Translation(Vector(center))
    if rot_z:
        matrix = matrix @ mathutils.Matrix.Rotation(rot_z, 4, "Z")
    res = bmesh.ops.create_cone(
        bm, cap_ends=not skip_caps, cap_tris=False, segments=sides,
        radius1=radius, radius2=r2, depth=depth, matrix=matrix)
    for f in res.get("faces", []):
        f.material_index = mat
    return res.get("faces", [])


def bm_ico(bm, center, radius, mat=0, scale=(1, 1, 1)):
    import mathutils
    matrix = mathutils.Matrix.Translation(Vector(center)) @ mathutils.Matrix.Diagonal(
        mathutils.Vector((*scale, 1.0)))
    res = bmesh.ops.create_icosphere(bm, subdivisions=0, radius=radius, matrix=matrix)
    for f in res.get("faces", []):
        f.material_index = mat
    return res.get("faces", [])


def pack_uvs(bm):
    """Unique, non-overlapping UV0 islands (one square cell per polygon)."""
    if not bm.faces:
        return
    uv = bm.loops.layers.uv.verify()
    cols = max(1, math.ceil(math.sqrt(len(bm.faces))))
    cell = 1.0 / cols
    inset = cell * 0.06
    for i, f in enumerate(bm.faces):
        cx, cy = (i % cols) * cell, (i // cols) * cell
        n = len(f.verts)
        if n <= 1:
            continue
        for k, loop in enumerate(f.loops):
            if n == 4:
                u, v = ((0, 0), (1, 0), (1, 1), (0, 1))[k]
            else:
                a = 2 * math.pi * k / n
                u, v = 0.5 + 0.42 * math.cos(a), 0.5 + 0.42 * math.sin(a)
            loop[uv].uv = (cx + inset + u * (cell - 2 * inset),
                           cy + inset + v * (cell - 2 * inset))


def finish(name, bm, materials, col, bevel=0.0, bevel_segments=1):
    """Bevel, recalc normals, pack UV0, apply weighted normals, link to col."""
    if bevel > 0:
        bmesh.ops.bevel(bm, geom=list(bm.edges), offset=bevel,
                        segments=bevel_segments, affect="EDGES")
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    pack_uvs(bm)
    me = bpy.data.meshes.new(name + "_mesh")
    bm.to_mesh(me)
    bm.free()
    for m in materials:
        me.materials.append(m)
    for p in me.polygons:
        p.use_smooth = False
    obj = bpy.data.objects.new(name, me)
    col.objects.link(obj)
    # Weighted normals are applied even though the look is flat shaded: the
    # asset contract requires them, and they keep any future smooth LOD sane.
    try:
        mod = obj.modifiers.new("WeightedNormal", "WEIGHTED_NORMAL")
        mod.keep_sharp = True
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=mod.name)
    except Exception as exc:  # pragma: no cover - Blender context dependent
        print("QA_WEIGHTED_NORMAL_SKIPPED", name, exc)
    return obj


def make_bm():
    return bmesh.new()


def tri_count(col):
    total = 0
    for o in col.objects:
        if o.type == "MESH":
            o.data.calc_loop_triangles()
            total += len(o.data.loop_triangles)
    return total
