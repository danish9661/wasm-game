"""Shared bmesh construction helpers for the character visual-quality rebuild.

Uses the game-asset-modeling skill's gam.py for bmesh parts, selective bevels,
weighted normals and UV packing. No textures: art direction is flat-shaded
palette only, so UV0 is packed for contract completeness but never sampled.

Authoring axes: +Z up, -Y front, 1u = 1 tile, base at z=0 before export.
"""
import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

SKILL_SCRIPTS = os.environ.get(
    "GAM_SCRIPTS",
    os.path.expanduser("~/.config/opencode/skills/game-asset-modeling/scripts"),
)
if SKILL_SCRIPTS not in sys.path:
    sys.path.insert(0, SKILL_SCRIPTS)
import gam  # noqa: E402

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
MODELS = os.path.join(ROOT, "assets", "models")

# ART_DIRECTION.md palette + the project's established neutral/character hues.
PAL = {
    "player": "#FF8C26", "skin": "#EAB88C", "hair": "#4A2E1A",
    "wood": "#6B4423", "leather": "#592F1A", "cloth": "#39527A",
    "bone": "#E8E0CC", "stone": "#7A7F87", "iron": "#B8C0C8",
    "steel": "#C8D0D8", "gold": "#D9A62E", "leaf": "#3A7A33",
    "grass": "#5B8C3A", "swamp": "#4A6741", "water": "#3A7CA5",
    "arcane": "#6B3FA0", "fire": "#FF6A2A", "enemy": "#C0392B",
    "sand": "#D9B46A", "snow": "#E8F0FA", "eye": "#1A1A1A",
    "trousers": "#3C4453", "sand_dark": "#A98548",
    "leaf_dark": "#2C5C28", "charcoal": "#2E3238",
}
METALLIC = {"steel": 0.6, "iron": 0.5, "gold": 0.5}
ROUGH = {"steel": 0.42, "iron": 0.55, "gold": 0.45, "fire": 0.6,
         "water": 0.45}


def new_palette():
    """Reset the Blender scene and create one Principled material per palette key."""
    gam.reset()
    mats, idx = [], {}
    for key, hexv in PAL.items():
        m = gam.material("m_" + key, gam.hex_rgba(hexv),
                         ROUGH.get(key, 0.85), METALLIC.get(key, 0.0))
        if key == "fire":
            bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
            bsdf.inputs["Emission Color"].default_value = gam.hex_rgba(PAL[key])
            bsdf.inputs["Emission Strength"].default_value = 0.9
        idx[key] = len(mats)
        mats.append(m)
    return idx, mats


class Builder:
    """One bmesh assembly -> one flat-shaded, UV-packed, bottom-pivoted mesh."""

    def __init__(self, idx, mats):
        self.idx, self.mats = idx, mats
        self.bm = bmesh.new()
        self.bevel_edges = []

    def _record(self, faces, bevel=False):
        if bevel:
            for f in faces:
                self.bevel_edges.extend(f.edges)

    # -- low-level parts ---------------------------------------------------
    def box(self, lo, hi, mat, bevel=False, skip=()):
        faces = gam.bm_box(self.bm, lo, hi, self.idx[mat], skip=skip)
        self._record(faces, bevel)
        return faces

    def box_at(self, center, size, mat, bevel=False, skip=()):
        c, s = Vector(center), Vector(size) * 0.5
        return self.box(tuple(c - s), tuple(c + s), mat, bevel, skip)

    def cyl(self, center, r, z0, z1, mat, n=6, r1=None, bevel=False,
            axis=(0, 0, 1)):
        if tuple(axis) == (0, 0, 1):
            start = Vector(center) + Vector((0, 0, z0))
            end = Vector(center) + Vector((0, 0, z1))
        else:
            # For a custom axis, z0/z1 are signed distances from `center`.
            d = Vector(axis).normalized()
            start = Vector(center) + d * z0
            end = Vector(center) + d * z1
        direction = (end - start).normalized()
        rot = Vector((0, 0, 1)).rotation_difference(direction).to_matrix().to_4x4()
        mat4 = Matrix.Translation(start) @ rot
        faces = gam.bm_cylinder(self.bm, r, 0, (end - start).length, n=n,
                                r1=r1, matrix=mat4, mat=self.idx[mat])
        self._record(faces, bevel)
        return faces

    def limb(self, p0, p1, r0, r1, mat, n=6):
        p0, p1 = Vector(p0), Vector(p1)
        return self.cyl(p0, r0, 0, (p1 - p0).length, mat, n=n, r1=r1,
                        axis=tuple(p1 - p0))

    def ico(self, center, r, mat, scale=(1, 1, 1), subdiv=0, bevel=False):
        bm = self.bm
        before = len(bm.verts)
        bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=r,
                                   matrix=Matrix.Translation(Vector(center)))
        verts = list(bm.verts)[before:]
        c = Vector(center)
        for v in verts:
            d = v.co - c
            v.co = c + Vector((d.x * scale[0], d.y * scale[1], d.z * scale[2]))
        faces = []
        for v in verts:
            for f in v.link_faces:
                if f not in faces and all(x in verts for x in f.verts):
                    faces.append(f)
        for f in faces:
            f.material_index = self.idx[mat]
        lay = bm.loops.layers.uv.get("UVMap") or bm.loops.layers.uv.new("UVMap")
        for i, f in enumerate(faces):
            ax = max(range(3), key=lambda k: abs(f.normal[k]))
            for l in f.loops:
                p = l.vert.co
                u, v = ((p.y, p.z) if ax == 0 else
                        (p.x, p.z) if ax == 1 else (p.x, p.y))
                l[lay].uv = (0.5 + 0.3 * u + 0.02 * (i % 8),
                             0.5 + 0.3 * v + 0.02 * (i // 8))
        self._record(faces, bevel)
        return faces

    def prism(self, poly, offset, mat):
        """Extrude a planar XY outline along `offset` (for capes, fins, crests)."""
        bm = self.bm
        pts = [bm.verts.new((x, y, 0)) for x, y in poly]
        top = [bm.verts.new((x + offset[0], y + offset[1], offset[2]))
               for x, y in poly]
        faces = []
        for verts in (pts, list(reversed(top))):
            f = bm.faces.new(verts); f.material_index = self.idx[mat]
            faces.append(f)
        for i in range(len(pts)):
            j = (i + 1) % len(pts)
            f = bm.faces.new((pts[i], pts[j], top[j], top[i]))
            f.material_index = self.idx[mat]
            faces.append(f)
        self._record(faces)
        return faces

    # -- finishing ---------------------------------------------------------
    def finish(self, name, bevel_width=0.006, object_name=None, pivot=True):
        bm = self.bm
        if self.bevel_edges and bevel_width > 0:
            layer = bm.edges.layers.float.get("bevel_weight_edge")
            if layer is None:
                layer = bm.edges.layers.float.new("bevel_weight_edge")
            for e in set(self.bevel_edges):
                if e.is_valid:
                    e[layer] = 1.0
        if not bm.loops.layers.uv.get("UVMap"):
            bm.loops.layers.uv.new("UVMap")
        bm.normal_update()
        me = bpy.data.meshes.new(name + "_mesh")
        bm.to_mesh(me)
        bm.free()
        for m in self.mats:
            me.materials.append(m)
        obj = bpy.data.objects.new(object_name or (name + "_lod0"), me)
        gam.link(obj)
        if self.bevel_edges and bevel_width > 0:
            print("    BEVEL", name, flush=True)
            gam.bevel(obj, bevel_width, 1, 35.0, limit="WEIGHT")
        print("    NORMALS", name, flush=True)
        gam.shade_hard_surface(obj, 35.0, True)
        gam.triangulate_ngons(obj)
        print("    UVPACK", name, flush=True)
        pack_uv(obj)
        print("    PIVOT", name, flush=True)
        if pivot:
            gam.set_pivot(obj, "bottom_center")
        return obj


def normalise(objs):
    """Shift all objects so the union base is z=0 and the footprint is centred."""
    pts = [o.matrix_world @ Vector(c) for o in objs for c in o.bound_box]
    lo = [min(p[i] for p in pts) for i in range(3)]
    hi = [max(p[i] for p in pts) for i in range(3)]
    off = Vector((-(lo[0] + hi[0]) / 2, -(lo[1] + hi[1]) / 2, -lo[2]))
    for o in objs:
        o.location += off
    return off


def pack_uv(obj):
    """Deterministic UV pack: one non-overlapping tile per face.

    Assets are texture-free (flat palette only), so a single uniform-density
    island grid is sufficient and avoids Blender's pack_islands stalling on the
    degenerate UVs of tiny cone tips.
    """
    me = obj.data
    lay = me.uv_layers.get("UVMap") or me.uv_layers.new(name="UVMap")
    polys = list(me.polygons)
    cols = max(1, math.ceil(math.sqrt(max(1, len(polys)))))
    cell = 1.0 / cols
    for i, poly in enumerate(polys):
        ax = max(range(3), key=lambda k: abs(poly.normal[k]))
        pts = [(v.co.x, v.co.y, v.co.z) for v in
               (me.vertices[vi] for vi in poly.vertices)]
        proj = [(p[1], p[2]) if ax == 0 else
                (p[0], p[2]) if ax == 1 else (p[0], p[1]) for p in pts]
        us = [q[0] for q in proj]; vs = [q[1] for q in proj]
        min_u, max_u = min(us), max(us)
        min_v, max_v = min(vs), max(vs)
        su = max(max_u - min_u, 1e-4)
        sv = max(max_v - min_v, 1e-4)
        ox, oy = (i % cols) * cell, (i // cols) * cell
        for li in poly.loop_indices:
            u, v = proj[li - poly.loop_start]
            lay.data[li].uv = (ox + 0.06 * cell + 0.88 * cell * (u - min_u) / su,
                               oy + 0.06 * cell + 0.88 * cell * (v - min_v) / sv)
    return len(polys)


def export(objs, name):
    if not isinstance(objs, (list, tuple)):
        objs = [objs]
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    path = os.path.join(MODELS, name + ".glb")
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB",
                              use_selection=True, export_apply=True,
                              export_yup=True)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in objs)
    print("REBUILT", name, tris, "tris", len(objs), "nodes ->", path, flush=True)
    return path, tris
