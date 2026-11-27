"""Headless Blender renderer for the visual QA contact sheets.

Renders every model in the props/weapons/tools/vehicles family at *gameplay
scale*: one Blender unit is one game tile and a 1.55 u human silhouette is
placed beside each model.  The camera is orthographic, yaw 45 deg / pitch
30 deg, matching docs/ART_DIRECTION.md.

Usage:
    ~/.local/blender/blender -b --python scripts/blender/render_gameplay_sheet.py -- \
        --config scripts/qa/family_props.json --out assets/previews/qa_props

Writes one PNG per model into <out>/cells/<group>/<name>.png plus a manifest
(<out>/cells/cells.json).  Compose the labelled sheets with
scripts/assemble_gameplay_sheet.py (CPython + Pillow), since Blender has no PIL.
"""
import argparse
import json
import math
import os
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, os.path.dirname(__file__))
from common import ROOT, C, mat, rgb  # noqa: E402


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    p = argparse.ArgumentParser()
    p.add_argument("--config", default="scripts/qa/family_props.json")
    p.add_argument("--out", default="assets/previews/qa_props")
    p.add_argument("--res", type=int, default=512)
    p.add_argument("--only", default="", help="comma separated model names")
    return p.parse_args(os.path.abspath(a) if a and "/" in a and a.startswith("/") else a
                        for a in argv)


def clear_scene():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)


def new_mat(name, hex_color, emission=None, strength=0.0):
    return mat(name, hex_color, emission=emission, strength=strength)


def add_box(name, loc, scale, material, rot=None):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    o = bpy.context.object
    o.name = name
    o.scale = scale
    if rot:
        o.rotation_euler = rot
    o.data.materials.append(material)
    for p in o.data.polygons:
        p.use_smooth = False
    return o


def add_cone(name, loc, r1, r2, depth, material, vertices=6, rot=None):
    bpy.ops.mesh.primitive_cone_add(vertices=vertices, radius1=r1, radius2=r2,
                                    depth=depth, location=loc)
    o = bpy.context.object
    o.name = name
    if rot:
        o.rotation_euler = rot
    o.data.materials.append(material)
    for p in o.data.polygons:
        p.use_smooth = False
    return o


def add_ico(name, loc, radius, material):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1, radius=radius, location=loc)
    o = bpy.context.object
    o.name = name
    o.data.materials.append(material)
    for p in o.data.polygons:
        p.use_smooth = False
    return o


def world_bbox(objects):
    pts = [o.matrix_world @ Vector(c) for o in objects if o.type == "MESH"
           for c in o.bound_box]
    if not pts:
        return None
    return (Vector((min(p.x for p in pts), min(p.y for p in pts), min(p.z for p in pts))),
            Vector((max(p.x for p in pts), max(p.y for p in pts), max(p.z for p in pts))))


def build_ground(mats):
    for x in range(-3, 4):
        for y in range(-3, 4):
            m = mats["grass_a"] if (x + y) % 2 == 0 else mats["grass_b"]
            add_box("tile_%d_%d" % (x, y), (x + 0.5, y + 0.5, -0.025),
                    (0.98, 0.98, 0.05), m)


def build_human(mats, x):
    """1.55 u tall, flat-shaded human silhouette (orange tunic / skin)."""
    parts = []
    add = lambda o: parts.append(o) or o
    add(add_box("ref_boots", (x, 0.0, 0.06), (0.27, 0.19, 0.12), mats["leather"]))
    for s in (-1, 1):
        add(add_box("ref_leg_%d" % s, (x + s * 0.075, 0, 0.29),
                    (0.105, 0.12, 0.42), mats["trousers"]))
    add(add_cone("ref_tunic", (x, 0, 0.83), 0.20, 0.15, 0.58, mats["tunic"], 6))
    for s in (-1, 1):
        add(add_box("ref_arm_%d" % s, (x + s * 0.23, 0, 0.88),
                    (0.085, 0.11, 0.48), mats["tunic"]))
        add(add_box("ref_hand_%d" % s, (x + s * 0.23, 0, 0.62),
                    (0.075, 0.09, 0.10), mats["skin"]))
    add(add_box("ref_neck", (x, 0, 1.16), (0.08, 0.08, 0.08), mats["skin"]))
    add(add_ico("ref_head", (x, 0, 1.32), 0.115, mats["skin"]))
    add(add_cone("ref_hair", (x, 0, 1.42), 0.118, 0.07, 0.11, mats["hair"], 6))
    return parts


def setup_lighting():
    bpy.ops.object.light_add(type="SUN", location=(4, -5, 9))
    sun = bpy.context.object
    sun.name = "Sun_Key"
    sun.data.energy = 2.4
    sun.data.angle = math.radians(8)
    sun.rotation_euler = (math.radians(32), 0.0, math.radians(38))
    bpy.ops.object.light_add(type="AREA", location=(-4, 3, 6))
    fill = bpy.context.object
    fill.name = "Cool_Fill"
    fill.data.energy = 480
    fill.data.size = 9
    fill.data.color = (0.66, 0.78, 1.0)
    world = bpy.data.worlds.new("QA_Studio")
    world.use_nodes = True
    bg = next(n for n in world.node_tree.nodes if n.type == "BACKGROUND")
    bg.inputs["Color"].default_value = (0.20, 0.22, 0.25, 1.0)
    bg.inputs["Strength"].default_value = 0.8
    bpy.context.scene.world = world


def setup_camera(target, ortho, res):
    bpy.ops.object.camera_add()
    cam = bpy.context.object
    cam.name = "QA_Iso"
    yaw = math.radians(45.0)
    pitch = math.radians(30.0)
    dist = 30.0
    cam.location = target + Vector((dist * math.cos(pitch) * math.cos(yaw),
                                    -dist * math.cos(pitch) * math.sin(yaw),
                                    dist * math.sin(pitch)))
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    cam.data.type = "ORTHO"
    cam.data.ortho_scale = ortho
    bpy.context.scene.camera = cam


def render_model(name, path, group, ortho, res, mats, cell_dir):
    clear_scene()
    before = set(bpy.data.objects)
    try:
        bpy.ops.import_scene.gltf(filepath=path)
    except Exception as exc:  # pragma: no cover - one bad GLB must not stop the sheet
        print("IMPORT_FAIL", name, exc)
        return None
    new_objects = [o for o in bpy.data.objects if o not in before]
    # Skill-built GLBs ship a hidden-in-engine collider mesh. Hide it here too,
    # otherwise its grey box covers the render mesh in the contact sheet.
    for o in new_objects:
        if o.type == "MESH" and (o.name.endswith("_col") or "ig_collider" in o):
            o.hide_render = True
    imported = [o for o in new_objects if o.type == "MESH" and not o.hide_render]
    if not imported:
        print("NO_MESH", name)
        return None
    # Workbench reads material.diffuse_color, while glTF only carries the
    # Principled Base Color. Copy it across so skill-built palette materials
    # show their real colours in the sheet (Babylon review is unaffected).
    for m in {m for o in imported for m in o.data.materials if m}:
        bs = next((n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None) if m.node_tree else None
        if bs:
            c = bs.inputs["Base Color"].default_value
            # glTF stores linear factors; the generator palette writes sRGB
            # fractions into diffuse_color. Convert so both families look the
            # same in Workbench.
            srgb = tuple(v / 12.92 if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055
                         for v in c[:3])
            m.diffuse_color = (*srgb, 1.0)
        m.metallic = 0.0
    bb = world_bbox(imported)
    # Place the human clear of the model's widest X extent.
    human_x = max(0.62, bb[1].x + 0.45)
    human = build_human(mats, human_x)
    build_ground(mats)
    full = world_bbox(imported + human)
    target = Vector(((full[0].x + full[1].x) * 0.5,
                     (full[0].y + full[1].y) * 0.5,
                     max(0.80, full[1].z * 0.46)))
    # Keep the group's fixed gameplay scale, but zoom out just enough for
    # oversized props (market stalls, carts, etc.) to fit the frame.
    yaw = math.radians(45.0)
    pitch = math.radians(30.0)
    dist = 30.0
    cam_pos = target + Vector((dist * math.cos(pitch) * math.cos(yaw),
                               -dist * math.cos(pitch) * math.sin(yaw),
                               dist * math.sin(pitch)))
    forward = (target - cam_pos).normalized()
    right = forward.cross(Vector((0, 0, 1))).normalized()
    up = right.cross(forward).normalized()
    corners = [Vector((x, y, z)) for x in (full[0].x, full[1].x)
               for y in (full[0].y, full[1].y) for z in (full[0].z, full[1].z)]
    span_x = max((p - target).dot(right) for p in corners) - min((p - target).dot(right) for p in corners)
    span_y = max((p - target).dot(up) for p in corners) - min((p - target).dot(up) for p in corners)
    ortho = max(ortho, span_x * 1.12, span_y * 1.12)
    setup_lighting()
    setup_camera(target, ortho, res)
    sc = bpy.context.scene
    # Workbench keeps the flat, faceted palette look of the in-game renderer
    # and is fast enough to shoot the whole family in one pass. Eevee's
    # emissive blow-out also hides the fire colours we need to audit.
    sc.render.engine = "BLENDER_WORKBENCH"
    shading = sc.display.shading
    shading.light = "STUDIO"
    shading.color_type = "MATERIAL"
    shading.show_shadows = True
    shading.show_cavity = True
    shading.cavity_type = "BOTH"
    shading.background_type = "WORLD"
    sc.render.resolution_x = res
    sc.render.resolution_y = res
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = "PNG"
    sc.render.film_transparent = False
    sc.view_settings.view_transform = "Standard"
    sc.view_settings.look = "Medium High Contrast"
    sc.view_settings.exposure = 0.0
    sc.view_settings.gamma = 1.0
    out = os.path.join(cell_dir, group, name + ".png")
    sc.render.filepath = out
    bpy.ops.render.render(write_still=True)
    return {"name": name, "group": group, "cell": os.path.relpath(out, cell_dir),
            "model_min": list(bb[0]), "model_max": list(bb[1]),
            "height": bb[1].z - bb[0].z, "ortho": ortho}


def main():
    args = parse_args()
    config_path = os.path.abspath(os.path.join(ROOT, args.config))
    out_dir = os.path.abspath(os.path.join(ROOT, args.out))
    cell_dir = os.path.join(out_dir, "cells")
    os.makedirs(cell_dir, exist_ok=True)
    config = json.load(open(config_path, encoding="utf-8"))
    mats = {
        "grass_a": new_mat("qa_grass_a", C["grass"]),
        "grass_b": new_mat("qa_grass_b", "#4E7C32"),
        "leather": new_mat("qa_leather", C["leather"]),
        "trousers": new_mat("qa_trousers", "#3C4453"),
        "tunic": new_mat("qa_tunic", C["player"]),
        "skin": new_mat("qa_skin", C["skin"]),
        "hair": new_mat("qa_hair", "#4A2E1A"),
    }
    only = set(x.strip() for x in args.only.split(",") if x.strip())
    records = []
    for group in config["groups"]:
        gname = group["name"]
        os.makedirs(os.path.join(cell_dir, gname), exist_ok=True)
        for name in group["models"]:
            if only and name not in only:
                continue
            path = os.path.join(ROOT, "assets", "models", name + ".glb")
            if not os.path.isfile(path):
                print("MISSING", name)
                continue
            rec = render_model(name, path, gname, group["ortho"], args.res, mats, cell_dir)
            if rec:
                records.append(rec)
                print("CELL", gname, name, "h=%.2f" % rec["height"])
    # One manifest per group: parallel Blender processes never clobber each other.
    manifests = []
    for gname in sorted({r["group"] for r in records}):
        manifest = os.path.join(cell_dir, "cells_%s.json" % gname)
        with open(manifest, "w", encoding="utf-8") as f:
            json.dump({"res": args.res,
                       "models": [r for r in records if r["group"] == gname],
                       "config": config_path}, f, indent=2)
        manifests.append(manifest)
    print("SHEET_CELLS_DONE", len(records), manifests)


if __name__ == "__main__":
    main()
