"""Headless Workbench frame strips for the final player clips.

    blender -b --python scripts/render_player_final.py
"""

import json
import os

import bpy
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = "/tmp/opencode/player_frames"
os.makedirs(OUT, exist_ok=True)

SHOTS = {
    "idle": [0, 30, 60, 90],
    "walk": [0, 6, 12, 18],
    "run": [0, 5, 10, 14],
    "attack_1": [0, 8, 14, 20, 35],
    "attack_2": [0, 8, 14, 20, 35],
    "attack_3": [0, 10, 18, 26, 40],
    "heavy_attack": [0, 14, 26, 34, 50],
    "block": [0, 10, 24, 45],
    "dodge_roll": [0, 8, 18, 28, 40],
    "hit_light": [0, 4, 8, 20],
    "hit_heavy": [0, 5, 12, 28],
    "death": [0, 15, 30, 45, 65],
    "interact": [0, 10, 20, 35],
}

with open(os.path.join(ROOT, "assets", "animations", "player", "manifest.json")) as fh:
    manifest = json.load(fh)
paths = {c["clip"]: os.path.join(ROOT, c["path"]) for c in manifest["clips"]}

for name, frames in SHOTS.items():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.render.fps = 30
    bpy.ops.import_scene.gltf(filepath=paths[name])
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = 320
    scene.render.resolution_y = 320
    scene.render.image_settings.file_format = "PNG"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.color_type = "MATERIAL"
    scene.display.shading.show_shadows = True
    scene.display.shading.background_type = "VIEWPORT"
    scene.display.shading.background_color = (0.32, 0.34, 0.36)
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = 2.3
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    cam.location = (2.4, -2.4, 2.4)
    cam.rotation_euler = (Vector((0, -0.1, 0.78)) - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam
    for f in frames:
        scene.frame_set(f)
        scene.render.filepath = os.path.join(OUT, "%s_%03d.png" % (name, f))
        bpy.ops.render.render(write_still=True)
    print("rendered", name)
print("frames in", OUT)
