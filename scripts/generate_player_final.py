"""From-scratch final player animation set for the verified 1.551 u character.

    blender -b --python scripts/generate_player_final.py

The rig is DERIVED from assets/models/player.glb mesh bounds (not hardcoded),
so a changed final player GLB is re-bound automatically on the next run.
All travel, stride and limb targets are expressed in metres against those
derived joints. 30 fps, LINEAR samplers, root carries horizontal travel.

Outputs assets/animations/player/*.glb, manifest.json, review sheet inputs.
"""

import hashlib
import json
import math
import os
import sys
from math import atan2, cos, pi, sin, sqrt

import bpy
from mathutils import Euler, Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
PLAYER = os.path.join(ROOT, "assets", "models", "player.glb")
METRICS = os.path.join(ROOT, "assets", "models", "metrics.json")
OUT = os.path.join(ROOT, "assets", "animations", "player")
FPS = 30

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from generate_combat_animations import iter_fcurves  # noqa: E402

DIRS = {
    "f": (0.0, 1.0), "fl": (0.7071, 0.7071), "l": (1.0, 0.0),
    "bl": (0.7071, -0.7071), "b": (0.0, -1.0), "br": (-0.7071, -0.7071),
    "r": (-1.0, 0.0), "fr": (-0.7071, 0.7071),
}
# [warn, fail] gates from game-animation references/validation.md
GATES = {
    "seam_position_cm": (0.5, 2.0),
    "seam_velocity_mps": (0.3, 0.8),
    "root_velocity_mps": (0.1, 0.3),
    "foot_slide_cm": (2.0, 5.0),
    "ground_penetration_cm": (1.0, 3.0),
    "float_p10_cm": (2.0, 5.0),
    "end_settled_mps": (0.05, 0.15),
}


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


def solve_2bone(hy, hz, ty, tz, l1, l2):
    vy, vz = ty - hy, tz - hz
    d = clamp(math.hypot(vy, vz), abs(l1 - l2) + 1e-4, l1 + l2 - 1e-4)
    uy, uz = vy / max(d, 1e-9), vz / max(d, 1e-9)
    wy, wz = uz, -uy
    if wy > 0:
        wy, wz = -wy, -wz
    a = math.acos(clamp((l1 * l1 + d * d - l2 * l2) / (2 * l1 * d), -1, 1))
    ky, kz = hy + uy * l1 * cos(a) + wy * l1 * sin(a), hz + uz * l1 * cos(a) + wz * l1 * sin(a)
    ang = lambda dy, dz: atan2(dy, -dz)
    up = ang(ky - hy, kz - hz)
    return up, ang(ty - ky, tz - kz) - up


# ---------------------------------------------------------------------------
# Data-driven rig
# ---------------------------------------------------------------------------

PARTS = ("root", "hips", "spine", "chest", "neck", "head",
         "upper_arm_l", "lower_arm_l", "hand_l", "upper_arm_r", "lower_arm_r", "hand_r",
         "upper_leg_l", "lower_leg_l", "foot_l", "toes_l",
         "upper_leg_r", "lower_leg_r", "foot_r", "toes_r", "attach_hand_r")
MESH_TO_JOINT = {
    "tunic": "spine", "belt": "hips", "buckle": "hips", "cape": "chest",
    "neck": "neck", "head": "head", "hair": "head", "eye_l": "head", "eye_r": "head",
    "mouth": "head", "circlet": "head",
    "arm_l": "lower_arm_l", "arm_r": "lower_arm_r",
    "hand_l": "hand_l", "hand_r": "hand_r",
    "leg_l": "upper_leg_l", "leg_r": "upper_leg_r",
    "boot_l": "foot_l", "boot_r": "foot_r",
}
CHAIN = {
    "hips": "root", "spine": "hips", "chest": "spine", "neck": "chest",
    "head": "neck",
    "upper_arm_l": "chest", "lower_arm_l": "upper_arm_l", "hand_l": "lower_arm_l",
    "upper_arm_r": "chest", "lower_arm_r": "upper_arm_r", "hand_r": "lower_arm_r",
    "attach_hand_r": "hand_r",
    "upper_leg_l": "hips", "lower_leg_l": "upper_leg_l", "foot_l": "lower_leg_l",
    "toes_l": "foot_l", "upper_leg_r": "hips", "lower_leg_r": "upper_leg_r",
    "foot_r": "lower_leg_r", "toes_r": "foot_r",
}


def bounds(o):
    pts = [o.matrix_world @ Vector(c) for c in o.bound_box]
    xs, ys, zs = [p.x for p in pts], [p.y for p in pts], [p.z for p in pts]
    return (min(xs), max(xs)), (min(ys), max(ys)), (min(zs), max(zs))


def centre(o):
    (x0, x1), (y0, y1), (z0, z1) = bounds(o)
    return Vector(((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2))


def build_rig():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=PLAYER)
    meshes = {o.name: o for o in bpy.data.objects if o.type == "MESH"}
    missing = set(MESH_TO_JOINT) - set(meshes)
    if missing:
        raise RuntimeError("player.glb changed; missing parts: %s" % sorted(missing))
    B = {n: bounds(o) for n, o in meshes.items()}
    C = {n: centre(o) for n, o in meshes.items()}

    # Joints derived from the real mesh bounds: all values scale with the GLB.
    height = max(b[2][1] for b in B.values())
    boot_z = B["boot_l"][2]
    leg_z = B["leg_l"][2]
    arm_z = B["arm_l"][2]
    # Hip pivot at the top of the thigh, not the belt's lower edge, so the
    # derived leg reach covers the stride without IK clamping.
    hip_z = leg_z[1] - 0.01
    knee_z = leg_z[0] + 0.4 * (leg_z[1] - leg_z[0])
    ankle_z = C["boot_l"].z + 0.03
    shoulder_z = arm_z[1]
    elbow_z = C["arm_l"].z - 0.02
    wrist_z = C["hand_l"].z
    arm_x = abs(C["arm_l"].x) - 0.06
    leg_x = abs(C["leg_l"].x)
    foot_x = abs(C["boot_l"].x)
    J = {
        "root": (0.0, 0.0, 0.0),
        "hips": (0.0, 0.0, hip_z),
        "spine": (0.0, 0.0, C["tunic"].z - 0.11),
        "chest": (0.0, 0.0, B["tunic"][2][1] - 0.08),
        "neck": (0.0, 0.0, C["neck"].z),
        "head": (0.0, 0.0, C["head"].z - 0.06),
    }
    for s, sign in (("l", -1), ("r", 1)):
        J.update({
            "upper_arm_" + s: (sign * arm_x, 0.0, shoulder_z),
            "lower_arm_" + s: (sign * (abs(C["arm_" + s].x)), 0.0, elbow_z),
            "hand_" + s: (sign * abs(C["hand_" + s].x), 0.0, wrist_z),
            "upper_leg_" + s: (sign * leg_x, 0.0, hip_z),
            "lower_leg_" + s: (sign * leg_x, 0.0, knee_z),
            "foot_" + s: (sign * foot_x, 0.0, ankle_z),
            "toes_" + s: (sign * foot_x, -0.12, ankle_z - 0.03),
        })
    J["attach_hand_r"] = (J["hand_r"][0], -0.06, J["hand_r"][2])
    rig = {}
    for name in PARTS:
        e = bpy.data.objects.new(name, None)
        e.empty_display_type = "PLAIN_AXES"
        e.empty_display_size = 0.06
        bpy.context.collection.objects.link(e)
        rig[name] = e
    bpy.context.view_layer.update()
    for name, parent in CHAIN.items():
        e = rig[name]
        e.parent = rig[parent]
        e.location = tuple(j - p for j, p in zip(J[name], J[parent]))
    bpy.context.view_layer.update()
    for name, j in MESH_TO_JOINT.items():
        o = meshes[name]
        mw = o.matrix_world.copy()
        o.parent = rig[j]
        o.matrix_parent_inverse = rig[j].matrix_world.inverted()
        o.matrix_world = mw
    bpy.context.view_layer.update()
    dims = {
        "height_m": round(height, 4),
        "boot_bottom_m": round(boot_z[0], 4),
        "leg_l1_m": round(hip_z - knee_z, 4),
        "leg_l2_m": round(knee_z - ankle_z, 4),
        "arm_l1_m": round(shoulder_z - elbow_z, 4),
        "arm_l2_m": round(elbow_z - wrist_z, 4),
        "foot_sole_m": round(ankle_z - 0.12, 4),
    }
    rest_loc = {n: tuple(e.location) for n, e in rig.items()}
    return rig, meshes, J, rest_loc, dims


REST = {}
JOINTS = {}
DIMS = {}


def pose():
    return {n: {"loc": (0.0, 0.0, 0.0), "rot": (0.0, 0.0, 0.0)} for n in PARTS}


def apply(rig, p):
    for n, tr in p.items():
        o = rig[n]
        o.location = tuple(r + d for r, d in zip(REST[n], tr["loc"]))
        o.rotation_mode = "XYZ"
        o.rotation_euler = tr["rot"]


def leg_ik(p, side, ankle_y, ankle_z, flat=0.0, foot_x=None):
    """Plant an ankle in root space; compensate hips rotation and yaw."""
    hips_pos = Vector((0.0, 0.0, JOINTS["hips"][2])) + Vector(p["hips"]["loc"])
    hip_off = Vector(JOINTS["upper_leg_" + side]) - Vector(JOINTS["hips"])
    rot = Euler(p["hips"]["rot"], "XYZ").to_matrix()
    hip_root = hips_pos + rot @ hip_off
    want = Vector((JOINTS["foot_" + side][0] if foot_x is None else foot_x,
                   ankle_y, ankle_z))
    local = rot.inverted() @ (want - hip_root)
    up, low = solve_2bone(0.0, 0.0, local.y, local.z, DIMS["leg_l1_m"], DIMS["leg_l2_m"])
    roll = atan2(-local.x, max(0.05, -local.z))
    p["upper_leg_" + side]["rot"] = (up, roll, 0.0)
    p["lower_leg_" + side]["rot"] = (low, 0.0, 0.0)
    leg_world = (rot @ Euler((up, roll, 0.0), "XYZ").to_matrix()
                 @ Euler((low, 0.0, 0.0), "XYZ").to_matrix())
    p["foot_" + side]["rot"] = tuple(
        (leg_world.inverted() @ Euler((flat, 0.0, 0.0), "XYZ").to_matrix()).to_euler("XYZ"))


def arm_ik(p, side, hy, hz):
    up, low = solve_2bone(JOINTS["upper_arm_" + side][1], JOINTS["upper_arm_" + side][2],
                          hy, hz, DIMS["arm_l1_m"], DIMS["arm_l2_m"])
    p["upper_arm_" + side]["rot"] = (up, 0.0, 0.0)
    p["lower_arm_" + side]["rot"] = (low, 0.0, 0.0)


def guard(p, breath=0.0):
    arm_ik(p, "r", -0.30, JOINTS["hand_r"][2] + 0.32)
    arm_ik(p, "l", -0.28, JOINTS["hand_l"][2] + 0.30)
    p["hand_r"]["rot"] = (0.10, 0.0, 0.0)
    p["hand_l"]["rot"] = (0.10, 0.0, 0.0)
    p["spine"]["rot"] = (0.015 * breath, 0.0, 0.01 * breath)


# ---------------------------------------------------------------------------
# Locomotion: stride and travel derived from the real leg length
# ---------------------------------------------------------------------------

LOCO = {
    "walk": {"cycle": 24, "speed": 1.05, "drop": 0.08, "arm": 0.30, "lean": 0.0},
    "run": {"cycle": 18, "speed": 1.60, "drop": 0.10, "arm": 0.55, "lean": -0.10},
}


def loco_pose(f, pf, band="walk", direction="f"):
    cfg = LOCO[band]
    p = pose()
    c = cfg["cycle"]
    ph = (pf % c) / c
    dx, dz = DIRS[direction]
    yaw = atan2(dx, dz)
    travel = cfg["speed"] * f / FPS
    p["root"]["rot"] = (0.0, 0.0, yaw)
    p["root"]["loc"] = (dx * travel, -dz * travel, 0.0)
    p["hips"]["loc"] = (0.0, 0.0, -cfg["drop"] + 0.009 * cos(4 * pi * ph))
    p["spine"]["rot"] = (cfg["lean"], 0.0, 0.05 * sin(2 * pi * ph))
    p["hips"]["rot"] = (0.0, 0.0, -0.035 * sin(2 * pi * ph))
    p["head"]["rot"] = (0.0, 0.0, -0.04 * sin(2 * pi * ph))
    stride = cfg["speed"] * c / (2 * FPS)          # root travel in one stance
    half = stride / 2.0
    hip_z = JOINTS["hips"][2] - cfg["drop"]
    for side, off in (("l", 0.0), ("r", 0.5)):
        q = (ph + off) % 1.0
        if q < 0.5:
            u = q / 0.5
            ay = -half + stride * u
            az = JOINTS["foot_" + side][2] + 0.018 - 0.018 * sin(pi * u) ** 2
            pitch = 0.0   # flat sole: heel/ball stay fixed while planted
        else:
            u = (q - 0.5) / 0.5
            h00, h10 = 2 * u ** 3 - 3 * u ** 2 + 1, u ** 3 - 2 * u ** 2 + u
            h01, h11 = -2 * u ** 3 + 3 * u ** 2, u ** 3 - u ** 2
            ay = h00 * half + h10 * stride + h01 * (-half) + h11 * stride
            az = JOINTS["foot_" + side][2] + 0.018 + 0.13 * sin(pi * u) ** 2
            pitch = 0.0
        up, low = solve_2bone(0.0, hip_z, ay, az, DIMS["leg_l1_m"], DIMS["leg_l2_m"])
        p["upper_leg_" + side]["rot"] = (up, 0.0, 0.0)
        p["lower_leg_" + side]["rot"] = (low, 0.0, 0.0)
        # Keep the sole level in world space; otherwise the shin angle sweeps
        # the heel/ball points and a planted foot is never detected.
        chain = (Euler(p["hips"]["rot"], "XYZ").to_matrix()
                 @ Euler((up, 0.0, 0.0), "XYZ").to_matrix()
                 @ Euler((low, 0.0, 0.0), "XYZ").to_matrix())
        p["foot_" + side]["rot"] = tuple(chain.inverted().to_euler("XYZ"))
        p["toes_" + side]["rot"] = (-0.35 * pitch, 0.0, 0.0)
    for side, sgn in (("l", -1), ("r", 1)):
        swing = sin(2 * pi * ph) * (-sgn)
        arm_ik(p, side, -0.22 - cfg["arm"] * swing * 0.5,
               JOINTS["hand_" + side][2] + 0.10 + 0.12 * max(0.0, swing))
    return p


def idle_pose(f, pf):
    p = pose()
    ph = 2 * pi * pf / 120.0
    p["hips"]["loc"] = (0.0, 0.0, -0.02 + 0.006 * sin(ph))
    guard(p, breath=sin(ph))
    for side in ("l", "r"):
        leg_ik(p, side, 0.0, JOINTS["foot_" + side][2])
    return p


# ---------------------------------------------------------------------------
# Attacks: explicit windup / release / recovery with measured root offsets
# ---------------------------------------------------------------------------

ATTACKS = {
    "attack_1": dict(frames=36, step=0.14, wind=(0.0, 0.33), release=(0.33, 0.53),
                     kind="overhead", notes="first hit: overhead cut"),
    "attack_2": dict(frames=36, step=0.16, wind=(0.0, 0.33), release=(0.33, 0.53),
                     kind="horizontal", notes="second hit: horizontal cut"),
    "attack_3": dict(frames=42, step=0.22, wind=(0.0, 0.36), release=(0.36, 0.60),
                     kind="low", notes="finisher: rising low cut"),
    "heavy_attack": dict(frames=54, step=0.25, wind=(0.0, 0.42), release=(0.42, 0.62),
                         kind="heavy", notes="slow heavy overhead, long recovery"),
}


def attack_pose(name):
    cfg = ATTACKS[name]
    frames = cfg["frames"]
    wind_end = cfg["wind"][1]
    rel_end = cfg["release"][1]

    def pose_fn(f, pf):
        t = f / frames
        p = pose()
        step = keys_at(t, [(0, 0), (wind_end, 0), (rel_end, cfg["step"]),
                           (1.0, cfg["step"])])
        p["root"]["loc"] = (0.0, -step, 0.0)
        p["hips"]["loc"] = (0.0, 0.0, keys_at(t, [(0, -0.02), (wind_end, -0.10),
                                                  (rel_end, -0.14), (1.0, -0.07)]))
        yaw = keys_at(t, [(0, 0), (wind_end, 0.50), (rel_end, -0.55),
                          (0.85, -0.10), (1.0, 0.0)])
        p["hips"]["rot"] = (0.0, 0.0, 0.30 * yaw)
        p["spine"]["rot"] = (keys_at(t, [(0, 0), (wind_end, -0.12),
                                         (rel_end, 0.16), (1.0, 0.0)]), 0.0, 0.65 * yaw)
        p["chest"]["rot"] = (0.0, 0.0, 0.25 * yaw)
        p["head"]["rot"] = (0.0, 0.0, -0.45 * yaw)
        kind = cfg["kind"]
        if kind == "overhead":
            up = keys_at(t, [(0, -0.9), (wind_end, 2.1), (rel_end, -1.5), (1.0, -0.9)])
            low = keys_at(t, [(0, -1.1), (wind_end, -0.5), (rel_end, -0.15), (1.0, -1.1)])
        elif kind == "horizontal":
            up = keys_at(t, [(0, -0.9), (wind_end, -1.9), (rel_end, 0.4), (1.0, -0.9)])
            low = keys_at(t, [(0, -1.1), (wind_end, -0.7), (rel_end, -0.2), (1.0, -1.1)])
        elif kind == "low":
            up = keys_at(t, [(0, -0.9), (wind_end, -1.7), (rel_end, 0.9), (1.0, -0.9)])
            low = keys_at(t, [(0, -1.1), (wind_end, -0.4), (rel_end, -0.1), (1.0, -1.1)])
        else:
            up = keys_at(t, [(0, -0.9), (wind_end, 2.5), (rel_end, -1.8), (1.0, -0.9)])
            low = keys_at(t, [(0, -1.1), (wind_end, -0.35), (rel_end, -0.05), (1.0, -1.1)])
        p["upper_arm_r"]["rot"] = (up, 0.0, 0.10)
        p["lower_arm_r"]["rot"] = (low, 0.0, 0.0)
        arm_ik(p, "l", -0.28, JOINTS["hand_l"][2] + 0.30 + 0.05 * sin(pi * t))
        back = keys_at(t, [(0, 0), (wind_end, 0.12), (rel_end, -0.06), (1.0, -0.06)])
        lift = keys_at(t, [(0, 0), (0.06, 0.08), (wind_end, 0.13),
                           (rel_end, 0.10), (1.0, 0.0)])
        leg_ik(p, "l", step, JOINTS["foot_l"][2])
        leg_ik(p, "r", step + back, JOINTS["foot_r"][2] + lift, flat=0.02)
        return p
    return pose_fn, frames


def block_pose(f, pf):
    t = f / 48.0
    p = pose()
    g = keys_at(t, [(0, 0), (0.2, 1), (0.7, 1), (1.0, 0)])
    p["hips"]["loc"] = (0.0, 0.0, -0.02 - 0.05 * g)
    p["spine"]["rot"] = (-0.10 * g, 0.0, 0.0)
    arm_ik(p, "r", -0.30, JOINTS["hand_r"][2] + 0.34 * g + 0.05)
    arm_ik(p, "l", -0.34, JOINTS["hand_l"][2] + 0.30 * g + 0.05)
    for side in ("l", "r"):
        leg_ik(p, side, 0.0, JOINTS["foot_" + side][2])
    return p


def dodge_roll_pose(f, pf):
    """Forward roll: 0.65 m on root, tuck, tumble, end in a low guard."""
    t = f / 42.0
    p = pose()
    travel = keys_at(t, [(0, 0), (0.18, 0.14), (0.6, 0.60), (0.9, 0.65), (1.0, 0.65)])
    p["root"]["loc"] = (0.0, -travel, 0.0)
    tuck = keys_at(t, [(0, 0), (0.18, 1), (0.75, 1), (0.95, 0), (1.0, 0)])
    roll = keys_at(t, [(0, 0), (0.18, 0), (0.55, -1.5), (0.75, -2.4),
                       (0.9, -1.5), (1.0, -1.2)])
    p["hips"]["rot"] = (roll, 0.0, 0.0)
    p["hips"]["loc"] = (0.0, 0.0, -0.02 - 0.22 * tuck)
    p["spine"]["rot"] = (0.35 * roll, 0.0, 0.0)
    p["head"]["rot"] = (-0.3 * roll, 0.0, 0.0)
    for side, sgn in (("l", -1), ("r", 1)):
        p["upper_arm_" + side]["rot"] = (-0.8 - 0.4 * tuck, 0.0, sgn * 0.2)
        p["lower_arm_" + side]["rot"] = (-0.9 - 0.5 * tuck, 0.0, 0.0)
        p["upper_leg_" + side]["rot"] = (0.3 * tuck + 0.5 * sin(pi * clamp(t / 0.8, 0, 1)),
                                         0.0, 0.0)
        p["lower_leg_" + side]["rot"] = (-1.1 * tuck, 0.0, 0.0)
        p["foot_" + side]["rot"] = (0.3 * tuck, 0.0, 0.0)
    return p


def hit_pose(heavy=False):
    frames = 36 if heavy else 30
    def pose_fn(f, pf):
        t = f / frames
        p = pose()
        back = keys_at(t, [(0, 0), (0.2, 0.18 if heavy else 0.10),
                           (0.5, 0.20 if heavy else 0.13), (1.0, 0.20 if heavy else 0.13)])
        p["root"]["loc"] = (0.0, back, 0.0)
        s = keys_at(t, [(0, 0), (0.1, 1), (0.3, 0.55), (0.6, 0.15), (1.0, 0)])
        p["hips"]["loc"] = (0.0, 0.0, -0.02 - 0.06 * s)
        p["hips"]["rot"] = (-0.12 * s, 0.0, 0.06 * s)
        p["spine"]["rot"] = (-0.45 * s, 0.0, -0.10 * s)
        p["head"]["rot"] = (-0.55 * s, 0.0, 0.12 * s)
        for side in ("l", "r"):
            p["upper_arm_" + side]["rot"] = (0.35 * s - 0.12, 0.0, 0.25 * s * (-1 if side == "l" else 1))
            p["lower_arm_" + side]["rot"] = (-0.7 * s - 0.25, 0.0, 0.0)
            leg_ik(p, side, -back, JOINTS["foot_" + side][2] + 0.02 * s,
                   flat=0.02 * s)
        return p
    return pose_fn, frames


def death_pose(f, pf):
    t = f / 60.0
    p = pose()
    back = keys_at(t, [(0, 0), (0.25, 0.18), (0.5, 0.42), (0.75, 0.56), (1.0, 0.56)])
    p["root"]["loc"] = (0.0, back, 0.0)
    drop = keys_at(t, [(0, -0.02), (0.25, -0.08), (0.5, -0.18),
                       (0.75, -0.28), (1.0, -0.28)])
    p["hips"]["loc"] = (0.0, 0.0, drop)
    fall = keys_at(t, [(0, 0), (0.25, -0.12), (0.5, -0.6),
                       (0.8, -1.45), (1.0, -1.48)])
    p["hips"]["rot"] = (fall, 0.0, 0.0)
    p["spine"]["rot"] = (0.35 * fall, 0.0, 0.08 * fall)
    p["head"]["rot"] = (-0.25 * fall, 0.0, 0.0)
    for side in ("l", "r"):
        p["upper_arm_" + side]["rot"] = (0.55 * abs(fall) - 0.1, 0.0,
                                         0.2 * (-1 if side == "l" else 1))
        p["lower_arm_" + side]["rot"] = (-0.4, 0.0, 0.0)
        w = smooth((t - 0.3) / 0.4)
        lift = 0.20 * w + 0.20 * smooth((t - 0.02) / 0.18)
        leg_ik(p, side, -back, JOINTS["foot_" + side][2] + lift)
        up = p["upper_leg_" + side]["rot"][0]
        low = p["lower_leg_" + side]["rot"][0]
        p["upper_leg_" + side]["rot"] = (up * (1 - w) + 0.08 * w, 0.0, 0.0)
        p["lower_leg_" + side]["rot"] = (low * (1 - w) - 0.25 * w, 0.0, 0.0)
    return p


def interact_pose(f, pf):
    t = f / 42.0
    p = pose()
    r = keys_at(t, [(0, 0), (0.35, 1), (0.65, 1), (1.0, 0)])
    p["root"]["loc"] = (0.0, -0.12 * r, 0.0)
    p["hips"]["loc"] = (0.0, 0.0, -0.02 - 0.07 * r)
    p["spine"]["rot"] = (0.22 * r, 0.0, 0.0)
    p["head"]["rot"] = (0.10 * r, 0.0, 0.0)
    arm_ik(p, "r", -0.55 * r - 0.20, JOINTS["hand_r"][2] + 0.10 * r)
    arm_ik(p, "l", -0.25, JOINTS["hand_l"][2] + 0.25)
    leg_ik(p, "l", 0.12 * r, JOINTS["foot_l"][2])
    leg_ik(p, "r", 0.12 * r, JOINTS["foot_r"][2])
    return p


# ---------------------------------------------------------------------------
# Keying, ground clamp, measurement
# ---------------------------------------------------------------------------


def export_glb(name, path):
    bpy.ops.export_scene.gltf(
        filepath=path, export_format="GLB", export_yup=True,
        export_animations=True, export_animation_mode="ACTIONS",
        export_nla_strips=False, export_extras=True, export_apply=False,
        export_optimize_animation_size=False, use_selection=False)


def author(rig, name, fn, frames, loop):
    action = bpy.data.actions.new(name)
    slots = {n: action.slots.new(id_type="OBJECT", name=n) for n in PARTS}
    for n, e in rig.items():
        ad = e.animation_data_create()
        ad.action = action
        ad.action_slot = slots[n]
    n = frames + 1 if loop else frames
    for f in range(n):
        apply(rig, fn(f, f % frames if loop else f))
        for e in rig.values():
            e.keyframe_insert(data_path="location", frame=f, group=e.name)
            e.keyframe_insert(data_path="rotation_euler", frame=f, group=e.name)
    for fc in iter_fcurves(action):
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
    action.use_fake_user = True
    return action, slots


def bind(rig, action, slots):
    for n, e in rig.items():
        ad = e.animation_data_create()
        ad.action = action
        ad.action_slot = slots[n]


def min_z(meshes):
    return min((mesh.matrix_world @ v.co).z
               for mesh in meshes.values() for v in mesh.data.vertices)


def ground_clamp(rig, meshes, n, loop, uniform=False):
    scene = bpy.context.scene
    corr = []
    for f in range(n):
        scene.frame_set(f)
        bpy.context.view_layer.update()
        corr.append(max(0.0, 0.001 - min_z(meshes)))
    if uniform:
        corr = [max(corr)] * n
    elif loop:
        corr[-1] = corr[0]
    for f, c in enumerate(corr):
        if c <= 0:
            continue
        scene.frame_set(f)
        bpy.context.view_layer.update()
        rig["root"].location.z += c
        rig["root"].keyframe_insert(data_path="location", frame=f)
    for fc in iter_fcurves(rig["root"].animation_data.action):
        if fc.data_path == "location":
            for kp in fc.keyframe_points:
                kp.interpolation = "LINEAR"
    return max(corr) if corr else 0.0


def measure(rig, meshes, frames, loop):
    scene = bpy.context.scene
    n = frames + 1 if loop else frames
    samples = []
    for f in range(n):
        scene.frame_set(f)
        bpy.context.view_layer.update()
        root = rig["root"]
        inv = root.matrix_world.inverted()
        rel = {name: (inv @ e.matrix_world).translation.copy() for name, e in rig.items()}
        feet = {}
        for side in ("l", "r"):
            fo = rig["foot_" + side]
            ball = fo.matrix_world @ Vector((0, -0.10, -0.12))
            heel = fo.matrix_world @ Vector((0, 0.10, -0.12))
            feet[side] = {
                "ball": ball, "heel": heel,
                "yaw": math.degrees(atan2(ball.x - heel.x, -(ball.y - heel.y))),
            }
        samples.append({"root": root.matrix_world.translation.copy(),
                        "rel": rel, "feet": feet, "z": min_z(meshes)})

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

    def spans_of(idx):
        if not idx:
            return []
        out, st, pr = [], idx[0], idx[0]
        for i in idx[1:]:
            if i > pr + 1:
                out.append((st, pr)); st = i
            pr = i
        out.append((st, pr))
        return out

    slide = 0.0
    plants = {"l": 0, "r": 0}
    for side in ("l", "r"):
        ps = {}
        for point in ("ball", "heel"):
            idx = []
            for i, s in enumerate(samples):
                pt = s["feet"][side][point]
                v = ((samples[i + 1]["feet"][side][point] - pt) * FPS
                     if i + 1 < len(samples) else Vector((0, 0, 0)))
                if pt.z < 0.035 and v.length < 0.45:
                    idx.append(i)
            ps[point] = spans_of(idx)
        for a, b in {(a, b) for spans in ps.values() for a, b in spans if b - a >= 2}:
            drifts = []
            for point in ("ball", "heel"):
                if any(s0 <= a and b <= s1 for s0, s1 in ps[point]):
                    xs = [samples[j]["feet"][side][point] for j in range(a, b + 1)]
                    dx = max(p.x for p in xs) - min(p.x for p in xs)
                    dy = max(p.y for p in xs) - min(p.y for p in xs)
                    drifts.append(math.hypot(dx, dy) * 100)
            if not drifts:
                continue
            turn = abs((samples[b]["feet"][side]["yaw"] -
                        samples[a]["feet"][side]["yaw"] + 180) % 360 - 180)
            slide = max(slide, min(drifts) if turn > 15 else max(drifts))
            plants[side] += 1

    settled = 0.0
    if not loop:
        k = max(1, int(0.3 * FPS))
        settled = max(v["rel"][key].length for v in vel[-k:]
                      for key in v["rel"])
    zs = sorted(s["z"] for s in samples)
    p10 = zs[int(0.1 * (len(zs) - 1))] * 100
    pen = max(0.0, -min(zs)) * 100
    offset = samples[-1]["root"] - samples[0]["root"]
    return {
        "seam_position_cm": round(seam, 4),
        "seam_velocity_mps": round(seam_v, 4),
        "root_velocity_mps": round(root_v, 4),
        "foot_slide_cm": round(slide, 4),
        "ground_penetration_cm": round(pen, 4),
        "float_p10_cm": round(p10, 4),
        "end_settled_mps": round(settled, 4),
        "root_offset_m": {"x": round(offset.x, 4),
                          "gltf_z": round(-offset.y, 4),
                          "gltf_y": round(offset.z, 4),
                          "magnitude": round(offset.length, 4)},
        "mean_speed_mps": round(sum(v["root"].length for v in vel) / max(1, len(vel)), 4),
        "foot_plants": plants,
        "min_z_m": round(min(zs), 5),
    }


def gate(res, loop, name):
    fails, warns = [], []
    for key, (warn, fail) in GATES.items():
        if key == "end_settled_mps" and loop:
            continue
        if key == "float_p10_cm" and name in ("dodge_roll", "death"):
            continue
        v = res[key]
        if v > fail:
            fails.append("%s %.4f > %s" % (key, v, fail))
        elif v > warn:
            warns.append("%s %.4f > %s" % (key, v, warn))
    if loop and (res["foot_plants"]["l"] < 1 or res["foot_plants"]["r"] < 1):
        warns.append("locomotion loop missing one plant per foot")
    return fails, warns


SETTLE_FRAMES = 12


def with_settle(fn, frames):
    """Play the authored motion then hold its last pose for SETTLE_FRAMES."""
    hold_at = frames

    def wrapped(f, pf):
        return fn(min(f, hold_at), min(pf, hold_at))
    return wrapped


NOTES = {
    "idle": "Combat-ready guard, 4 s loop, in place; weight shift and breath.",
    "walk": "1.05 m/s forward, 24 f cycle; planted feet, root carries travel.",
    "run": "1.60 m/s forward, 18 f cycle; deeper crouch and arm drive.",
    "attack_1": "Windup 0-12 f, cut 12-19 f, recovery 19-36 f; 0.14 m step-in.",
    "attack_2": "Windup 0-12 f, cut 12-19 f, recovery 19-36 f; 0.16 m step-in.",
    "attack_3": "Windup 0-15 f, cut 15-25 f, recovery 25-42 f; 0.22 m step-in.",
    "heavy_attack": "Windup 0-23 f, cut 23-34 f, recovery 34-54 f; 0.25 m lunge.",
    "block": "Guard raised by frame 10, held to frame 34, released by 48; in place.",
    "dodge_roll": "0.65 m forward tumble, tucked through frame 32, low guard at end.",
    "hit_light": "Small recoil, 0.13 m back, settles by frame 30.",
    "hit_heavy": "Large recoil, 0.20 m back, settles by frame 36.",
    "death": "Two staggers, 0.56 m back, laid-out ragdoll-ready end pose.",
    "interact": "Reach forward with 0.12 m step at the apex, hold, return to origin (net root offset 0).",
}


def main():
    os.makedirs(OUT, exist_ok=True)
    rig, meshes, JOINTS_LOCAL, REST_LOCAL, dims = build_rig()
    JOINTS.clear(); JOINTS.update(JOINTS_LOCAL)
    REST.clear(); REST.update(REST_LOCAL)
    DIMS.clear(); DIMS.update(dims)
    scene = bpy.context.scene
    scene.render.fps = FPS
    print("RIG", json.dumps(dims))

    specs = []
    for band, direction in (("walk", "f"), ("run", "f")):
        name = band
        specs.append((name, LOCO[band]["cycle"], True,
                      lambda f, pf, band=band, direction=direction:
                          loco_pose(f, pf, band, direction)))
    specs.append(("idle", 120, True, idle_pose))
    for name in ("attack_1", "attack_2", "attack_3", "heavy_attack"):
        fn, frames = attack_pose(name)
        specs.append((name, frames, False, fn))
    hf, hframes = hit_pose(False)
    hf2, hframes2 = hit_pose(True)
    specs += [
        ("block", 48, False, block_pose),
        ("dodge_roll", 42, False, dodge_roll_pose),
        ("hit_light", hframes, False, hf),
        ("hit_heavy", hframes2, False, hf2),
        ("death", 60, False, death_pose),
        ("interact", 42, False, interact_pose),
    ]

    specs = [(name, frames + (0 if loop else SETTLE_FRAMES), loop,
              fn if loop else with_settle(fn, frames))
             for name, frames, loop, fn in specs]

    records, fails_all, warns_all = [], [], []
    for name, frames, loop, fn in specs:
        action, slots = author(rig, name, fn, frames, loop)
        bind(rig, action, slots)
        lift = ground_clamp(rig, meshes, frames + 1 if loop else frames, loop)
        export_glb(name, os.path.join(OUT, name + ".glb"))
        res = measure(rig, meshes, frames, loop)
        fails, warns = gate(res, loop, name)
        res["max_ground_lift_cm"] = round(lift * 100, 3)
        res["duration_s"] = round((frames + 1 if loop else frames) / FPS, 4)
        res["frames"] = frames + 1 if loop else frames
        res["fps"] = FPS
        res["loop"] = loop
        res["root_motion"] = bool(loop or name in (
            "attack_1", "attack_2", "attack_3", "heavy_attack", "dodge_roll",
            "hit_light", "hit_heavy", "death", "interact"))
        res["status"] = "FAIL" if fails else ("WARN" if warns else "PASS")
        res["fails"] = fails
        res["warns"] = warns
        res["notes"] = NOTES[name]
        res["path"] = "assets/animations/player/%s.glb" % name
        records.append({"clip": name, **res})
        fails_all += [(name, x) for x in fails]
        warns_all += [(name, x) for x in warns]
        print("[%s] %-14s %3df seam=%.3fcm slide=%.3fcm pen=%.2fcm p10=%.2fcm off=%.2fm" %
              (res["status"], name, frames, res["seam_position_cm"],
               res["foot_slide_cm"], res["ground_penetration_cm"],
               res["float_p10_cm"], res["root_offset_m"]["magnitude"]))

    with open(METRICS) as fh:
        metrics = json.load(fh)
    model = next((m for m in metrics["models"] if m.get("name") == "player"), {})
    with open(PLAYER, "rb") as fh:
        digest = hashlib.sha256(fh.read()).hexdigest()
    manifest = {
        "schema": "fight-animations.player-final/1",
        "generator": "scripts/generate_player_final.py",
        "status": "final redesign - supersedes the earlier root-level humanoid clips",
        "source": {
            "glb": "assets/models/player.glb", "sha256": digest,
            "metrics_height_m": model.get("height"),
            "metrics_min": model.get("min"), "metrics_max": model.get("max"),
        },
        "rig": {
            "kind": "object empties derived from mesh bounds",
            "derived_dimensions_m": dims,
            "joints_m": {k: [round(v, 4) for v in val] for k, val in JOINTS.items()},
            "root_motion": "all horizontal travel on root; hips vertical bob only",
        },
        "conventions": {
            "fps": FPS, "samplers": "LINEAR", "scale_keys": False,
            "direction_suffix": "f is glTF +Z / Blender -Y",
            "loop": "last key duplicates frame 0 in root-relative pose; root advances one cycle",
        },
        "gates_warn_fail": GATES,
        "clips": records,
        "summary": {
            "count": len(records),
            "pass": sum(1 for r in records if r["status"] == "PASS"),
            "warn": sum(1 for r in records if r["status"] == "WARN"),
            "fail": sum(1 for r in records if r["status"] == "FAIL"),
        },
        "fails": [{"clip": c, "check": x} for c, x in fails_all],
        "warnings": [{"clip": c, "check": x} for c, x in warns_all],
        "review_sheet": "assets/animations/player/review/contact_sheet.png",
    }
    path = os.path.join(OUT, "manifest.json")
    with open(path, "w") as fh:
        json.dump(manifest, fh, indent=2)
    print("SUMMARY", manifest["summary"])
    return 1 if fails_all else 0


if __name__ == "__main__":
    sys.exit(main())
