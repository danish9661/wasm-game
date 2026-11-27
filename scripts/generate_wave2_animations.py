"""Second animation wave: enemy, boss and NPC clips on bound-derived pivot rigs.

    blender -b --python scripts/generate_wave2_animations.py

Every rig is built from the *actual* node bounds of its GLB at run time (no
hardcoded joint coordinates); travel, leans and contact compensation are scaled
by model_height / 1.55 u, the verified player scale. 30 fps, LINEAR samplers,
no scale keys, one named Action per clip, all horizontal travel on `root`.

Outputs assets/animations/wave2/<clip>.glb and wave2/manifest.json with the
per-clip duration, loop flag, root offset in metres and [warn, fail] gates
used by generate_player_final.py.
"""

import hashlib
import json
import math
import os
import sys
from math import cos, pi, sin

import bpy
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MODELS = os.path.join(ROOT, "assets", "models")
OUT = os.path.join(ROOT, "assets", "animations", "wave2")
PLAYER = os.path.join(MODELS, "player.glb")
FPS = 30

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from generate_combat_animations import iter_fcurves  # noqa: E402
from generate_player_final import keys_at  # noqa: E402

# [warn, fail], game-animation references/validation.md (same as player set)
GATES = {
    "seam_position_cm": (0.5, 2.0),
    "seam_velocity_mps": (0.3, 0.8),
    "root_velocity_mps": (0.1, 0.3),
    "foot_slide_cm": (2.0, 5.0),
    "ground_penetration_cm": (1.0, 3.0),
    "float_p10_cm": (2.0, 5.0),
    "end_settled_mps": (0.05, 0.15),
}
SETTLE_FRAMES = 12


# ---------------------------------------------------------------------------
# Bound-derived pivot rigs: (pivot, mesh parts, pivot mode, parent pivot)
# mode = "bottom" | "top" | "center" of the union of the parts' world bounds.
# Parents always precede children. Every mesh node of a model is covered.
# ---------------------------------------------------------------------------

RIGS = {
    "slime": [
        ("base", ["puddle"], "bottom", "root"),
        ("torso", ["body.001", "core", "eye_l.003", "eye_r.003", "pupil_l",
                   "pupil_r", "mouth.001"], "bottom", "root"),
    ],
    "goblin": [
        ("torso", ["body", "loincloth"], "bottom", "root"),
        ("head", ["head.001", "ear_l", "ear_r", "eye_l.001", "eye_r.001",
                  "nose", "tooth"], "bottom", "torso"),
        ("arm_l", ["arm_l.001"], "top", "torso"),
        ("arm_r", ["arm_r.001"], "top", "torso"),
        ("leg_l", ["leg_l.001", "foot_l"], "top", "torso"),
        ("leg_r", ["leg_r.001", "foot_r"], "top", "torso"),
    ],
    "skeleton": [
        ("torso", ["pelvis", "spine", "rib_l0", "rib_l1", "rib_l2",
                   "rib_r0", "rib_r1", "rib_r2"], "bottom", "root"),
        ("head", ["skull", "jaw", "eye_l.002", "eye_r.002"], "bottom", "torso"),
        ("arm_l", ["shoulder_l", "upperarm_l"], "top", "torso"),
        ("lower_arm_l", ["forearm_l"], "top", "arm_l"),
        ("arm_r", ["shoulder_r", "upperarm_r"], "top", "torso"),
        ("lower_arm_r", ["forearm_r"], "top", "arm_r"),
        ("leg_l", ["thigh_l"], "top", "torso"),
        ("lower_leg_l", ["shin_l", "foot_l.001"], "top", "leg_l"),
        ("leg_r", ["thigh_r"], "top", "torso"),
        ("lower_leg_r", ["shin_r", "foot_r.001"], "top", "leg_r"),
    ],
    "npc_dire_wolf": [
        ("torso", ["dire_body"], "center", "root"),
        ("head", ["dire_head", "ear_0.001", "ear_1.001", "fang_0", "fang_1",
                  "mane_0", "mane_1", "mane_2", "snout"], "center", "torso"),
        ("leg_0", ["dire_leg_0"], "top", "torso"),
        ("leg_1", ["dire_leg_1"], "top", "torso"),
        ("leg_2", ["dire_leg_2"], "top", "torso"),
        ("leg_3", ["dire_leg_3"], "top", "torso"),
        ("tail", ["tail.001"], "center", "torso"),
    ],
    "spider": [
        ("torso", ["thorax", "abdomen", "eye_l.006", "eye_r.006",
                   "fang_1", "fang_2"], "center", "root"),
        ("upper_l0", ["upper_l0"], "top", "torso"),
        ("lower_l0", ["lower_l0"], "top", "upper_l0"),
        ("upper_l1", ["upper_l1"], "top", "torso"),
        ("lower_l1", ["lower_l1"], "top", "upper_l1"),
        ("upper_l2", ["upper_l2"], "top", "torso"),
        ("lower_l2", ["lower_l2"], "top", "upper_l2"),
        ("upper_l3", ["upper_l3"], "top", "torso"),
        ("lower_l3", ["lower_l3"], "top", "upper_l3"),
        ("upper_r0", ["upper_r0"], "top", "torso"),
        ("lower_r0", ["lower_r0"], "top", "upper_r0"),
        ("upper_r1", ["upper_r1"], "top", "torso"),
        ("lower_r1", ["lower_r1"], "top", "upper_r1"),
        ("upper_r2", ["upper_r2"], "top", "torso"),
        ("lower_r2", ["lower_r2"], "top", "upper_r2"),
        ("upper_r3", ["upper_r3"], "top", "torso"),
        ("lower_r3", ["lower_r3"], "top", "upper_r3"),
    ],
    "archer": [
        ("torso", ["torso", "belt.001", "quiver", "shaft_0", "tip_0",
                   "shaft_1", "tip_1", "shaft_2", "tip_2"], "bottom", "root"),
        ("head", ["hood", "shade", "eye_l.007", "eye_r.007"], "bottom", "torso"),
        ("arm_l", ["arm_l.002"], "top", "torso"),
        ("lower_arm_l", ["hand_l.001"], "top", "arm_l"),
        ("arm_r", ["arm_r.002"], "top", "torso"),
        ("lower_arm_r", ["hand_r.001", "bow_grip", "bow_top", "bow_low",
                         "bow_string"], "top", "arm_r"),
        ("leg_l", ["leg_l.002"], "top", "torso"),
        ("lower_leg_l", ["boot_l.001"], "top", "leg_l"),
        ("leg_r", ["leg_r.002"], "top", "torso"),
        ("lower_leg_r", ["boot_r.001"], "top", "leg_r"),
    ],
    "brute": [
        ("torso", ["torso.002", "belt.003"], "bottom", "root"),
        ("head", ["head.004", "jaw.001", "eye_l.009", "eye_r.009",
                  "horn_l", "horn_r"], "bottom", "torso"),
        ("arm_l", ["shoulder_l.001", "spike_l", "arm_l.003"], "top", "torso"),
        ("lower_arm_l", ["fist_l"], "top", "arm_l"),
        ("arm_r", ["shoulder_r.001", "spike_r", "arm_r.003"], "top", "torso"),
        ("lower_arm_r", ["fist_r"], "top", "arm_r"),
        ("leg_l", ["leg_l.004"], "top", "torso"),
        ("lower_leg_l", ["foot_l.003"], "top", "leg_l"),
        ("leg_r", ["leg_r.004"], "top", "torso"),
        ("lower_leg_r", ["foot_r.003"], "top", "leg_r"),
    ],
    "scorpion_queen": [
        ("torso", ["body.004", "crown_l", "crown_r"], "center", "root"),
        ("head", ["head.005", "eye_l.010", "eye_r.010"], "center", "torso"),
        ("arm_l", ["arm_l.004", "pincer_l_up", "pincer_l_lo"], "top", "torso"),
        ("arm_r", ["arm_r.004", "pincer_r_up", "pincer_r_lo"], "top", "torso"),
        ("tail_1", ["tail_1"], "center", "torso"),
        ("tail_2", ["tail_2"], "center", "tail_1"),
        ("tail_3", ["tail_3", "stinger"], "center", "tail_2"),
        ("leg_l0", ["leg_l0"], "top", "torso"),
        ("leg_l1", ["leg_l1"], "top", "torso"),
        ("leg_l2", ["leg_l2"], "top", "torso"),
        ("leg_r0", ["leg_r0"], "top", "torso"),
        ("leg_r1", ["leg_r1"], "top", "torso"),
        ("leg_r2", ["leg_r2"], "top", "torso"),
    ],
    "toad_king": [
        ("torso", ["body.005", "belly", "wart_0", "wart_1", "wart_2"],
         "center", "root"),
        ("head", ["head.006", "mouth.002", "eye_l.011", "eye_r.011",
                  "pupil_l.001", "pupil_r.001"], "center", "torso"),
        ("crown", ["crown", "crown_spike_0", "crown_spike_1",
                   "crown_spike_2"], "bottom", "head"),
        ("leg_l", ["leg_l.005"], "top", "torso"),
        ("leg_r", ["leg_r.005"], "top", "torso"),
    ],
    "stormcaller": [
        ("torso", ["robe", "hem", "torso.003", "rune_0", "rune_1", "rune_2"],
         "bottom", "root"),
        ("head", ["hood.002", "shade.001", "eye_l.012", "eye_r.012"],
         "bottom", "torso"),
        ("arm_l", ["arm_l.005"], "top", "torso"),
        ("arm_r", ["arm_r.005", "staff", "orb", "bolt_0", "bolt_1",
                   "bolt_2", "bolt_3"], "top", "torso"),
    ],
    "ocean_leviathan": [
        ("torso", ["coil_0", "coil_1", "coil_2", "coil_3", "foam",
                   "fin_0", "fin_1", "fin_2", "fin_3"], "center", "root"),
        ("head", ["head.007", "eye_l.013", "eye_r.013", "crest_l", "crest_r"],
         "center", "torso"),
        ("jaw", ["jaw.002"], "center", "head"),
        ("tentacle_l", ["tentacle_l"], "center", "torso"),
        ("tentacle_r", ["tentacle_r"], "center", "torso"),
    ],
    "colossus": [
        ("torso", ["pelvis.001", "torso.004", "crack_l", "crack_r"],
         "bottom", "root"),
        ("head", ["head.008", "eye_l.014", "eye_r.014", "crown_0",
                  "crown_1", "crown_2"], "bottom", "torso"),
        ("arm_l", ["shoulder_l.002", "arm_l.006"], "top", "torso"),
        ("lower_arm_l", ["fist_l.001"], "top", "arm_l"),
        ("arm_r", ["shoulder_r.002", "arm_r.006"], "top", "torso"),
        ("lower_arm_r", ["fist_r.001"], "top", "arm_r"),
        ("leg_l", ["leg_l.006"], "top", "torso"),
        ("lower_leg_l", ["foot_l.004"], "top", "leg_l"),
        ("leg_r", ["leg_r.006"], "top", "torso"),
        ("lower_leg_r", ["foot_r.004"], "top", "leg_r"),
    ],
    "npc_villager_child": [
        ("torso", ["smock"], "bottom", "root"),
        ("head", ["head.002", "hair", "eye_l", "eye_r", "patch"],
         "bottom", "torso"),
        ("arm_l", ["arm_0.002"], "top", "torso"),
        ("arm_r", ["arm_1.002"], "top", "torso"),
        ("leg_l", ["leg_0.001", "shoe_0"], "top", "torso"),
        ("leg_r", ["leg_1.001", "shoe_1"], "top", "torso"),
    ],
    "guard": [
        ("torso", ["tunic.001", "belt.004"], "bottom", "root"),
        ("head", ["head.009", "helm", "eye_l.015", "eye_r.015"],
         "bottom", "torso"),
        ("arm_l", ["arm_l.007", "shield", "shield_boss"], "top", "torso"),
        ("arm_r", ["arm_r.007"], "top", "torso"),
        ("leg_l", ["leg_l.007", "boot_l.003"], "top", "torso"),
        ("leg_r", ["leg_r.007", "boot_r.003"], "top", "torso"),
    ],
    "npc_merchant": [
        ("torso", ["tunic", "belt", "pack", "pack_flap", "satchel", "coin"],
         "bottom", "root"),
        ("head", ["head", "hood"], "bottom", "torso"),
        ("arm_l", ["arm_0"], "top", "torso"),
        ("arm_r", ["arm_1"], "top", "torso"),
        ("leg_l", ["leg_0"], "top", "torso"),
        ("leg_r", ["leg_1"], "top", "torso"),
    ],
    "golem": [
        ("torso", ["body.006", "crystal"], "bottom", "root"),
        ("head", ["head.010", "eye_l.016", "eye_r.016"], "bottom", "torso"),
        ("arm_l", ["shoulder_l.003", "arm_l.008", "fist_l.002"], "top", "torso"),
        ("arm_r", ["shoulder_r.003", "arm_r.008", "fist_r.002"], "top", "torso"),
        ("leg_l", ["leg_l.008", "foot_l.005"], "top", "torso"),
        ("leg_r", ["leg_r.008", "foot_r.005"], "top", "torso"),
    ],
}

# Contact pivots: pivots whose geometry-bottom tip is tracked for planted slide.
CONTACTS = {
    "slime": ["base"],
    "goblin": ["leg_l", "leg_r"],
    "skeleton": ["lower_leg_l", "lower_leg_r"],
    "npc_dire_wolf": ["leg_0", "leg_1", "leg_2", "leg_3"],
    "spider": ["lower_l0", "lower_l1", "lower_l2", "lower_l3",
               "lower_r0", "lower_r1", "lower_r2", "lower_r3"],
    "archer": ["lower_leg_l", "lower_leg_r"],
    "brute": ["lower_leg_l", "lower_leg_r"],
    "scorpion_queen": ["leg_l0", "leg_l1", "leg_l2",
                       "leg_r0", "leg_r1", "leg_r2"],
    "toad_king": ["leg_l", "leg_r"],
    "stormcaller": [],
    "ocean_leviathan": [],
    "colossus": ["lower_leg_l", "lower_leg_r"],
    "npc_villager_child": ["leg_l", "leg_r"],
    "guard": ["leg_l", "leg_r"],
    "npc_merchant": ["leg_l", "leg_r"],
    "golem": ["leg_l", "leg_r"],
}

MODELS_USED = sorted(RIGS)


# ---------------------------------------------------------------------------
# Rig construction from measured GLB bounds
# ---------------------------------------------------------------------------


def node_bounds(o):
    pts = [o.matrix_world @ Vector(c) for c in o.bound_box]
    xs, ys, zs = [p.x for p in pts], [p.y for p in pts], [p.z for p in pts]
    return (min(xs), max(xs)), (min(ys), max(ys)), (min(zs), max(zs))


def sha256(path):
    with open(path, "rb") as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def build_rig(model):
    """Import the model, derive every pivot from real mesh bounds, parent parts."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=os.path.join(MODELS, model + ".glb"))
    meshes = {o.name: o for o in bpy.data.objects if o.type == "MESH"}
    claimed = [part for _, parts, _, _ in RIGS[model] for part in parts]
    if len(claimed) != len(set(claimed)) or set(claimed) != set(meshes):
        raise RuntimeError("%s: rig parts do not cover mesh nodes (%d vs %d)" %
                           (model, len(set(claimed)), len(meshes)))
    all_pts = [o.matrix_world @ v.co for o in meshes.values() for v in o.data.vertices]
    gz = (min(p.z for p in all_pts), max(p.z for p in all_pts))
    height = gz[1] - gz[0]

    root = bpy.data.objects.new("root", None)
    root.empty_display_type = "PLAIN_AXES"
    root.empty_display_size = 0.08
    bpy.context.collection.objects.link(root)
    empties = {"root": root}
    rest = {"root": (0.0, 0.0, 0.0)}
    tips = {}
    lengths = {}
    for name, parts, mode, parent in RIGS[model]:
        objs = [meshes[p] for p in parts]
        pts = [o.matrix_world @ Vector(c) for o in objs for c in o.bound_box]
        xs, ys, zs = [p.x for p in pts], [p.y for p in pts], [p.z for p in pts]
        px, py = (min(xs) + max(xs)) / 2.0, (min(ys) + max(ys)) / 2.0
        pz = {"bottom": min(zs), "top": max(zs),
              "center": (min(zs) + max(zs)) / 2.0}[mode]
        pivot = Vector((px, py, pz))
        e = bpy.data.objects.new(name, None)
        e.empty_display_type = "PLAIN_AXES"
        e.empty_display_size = 0.06
        bpy.context.collection.objects.link(e)
        e.parent = empties[parent]
        bpy.context.view_layer.update()
        e.location = pivot - e.parent.matrix_world.translation
        bpy.context.view_layer.update()
        empties[name] = e
        rest[name] = tuple(e.location)
        tips[name] = Vector((0.0, 0.0, min(zs) - pz))
        lengths[name] = max(0.0, pz - min(zs))
        for o in objs:
            mw = o.matrix_world.copy()
            o.parent = e
            o.matrix_parent_inverse = e.matrix_world.inverted()
            o.matrix_world = mw
    bpy.context.view_layer.update()
    rest_tip_world = {k: empties[k].matrix_world @ tips[k]
                      for k in CONTACTS[model]}
    contact_meshes = {}
    for pivot, parts, mode, parent in RIGS[model]:
        if pivot in CONTACTS[model]:
            contact_meshes[pivot] = [meshes[p] for p in parts]
    rest_low_z = {k: min((m.matrix_world @ v.co).z for m in objs
                         for v in m.data.vertices)
                  for k, objs in contact_meshes.items()}
    return {
        "model": model, "empties": empties, "meshes": meshes, "rest": rest,
        "tips": tips, "lengths": lengths, "height": height,
        "scale": height / 1.55, "contacts": CONTACTS[model],
        "rest_tip_world": rest_tip_world, "contact_meshes": contact_meshes,
        "rest_low_z": rest_low_z,
    }


def blank(info):
    return {n: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0) for n in info["empties"]}


def put(p, name, rx=0.0, ry=0.0, rz=0.0, dx=0.0, dy=0.0, dz=0.0):
    p[name] = (rx, ry, rz, dx, dy, dz)
    return p


def step_forward(p, info, amount, pivots=None):
    """Root travel on -Y (Blender front); derived leg lengths plant the soles."""
    p["root"] = (0.0, 0.0, 0.0, 0.0, -amount, 0.0)
    for name in (pivots if pivots is not None else info["contacts"]):
        length = info["lengths"].get(name, 0.0)
        if length > 0.02:
            p[name] = (amount / length, 0.0, 0.0, 0.0, 0.0, 0.0)


def apply_pose(info, pose):
    for name, vals in pose.items():
        e = info["empties"][name]
        e.rotation_mode = "XYZ"
        e.rotation_euler = vals[:3]
        e.location = tuple(r + d for r, d in zip(info["rest"][name], vals[3:]))


def lock_feet(info):
    """Pin a near-ground contact tip to its rest world point (single-pivot IK).

    A limb pivot at the top of a mesh group swings its sole when rotated, so a
    torso lean or a leg pose alone reads as foot skate.  Translate the pivot by
    the inverse parent rotation to hold the sole still, exactly like the
    player set's 2-bone leg lock but on one bound-derived pivot.
    """
    bpy.context.view_layer.update()   # apply_pose just changed the transforms
    # Horizontal planting uses the bottom-centre tip. Vertical planting uses
    # the lowest *vertex*: a tilted sole has corners below that centre point,
    # so correcting Z from the tip alone would drive corners underground.
    for k in info["contacts"]:
        e = info["empties"][k]
        tip = e.matrix_world @ info["tips"][k]
        if tip.z > 0.12:          # airborne / folded: let the limb move freely
            continue
        delta = info["rest_tip_world"][k] - tip
        delta.z = 0.0
        e.location += e.parent.matrix_world.to_3x3().inverted() @ delta
    bpy.context.view_layer.update()
    for k in info["contacts"]:
        e = info["empties"][k]
        low = min((m.matrix_world @ v.co).z
                  for m in info["contact_meshes"][k] for v in m.data.vertices)
        if low > 0.12:            # limb folded away from the floor
            continue
        dz = info["rest_low_z"][k] - low
        e.location += e.parent.matrix_world.to_3x3().inverted() @ Vector((0, 0, dz))
    bpy.context.view_layer.update()


def author(info, name, fn, frames, loop, lock=False):
    action = bpy.data.actions.new(name)
    slots = {}
    for n, e in info["empties"].items():
        ad = e.animation_data_create()
        ad.action = action
        slot = action.slots.new(id_type="OBJECT", name=n)
        ad.action_slot = slot
        slots[n] = slot
    for f in range(frames + 1 if loop else frames):
        apply_pose(info, fn(f, f % frames if loop else f))
        if lock:
            lock_feet(info)
        for n, e in info["empties"].items():
            e.keyframe_insert(data_path="location", frame=f, group=n)
            e.keyframe_insert(data_path="rotation_euler", frame=f, group=n)
    for fc in iter_fcurves(action):
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
    action.use_fake_user = True
    return action, slots


def bind(info, action, slots):
    for n, e in info["empties"].items():
        ad = e.animation_data_create()
        ad.action = action
        ad.action_slot = slots[n]


def min_z(info):
    return min((m.matrix_world @ v.co).z
               for m in info["meshes"].values() for v in m.data.vertices)


def ground_clamp(info, n_keys, loop):
    scene = bpy.context.scene
    corr = []
    for f in range(n_keys):
        scene.frame_set(f)
        bpy.context.view_layer.update()
        corr.append(max(0.0, 0.001 - min_z(info)))
    if loop:
        corr[-1] = corr[0]
    # Patch the existing root Z keys in place. keyframe_insert() on a slotted
    # action can append duplicate, out-of-order keys, which glTF exports as an
    # unsorted sampler and makes a reloaded clip play its settle tail early.
    for fc in iter_fcurves(info["empties"]["root"].animation_data.action):
        if fc.data_path != "location" or fc.array_index != 2:
            continue
        keymap = {round(kp.co.x): kp for kp in fc.keyframe_points}
        for f, c in enumerate(corr):
            if c <= 0:
                continue
            kp = keymap.get(f)
            if kp is not None:
                kp.co.y += c
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
        fc.update()
    return max(corr) if corr else 0.0


# ---------------------------------------------------------------------------
# Measurement and gates (same checks as the player set, on pivot tips)
# ---------------------------------------------------------------------------


def spans_of(idx):
    if not idx:
        return []
    out, st, pr = [], idx[0], idx[0]
    for i in idx[1:]:
        if i > pr + 1:
            out.append((st, pr))
            st = i
        pr = i
    out.append((st, pr))
    return out


def measure(info, frames, loop):
    scene = bpy.context.scene
    n = frames + 1 if loop else frames
    samples = []
    for f in range(n):
        scene.frame_set(f)
        bpy.context.view_layer.update()
        root = info["empties"]["root"]
        inv = root.matrix_world.inverted()
        rel = {k: (inv @ e.matrix_world).translation.copy()
               for k, e in info["empties"].items()}
        tips = {k: info["empties"][k].matrix_world @ info["tips"][k]
                for k in info["contacts"]}
        samples.append({"root": root.matrix_world.translation.copy(),
                        "rel": rel, "tips": tips, "z": min_z(info)})

    seam = 0.0
    if loop:
        seam = max((samples[0]["rel"][k] - samples[-1]["rel"][k]).length * 100
                   for k in samples[0]["rel"])
    vel = [{"root": (b["root"] - a["root"]) * FPS,
            "rel": {k: (b["rel"][k] - a["rel"][k]) * FPS for k in a["rel"]}}
           for a, b in zip(samples, samples[1:])]
    seam_v = root_v = 0.0
    if loop and len(vel) >= 3:
        for k in vel[0]["rel"]:
            own = max((b["rel"][k] - a["rel"][k]).length for a, b in zip(vel, vel[1:]))
            seam_v = max(seam_v, max(0.0, (vel[-1]["rel"][k] - vel[0]["rel"][k]).length - own))
        own = max((b["root"] - a["root"]).length for a, b in zip(vel, vel[1:]))
        root_v = max(0.0, (vel[-1]["root"] - vel[0]["root"]).length - own)

    slide = 0.0
    for k in info["contacts"]:
        idx = []
        for i, s in enumerate(samples):
            pt = s["tips"][k]
            v = ((samples[i + 1]["tips"][k] - pt) * FPS
                 if i + 1 < len(samples) else Vector((0, 0, 0)))
            if pt.z < 0.045 and v.length < 0.45:
                idx.append(i)
        for a, b in spans_of(idx):
            if b - a < 2:
                continue
            xs = [samples[j]["tips"][k].x for j in range(a, b + 1)]
            ys = [samples[j]["tips"][k].y for j in range(a, b + 1)]
            slide = max(slide, math.hypot(max(xs) - min(xs), max(ys) - min(ys)) * 100)

    settled = 0.0
    if not loop:
        k = max(1, int(0.3 * FPS))
        settled = max(v["rel"][key].length for v in vel[-k:] for key in v["rel"])
    zs = sorted(s["z"] for s in samples)
    off = samples[-1]["root"] - samples[0]["root"]
    return {
        "seam_position_cm": round(seam, 4),
        "seam_velocity_mps": round(seam_v, 4),
        "root_velocity_mps": round(root_v, 4),
        "foot_slide_cm": round(slide, 4),
        "ground_penetration_cm": round(max(0.0, -min(zs)) * 100, 4),
        "float_p10_cm": round(zs[int(0.1 * (len(zs) - 1))] * 100, 4),
        "end_settled_mps": round(settled, 4),
        "root_offset_m": {"x": round(off.x, 4), "gltf_z": round(-off.y, 4),
                          "gltf_y": round(off.z, 4),
                          "magnitude": round(off.length, 4)},
        "mean_speed_mps": round(sum(v["root"].length for v in vel) /
                                max(1, len(vel)), 4),
        "min_z_m": round(min(zs), 5),
    }


def gate(res, loop, name):
    fails, warns = [], []
    for key, (warn, fail) in GATES.items():
        if (key == "end_settled_mps" and loop) or \
           (key == "float_p10_cm" and name in SKIP_FLOAT):
            continue
        if res[key] > fail:
            fails.append("%s %.4f > %s" % (key, res[key], fail))
        elif res[key] > warn:
            warns.append("%s %.4f > %s" % (key, res[key], warn))
    return fails, warns


# ---------------------------------------------------------------------------
# Pose authoring. All amplitudes scale with the model's height / 1.55 u.
# ---------------------------------------------------------------------------


def with_settle(fn, frames):
    hold_at = frames
    return lambda f, pf: fn(min(f, hold_at), min(pf, hold_at))


def humanoid_attack(name, info, t, S, extra=None):
    p = blank(info)
    a = keys_at(t, [(0, 0), (0.30, -0.55), (0.48, 1.0), (0.70, 0.18), (1, 0)])
    step_forward(p, info, 0.22 * S * a)  # negative during windup = steps back
    put(p, "torso", 0.02 * a, 0.0, 0.22 * a, dz=-0.03 * S * abs(a))
    put(p, "head", -0.10 * a, 0.0, -0.08 * a)
    arm = "arm_r"
    if arm in info["empties"]:
        put(p, arm, -0.50 + 2.0 * a, 0.0, 0.10)
    if "lower_arm_r" in info["empties"]:
        put(p, "lower_arm_r", -0.55 - 0.35 * max(0.0, a))
    if "arm_l" in info["empties"]:
        put(p, "arm_l", -0.40 - 0.45 * a, 0.0, -0.08)
    if extra:
        extra(p, a, S)
    return p


def humanoid_hurt(info, t, S):
    p = blank(info)
    s = keys_at(t, [(0, 0), (0.12, 1), (0.40, 0.55), (1, 0)])
    back = 0.13 * S * keys_at(t, [(0, 0), (0.3, 1), (1, 1)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, back, 0.0)
    for k in info["contacts"]:
        L = info["lengths"].get(k, 0.0)
        if L > 0.02:
            put(p, k, -back / L)
    put(p, "torso", -0.10 * s, 0.0, -0.10 * s)
    put(p, "head", -0.48 * s, 0.0, 0.12 * s)
    if "arm_l" in info["empties"]:
        put(p, "arm_l", 0.35 * s - 0.15, 0.0, -0.22 * s)
    if "arm_r" in info["empties"]:
        put(p, "arm_r", 0.30 * s - 0.15, 0.0, 0.22 * s)
    return p


def humanoid_death(info, t, S):
    p = blank(info)
    back = 0.30 * S * keys_at(t, [(0, 0), (0.55, 1), (1, 1)])
    fall = keys_at(t, [(0, 0), (0.25, -0.12), (0.70, -0.95), (1, -1.35)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, back, -0.04 * S * abs(fall))
    put(p, "torso", fall, 0.0, 0.06 * fall, dz=-0.05 * S * abs(fall))
    put(p, "head", -0.28 * fall, 0.0, 0.0)
    for k in info["contacts"]:
        L = info["lengths"].get(k, 0.0)
        if L > 0.02:
            put(p, k, -back / L + 0.30 * fall)
    if "arm_l" in info["empties"]:
        put(p, "arm_l", 0.55 * abs(fall) - 0.10, 0.0, -0.20)
    if "arm_r" in info["empties"]:
        put(p, "arm_r", 0.65 * abs(fall) - 0.10, 0.0, 0.20)
    return p


POSES = {}


def reg(name, frames, loop, fn, notes, kind, skip_float=False):
    POSES[name] = {"frames": frames, "loop": loop, "fn": fn, "notes": notes,
                   "kind": kind, "skip_float": skip_float}


# --- enemy clips -----------------------------------------------------------

reg("slime_hop", 24, True,
    lambda info, t, ph, S: (
        put(put(blank(info), "root", dz=0.24 * S * max(0.0, sin(2 * pi * ph)) ** 1.4),
            "torso", 0.20 * sin(2 * pi * ph + 0.5), 0.0, 0.12 * sin(4 * pi * ph))),
    "One bound-and-land hop per 24 f cycle; vertical on root, body wobble, "
    "puddle planted; in place (root offset 0).", "enemy", skip_float=True)

for _model in ("goblin", "skeleton"):
    _tag = "goblin" if _model == "goblin" else "skeleton"
    reg(_tag + "_attack", 36, False,
        (lambda info, t, ph, S, m=_model: humanoid_attack(m, info, t, S)),
        "Back-step windup then overhead chop, returning to origin; plant "
        "compensated from derived leg lengths.", "enemy")
    reg(_tag + "_hurt", 30, False,
        (lambda info, t, ph, S: humanoid_hurt(info, t, S)),
        "Impact recoil 0-6 f, backwards step on root, settles by frame 30.",
        "enemy")
    reg(_tag + "_death", 48, False,
        (lambda info, t, ph, S: humanoid_death(info, t, S)),
        "Stagger, buckle backwards, laid-out finish for ragdoll handover.",
        "enemy", skip_float=True)


def wolf_attack(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, 0), (0.30, -0.35), (0.50, 1), (0.75, 0.2), (1, 0)])
    step_forward(p, info, 0.28 * S * max(0.0, a))
    put(p, "torso", -0.16 * max(0.0, -a) + 0.22 * max(0.0, a), dz=-0.05 * S * abs(a))
    put(p, "head", -0.50 * a, 0.0, 0.04 * sin(2 * pi * ph))
    for k in ("leg_0", "leg_1", "leg_2", "leg_3"):
        put(p, k, 0.30 * a)
    put(p, "tail", 0.0, 0.35 * a, 0.0)
    return p


def wolf_hurt(info, t, ph, S):
    p = blank(info)
    s = keys_at(t, [(0, 0), (0.12, 1), (0.45, 0.45), (1, 0)])
    back = 0.16 * S * keys_at(t, [(0, 0), (0.3, 1), (1, 1)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, back, 0.0)
    put(p, "torso", -0.28 * s)
    put(p, "head", -0.55 * s, 0.0, 0.12 * s)
    for k in ("leg_0", "leg_1", "leg_2", "leg_3"):
        put(p, k, -0.22 * s)
    put(p, "tail", 0.0, -0.30 * s)
    return p


def wolf_death(info, t, ph, S):
    p = blank(info)
    fall = keys_at(t, [(0, 0), (0.28, -0.20), (0.72, -1.1), (1, -1.45)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, 0.18 * S, -0.03 * S)
    put(p, "torso", 0.20 * fall, 1.15 * fall, 0.0, dz=-0.12 * S * abs(fall))
    put(p, "head", -0.25 * fall, 0.0, 0.0)
    for k in ("leg_0", "leg_1", "leg_2", "leg_3"):
        put(p, k, 0.45 * abs(fall))
    return p


reg("wolf_attack", 36, False, wolf_attack,
    "Dire wolf lunge up to 0.28 u scaled and recover; head snaps into the bite, "
    "paws plant.", "enemy")
reg("wolf_hurt", 30, False, wolf_hurt,
    "Flinch back with head up and forelegs buckling; settles before frame 30.",
    "enemy")
reg("wolf_death", 48, False, wolf_death,
    "Roll onto the side, legs relax, laid-out finish for ragdoll handover.",
    "enemy", skip_float=True)


def spider_scuttle(info, t, ph, S):
    p = blank(info)
    put(p, "torso", 0.04 * sin(4 * pi * ph), 0.0, 0.03 * sin(2 * pi * ph),
        dz=0.012 * S * sin(4 * pi * ph))
    for i in range(4):
        for side in ("l", "r"):
            phase = 2 * pi * (ph + i * 0.25 + (0.5 if side == "r" else 0.0))
            put(p, "upper_%s%d" % (side, i), 0.32 * sin(phase), 0.0,
                0.06 * sin(phase))
            put(p, "lower_%s%d" % (side, i), -0.22 * max(0.0, sin(phase)))
    return p


def spider_attack(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, 0), (0.32, -0.4), (0.55, 1), (1, 0.15)])
    step_forward(p, info, 0.20 * S * max(0.0, a))
    put(p, "torso", -0.30 * max(0.0, -a) + 0.38 * max(0.0, a),
        dz=0.05 * S * abs(a))
    for side in ("l", "r"):
        for i in range(4):
            put(p, "upper_%s%d" % (side, i), 0.25 * a, 0.0,
                (0.20 if side == "l" else -0.20) * a)
    return p


reg("spider_scuttle", 24, True, spider_scuttle,
    "In-place rapid scuttle: eight legs in alternating waves, body bob; "
    "root offset 0.", "enemy")
reg("spider_attack", 36, False, spider_attack,
    "Rear back then lunge 0.20 u scaled with all legs spread and fangs forward.",
    "enemy")


def archer_shoot(info, t, ph, S):
    p = blank(info)
    d = keys_at(t, [(0, 0), (0.35, 1), (0.50, 1), (0.62, 0), (1, 0)])
    put(p, "torso", 0.0, 0.0, -0.10 * d)
    put(p, "head", 0.05 * d)
    put(p, "arm_l", -0.95 - 0.35 * d)
    put(p, "lower_arm_l", -0.30)
    put(p, "arm_r", -1.00 + 0.85 * d)
    put(p, "lower_arm_r", -0.95 - 0.25 * d)
    return p


reg("archer_shoot", 36, False, archer_shoot,
    "Draw the bow 0-11 f, loose at 15 f, recover; in place, feet planted.",
    "enemy")


def brute_smash(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, 0), (0.42, -0.7), (0.62, 1), (1, 0.12)])
    step_forward(p, info, 0.22 * S * max(0.0, a))
    put(p, "torso", -0.16 * max(0.0, -a) + 0.30 * max(0.0, a),
        dz=-0.04 * S * abs(a))
    put(p, "head", 0.18 * max(0.0, a))
    for side in ("l", "r"):
        put(p, "arm_" + side, -0.60 + 2.7 * a)
        if "lower_arm_" + side in info["empties"]:
            put(p, "lower_arm_" + side, -0.45 - 0.35 * max(0.0, a))
    return p


reg("brute_smash", 42, False, brute_smash,
    "Two-fisted overhead slam, up to 0.22 u scaled step-in, heavy recovery.",
    "enemy")

# --- boss telegraph / attack ----------------------------------------------


def scorpion_telegraph(info, t, ph, S):
    p = blank(info)
    g = keys_at(t, [(0, 0), (0.55, 1), (1, 1)])
    put(p, "torso", -0.12 * g, dz=-0.03 * S * g)
    put(p, "head", -0.16 * g)
    put(p, "tail_1", -0.38 * g)
    put(p, "tail_2", -0.45 * g)
    put(p, "tail_3", -0.50 * g)
    put(p, "arm_l", 0.0, 0.0, -0.32 * g)
    put(p, "arm_r", 0.0, 0.0, 0.32 * g)
    return p


def scorpion_attack(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, -0.8), (0.42, 1), (1, 0.05)])
    step_forward(p, info, 0.30 * S * max(0.0, a))
    put(p, "torso", 0.18 * max(0.0, a), dz=-0.03 * S * abs(a))
    put(p, "tail_1", 0.70 * a)
    put(p, "tail_2", 0.55 * a)
    put(p, "tail_3", 0.35 * a)
    put(p, "arm_l", 0.0, 0.0, 0.30 * a)
    put(p, "arm_r", 0.0, 0.0, -0.30 * a)
    return p


def toad_telegraph(info, t, ph, S):
    p = blank(info)
    g = keys_at(t, [(0, 0), (0.55, 1), (1, 1)])
    put(p, "torso", 0.0, 0.0, 0.0, dz=-0.07 * S * g)
    put(p, "head", 0.24 * g)
    put(p, "crown", -0.10 * g)
    put(p, "leg_l", 0.32 * g)
    put(p, "leg_r", 0.32 * g)
    return p


def toad_attack(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, 0), (0.34, 1), (1, 0.1)])
    step_forward(p, info, 0.28 * S * max(0.0, a))
    put(p, "torso", 0.38 * a, dz=0.06 * S * a)
    put(p, "head", -0.48 * a)
    put(p, "leg_l", -0.28 * a)
    put(p, "leg_r", -0.28 * a)
    return p


def storm_telegraph(info, t, ph, S):
    p = blank(info)
    g = keys_at(t, [(0, 0), (0.55, 1), (1, 1)])
    put(p, "torso", -0.10 * g, 0.0, 0.04 * g)
    put(p, "head", -0.22 * g)
    put(p, "arm_l", -0.60 * g)
    put(p, "arm_r", -2.20 * g)
    return p


def storm_attack(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, -1), (0.40, 1), (1, 0.1)])
    step_forward(p, info, 0.18 * S * max(0.0, a))
    put(p, "torso", 0.30 * max(0.0, a))
    put(p, "head", 0.16 * max(0.0, a))
    put(p, "arm_l", -0.45 * a)
    put(p, "arm_r", -1.10 + 0.90 * a)
    return p


def leviathan_telegraph(info, t, ph, S):
    p = blank(info)
    g = keys_at(t, [(0, 0), (0.55, 1), (1, 1)])
    put(p, "torso", -0.24 * g, 0.0, 0.0, dz=0.05 * S * g)
    put(p, "head", -0.42 * g)
    put(p, "jaw", 0.38 * g)
    put(p, "tentacle_l", 0.0, 0.42 * g, 0.0)
    put(p, "tentacle_r", 0.0, -0.42 * g, 0.0)
    return p


def leviathan_attack(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, -0.5), (0.42, 1), (1, 0.05)])
    step_forward(p, info, 0.32 * S * max(0.0, a))
    put(p, "torso", 0.20 * a, dz=-0.03 * S * abs(a))
    put(p, "head", 0.48 * a)
    put(p, "jaw", -0.52 * a)
    put(p, "tentacle_l", 0.0, -0.55 * a, 0.0)
    put(p, "tentacle_r", 0.0, 0.55 * a, 0.0)
    return p


def colossus_telegraph(info, t, ph, S):
    p = blank(info)
    g = keys_at(t, [(0, 0), (0.55, 1), (1, 1)])
    put(p, "torso", -0.12 * g, dz=-0.04 * S * g)
    put(p, "head", -0.16 * g)
    for side in ("l", "r"):
        put(p, "arm_" + side, -1.80 * g)
    return p


def colossus_attack(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, -1), (0.45, 1), (1, 0.1)])
    step_forward(p, info, 0.34 * S * max(0.0, a))
    put(p, "torso", 0.28 * max(0.0, a), dz=-0.06 * S * abs(a))
    put(p, "head", 0.20 * max(0.0, a))
    for side in ("l", "r"):
        put(p, "arm_" + side, -0.45 + 1.05 * a)
    return p


BOSSES = [
    ("scorpion_queen", "scorpion_queen_telegraph", scorpion_telegraph, 48,
     "Rear the tail over the head and spread both pincers; telegraph hold."),
    ("scorpion_queen", "scorpion_queen_attack", scorpion_attack, 42,
     "Tail stinger strikes down and pincers snap; up to 0.30 u scaled lunge."),
    ("toad_king", "toad_king_telegraph", toad_telegraph, 48,
     "Crouch, tilt the crowned head up and brace the legs; telegraph hold."),
    ("toad_king", "toad_king_attack", toad_attack, 36,
     "Explosive forward lunge with a downward head snap; up to 0.28 u scaled."),
    ("stormcaller", "stormcaller_telegraph", storm_telegraph, 48,
     "Raise staff and orb overhead, robe settles; telegraph hold."),
    ("stormcaller", "stormcaller_attack", storm_attack, 36,
     "Slam the staff down, bolts sweep forward; up to 0.18 u scaled step-in."),
    ("ocean_leviathan", "ocean_leviathan_telegraph", leviathan_telegraph, 48,
     "Rear the coils, open the jaw and spread both tentacles; telegraph hold."),
    ("ocean_leviathan", "ocean_leviathan_attack", leviathan_attack, 42,
     "Head-and-jaw strike with a tentacle sweep; up to 0.32 u scaled lunge."),
    ("colossus", "colossus_telegraph", colossus_telegraph, 48,
     "Both fists rise, torso leans back; telegraph hold."),
    ("colossus", "colossus_attack", colossus_attack, 42,
     "Two-fisted ground slam with up to 0.34 u scaled step-in."),
]
for _model, _name, _fn, _frames, _notes in BOSSES:
    reg(_name, _frames, False, _fn, _notes, "boss")

# --- NPC clips -------------------------------------------------------------


def villager_idle(info, t, ph, S):
    p = blank(info)
    put(p, "torso", 0.02 * sin(ph), 0.0, 0.015 * sin(ph + 0.6))
    put(p, "head", -0.01 * sin(ph), 0.0, 0.05 * sin(ph + 1.1))
    put(p, "arm_l", -0.12 + 0.03 * sin(ph))
    put(p, "arm_r", -0.12 - 0.03 * sin(ph))
    return p


def villager_wave(info, t, ph, S):
    p = blank(info)
    g = keys_at(t, [(0, 0), (0.22, 1), (1, 1)])
    put(p, "arm_r", -2.30 * g, 0.0, -0.25 * g + 0.38 * g * sin(4 * pi * ph))
    put(p, "arm_l", -0.15)
    put(p, "head", 0.0, 0.0, -0.08 * g)
    put(p, "torso", 0.0, 0.0, 0.04 * g)
    return p


def guard_salute(info, t, ph, S):
    p = blank(info)
    a = keys_at(t, [(0, 0), (0.28, 1), (0.72, 1), (1, 0)])
    put(p, "arm_r", -2.05 * a, 0.0, -0.38 * a)
    put(p, "arm_l", -0.85)
    put(p, "torso", -0.06 * a)
    put(p, "head", -0.06 * a)
    return p


def guard_patrol(info, t, ph, S):
    p = blank(info)
    swing = sin(ph)
    put(p, "torso", 0.0, 0.0, 0.04 * swing, dz=0.008 * S * cos(2 * ph))
    put(p, "head", 0.0, 0.0, -0.04 * swing)
    put(p, "arm_l", -0.85)
    put(p, "arm_r", -0.25 - 0.28 * swing)
    put(p, "leg_l", 0.35 * swing)
    put(p, "leg_r", -0.35 * swing)
    return p


def merchant_greet(info, t, ph, S):
    p = blank(info)
    g = keys_at(t, [(0, 0), (0.28, 1), (0.62, 1), (1, 0)])
    put(p, "torso", 0.12 * g)
    put(p, "head", 0.08 * g)
    put(p, "arm_l", -1.05 * g - 0.12)
    put(p, "arm_r", -0.95 * g - 0.12)
    return p


def golem_standby(info, t, ph, S):
    p = blank(info)
    put(p, "torso", 0.0, 0.0, 0.02 * sin(ph), dz=0.008 * S * sin(2 * ph))
    put(p, "head", 0.0, 0.0, -0.03 * sin(ph))
    put(p, "arm_l", -0.10 + 0.02 * sin(ph))
    put(p, "arm_r", -0.10 - 0.02 * sin(ph))
    return p


NPC = [
    ("npc_villager_child", "villager_idle", villager_idle, 120, True,
     "Idle villager: small weight shift and glance; feet planted, in place."),
    ("npc_villager_child", "villager_wave", villager_wave, 42, False,
     "Raise the right arm and wave side to side, then lower; in place."),
    ("guard", "guard_salute", guard_salute, 36, False,
     "Right hand to helm, hold the salute, lower; shield arm steady."),
    ("guard", "guard_patrol", guard_patrol, 24, True,
     "In-place guard patrol gait: arm swing and small torso counter-rotation."),
    ("npc_merchant", "merchant_greet", merchant_greet, 42, False,
     "Merchant bows slightly and opens both arms in greeting; in place."),
    ("golem", "golem_standby", golem_standby, 120, True,
     "Stone golem standby: slow crystal breath and tiny torso sway, feet planted."),
]
for _model, _name, _fn, _frames, _loop, _notes in NPC:
    reg(_name, _frames, _loop, _fn, _notes, "npc")

CLIP_MODEL = {}
for _name in POSES:
    if _name in ("slime_hop",):
        CLIP_MODEL[_name] = "slime"
    elif _name.startswith("goblin_"):
        CLIP_MODEL[_name] = "goblin"
    elif _name.startswith("skeleton_"):
        CLIP_MODEL[_name] = "skeleton"
    elif _name.startswith("wolf_"):
        CLIP_MODEL[_name] = "npc_dire_wolf"
    elif _name.startswith("spider_"):
        CLIP_MODEL[_name] = "spider"
    elif _name.startswith("archer_"):
        CLIP_MODEL[_name] = "archer"
    elif _name.startswith("brute_"):
        CLIP_MODEL[_name] = "brute"
    elif _name.startswith("villager_"):
        CLIP_MODEL[_name] = "npc_villager_child"
    elif _name.startswith("guard_"):
        CLIP_MODEL[_name] = "guard"
    elif _name.startswith("merchant_"):
        CLIP_MODEL[_name] = "npc_merchant"
    elif _name.startswith("golem_"):
        CLIP_MODEL[_name] = "golem"
for _model, _name, _fn, _frames, _notes in BOSSES:
    CLIP_MODEL[_name] = _model

SKIP_FLOAT = {n for n, cfg in POSES.items() if cfg["skip_float"]}


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    os.makedirs(OUT, exist_ok=True)
    scene = bpy.context.scene
    scene.render.fps = FPS
    scene.frame_start = 0
    scene.frame_end = 120

    model_info = {}
    for model in MODELS_USED:
        path = os.path.join(MODELS, model + ".glb")
        model_info[model] = {
            "glb": "assets/models/%s.glb" % model,
            "sha256": sha256(path),
            "height_m": 0.0, "scale_to_1p55u": 0.0,
        }

    records, fails_all, warns_all = [], [], []
    for name, cfg in POSES.items():
        model = CLIP_MODEL[name]
        frames, loop, fn = cfg["frames"], cfg["loop"], cfg["fn"]
        info = build_rig(model)  # imports + derives bounds for every clip
        bpy.context.scene.render.fps = FPS  # build_rig's factory reset -> 24 fps
        model_info[model]["height_m"] = round(info["height"], 4)
        model_info[model]["scale_to_1p55u"] = round(info["scale"], 5)
        S = info["scale"]
        pose_fn = (lambda f, pf, fn=fn, S=S: fn(info, f / frames,
                                               (pf / frames) * 2 * pi, S))
        total = frames + (0 if loop else SETTLE_FRAMES)
        pose_fn = pose_fn if loop else with_settle(pose_fn, frames)
        action, slots = author(info, name, pose_fn, total, loop, lock=not loop)
        bind(info, action, slots)
        lift = ground_clamp(info, total + 1 if loop else total, loop)
        res = measure(info, total, loop)
        fails, warns = gate(res, loop, name)
        res["max_ground_lift_cm"] = round(lift * 100, 3)
        res.update({
            "clip": name, "model": model, "kind": cfg["kind"],
            "frames": total + 1 if loop else total,
            "fps": FPS, "duration_s": round((total + 1 if loop else total) / FPS, 4),
            "loop": loop,
            "root_motion": (res["root_offset_m"]["magnitude"] > 0.005 or
                            res["mean_speed_mps"] > 0.01),
            "scale_to_1p55u": round(S, 5),
            "contacts": info["contacts"],
            "source_sha256": model_info[model]["sha256"],
            "status": "FAIL" if fails else ("WARN" if warns else "PASS"),
            "fails": fails, "warns": warns, "notes": cfg["notes"],
            "path": "assets/animations/wave2/%s.glb" % name,
        })
        bpy.ops.export_scene.gltf(
            filepath=os.path.join(OUT, name + ".glb"), export_format="GLB",
            export_yup=True, export_animations=True,
            export_animation_mode="ACTIONS", export_nla_strips=False,
            export_extras=True, export_apply=False, use_selection=False)
        records.append(res)
        fails_all += [(name, x) for x in fails]
        warns_all += [(name, x) for x in warns]
        print("[%s] %-26s %-14s %3df seam=%.3fcm slide=%.3fcm pen=%.2fcm "
              "p10=%.2fcm off=%.2fm" %
              (res["status"], name, model, frames, res["seam_position_cm"],
               res["foot_slide_cm"], res["ground_penetration_cm"],
               res["float_p10_cm"], res["root_offset_m"]["magnitude"]))

    with open(PLAYER, "rb") as fh:
        player_digest = hashlib.sha256(fh.read()).hexdigest()
    manifest = {
        "schema": "fight-animations.wave2/1",
        "generator": "scripts/generate_wave2_animations.py",
        "status": "second wave: enemy, boss and NPC clips on bound-derived pivot rigs",
        "tooling": {"blender": bpy.app.version_string, "fps": FPS,
                    "pipeline": "procedural keyframes on GLB node bounds; no mocap",
                    "gvhmr_available": False, "mixamo_available": False},
        "scale_reference": {
            "glb": "assets/models/player.glb", "sha256": player_digest,
            "height_m": 1.55,
            "note": "all travel, leans and contact compensation scale as model_height / 1.55",
        },
        "conventions": {
            "fps": FPS, "samplers": "LINEAR", "scale_keys": False,
            "direction": "front is glTF +Z / Blender -Y",
            "root_motion": "horizontal travel on root only; no scale keys",
            "loop": "last key duplicates frame 0 in root-relative pose",
            "gates_warn_fail": GATES,
        },
        "models": model_info,
        "clips": records,
        "summary": {
            "count": len(records),
            "pass": sum(1 for r in records if r["status"] == "PASS"),
            "warn": sum(1 for r in records if r["status"] == "WARN"),
            "fail": sum(1 for r in records if r["status"] == "FAIL"),
            "enemy": sum(1 for r in records if r["kind"] == "enemy"),
            "boss": sum(1 for r in records if r["kind"] == "boss"),
            "npc": sum(1 for r in records if r["kind"] == "npc"),
        },
        "fails": [{"clip": c, "check": x} for c, x in fails_all],
        "warnings": [{"clip": c, "check": x} for c, x in warns_all],
    }
    path = os.path.join(OUT, "manifest.json")
    with open(path, "w") as fh:
        json.dump(manifest, fh, indent=2)
    print("SUMMARY", manifest["summary"])
    return 1 if fails_all else 0


if __name__ == "__main__":
    sys.exit(main())
