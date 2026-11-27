"""Wave-5 palette additions and bmesh builders for the hero interiors.

The long tail of props uses the lightweight ``common.py`` primitives (as in
waves 3 and 4). The three hero interiors are authored with bmesh primitives so
they go through the game-asset-modeling finishing path explicitly:

    bmesh forms -> selective bevel -> weighted normals -> flat shading
                -> UV pack  (``expansion_common.hero_finish``)

Colours are lightness variants of the ART_DIRECTION.md palette; no new hues
and no textures.
"""
import bmesh
import bpy
from mathutils import Euler, Matrix, Vector

from common import C, mat
from expansion_common import P


def palette_v5():
    """Palette shades needed by wave 5, layered on the expansion palette."""
    V = dict(P)
    V["obsidian"] = mat("v5_obsidian", "#333840", rough=0.7)
    V["char"] = mat("v5_charcoal", "#2F333A", rough=0.9)
    V["ash"] = mat("v5_ash", "#6E6A66", rough=0.95)
    V["frost"] = mat("v5_frost", "#A8C6DC", rough=0.45)
    V["rust"] = mat("v5_rust", "#8C5A32", rough=0.9)
    V["bone_dark"] = mat("v5_bone_dark", "#C9BFA6", rough=0.9)
    V["copper"] = mat("v5_copper", "#B87333", rough=0.5, metallic=0.35)
    V["crimson"] = mat("v5_crimson", "#9E2F28", rough=0.85)
    V["amber"] = mat("v5_amber", "#D9A62E", rough=0.5,
                     emission="#FF6A2A", strength=0.25)
    # The shared wave-3/4 emission (0.5-0.6) washes saturated accents out to
    # pastel in Babylon. Wave 5 keeps the palette hue readable and reserves a
    # strong glow for a tiny inner facet only.
    V["fire"] = mat("v5_fire", C["fire"], emission=C["fire"], strength=0.15)
    V["arcane"] = mat("v5_arcane", C["arcane"], emission=C["arcane"],
                      strength=0.2)
    V["arcane_light"] = mat("v5_arcane_soft", C["arcane"],
                            emission=C["arcane"], strength=0.12)
    V["arcane_core"] = mat("v5_arcane_core", C["arcane"],
                           emission=C["arcane"], strength=0.6)
    return V


V = palette_v5()


def _matrix(loc, scale=(1.0, 1.0, 1.0), rot=(0.0, 0.0, 0.0)):
    return Matrix.LocRotScale(Vector(loc), Euler(rot, "XYZ").to_quaternion(),
                              Vector(scale))


def _new_object(name, bm, material, col):
    bm.normal_update()
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    uv = bm.loops.layers.uv.verify()
    for face in bm.faces:
        n = face.normal
        for loop in face.loops:
            co = loop.vert.co
            if abs(n.z) >= abs(n.x) and abs(n.z) >= abs(n.y):
                u, v = co.x, co.y
            elif abs(n.x) >= abs(n.y):
                u, v = co.y, co.z
            else:
                u, v = co.x, co.z
            loop[uv].uv = (u, v)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(material)
    for poly in me.polygons:
        poly.use_smooth = False
    obj = bpy.data.objects.new(name, me)
    col.objects.link(obj)
    return obj


def bm_box(name, loc, scale, key, col, rot=(0.0, 0.0, 0.0)):
    """A box authored directly as bmesh geometry (1u grid friendly)."""
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0, matrix=_matrix(loc, scale, rot))
    return _new_object(name, bm, V[key], col)


def bm_cone(name, loc, r1, r2, depth, key, col, verts=6,
            rot=(0.0, 0.0, 0.0)):
    """A faceted frustum/cone authored directly as bmesh geometry."""
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=verts,
                          radius1=r1, radius2=r2, depth=depth,
                          matrix=_matrix(loc, (1.0, 1.0, 1.0), rot))
    return _new_object(name, bm, V[key], col)
