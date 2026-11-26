"""Headless Workbench frame strips for the combat clips (visual review).

    blender -b assets/animations/player_combat.blend \
        --python scripts/render_combat_review.py

Writes individual PNGs to /tmp/opencode/anim_frames; assemble with
scripts/assemble_combat_sheet.py.
"""

import os

import bpy
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = "/tmp/opencode/anim_frames"
os.makedirs(OUT, exist_ok=True)

SHOTS = [
    ("idle", [0, 30, 60, 90]),
    ("walk_f", [0, 6, 12, 18]),
    ("run_f", [0, 5, 10, 15]),
    ("sprint_f", [0, 4, 8, 12]),
    ("attack_sword_slash", [0, 12, 16, 20, 30, 41]),
    ("block", [0, 8, 20, 40]),
    ("dodge", [0, 6, 12, 20]),
    ("hit", [0, 6, 12, 29]),
    ("jump", [0, 10, 18, 26, 40]),
    ("death", [0, 15, 30, 42, 53]),
    ("climb", [0, 9, 18, 27]),
    ("bow", [0, 12, 24, 40]),
    ("sleep", [0, 30, 60, 90]),
    ("drink", [0, 10, 20, 30]),
    ("eat", [0, 8, 16, 24, 35]),
    ("cook", [0, 15, 30, 45]),
    ("craft", [0, 12, 24, 36]),
    ("door_open", [0, 12, 20, 30]),
    ("interact", [0, 10, 18, 30]),
]

scene = bpy.context.scene
scene.render.engine = "BLENDER_WORKBENCH"
scene.render.resolution_x = 320
scene.render.resolution_y = 320
scene.render.resolution_percentage = 100
scene.render.image_settings.file_format = "PNG"
scene.display.shading.light = "STUDIO"
scene.display.shading.color_type = "MATERIAL"
scene.display.shading.show_shadows = True
scene.display.shading.show_cavity = True
scene.display.shading.background_type = "VIEWPORT"
scene.display.shading.background_color = (0.32, 0.34, 0.36)

cam_data = bpy.data.cameras.new("review_cam")
cam_data.type = "ORTHO"
cam_data.ortho_scale = 2.3
cam = bpy.data.objects.new("review_cam", cam_data)
scene.collection.objects.link(cam)
cam.location = (2.4, -2.4, 2.4)
target = Vector((0.0, -0.1, 0.78))
cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
scene.camera = cam

objs = list(bpy.data.objects)
for clip_name, frames in SHOTS:
    action = bpy.data.actions.get(clip_name)
    if not action:
        raise RuntimeError("missing action " + clip_name)
    for slot in action.slots:
        ident = slot.identifier[2:] if slot.identifier.startswith("OB") else slot.identifier
        o = bpy.data.objects.get(ident)
        if o:
            ad = o.animation_data_create()
            ad.action = action
            ad.action_slot = slot
    for f in frames:
        scene.frame_set(f)
        scene.render.filepath = os.path.join(OUT, "%s_%03d.png" % (clip_name, f))
        bpy.ops.render.render(write_still=True)
    print("rendered", clip_name)
print("frames in", OUT)
