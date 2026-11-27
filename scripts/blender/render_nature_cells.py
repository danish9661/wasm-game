"""Render one gameplay-scale cell per nature/biome model for the visual QA sheet.

Every cell shows the model at TRUE scale (1 unit = 1 tile) beside a 1.55u
player silhouette, on a 1u checkerboard, from the game's 45deg / 30deg
orthographic camera. This makes wrong scale and 60px unreadability obvious.

Usage:
    ~/.local/blender/blender -b --python scripts/blender/render_nature_cells.py -- \
        --out /tmp/opencode/nature_qa/cells_before

The cell PNGs are composed into labelled contact sheets by
scripts/build_nature_sheet.py (CPython + PIL).
"""
import argparse
import json
import math
import os
import sys

import bpy
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
MODELS = os.path.join(ROOT, "assets", "models")
sys.path.insert(0, HERE)
from common import C, mat  # noqa: E402

CELL_W, CELL_H = 440, 380
# Human offset along the camera's screen-right vector (world +X/+Y diagonal).
HUMAN_XY = 1.05
CAM_TARGET_Z = 1.02
ORTHO_SCALE = 4.6  # horizontal world units visible in a cell


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    p = argparse.ArgumentParser()
    p.add_argument("--out", required=True)
    p.add_argument("--only", default=None, help="comma-separated model names")
    p.add_argument("--family", default=os.path.join(HERE, "nature_family.json"))
    return p.parse_args(argv)


def clear_scene():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for c in list(bpy.data.collections):
        if c.name != "Collection":
            bpy.data.collections.remove(c)
    for m in list(bpy.data.materials):
        if m.users == 0:
            bpy.data.materials.remove(m)


def import_glb(path, hide_colliders=False):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    objs = [o for o in bpy.data.objects if o not in before]
    if hide_colliders:
        # Gam-built GLBs carry an invisible-in-engine collider mesh. Hide it so
        # the QA sheet shows only what the game renderer draws.
        for o in objs:
            if (o.name.endswith("_col") or o.name.endswith("_col_src")
                    or o.get("ig_collider") is not None
                    or (o.type == "MESH" and not o.data.materials)):
                o.hide_render = True
    return objs


def tame_emission():
    """Clamp HDR emission in the audit harness so the base hue stays legible.

    Several generated materials use 1.4 strength, which clips to white under a
    Standard view transform. The in-game renderer tone-maps this; the sheet
    needs the colour identity, not the bloom.
    """
    for m in bpy.data.materials:
        if not m.use_nodes or not m.node_tree:
            continue
        for n in m.node_tree.nodes:
            if n.type == "BSDF_PRINCIPLED":
                s = n.inputs.get("Emission Strength")
                if s and s.default_value > 0.55:
                    s.default_value = 0.45


def build_ground():
    """1u checkerboard like the game's tile grid, plus a faint plinth edge."""
    a = mat("qa_tile_a", "#6F7469", rough=1.0)
    b = mat("qa_tile_b", "#797E72", rough=1.0)
    for i in range(-3, 4):
        for j in range(-3, 4):
            bpy.ops.mesh.primitive_cube_add(size=1.0, location=(i + 0.5, j + 0.5, -0.06))
            o = bpy.context.object
            o.scale = (0.99, 0.99, 0.10)
            o.data.materials.append(a if (i + j) % 2 == 0 else b)


def build_human():
    """A neutral 1.55u player stand-in, imported from player.glb and flattened."""
    path = os.path.join(MODELS, "player.glb")
    if os.path.isfile(path):
        objs = import_glb(path)
        grey = mat("qa_silhouette", "#39414D", rough=1.0)
        bsdf = next((n for n in grey.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
        if bsdf:
            bsdf.inputs["Emission Color"].default_value = (0.16, 0.19, 0.24, 1.0)
            bsdf.inputs["Emission Strength"].default_value = 0.6
        for o in objs:
            if o.type == "MESH":
                o.data.materials.clear()
                o.data.materials.append(grey)
        roots = [o for o in objs if o.parent is None or o.parent not in objs]
        for o in roots:
            o.location = Vector(o.location) + Vector((HUMAN_XY, HUMAN_XY, 0))
        return
    # Fallback built from primitives if player.glb is unavailable.
    grey = mat("qa_silhouette", "#39414D", rough=1.0)
    parts = [("head", (HUMAN_XY, HUMAN_XY, 1.40), 0.11),
             ("torso", (HUMAN_XY, HUMAN_XY, 0.95), 0.28),
             ("legs", (HUMAN_XY, HUMAN_XY, 0.35), 0.25)]
    for name, loc, r in parts:
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1, radius=r, location=loc)
        bpy.context.object.data.materials.append(grey)


def setup_lights_camera():
    sun = bpy.data.lights.new("qa_sun", type="SUN")
    sun.energy = 1.8
    sun.angle = math.radians(8)
    so = bpy.data.objects.new("qa_sun", sun)
    bpy.context.collection.objects.link(so)
    so.rotation_euler = (math.radians(35), 0, math.radians(35))

    fill = bpy.data.lights.new("qa_fill", type="AREA")
    fill.energy = 850
    fill.size = 14
    fill.color = (0.68, 0.78, 1.0)
    fo = bpy.data.objects.new("qa_fill", fill)
    bpy.context.collection.objects.link(fo)
    fo.location = (0, 0, 8)

    world = bpy.data.worlds.new("qa_world")
    world.use_nodes = True
    bg = next(n for n in world.node_tree.nodes if n.type == "BACKGROUND")
    bg.inputs["Color"].default_value = (0.22, 0.24, 0.27, 1)
    bg.inputs["Strength"].default_value = 0.5
    bpy.context.scene.world = world

    cam_data = bpy.data.cameras.new("qa_cam")
    cam = bpy.data.objects.new("qa_cam", cam_data)
    bpy.context.collection.objects.link(cam)
    bpy.context.scene.camera = cam
    pitch, yaw, dist = math.radians(30), math.radians(45), 12.0
    target = Vector((0.0, 0.0, CAM_TARGET_Z))
    cam.location = target + Vector((
        dist * math.cos(pitch) * math.cos(yaw),
        -dist * math.cos(pitch) * math.sin(yaw),
        dist * math.sin(pitch)))
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = ORTHO_SCALE


def setup_render(out_path):
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_EEVEE"
    sc.render.resolution_x = CELL_W
    sc.render.resolution_y = CELL_H
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = "PNG"
    sc.render.film_transparent = False
    sc.render.filepath = out_path
    sc.view_settings.view_transform = "Standard"
    try:
        sc.view_settings.look = "Medium High Contrast"
    except TypeError:
        sc.view_settings.look = "None"


def main():
    args = parse_args()
    os.makedirs(args.out, exist_ok=True)
    with open(args.family, encoding="utf-8") as f:
        fam = json.load(f)
    names = fam["models"]
    if args.only:
        wanted = set(args.only.split(","))
        names = [n for n in names if n in wanted]
    for name in names:
        path = os.path.join(MODELS, name + ".glb")
        if not os.path.isfile(path):
            print("SKIP missing", name)
            continue
        clear_scene()
        objs = import_glb(path, hide_colliders=True)
        tame_emission()
        # Re-centre the imported model on (0,0) in XY and put base at z=0.
        pts = [o.matrix_world @ Vector(c) for o in objs
               if o.type == "MESH" and not o.hide_render for c in o.bound_box]
        if pts:
            cx = (min(p.x for p in pts) + max(p.x for p in pts)) / 2
            cy = (min(p.y for p in pts) + max(p.y for p in pts)) / 2
            zmin = min(p.z for p in pts)
            roots = [o for o in objs if o.parent is None or o.parent not in objs]
            for o in roots:
                o.location = Vector(o.location) + Vector((-cx, -cy, -zmin))
        build_ground()
        build_human()
        setup_lights_camera()
        cell = os.path.join(args.out, name + ".png")
        setup_render(cell)
        bpy.ops.render.render(write_still=True)
        print("CELL", name, cell)
    print("CELLS_DONE", len(names), args.out)


if __name__ == "__main__":
    main()
