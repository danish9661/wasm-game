"""Build a full isometric diorama from every generated GLB.

Run: ~/.local/blender/blender -b --python scripts/blender/showcase.py
Output: assets/previews/diorama.png and assets/models/starfall_diorama.blend
"""
import bpy
import os
import math
import sys
from mathutils import Vector

sys.path.insert(0, os.path.dirname(__file__))
from common import mat, box, cone, ico, C, new_scene, ROOT

MODELS = os.path.join(ROOT, "assets", "models")
PREVIEWS = os.path.join(ROOT, "assets", "previews")
os.makedirs(PREVIEWS, exist_ok=True)

new_scene()

# ---------------------------------------------------------------- materials
mg = mat("M_Grass", C["grass"])
mg2 = mat("M_Grass2", "#4E7C32")
mswamp = mat("M_Swamp", C["swamp"])
msand = mat("M_Sand", C["sand"])
mwater = mat("M_Water", C["water"], rough=0.35)
msnow = mat("M_Snow", C["snow"])

# ------------------------------------------------------------------ terrain
T = bpy.data.collections.new("terrain")
bpy.context.scene.collection.children.link(T)
for x in range(-6, 6):
    for y in range(-6, 6):
        if x <= -4 and y <= -3:
            m = mwater
        elif x >= 4 and y >= 3:
            m = msnow
        elif x >= 3 and y <= -3:
            m = msand
        elif x <= -4 and y >= 2:
            m = mswamp
        else:
            m = mg if (x + y) % 3 else mg2
        h = 0.10 + (0.06 if (x * 7 + y * 13) % 5 == 0 else 0.0)
        box("tile_%d_%d" % (x, y), (x, y, -h / 2), (0.98, 0.98, h), m, T)

# ------------------------------------------------------------------- import
def place(name, x, y, z=0.0, scale=1.0, yaw=0.0):
    path = os.path.join(MODELS, name + ".glb")
    if not os.path.exists(path):
        print("SKIP missing", name)
        return
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    new = [o for o in bpy.data.objects if o not in before]
    roots = [o for o in new if o.parent is None or o.parent not in new]
    for o in roots:
        o.location = Vector(o.location) + Vector((x, y, z))
        if scale != 1.0:
            o.scale = Vector(o.scale) * scale
        if yaw:
            o.rotation_euler.z += yaw

# player + settlement
place("player", 0, 0, yaw=math.radians(-35))
place("house", 2, 2, scale=0.85)
place("cabin", 4, 1, scale=0.8)
place("hut", 1, 3, scale=0.8)
place("inn", 4, 3, scale=0.8)
place("campfire", 0.6, -0.7)
place("chest", 1.6, -1.2)
place("anvil", 3.0, 0.0)
place("bed", 2.0, 3.3)
place("fence", 3.2, 2.8)
place("wall", 4.4, 0.2)
place("lantern", 2.6, 1.0)
place("torch", 4.0, 2.1)

# story points
place("altar", -1.2, 2.0)
place("portal", -4.5, 1.0, scale=0.9)
place("dungeon", 5.0, -4.0, scale=0.9)
place("enchanting_table", -2.0, 3.4)
place("banner", 3.6, 3.4)

# nature
for x, y, s in [(-3, 2, 1.0), (-4, 3, 0.85), (-2, 4, 0.9), (5, -2, 0.8)]:
    place("tree", x, y, scale=s)
place("pine", -5, -1, scale=0.9)
place("rock", -2, -2)
place("rock_pile", -3, -1)
place("crystal", -4, -2)
place("mushroom", -3.2, 1.2)
place("bush", -1, 4)
place("ore", 2, -3)
place("cactus", 4.5, -2.5)
place("lilypad", -4.5, -4)

# Wildlife & foes
place("slime", -2.5, 0.0)
place("goblin", 2.5, -2.0)
place("skeleton", 4.5, -1.0)
place("wolf", -1.0, 4.5)
place("spider", 2.0, -4.0)
place("archer", 5.0, 0.5)
place("raider", -3.5, -2.5)
place("scorpion_queen", 5.0, 4.5, scale=0.9)

# ------------------------------------------------------------------- lights
bpy.ops.object.light_add(type="SUN", location=(6, -7, 11))
sun = bpy.context.object
sun.name = "Sun_Key"
sun.data.energy = 2.1
sun.data.angle = math.radians(6)
sun.rotation_euler = (math.radians(35), 0, math.radians(35))

bpy.ops.object.light_add(type="AREA", location=(0, 0, 8))
fill = bpy.context.object
fill.name = "Sky_Fill"
fill.data.energy = 1000
fill.data.size = 16
fill.data.color = (0.62, 0.75, 1.0)

for px, py in [(0.6, -0.7), (-1.2, 2.0), (-4.5, 1.0)]:
    bpy.ops.object.light_add(type="POINT", location=(px, py, 0.7))
    l = bpy.context.object
    l.data.energy = 130
    l.data.color = (1.0, 0.5, 0.2)

world = bpy.data.worlds.new("Dusk")
world.use_nodes = True
bg = next(n for n in world.node_tree.nodes if n.type == "BACKGROUND")
bg.inputs["Color"].default_value = (0.10, 0.13, 0.22, 1)
bg.inputs["Strength"].default_value = 0.7
bpy.context.scene.world = world

# ------------------------------------------------------------------- camera
bpy.ops.object.camera_add(location=(15, -15, 12))
cam = bpy.context.object
cam.name = "Iso_Camera"
target = Vector((0, 0, 0.9))
cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
cam.data.type = "ORTHO"
cam.data.ortho_scale = 17
bpy.context.scene.camera = cam

# ------------------------------------------------------------------- render
sc = bpy.context.scene
sc.render.engine = "BLENDER_EEVEE"
sc.render.resolution_x = 1600
sc.render.resolution_y = 900
sc.render.resolution_percentage = 100
sc.render.image_settings.file_format = "PNG"
sc.render.filepath = os.path.join(PREVIEWS, "diorama.png")
sc.view_settings.view_transform = "AgX"
bpy.ops.render.render(write_still=True)
bpy.ops.wm.save_as_mainfile(filepath=os.path.join(MODELS, "starfall_diorama.blend"))
print("DIORAMA_DONE", sc.render.filepath)
