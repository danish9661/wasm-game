"""Weather effects and NPC work animations on the existing unskinned models.

    blender -b --python scripts/generate_world_animations.py

Two families, both 30 fps with duplicate-first-frame loops:

  fx_*  - node-keyed effects on torch/campfire/tree/pine/grass plus simple
          procedural rain, snow, hail, water, wind leaves, fog and lightning.
          Scale keys are intentional here (fire flicker, ripples, flashes).
  *_<work> - small pivot rigs built over the NPC models (archer, guard, raider,
          goblin, imp, skeleton, wolf) with their own work/idle loops.

Nothing here is mocap. Output: assets/animations/<name>.glb and
assets/animations/world_manifest.json.
"""

import json
import math
import os
import sys
from math import cos, pi, sin

import bpy
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MODELS = os.path.join(ROOT, "assets", "models")
OUT = os.path.join(ROOT, "assets", "animations")
FPS = 30

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from generate_combat_animations import iter_fcurves  # noqa: E402


# ---------------------------------------------------------------------------
# Generic helpers
# ---------------------------------------------------------------------------


def clamp(v, lo, hi):
    return max(lo, min(hi, v))


def smooth(t):
    t = clamp(t, 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def keys_at(t, keys):
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * smooth((t - t0) / (t1 - t0))
    return keys[-1][1]


def clean():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def import_model(name):
    bpy.ops.import_scene.gltf(filepath=os.path.join(MODELS, name + ".glb"))
    return {o.name: o for o in bpy.data.objects if o.type == "MESH"}


def cube(name, size, loc):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    o = bpy.context.object
    o.name = name
    o.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    return o


def ico(name, radius, loc):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=0, radius=radius, location=loc)
    o = bpy.context.object
    o.name = name
    return o


def make_action(objects, name, frames, loop, key_fn, allow_scale=True):
    action = bpy.data.actions.new(name)
    for o in objects:
        ad = o.animation_data_create()
        ad.action = action
        ad.action_slot = action.slots.new(id_type="OBJECT", name=o.name)
    n = frames + 1 if loop else frames
    for f in range(n):
        key_fn(f, f % frames if loop else f)
        for o in objects:
            o.keyframe_insert(data_path="location", frame=f, group=o.name)
            o.keyframe_insert(data_path="rotation_euler", frame=f, group=o.name)
            if allow_scale:
                o.keyframe_insert(data_path="scale", frame=f, group=o.name)
    for fc in iter_fcurves(action):
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
    action.use_fake_user = True
    return action


def measure(objects, frames, loop, reference=None):
    scene = bpy.context.scene
    samples = []
    for f in range(frames + 1 if loop else frames):
        scene.frame_set(f)
        bpy.context.view_layer.update()
        if reference:
            ref_obj = (reference if hasattr(reference, "matrix_world")
                       else objects[reference] if isinstance(objects, dict)
                       else next(o for o in objects if o.name == reference))
            ref = ref_obj.matrix_world.inverted()
        else:
            ref = None
        samples.append({o.name: ((ref @ o.matrix_world).translation.copy() if ref
                                 else o.matrix_world.translation.copy())
                        for o in objects})
    seam = 0.0
    if loop:
        for name in samples[0]:
            seam = max(seam, (samples[0][name] - samples[-1][name]).length * 100.0)
    return round(seam, 4)


def export(name, path):
    bpy.ops.export_scene.gltf(
        filepath=path, export_format="GLB", export_yup=True,
        export_animations=True, export_animation_mode="ACTIONS",
        export_nla_strips=False, export_extras=True, export_apply=False,
        use_selection=False)


# ---------------------------------------------------------------------------
# Weather / FX clips
# ---------------------------------------------------------------------------


def fire_key(model, names, frames):
    """Node-keyed flame flicker (scale + roll + a small vertical bob)."""
    objs = import_model(model)
    animated = [objs[n] for n in names]
    rest = {o.name: (o.location.copy(), o.scale.copy()) for o in animated}

    def key(f, pf):
        ph = 2 * pi * pf / frames
        for i, o in enumerate(animated):
            loc0, sc0 = rest[o.name]
            w = sin(ph * (2 + i) + i * 1.7) + 0.5 * sin(3 * ph + i * 2.3)
            o.location = loc0 + Vector((0.0, 0.0, 0.015 * w))
            o.rotation_euler = (0.0, 0.0, 0.10 * w)
            o.scale = (sc0.x * (1.0 + 0.10 * w),
                       sc0.y * (1.0 + 0.08 * w),
                       sc0.z * (1.0 + 0.22 * w))
    return animated, key, False


def falling_key(kind, count, frames, fall=3.6, speed=1.0):
    """Wrapped falling particles; last frame == first frame exactly."""
    objs = []
    for i in range(count):
        bx = -1.0 + 2.0 * ((i * 7 % count) / max(1, count - 1))
        bz = -1.0 + 2.0 * ((i * 11 % count) / max(1, count - 1))
        if kind == "rain":
            o = cube("drop_%02d" % i, (0.016, 0.24, 0.016), (bx, 2.0, bz))
        elif kind == "hail":
            o = ico("hail_%02d" % i, 0.035, (bx, 2.0, bz))
        else:
            o = ico("flake_%02d" % i, 0.028, (bx, 2.0, bz))
        objs.append(o)
    bases = [(o.location.x, o.location.z) for o in objs]

    def key(f, pf):
        for i, o in enumerate(objs):
            bx, bz = bases[i]
            phase = (i / count + pf / frames * speed) % 1.0
            sway = 0.16 * sin(2 * pi * (phase + i * 0.17)) if kind != "rain" else 0.0
            o.location = (bx + sway, 2.0 - fall * phase, bz + 0.5 * sway)
            o.rotation_euler = (0.0, 0.0, 0.0 if kind == "rain" else pi * phase)
            o.scale = (1.0, 1.0, 1.0)
    return objs, key, False


def water_key(frames):
    objs = []
    for ix in range(4):
        for iz in range(4):
            objs.append(cube("wave_%d_%d" % (ix, iz), (0.24, 0.02, 0.24),
                             (-0.4 + ix * 0.27, 0.0, -0.4 + iz * 0.27)))
    for i in range(4):
        objs.append(ico("ripple_%d" % i, 0.18, (-0.3 + 0.2 * i, 0.02, 0.0)))

    def key(f, pf):
        ph = 2 * pi * pf / frames
        for ix in range(4):
            for iz in range(4):
                o = objs[ix * 4 + iz]
                z = 0.05 * sin(ph + ix * 0.7) + 0.035 * sin(2 * ph + iz * 0.9)
                o.location = (o.location.x, z, o.location.z)
                o.rotation_euler = (0.05 * sin(ph + ix), 0.0, 0.04 * cos(ph + iz))
        for i in range(4):
            o = objs[16 + i]
            phase = (i / 4.0 + pf / frames) % 1.0
            s = 0.4 + 1.2 * phase
            o.location = (o.location.x, 0.02 + 0.01 * phase, o.location.z)
            o.scale = (s, 0.15, s)
    return objs, key, False


def wind_key(frames):
    objs = list(import_model("grass_tuft").values())
    base = {o.name: (o.location.copy(), o.rotation_euler.copy()) for o in objs}
    for i in range(8):
        objs.append(cube("leaf_%d" % i, (0.06, 0.008, 0.04),
                         (-1.0 + i * 0.25, 0.5 + (i % 3) * 0.4, -0.5 + (i % 2) * 0.9)))
    leaf_base = [(o.location.y, o.location.z) for o in objs[len(base):]]

    def key(f, pf):
        ph = 2 * pi * pf / frames
        for i, o in enumerate(objs[:len(base)]):
            loc0, rot0 = base[o.name]
            w = sin(ph + i * 0.4) + 0.35 * sin(2 * ph + i)
            o.location = loc0 + Vector((0.02 * w, 0.0, 0.0))
            o.rotation_euler = (0.28 * w, 0.12 * w, 0.35 * w)
            o.scale = (1.0, 1.0, 1.0)
        for i, o in enumerate(objs[len(base):]):
            y0, z0 = leaf_base[i]
            phase = (i / 8.0 + pf / frames) % 1.0
            o.location = (-1.4 + 2.8 * phase, y0 + 0.10 * sin(ph + i), z0)
            o.rotation_euler = (0.3 * phase, 0.2 * sin(ph + i), ph * 0.5)
    return objs, key, False


def sway_key(model, frames, amount):
    objs = list(import_model(model).values())
    base = {o.name: (o.location.copy(), o.rotation_euler.copy()) for o in objs}

    def key(f, pf):
        ph = 2 * pi * pf / frames
        for i, o in enumerate(objs):
            loc0, rot0 = base[o.name]
            w = sin(ph + i * 0.5) + 0.3 * sin(2 * ph + i * 0.9)
            o.location = loc0 + Vector((amount * 0.25 * w, 0.0, 0.0))
            o.rotation_euler = (rot0.x + amount * w,
                                rot0.y + amount * 0.4 * sin(ph + i),
                                rot0.z + amount * 0.6 * w)
            o.scale = (1.0, 1.0, 1.0)
    return objs, key, False


def lightning_key(frames):
    objs = []
    for i in range(5):
        objs.append(cube("bolt_%d" % i, (0.06, 0.28, 0.06),
                         (0.05 * sin(i * 1.7), 1.9 - i * 0.3, 0.04 * cos(i * 1.3))))
    base = [o.location.copy() for o in objs]

    def key(f, pf):
        t = pf / frames
        flash = keys_at(t, [(0.0, 0.0), (0.18, 0.0), (0.20, 1.0), (0.23, 0.2),
                            (0.25, 1.0), (0.30, 0.0), (1.0, 0.0)])
        for i, o in enumerate(objs):
            o.location = base[i]
            o.rotation_euler = (0.05 * i, 0.0, 0.08 * i)
            o.scale = (flash, flash, flash)
    return objs, key, True


def fog_key(frames):
    objs = []
    for i in range(6):
        o = ico("fog_%d" % i, 0.4, (-1.2 + i * 0.45, 0.3 + (i % 2) * 0.25,
                                    -0.6 + (i % 3) * 0.5))
        o.scale = (1.5, 0.3, 1.0)
        objs.append(o)
    base = [o.location.copy() for o in objs]

    def key(f, pf):
        ph = 2 * pi * pf / frames
        for i, o in enumerate(objs):
            phase = (i / 6.0 + pf / frames) % 1.0
            p = base[i]
            o.location = (-2.0 + 4.0 * phase, p.y + 0.05 * sin(ph + i), p.z)
            s = 1.0 + 0.15 * sin(ph + i)
            o.scale = (1.5 * s, 0.3 * s, 1.0 * s)
            o.rotation_euler = (0.0, 0.0, 0.1 * sin(ph + i))
    return objs, key, True


FX = [
    ("fx_fire", 48, True, lambda: fire_key("campfire", ["flame_outer", "flame_inner", "ember"], 48)),
    ("fx_torch_fire", 36, True, lambda: fire_key("torch", ["flame", "core.001"], 36)),
    ("fx_brazier_fire", 48, True, lambda: fire_key("brazier", ["flame.001", "coal"], 48)),
    ("fx_rain", 60, True, lambda: falling_key("rain", 24, 60)),
    ("fx_snow", 90, True, lambda: falling_key("snow", 20, 90)),
    ("fx_hail", 60, True, lambda: falling_key("hail", 16, 60, fall=4.0, speed=1.4)),
    ("fx_water", 60, True, lambda: water_key(60)),
    ("fx_wind", 48, True, lambda: wind_key(48)),
    ("fx_tree_sway", 90, True, lambda: sway_key("tree", 90, 0.05)),
    ("fx_pine_sway", 90, True, lambda: sway_key("pine", 90, 0.035)),
    ("fx_lightning", 90, True, lambda: lightning_key(90)),
    ("fx_fog", 120, True, lambda: fog_key(120)),
]


# ---------------------------------------------------------------------------
# NPC pivot rigs
# ---------------------------------------------------------------------------

# chain name -> (node names, pivot mode, parent chain)
NPC = {
    "archer": {
        "torso": (["torso"], "bottom", None),
        "head": (["hood", "shade", "eye_l.007", "eye_r.007"], "bottom", "torso"),
        "upper_arm_l": (["arm_l.002"], "top", "torso"),
        "lower_arm_l": (["hand_l.001"], "top", "upper_arm_l"),
        "upper_arm_r": (["arm_r.002"], "top", "torso"),
        "lower_arm_r": (["hand_r.001", "bow_grip", "bow_top", "bow_low", "bow_string"], "top", "upper_arm_r"),
        "upper_leg_l": (["leg_l.002"], "top", "torso"),
        "lower_leg_l": (["boot_l.001"], "top", "upper_leg_l"),
        "upper_leg_r": (["leg_r.002"], "top", "torso"),
        "lower_leg_r": (["boot_r.001"], "top", "upper_leg_r"),
    },
    "guard": {
        "torso": (["tunic.001"], "bottom", None),
        "head": (["head.009", "helm", "eye_l.015", "eye_r.015"], "bottom", "torso"),
        "upper_arm_l": (["arm_l.007", "shield", "shield_boss"], "top", "torso"),
        "upper_arm_r": (["arm_r.007"], "top", "torso"),
        "upper_leg_l": (["leg_l.007"], "top", "torso"),
        "lower_leg_l": (["boot_l.003"], "top", "upper_leg_l"),
        "upper_leg_r": (["leg_r.007"], "top", "torso"),
        "lower_leg_r": (["boot_r.003"], "top", "upper_leg_r"),
    },
    "raider": {
        "torso": (["torso.001", "cloak", "belt.002"], "bottom", None),
        "head": (["hood.001", "mask", "eye_l.008", "eye_r.008"], "bottom", "torso"),
        "upper_arm_l": (["pauldron_l"], "top", "torso"),
        "upper_arm_r": (["pauldron_r", "blade", "hilt", "guard"], "top", "torso"),
        "upper_leg_l": (["leg_l.003"], "top", "torso"),
        "lower_leg_l": (["boot_l.002"], "top", "upper_leg_l"),
        "upper_leg_r": (["leg_r.003"], "top", "torso"),
        "lower_leg_r": (["boot_r.002"], "top", "upper_leg_r"),
    },
    "goblin": {
        "torso": (["body", "loincloth"], "bottom", None),
        "head": (["head.001", "ear_l", "ear_r", "eye_l.001", "eye_r.001",
                  "nose", "tooth"], "bottom", "torso"),
        "upper_arm_l": (["arm_l.001"], "top", "torso"),
        "upper_arm_r": (["arm_r.001"], "top", "torso"),
        "upper_leg_l": (["leg_l.001"], "top", "torso"),
        "lower_leg_l": (["foot_l"], "top", "upper_leg_l"),
        "upper_leg_r": (["leg_r.001"], "top", "torso"),
        "lower_leg_r": (["foot_r"], "top", "upper_leg_r"),
    },
    "imp": {
        "torso": (["body.008"], "bottom", None),
        "head": (["head.013", "horn_l.001", "horn_r.001", "eye_l.019",
                  "eye_r.019"], "bottom", "torso"),
        "upper_arm_l": (["arm_l.011"], "top", "torso"),
        "upper_arm_r": (["arm_r.011"], "top", "torso"),
        "wing_l": (["wing_l.001"], "center", "torso"),
        "wing_r": (["wing_r.001"], "center", "torso"),
        "tail": (["tail.001", "tail_tip"], "center", "torso"),
        "upper_leg_l": (["foot_l.007"], "top", "torso"),
        "upper_leg_r": (["foot_r.007"], "top", "torso"),
    },
    "skeleton": {
        "torso": (["pelvis", "spine", "rib_l0", "rib_l1", "rib_l2",
                   "rib_r0", "rib_r1", "rib_r2"], "bottom", None),
        "head": (["skull", "jaw", "eye_l.002", "eye_r.002"], "bottom", "torso"),
        "upper_arm_l": (["shoulder_l", "upperarm_l"], "top", "torso"),
        "lower_arm_l": (["forearm_l"], "top", "upper_arm_l"),
        "upper_arm_r": (["shoulder_r", "upperarm_r"], "top", "torso"),
        "lower_arm_r": (["forearm_r"], "top", "upper_arm_r"),
        "upper_leg_l": (["thigh_l"], "top", "torso"),
        "lower_leg_l": (["shin_l", "foot_l.001"], "top", "upper_leg_l"),
        "upper_leg_r": (["thigh_r"], "top", "torso"),
        "lower_leg_r": (["shin_r", "foot_r.001"], "top", "upper_leg_r"),
    },
    "wolf": {
        "torso": (["body.002"], "center", None),
        "head": (["neck.001", "head.002", "snout", "nose.001", "ear_l.001",
                  "ear_r.001", "eye_l.004", "eye_r.004"], "center", "torso"),
        "leg_lf": (["leg_l_front", "paw_l_front"], "top", "torso"),
        "leg_rf": (["leg_r_front", "paw_r_front"], "top", "torso"),
        "leg_lb": (["leg_l_back", "paw_l_back"], "top", "torso"),
        "leg_rb": (["leg_r_back", "paw_r_back"], "top", "torso"),
        "tail": (["tail", "tuft_l", "tuft_r"], "center", "torso"),
    },
}


def build_puppet(name):
    clean()
    nodes = import_model(name)
    root = bpy.data.objects.new("root", None)
    bpy.context.collection.objects.link(root)
    bpy.context.view_layer.update()
    pivots, empties = {}, {"root": root}
    for cname, (parts, mode, parent) in NPC[name].items():
        objs = [nodes[p] for p in parts]
        pts = [o.matrix_world @ Vector(c) for o in objs for c in o.bound_box]
        xs = [p.x for p in pts]
        ys = [p.y for p in pts]
        zs = [p.z for p in pts]
        cx, cy = (min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2
        z = {"top": max(zs), "bottom": min(zs),
             "center": (min(zs) + max(zs)) / 2}[mode]
        pivot = Vector((cx, cy, z))
        e = bpy.data.objects.new(cname, None)
        bpy.context.collection.objects.link(e)
        e.parent = empties[parent] if parent else root
        bpy.context.view_layer.update()
        e.location = pivot - e.parent.matrix_world.translation
        bpy.context.view_layer.update()
        empties[cname] = e
        pivots[cname] = pivot
        for o in objs:
            mw = o.matrix_world.copy()
            o.parent = e
            o.matrix_parent_inverse = e.matrix_world.inverted()
            o.matrix_world = mw
    bpy.context.view_layer.update()
    return empties


def apply_npc_pose(empties, pose):
    for name, rot in pose.items():
        e = empties[name]
        e.rotation_mode = "XYZ"
        e.rotation_euler = rot[:3]
        if len(rot) == 6:
            e.location = rot[3:]


def npc_pose_fn(model, kind, frames):
    def pose(f, pf):
        t = f / FPS
        ph = 2 * pi * pf / frames
        p = {}
        if model == "archer":
            if kind == "shoot":
                d = keys_at(t, [(0, 0), (0.35, 1), (0.5, 1), (0.62, 0), (1.2, 0)])
                p["torso"] = (0.0, 0.0, -0.10 * d, 0, 0, 0)
                p["head"] = (0.05 * d, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_l"] = (-0.95 - 0.35 * d, 0.0, 0.0, 0, 0, 0)
                p["lower_arm_l"] = (-0.30, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-1.00 + 0.85 * d, 0.0, 0.0, 0, 0, 0)
                p["lower_arm_r"] = (-0.95 - 0.25 * d, 0.0, 0.0, 0, 0, 0)
            else:
                p["torso"] = (0.03 * sin(ph), 0.0, -0.12, 0, 0, 0)
                p["head"] = (0.02, 0.0, 0.04 * sin(ph), 0, 0, 0)
                p["upper_arm_l"] = (-1.30 + 0.03 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["lower_arm_l"] = (-0.30, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.20 + 0.03 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["lower_arm_r"] = (-1.20, 0.0, 0.0, 0, 0, 0)
            p["upper_leg_l"] = (0.08, 0.0, 0.0, 0, 0, 0)
            p["upper_leg_r"] = (-0.08, 0.0, 0.0, 0, 0, 0)
        elif model == "guard":
            if kind == "patrol":
                swing = sin(ph)
                p["torso"] = (0.0, 0.0, 0.04 * swing, 0, 0, 0.012 * cos(2 * ph))
                p["head"] = (0.0, 0.0, -0.04 * swing, 0, 0, 0)
                p["upper_arm_l"] = (-0.85, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.25 - 0.28 * swing, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_l"] = (0.35 * swing, 0.0, 0.0, 0, 0, 0)
                p["lower_leg_l"] = (-0.25 * max(0, -swing), 0.0, 0.0, 0, 0, 0)
                p["upper_leg_r"] = (-0.35 * swing, 0.0, 0.0, 0, 0, 0)
                p["lower_leg_r"] = (-0.25 * max(0, swing), 0.0, 0.0, 0, 0, 0)
            else:
                b = keys_at(t, [(0, 0), (0.3, 1), (0.5, 1), (0.9, 0), (1.3, 0)])
                p["torso"] = (-0.10 * b, 0.0, 0.12 * b, 0, 0, 0)
                p["upper_arm_l"] = (-1.20 - 0.30 * b, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (0.0, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_l"] = (0.15, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_r"] = (-0.10, 0.0, 0.0, 0, 0, 0)
        elif model == "raider":
            if kind == "slash":
                w = keys_at(t, [(0, 0), (0.35, 1), (0.5, 1), (0.68, -1),
                                (1.0, 0), (1.4, 0)])
                p["torso"] = (0.0, 0.0, 0.25 * w, 0, 0, 0)
                p["head"] = (0.0, 0.0, -0.10 * w, 0, 0, 0)
                p["upper_arm_r"] = (-0.60 + 1.80 * max(0, w) - 0.90 * max(0, -w),
                                    0.0, 0.0, 0, 0, 0)
                p["upper_arm_l"] = (-0.60, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_l"] = (0.15, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_r"] = (-0.15, 0.0, 0.0, 0, 0, 0)
            else:
                p["torso"] = (0.28, 0.0, 0.08 * sin(ph), 0, 0, -0.04)
                p["head"] = (-0.12, 0.0, 0.05 * sin(ph), 0, 0, 0)
                p["upper_arm_l"] = (-0.45 + 0.15 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.50 - 0.15 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["upper_leg_l"] = (0.30 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["upper_leg_r"] = (-0.30 * sin(ph), 0.0, 0.0, 0, 0, 0)
        elif model == "goblin":
            if kind == "scavenge":
                p["torso"] = (0.32 + 0.06 * sin(ph), 0.0, 0.06 * sin(ph * 2), 0, 0, 0)
                p["head"] = (-0.20 + 0.10 * sin(ph), 0.0, 0.10 * sin(ph), 0, 0, 0)
                p["upper_arm_l"] = (-0.70 + 0.30 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.70 - 0.30 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["upper_leg_l"] = (0.20, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_r"] = (-0.20, 0.0, 0.0, 0, 0, 0)
            else:
                a = keys_at(t, [(0, 0), (0.3, 1), (0.5, -0.5), (0.8, 0), (1.1, 0)])
                p["torso"] = (0.20 * a, 0.0, 0.18 * a, 0, 0, 0)
                p["head"] = (-0.15 * a, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_l"] = (-0.50 - 0.8 * a, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.50 - 0.5 * a, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_l"] = (0.25, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_r"] = (-0.25, 0.0, 0.0, 0, 0, 0)
        elif model == "imp":
            if kind == "hover":
                p["torso"] = (0.0, 0.0, 0.05 * sin(ph), 0, 0, 0.05 * sin(2 * ph))
                p["head"] = (0.0, 0.0, 0.08 * sin(ph + 1), 0, 0, 0)
                p["wing_l"] = (0.0, -0.7 * sin(ph), 0.0, 0, 0, 0)
                p["wing_r"] = (0.0, 0.7 * sin(ph), 0.0, 0, 0, 0)
                p["tail"] = (0.0, 0.25 * sin(ph + 0.7), 0.0, 0, 0, 0)
                p["upper_arm_l"] = (-0.35 + 0.15 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.35 - 0.15 * sin(ph), 0.0, 0.0, 0, 0, 0)
            else:
                m = keys_at(t, [(0, 0), (0.25, 1), (0.5, -1), (0.8, 0), (1.1, 0)])
                p["torso"] = (0.20 * m, 0.0, 0.20 * m, 0, 0, 0)
                p["head"] = (-0.15 * m, 0.0, -0.20 * m, 0, 0, 0)
                p["wing_l"] = (0.0, -0.9 * m, 0.0, 0, 0, 0)
                p["wing_r"] = (0.0, 0.9 * m, 0.0, 0, 0, 0)
                p["upper_arm_l"] = (-0.60 - 0.5 * m, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.60 + 0.5 * m, 0.0, 0.0, 0, 0, 0)
        elif model == "skeleton":
            if kind == "shamble":
                p["torso"] = (0.06 * sin(ph), 0.0, 0.10 * sin(ph),
                              0, 0, 0.015 * cos(2 * ph))
                p["head"] = (-0.05 * sin(ph), 0.0, -0.12 * sin(ph), 0, 0, 0)
                p["upper_arm_l"] = (-0.25 + 0.40 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.25 - 0.40 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["upper_leg_l"] = (0.38 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["lower_leg_l"] = (-0.30 * max(0, -sin(ph)), 0.0, 0.0, 0, 0, 0)
                p["upper_leg_r"] = (-0.38 * sin(ph), 0.0, 0.0, 0, 0, 0)
                p["lower_leg_r"] = (-0.30 * max(0, sin(ph)), 0.0, 0.0, 0, 0, 0)
            else:
                a = keys_at(t, [(0, 0), (0.3, 1), (0.45, 1), (0.8, 0), (1.2, 0)])
                p["torso"] = (-0.10 * a, 0.0, 0.20 * a, 0, 0, 0)
                p["head"] = (0.10 * a, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_l"] = (-0.30 - 1.0 * a, 0.0, 0.0, 0, 0, 0)
                p["lower_arm_l"] = (-0.25 - 0.3 * a, 0.0, 0.0, 0, 0, 0)
                p["upper_arm_r"] = (-0.30 - 0.6 * a, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_l"] = (0.20, 0.0, 0.0, 0, 0, 0)
                p["upper_leg_r"] = (-0.20, 0.0, 0.0, 0, 0, 0)
        elif model == "wolf":
            if kind == "run":
                swing = sin(ph)
                p["torso"] = (0.0, 0.0, 0.03 * swing, 0, 0, 0.03 * cos(2 * ph))
                p["head"] = (-0.05, 0.0, 0.04 * swing, 0, 0, 0)
                p["leg_lf"] = (0.55 * swing, 0.0, 0.0, 0, 0, 0)
                p["leg_rf"] = (-0.55 * swing, 0.0, 0.0, 0, 0, 0)
                p["leg_lb"] = (-0.50 * swing, 0.0, 0.0, 0, 0, 0)
                p["leg_rb"] = (0.50 * swing, 0.0, 0.0, 0, 0, 0)
                p["tail"] = (0.0, 0.20 * sin(ph + 0.8), 0.0, 0, 0, 0)
            elif kind == "howl":
                h = keys_at(t, [(0, 0), (0.25, 1), (0.65, 1), (1.0, 0), (1.4, 0)])
                p["torso"] = (-0.18 * h, 0.0, 0.0, 0, 0, 0)
                p["head"] = (-0.55 * h, 0.0, 0.0, 0, 0, 0)
                p["tail"] = (0.0, 0.25 * sin(ph), 0.0, 0, 0, 0)
            else:
                b = keys_at(t, [(0, 0), (0.25, 1), (0.4, -0.5), (0.7, 0), (1.0, 0)])
                p["head"] = (-0.30 * b, 0.0, 0.0, 0, 0, 0)
                p["torso"] = (0.20 * b, 0.0, 0.0, 0, 0, 0)
                p["leg_lf"] = (-0.50 * b, 0.0, 0.0, 0, 0, 0)
                p["leg_rf"] = (-0.50 * b, 0.0, 0.0, 0, 0, 0)
                p["tail"] = (0.0, 0.2 * sin(ph), 0.0, 0, 0, 0)
        return p
    return pose


NPC_CLIPS = [
    ("archer_shoot", "archer", "shoot", 36, False),
    ("archer_aim", "archer", "aim", 60, True),
    ("guard_patrol", "guard", "patrol", 24, True),
    ("guard_shield_bash", "guard", "bash", 40, False),
    ("raider_slash", "raider", "slash", 42, False),
    ("raider_sneak", "raider", "sneak", 48, True),
    ("goblin_scavenge", "goblin", "scavenge", 48, True),
    ("goblin_attack", "goblin", "attack", 36, False),
    ("imp_hover", "imp", "hover", 48, True),
    ("imp_mischief", "imp", "mischief", 36, False),
    ("skeleton_shamble", "skeleton", "shamble", 30, True),
    ("skeleton_attack", "skeleton", "attack", 36, False),
    ("wolf_run", "wolf", "run", 24, True),
    ("wolf_howl", "wolf", "howl", 42, False),
    ("wolf_bite", "wolf", "bite", 30, False),
]


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    os.makedirs(OUT, exist_ok=True)
    records = []
    scene = bpy.context.scene
    scene.render.fps = FPS

    for name, frames, loop, builder in FX:
        clean()
        objs, key, scale_keys = builder()
        make_action(objs, name, frames, loop,
                    lambda f, pf: key(f, pf), allow_scale=True)
        seam = measure(objs, frames, loop)
        path = os.path.join(OUT, name + ".glb")
        export(name, path)
        records.append({
            "clip": name, "type": "fx", "path": "assets/animations/" + name + ".glb",
            "source": "procedural / existing weather model", "frames": frames,
            "duration_s": round((frames + 1 if loop else frames) / FPS, 4),
            "loop": loop, "scale_keys": scale_keys, "seam_position_cm": seam,
            "notes": "weather effect; scale keys are intentional for flicker/ripples/flash",
        })
        print("[FX ] %-18s frames=%3d seam=%.3fcm" % (name, frames, seam))

    for name, model, kind, frames, loop in NPC_CLIPS:
        empties = build_puppet(model)   # calls clean()
        pose_fn = npc_pose_fn(model, kind, frames)
        objs = [e for n, e in empties.items() if n != "root"]
        make_action(objs, name, frames, loop,
                    lambda f, pf: apply_npc_pose(empties, pose_fn(f, pf)),
                    allow_scale=False)
        seam = measure(objs, frames, loop, reference=empties["root"])
        path = os.path.join(OUT, name + ".glb")
        export(name, path)
        records.append({
            "clip": name, "type": "npc_work", "path": "assets/animations/" + name + ".glb",
            "source": "assets/models/%s.glb" % model, "frames": frames,
            "duration_s": round((frames + 1 if loop else frames) / FPS, 4),
            "loop": loop, "scale_keys": False, "seam_position_cm": seam,
            "notes": "%s doing %s on its own pivot rig; 30 fps, no scale keys" % (model, kind),
        })
        print("[NPC] %-18s frames=%3d seam=%.3fcm" % (name, frames, seam))

    manifest = {
        "schema": "fight-animations.world-clips/1",
        "generator": "scripts/generate_world_animations.py",
        "tooling": {"blender": bpy.app.version_string, "fps": FPS,
                    "pipeline": "node-keyed FX + per-model pivot puppet rigs"},
        "conventions": {
            "fx_scale_keys": "allowed and intentional",
            "npc_scale_keys": "forbidden",
            "loop": "last key duplicates frame 0 in root-relative pose",
            "seam_gate_cm": [0.5, 2.0],
        },
        "clips": records,
        "summary": {
            "count": len(records),
            "fx": sum(1 for r in records if r["type"] == "fx"),
            "npc_work": sum(1 for r in records if r["type"] == "npc_work"),
            "fail": sum(1 for r in records if r["seam_position_cm"] > 2.0),
        },
    }
    path = os.path.join(OUT, "world_manifest.json")
    with open(path, "w") as fh:
        json.dump(manifest, fh, indent=2)
    print("wrote", os.path.relpath(path, ROOT), manifest["summary"])
    return 1 if manifest["summary"]["fail"] else 0


if __name__ == "__main__":
    sys.exit(main())
