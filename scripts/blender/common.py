"""Shared helpers for Starfall low-poly asset generators.

Read docs/ART_DIRECTION.md first. Every generator imports this module:

    import sys, os
    sys.path.insert(0, os.path.dirname(__file__))
    from common import new_scene, mat, collection, box, cyl, cone, ico, export_asset
"""
import bpy
import os

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = os.path.join(ROOT, "assets", "models")
PREVIEWS = os.path.join(ROOT, "assets", "previews")
os.makedirs(OUT, exist_ok=True)
os.makedirs(PREVIEWS, exist_ok=True)

# Palette from docs/ART_DIRECTION.md
C = {
    "player": "#FF8C26", "skin": "#EAB88C",
    "wood": "#6B4423", "leather": "#592F1A",
    "stone": "#7A7F87", "iron": "#B8C0C8", "steel": "#C8D0D8",
    "gold": "#D9A62E", "leaf": "#3A7A33", "grass": "#5B8C3A",
    "sand": "#D9B46A", "snow": "#E8F0FA", "swamp": "#4A6741",
    "water": "#3A7CA5", "arcane": "#6B3FA0", "fire": "#FF6A2A",
    "enemy": "#C0392B", "bone": "#E8E0CC", "cloth": "#39527A",
}


def new_scene():
    """Start from an empty scene."""
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for c in list(bpy.data.collections):
        if c.name != "Collection":
            bpy.data.collections.remove(c)


def rgb(hex_color):
    h = hex_color.lstrip("#")
    return tuple(int(h[i:i + 2], 16) / 255.0 for i in (0, 2, 4))


def mat(name, hex_color, rough=0.85, metallic=0.0, emission=None, strength=0.0):
    """Flat-colour Principled material. Emission is optional."""
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bsdf.inputs["Base Color"].default_value = (*rgb(hex_color), 1.0)
    bsdf.inputs["Roughness"].default_value = rough
    bsdf.inputs["Metallic"].default_value = metallic
    if emission:
        bsdf.inputs["Emission Color"].default_value = (*rgb(emission), 1.0)
        bsdf.inputs["Emission Strength"].default_value = strength
    m.diffuse_color = (*rgb(hex_color), 1.0)
    return m


def collection(name):
    c = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(c)
    return c


def _finish(obj, col, material):
    for old in list(obj.users_collection):
        old.objects.unlink(obj)
    col.objects.link(obj)
    obj.data.materials.append(material)
    for p in obj.data.polygons:
        p.use_smooth = False
    return obj


def box(name, loc, scale, material, col):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    o = bpy.context.object
    o.name = name
    o.scale = scale
    return _finish(o, col, material)


def cyl(name, loc, radius, depth, material, col, vertices=8):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius,
                                        depth=depth, location=loc)
    o = bpy.context.object
    o.name = name
    return _finish(o, col, material)


def cone(name, loc, r1, r2, depth, material, col, vertices=8, rotation=None):
    bpy.ops.mesh.primitive_cone_add(vertices=vertices, radius1=r1, radius2=r2,
                                    depth=depth, location=loc)
    o = bpy.context.object
    o.name = name
    if rotation:
        o.rotation_euler = rotation
    return _finish(o, col, material)


def ico(name, loc, radius, material, col, subdivisions=1, scale=None):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=subdivisions,
                                          radius=radius, location=loc)
    o = bpy.context.object
    o.name = name
    if scale:
        o.scale = scale
    return _finish(o, col, material)


def export_asset(name, col):
    """Export one collection to assets/models/<name>.glb (Y-up, flat)."""
    bpy.ops.object.select_all(action="DESELECT")
    for o in col.objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = next(iter(col.objects))
    path = os.path.join(OUT, name + ".glb")
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB",
                              use_selection=True, export_apply=True,
                              export_yup=True)
    n = sum(len(o.data.polygons) * 2 for o in col.objects if o.type == "MESH")
    print("EXPORTED", name, n, "tris ->", path)
    return path


def contact_sheet(sets, filename, width=1400, height=420):
    """Optional: workbench-render a row of all assets for visual review.

    sets: list of (name, collection) built at origin. They are spread on X.
    """
    for i, (_, col) in enumerate(sets):
        for o in col.objects:
            o.location.x += i * 2.4
    bpy.ops.object.camera_add(location=(len(sets) * 1.2, -14, 9))
    cam = bpy.context.object
    from mathutils import Vector
    d = Vector((len(sets) * 1.2, 0, 1.2)) - cam.location
    cam.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
    cam.data.type = "ORTHO"
    cam.data.ortho_scale = len(sets) * 2.6
    bpy.context.scene.camera = cam
    bpy.ops.object.light_add(type="AREA", location=(0, -4, 10))
    bpy.context.object.data.energy = 3000
    bpy.context.object.data.size = 12
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.display.shading.light = "STUDIO"
    sc.display.shading.color_type = "MATERIAL"
    sc.render.resolution_x = width
    sc.render.resolution_y = height
    sc.render.image_settings.file_format = "PNG"
    sc.render.filepath = os.path.join(PREVIEWS, filename)
    bpy.ops.render.render(write_still=True)
    print("PREVIEW", sc.render.filepath)
