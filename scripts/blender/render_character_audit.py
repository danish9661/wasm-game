"""Headless Blender renders of every character-family GLB at two fixed scales.

    ~/.local/blender/blender -b --python scripts/blender/render_character_audit.py
       -- [label]        label defaults to "before" (output subfolder)

Gameplay scale: ortho 5.2 u across a 220 px frame => a 1.55 u human is ~66 px.
Zoom scale:     ortho 3.2 u across a 320 px frame => a 1.55 u human is ~155 px.
A neutral 1.55 u human silhouette stands at the model's right in every frame.

Output: assets/qa/characters/<label>/{gameplay,zoom}/<name>.png
"""
import json
import os
import sys

import bpy
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
FAMILY = os.path.join(ROOT, "assets", "qa", "characters", "family.json")
MODELS = os.path.join(ROOT, "assets", "models")
label = sys.argv[sys.argv.index("--") + 1] if "--" in sys.argv else "before"
OUT = os.path.join(ROOT, "assets", "qa", "characters", label)
for d in ("gameplay", "zoom"):
    os.makedirs(os.path.join(OUT, d), exist_ok=True)

names = [e["name"] for e in json.load(open(FAMILY))["family"]]
if "--" in sys.argv:
    args = sys.argv[sys.argv.index("--") + 1:]
    if len(args) > 1:
        names = [n for n in names if n in set(args[1:])]

SCALES = {
    "gameplay": dict(size=220, ortho=5.2, ref_x=1.35, target=(0.0, 0.0, 0.78)),
    "zoom":     dict(size=320, ortho=3.2, ref_x=0.98, target=(0.0, 0.0, 0.78)),
}


def human(col, x):
    """Neutral 1.55 u human: boots 0.12, legs 0.42, torso 0.55, head to 1.55."""
    m = bpy.data.materials.new("ref_grey")
    m.use_nodes = True
    bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bsdf.inputs["Base Color"].default_value = (0.55, 0.57, 0.60, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.9
    m.diffuse_color = (0.55, 0.57, 0.60, 1.0)

    def mesh(kind, loc, dims, verts=None):
        if kind == "box":
            bpy.ops.mesh.primitive_cube_add(size=1, location=(x + loc[0], loc[1], loc[2]))
        elif kind == "cyl":
            bpy.ops.mesh.primitive_cylinder_add(vertices=verts or 6, radius=1, depth=1,
                                                location=(x + loc[0], loc[1], loc[2]))
        else:
            bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1, radius=loc[3],
                                                  location=(x + loc[0], loc[1], loc[2]))
            o = bpy.context.object
            o.data.materials.append(m)
            if col not in o.users_collection:
                col.objects.link(o)
            for sc in list(o.users_collection):
                if sc != col:
                    sc.objects.unlink(o)
            return o
        o = bpy.context.object
        o.scale = dims
        o.data.materials.append(m)
        if col not in o.users_collection:
            col.objects.link(o)
        for sc in list(o.users_collection):
            if sc != col:
                sc.objects.unlink(o)
        return o

    for s in (-1, 1):
        mesh("box", (s * 0.09, 0.0, 0.06), (0.16, 0.24, 0.12))
        mesh("box", (s * 0.09, 0.0, 0.33), (0.15, 0.16, 0.42))
        mesh("box", (s * 0.235, 0.0, 0.86), (0.10, 0.14, 0.44))
    mesh("cyl", (0.0, 0.0, 0.84), (0.17, 0.17, 0.55))
    mesh("ico", (0.0, 0.0, 1.395, 0.155), None)


def setup(scale):
    cfg = SCALES[scale]
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = cfg["size"]
    scene.render.resolution_y = cfg["size"]
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.color_type = "MATERIAL"
    scene.display.shading.show_shadows = True
    scene.display.shading.background_type = "VIEWPORT"
    scene.display.shading.background_color = (0.19, 0.21, 0.23)
    scene.display.shading.show_cavity = True
    scene.display.shading.cavity_type = "BOTH"
    cam_data = bpy.data.cameras.new("audit_cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = cfg["ortho"]
    cam = bpy.data.objects.new("audit_cam", cam_data)
    scene.collection.objects.link(cam)
    cam.location = (2.6, -2.6, 3.20)
    target = Vector(cfg["target"])
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam
    return cfg


def render_all():
    for name in names:
        path = os.path.join(MODELS, name + ".glb")
        if not os.path.exists(path):
            print("MISSING", path)
            continue
        for scale in SCALES:
            bpy.ops.wm.read_factory_settings(use_empty=True)
            cfg = setup(scale)
            before = set(bpy.data.objects)
            bpy.ops.import_scene.gltf(filepath=path)
            new_objs = [o for o in bpy.data.objects if o not in before]
            imported = [o for o in new_objs if o.type == "MESH"]
            # Normalise: base on z=0, footprint centred on origin.
            pts = [o.matrix_world @ Vector(c) for o in imported for c in o.bound_box]
            lo = [min(p[i] for p in pts) for i in range(3)]
            hi = [max(p[i] for p in pts) for i in range(3)]
            cx, cy, cz = [(lo[i] + hi[i]) / 2 for i in range(3)]
            roots = {o for o in new_objs if o.parent is None}
            for o in roots:
                o.location.x -= cx
                o.location.y -= cy
                o.location.z -= lo[2]
            # glTF base colours -> Workbench material colours.
            for m in bpy.data.materials:
                if m.use_nodes:
                    bsdf = next((n for n in m.node_tree.nodes
                                 if n.type == "BSDF_PRINCIPLED"), None)
                    if bsdf:
                        c = bsdf.inputs["Base Color"].default_value
                        m.diffuse_color = (c[0], c[1], c[2], 1.0)
            human(bpy.context.scene.collection, cfg["ref_x"])
            scene = bpy.context.scene
            scene.render.filepath = os.path.join(OUT, scale, name + ".png")
            bpy.ops.render.render(write_still=True)
        print("AUDIT_RENDERED", name, flush=True)
    print("AUDIT_DONE", label, len(names))


if __name__ == "__main__":
    render_all()
