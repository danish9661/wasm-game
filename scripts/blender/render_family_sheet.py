"""Headless Blender renderer for the building/structure family QA sheets.

Every model is rendered at a FIXED gameplay scale: 1 Blender unit = 1 tile,
the ortho camera's vertical field is GAMEPLAY_VIEW units, and a 1.55 u human
silhouette stands beside each model. No per-model zoom is applied; tall models
are framed by moving the camera target up, never by changing the scale.

Run:
    ~/.local/blender/blender -b --python scripts/blender/render_family_sheet.py -- \
        --out /tmp/opencode/qa/cells

Then compose the cell PNGs with scripts/make_family_sheet.py.
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
sys.path.insert(0, HERE)
from common import mat, C, ROOT as COMMON_ROOT  # noqa: E402

METRICS = os.path.join(ROOT, "assets", "models", "metrics.json")
MODELS = os.path.join(ROOT, "assets", "models")

# The family under audit: buildings + structures + interiors + kit pieces.
FAMILY_CATEGORIES = {"Building", "Structure", "Kit Expansion"}
FAMILY_EXTRA = {
    "altar", "portal", "wall", "well", "well_bucket", "fence", "fence_gate",
    "stone_wall_corner", "turret", "altar", "blood_altar", "brazier",
    "crypt_brazier", "banner", "torch_sconce", "lantern", "sandstone_pillar",
    "signpost", "sign", "farm_windmill", "farm_plot", "farm_cart",
    "vendor_cart", "handcart", "car", "train", "rail", "spike", "statue",
    "campfire", "totem", "healing_totem", "anvil", "chest", "bed",
    "barrel_open", "barrel_stacked", "enchanting_table",
}

GAMEPLAY_VIEW = 5.6          # vertical ortho extent in tiles
CELL_W, CELL_H = 420, 320    # px -> 57 px per tile; player ~= 89 px
HUMAN_H = 1.55
HUMAN_X = 1.45               # silhouette placed to the right of the model


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    p = argparse.ArgumentParser()
    p.add_argument("--out", default="/tmp/opencode/qa/cells")
    p.add_argument("--names", default="", help="optional comma-separated names")
    p.add_argument("--suffix", default="", help="suffix for cell filenames")
    return p.parse_args(argv)


def clear_objects():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)


def make_materials():
    return {
        "ground": mat("QA_Grass", "#6B8C57"),
        "tile": mat("QA_Tile", "#9AA38C"),
        "human": mat("QA_Human", "#1A1F26"),
        "line": mat("QA_Grid", "#4C6048"),
    }


def add_cube(name, loc, scale, material, rotation=None):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    o = bpy.context.object
    o.name = name
    o.scale = scale
    if rotation:
        o.rotation_euler = rotation
    o.data.materials.append(material)
    for p in o.data.polygons:
        p.use_smooth = False
    return o


def add_human(materials):
    """A deliberately plain 1.55 u tall human silhouette (head, body, limbs)."""
    m = materials["human"]
    group = []
    group.append(add_cube("qa_head", (HUMAN_X, 0, 1.39), (0.20, 0.19, 0.24), m))
    # tapered torso approximated by two stacked blocks
    group.append(add_cube("qa_torso", (HUMAN_X, 0, 1.06), (0.30, 0.17, 0.43), m))
    group.append(add_cube("qa_hips", (HUMAN_X, 0, 0.79), (0.24, 0.16, 0.14), m))
    for dy in (-0.11, 0.11):
        group.append(add_cube("qa_arm", (HUMAN_X, dy, 1.02), (0.09, 0.07, 0.50), m))
    for dx in (-0.07, 0.07):
        group.append(add_cube("qa_leg", (HUMAN_X + dx, 0, 0.35), (0.10, 0.11, 0.70), m))
    return group


def add_ground(materials):
    add_cube("qa_ground", (0.25, 0, -0.06), (7.0, 6.0, 0.1), materials["ground"])
    # 1x1 reference tile directly under the model, exactly one unit across.
    add_cube("qa_tile", (0, 0, -0.005), (1.0, 1.0, 0.01), materials["tile"])
    for dx in (-0.5, 0.5):
        add_cube("qa_grid_x", (dx, 0, 0.002), (0.012, 1.0, 0.004), materials["line"])
    for dy in (-0.5, 0.5):
        add_cube("qa_grid_y", (0, dy, 0.002), (1.0, 0.012, 0.004), materials["line"])


def add_lights_camera(height):
    # Tall landmark models need a wider vertical field, otherwise their roof
    # is cropped. The HUMAN and the model share the same field in every cell,
    # so their RELATIVE gameplay scale stays exact; the sheet records the zoom
    # factor for any cell wider than the standard view.
    bpy.ops.object.light_add(type="SUN", location=(4, -6, 9))
    sun = bpy.context.object
    sun.data.energy = 2.2
    sun.data.angle = math.radians(6)
    sun.rotation_euler = (math.radians(32), 0, math.radians(38))

    bpy.ops.object.light_add(type="AREA", location=(-3, 2, 7))
    fill = bpy.context.object
    fill.data.energy = 900
    fill.data.size = 9
    fill.data.color = (0.72, 0.82, 1.0)

    world = bpy.data.worlds.new("QA_World")
    world.use_nodes = True
    bg = next(n for n in world.node_tree.nodes if n.type == "BACKGROUND")
    bg.inputs["Color"].default_value = (0.34, 0.38, 0.44, 1)
    bg.inputs["Strength"].default_value = 0.8
    bpy.context.scene.world = world

    # Art direction: yaw 45 deg, pitch 30 deg, orthographic.
    zoom = max(1.0, (height + 0.8) / GAMEPLAY_VIEW)
    view = GAMEPLAY_VIEW * zoom
    lift = (view - GAMEPLAY_VIEW) * 0.5
    cam_pos = Vector((7.0, -7.0, 4.04 + lift))
    target = Vector((0.25, 0, 0.95 + lift))
    bpy.ops.object.camera_add(location=cam_pos)
    cam = bpy.context.object
    cam.rotation_euler = (target - cam_pos).to_track_quat("-Z", "Y").to_euler()
    cam.data.type = "ORTHO"
    cam.data.ortho_scale = view
    bpy.context.scene.camera = cam
    return zoom


def render_model(name, height, out_dir, suffix):
    clear_objects()
    materials = make_materials()
    path = os.path.join(MODELS, name + ".glb")
    if not os.path.isfile(path):
        print("QA_SKIP missing", name)
        return
    bpy.ops.import_scene.gltf(filepath=path)
    add_ground(materials)
    add_human(materials)
    add_lights_camera(height)

    sc = bpy.context.scene
    # Workbench with material colours is fast enough for a 100+ model family
    # and keeps the flat diorama look; EEVEE is used only for beauty shots.
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.display.shading.light = "STUDIO"
    sc.display.shading.color_type = "MATERIAL"
    sc.display.shading.show_shadows = True
    sc.display.shading.show_cavity = True
    sc.render.resolution_x = CELL_W
    sc.render.resolution_y = CELL_H
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = "PNG"
    sc.render.film_transparent = False
    sc.view_settings.view_transform = "Standard"
    sc.view_settings.look = "Medium High Contrast"
    sc.render.filepath = os.path.join(
        out_dir, ("%s%s.png" % (name, suffix)))
    bpy.ops.render.render(write_still=True)
    print("QA_CELL", name, "h=%.2f" % height, sc.render.filepath)


def main():
    args = parse_args()
    os.makedirs(args.out, exist_ok=True)
    report = json.loads(open(METRICS, encoding="utf-8").read())
    models = {m["name"]: m for m in report["models"]}
    if args.names:
        names = [n.strip() for n in args.names.split(",") if n.strip()]
    else:
        names = [n for n, m in models.items()
                 if m["category"] in FAMILY_CATEGORIES or n in FAMILY_EXTRA]
    print("QA_FAMILY_SIZE", len(names))
    for i, name in enumerate(names):
        height = models.get(name, {}).get("height", 2.5)
        render_model(name, height, args.out, args.suffix)
    with open(os.path.join(args.out, "family_names%s.json" % args.suffix), "w") as f:
        json.dump(names, f)


if __name__ == "__main__":
    main()
