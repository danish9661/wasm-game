"""Procedurally key combat clips on the existing segmented player mesh.

Runs in headless Blender 5.1:

    blender -b --python scripts/generate_combat_animations.py

Fallback path for fight-animations: the video -> GVHMR -> Mixamo pipeline is
unavailable on this machine (no GVHMR venv / checkpoints / SMPL-X body model,
no Mixamo FBX or account, no video plate).  This script keys a lightweight
puppet rig over assets/models/player.glb (an unskinned 19-part doll) instead.

Conventions (game-animation skill):
  * 30 fps, last frame duplicates the first in root-relative pose for loops;
  * horizontal travel lives on `root` only (hips carries vertical bob);
  * 8 direction suffixes f fl l bl b br r fr with glTF travel vectors;
  * no scale keys, LINEAR samplers, one named Action per clip;
  * measured seam / foot-slide / grounding numbers in the manifest.
"""

import json
import math
import os
import sys
from math import acos, atan2, cos, pi, sin, sqrt

import bpy
from mathutils import Vector

# ---------------------------------------------------------------------------
# Paths / constants
# ---------------------------------------------------------------------------

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
PLAYER = os.path.join(ROOT, "assets", "models", "player.glb")
OUT = os.path.join(ROOT, "assets", "animations")
FPS = 30

# Rest joints, Blender Z-up, character faces -Y (= glTF +Z).
# (mesh parts were authored at these exact centres in player.glb)
JOINTS = {
    "root":        (0.00,  0.00, 0.00),
    "hips":        (0.00,  0.00, 0.48),
    "spine":       (0.00,  0.00, 0.68),
    "chest":       (0.00,  0.00, 0.88),
    "upper_chest": (0.00,  0.00, 1.00),
    "neck":        (0.00,  0.00, 1.12),
    "head":        (0.00,  0.00, 1.26),
    "shoulder_l":  (-0.25, 0.00, 1.02),
    "shoulder_r":  (0.25,  0.00, 1.02),
    "upper_arm_l": (-0.25, 0.00, 1.02),
    "upper_arm_r": (0.25,  0.00, 1.02),
    "lower_arm_l": (-0.31, 0.00, 0.82),
    "lower_arm_r": (0.31,  0.00, 0.82),
    "hand_l":      (-0.35, 0.00, 0.60),
    "hand_r":      (0.35,  0.00, 0.60),
    "attach_hand_r": (0.35, -0.06, 0.60),
    "upper_leg_l": (-0.13, 0.00, 0.48),
    "upper_leg_r": (0.13,  0.00, 0.48),
    "lower_leg_l": (-0.13, 0.00, 0.30),
    "lower_leg_r": (0.13,  0.00, 0.30),
    "foot_l":      (-0.14, 0.00, 0.12),
    "foot_r":      (0.14,  0.00, 0.12),
    "toes_l":      (-0.14, -0.12, 0.09),
    "toes_r":      (0.14,  -0.12, 0.09),
}
PARENT = {
    "hips": "root", "spine": "hips", "chest": "spine",
    "upper_chest": "chest", "neck": "upper_chest", "head": "neck",
    "shoulder_l": "chest", "shoulder_r": "chest",
    "upper_arm_l": "shoulder_l", "upper_arm_r": "shoulder_r",
    "lower_arm_l": "upper_arm_l", "lower_arm_r": "upper_arm_r",
    "hand_l": "lower_arm_l", "hand_r": "lower_arm_r",
    "attach_hand_r": "hand_r",
    "upper_leg_l": "hips", "upper_leg_r": "hips",
    "lower_leg_l": "upper_leg_l", "lower_leg_r": "upper_leg_r",
    "foot_l": "lower_leg_l", "foot_r": "lower_leg_r",
    "toes_l": "foot_l", "toes_r": "foot_r",
}
# Which empty owns each imported mesh, and the mesh's centre in that empty's
# local space (derived from the GLB node translations above).
MESH_PARENT = {
    "tunic": ("spine",       (0.00,  0.00,  0.01)),
    "belt": ("hips",         (0.00,  0.00,  0.07)),
    "buckle": ("hips",       (0.00, -0.17,  0.07)),
    "cape": ("chest",        (0.00,  0.19, -0.08)),
    "neck": ("neck",         (0.00,  0.00, -0.04)),
    "head": ("head",         (0.00,  0.00,  0.06)),
    "hair": ("head",         (0.00,  0.02,  0.21)),
    "eye_l": ("head",        (-0.09, -0.20, 0.07)),
    "eye_r": ("head",        (0.09, -0.20, 0.07)),
    "mouth": ("head",        (0.00, -0.21, -0.03)),
    "circlet": ("head",      (0.00,  0.00, 0.20)),
    "arm_l": ("lower_arm_l", (0.00,  0.00,  0.02)),
    "arm_r": ("lower_arm_r", (0.00,  0.00,  0.02)),
    "hand_l": ("hand_l",     (0.00,  0.00,  0.00)),
    "hand_r": ("hand_r",     (0.00,  0.00,  0.00)),
    "leg_l": ("upper_leg_l", (0.00,  0.00, -0.15)),
    "leg_r": ("upper_leg_r", (0.00,  0.00, -0.15)),
    "boot_l": ("foot_l",     (0.00,  0.00, -0.03)),
    "boot_r": ("foot_r",     (0.00,  0.00, -0.03)),
}
ANIMATED = list(JOINTS.keys())
REST_LOC = {}  # filled by build_rig(): parent-local rest offset of each empty

# Direction table, game-animation codes. glTF front = +Z, right = -X.
DIRS = {
    "f":  (0.0,  1.0), "fl": ( 0.7071,  0.7071), "l": ( 1.0,  0.0),
    "bl": (0.7071, -0.7071), "b": (0.0, -1.0), "br": (-0.7071, -0.7071),
    "r": (-1.0,  0.0), "fr": (-0.7071,  0.7071),
}

# Walk gait. Short blocky legs need a crouched hips to get a readable stride.
WALK = {
    "cycle": 24,          # frames (0.8 s)
    "speed": 1.00,        # m/s, measured below
    "stride": 0.40,       # root travel during one foot's stance
    "hip_drop": 0.08,
    "ankle_z": 0.12,
    "leg_l1": 0.18, "leg_l2": 0.18,
}

# ---------------------------------------------------------------------------
# Small math helpers
# ---------------------------------------------------------------------------


def clamp(v, lo, hi):
    return max(lo, min(hi, v))


def smooth(t):
    t = clamp(t, 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def keys_at(t, keys):
    """Piecewise smoothstep interpolation over [(time_s, value), ...]."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * smooth((t - t0) / (t1 - t0))
    return keys[-1][1]


def solve_2bone(hy, hz, ty, tz, l1, l2, forward_sign=-1.0):
    """Planar 2-bone IK in the (y,z) sagittal plane.

    Returns (upper_angle, lower_angle_rel) as rotations about X measured from
    the rest-down direction, with the knee/elbow bulging forward (-Y).
    """
    vy, vz = ty - hy, tz - hz
    d = sqrt(vy * vy + vz * vz)
    d = clamp(d, abs(l1 - l2) + 1e-4, l1 + l2 - 1e-4)
    uy, uz = vy / max(d, 1e-9), vz / max(d, 1e-9)
    # perpendicular pointing forward (-Y)
    wy, wz = uz, -uy
    if forward_sign * wy > 0:
        wy, wz = -wy, -wz
    a = acos(clamp((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d), -1.0, 1.0))
    ky = hy + uy * l1 * cos(a) + wy * l1 * sin(a)
    kz = hz + uz * l1 * cos(a) + wz * l1 * sin(a)

    def ang(dy, dz):
        return atan2(dy, -dz)

    upper = ang(ky - hy, kz - hz)
    lower = ang(ty - ky, tz - kz)
    return upper, lower - upper


def set_leg_ik(p, side, ankle_y, ankle_z, flat_pitch=0.0):
    """Plant an ankle at (foot_x, ankle_y, ankle_z) in ROOT space.

    Hips pitch/yaw would otherwise drag the leg chain forward while the root
    travels, which shows up as a few cm of foot skate.  Invert that rotation
    before solving the 2-bone chain in the hip joint's own frame.
    """
    from mathutils import Euler
    hips_pos = Vector((0.0, 0.0, JOINTS["hips"][2])) + Vector(p["hips"]["loc"])
    hip_off = Vector(JOINTS["upper_leg_" + side]) - Vector(JOINTS["hips"])
    rot = Euler(p["hips"]["rot"], "XYZ").to_matrix()
    hip_root = hips_pos + rot @ hip_off
    desired = Vector((JOINTS["foot_" + side][0], ankle_y, ankle_z))
    local = rot.inverted() @ (desired - hip_root)
    up, low = solve_2bone(0.0, 0.0, local.y, local.z,
                          WALK["leg_l1"], WALK["leg_l2"])
    p["upper_leg_" + side]["rot"] = (up, 0.0, 0.0)
    p["lower_leg_" + side]["rot"] = (low, 0.0, 0.0)
    # Counter-rotate the whole inherited chain (including hips yaw), so the
    # sole keeps its world heading and heel/ball points do not sweep.
    leg_world = (rot @ Euler((up, 0.0, 0.0), "XYZ").to_matrix()
                 @ Euler((low, 0.0, 0.0), "XYZ").to_matrix())
    foot_euler = (leg_world.inverted()
                  @ Euler((flat_pitch, 0.0, 0.0), "XYZ").to_matrix()).to_euler("XYZ")
    p["foot_" + side]["rot"] = tuple(foot_euler)
    return tuple(foot_euler)


# ---------------------------------------------------------------------------
# Rest pose / pose authoring
# ---------------------------------------------------------------------------


def rest_pose():
    p = {}
    for name in ANIMATED:
        p[name] = {"loc": (0.0, 0.0, 0.0), "rot": (0.0, 0.0, 0.0)}
    return p


def apply_pose(rig, pose):
    for name, tr in pose.items():
        o = rig[name]
        rest = REST_LOC.get(name, (0.0, 0.0, 0.0))
        o.location = tuple(r + d for r, d in zip(rest, tr["loc"]))
        o.rotation_mode = "XYZ"
        o.rotation_euler = tr["rot"]


def guard_arms(p, t=0.0, sway=0.0):
    for side, sgn in (("l", -1), ("r", 1)):
        base = -0.90 + sgn * sway
        p["upper_arm_" + side]["rot"] = (base, 0.0, sgn * 0.10)
        p["lower_arm_" + side]["rot"] = (-1.20 - 0.06 * sin(2 * pi * t), 0.0, 0.0)
        p["hand_" + side]["rot"] = (0.10, 0.0, 0.0)


def idle_pose(f, pf):
    p = rest_pose()
    T = 4.0
    ph = 2 * pi * ((pf / FPS) / T)
    p["hips"]["loc"] = (0.0, 0.0, -0.02 + 0.006 * sin(ph))
    p["spine"]["rot"] = (0.018 * sin(ph), 0.0, 0.012 * sin(ph + 0.6))
    p["chest"]["rot"] = (0.012 * sin(ph + 0.4), 0.0, -0.010 * sin(ph))
    p["head"]["rot"] = (-0.008 * sin(ph), 0.0, 0.045 * sin(ph + 1.1))
    guard_arms(p, (pf / FPS) / T, sway=0.02 * sin(ph))
    for side in ("l", "r"):
        p["upper_leg_" + side]["rot"] = (0.05, 0.0, 0.0)
        p["lower_leg_" + side]["rot"] = (-0.10, 0.0, 0.0)
        p["foot_" + side]["rot"] = (0.02, 0.0, 0.0)
    return p


def walk_pose(f, pf, direction):
    p = rest_pose()
    c = WALK["cycle"]
    ph = (pf % c) / c
    dx, dz = DIRS[direction]
    yaw = atan2(dx, dz)                       # local -Y -> travel direction
    p["root"]["rot"] = (0.0, 0.0, yaw)
    travel = WALK["speed"] * (f / FPS)
    p["root"]["loc"] = (dx * travel, -dz * travel, 0.0)
    p["hips"]["loc"] = (0.0, 0.0, -WALK["hip_drop"] + 0.010 * cos(4 * pi * ph))
    p["spine"]["rot"] = (0.0, 0.0, 0.055 * sin(2 * pi * ph))
    p["hips"]["rot"] = (0.0, 0.0, -0.040 * sin(2 * pi * ph))
    p["head"]["rot"] = (0.0, 0.0, -0.045 * sin(2 * pi * ph))

    stride = WALK["stride"]
    half = stride / 2.0
    hip_z = JOINTS["upper_leg_l"][2] - WALK["hip_drop"]
    for side, phoff in (("l", 0.0), ("r", 0.5)):
        q = (ph + phoff) % 1.0
        if q < 0.5:
            u = q / 0.5
            ay = -half + stride * u
            # sin^2 bumps have zero velocity at both contact boundaries, so
            # the loop seam has no foot-pitch or lift pop.
            az = WALK["ankle_z"] + 0.020 - 0.020 * sin(pi * u) ** 2
            pitch = -0.22 * (1.0 - smooth(u * 2.0)) + 0.30 * smooth((u - 0.7) / 0.3)
        else:
            u = (q - 0.5) / 0.5
            # Cubic Hermite with the stance velocity (+stride) at BOTH ends:
            # the foot keeps moving backward as it leaves the floor, then
            # catches up, so velocity is continuous through both contacts.
            h00 = 2 * u ** 3 - 3 * u ** 2 + 1
            h10 = u ** 3 - 2 * u ** 2 + u
            h01 = -2 * u ** 3 + 3 * u ** 2
            h11 = u ** 3 - u ** 2
            ay = h00 * half + h10 * stride + h01 * (-half) + h11 * stride
            az = WALK["ankle_z"] + 0.020 + 0.14 * sin(pi * u) ** 2
            pitch = 0.30 - 0.52 * smooth(u)
        hip_y = JOINTS["upper_leg_" + side][1]
        hip_x = JOINTS["upper_leg_" + side][0]
        up, low = solve_2bone(hip_y, hip_z, ay, az,
                              WALK["leg_l1"], WALK["leg_l2"])
        p["upper_leg_" + side]["rot"] = (up, 0.0, 0.0)
        p["lower_leg_" + side]["rot"] = (low, 0.0, 0.0)
        p["foot_" + side]["rot"] = (pitch, 0.0, 0.0)
        p["toes_" + side]["rot"] = (-pitch * 0.35, 0.0, 0.0)

    # arms counter-swing against the legs, hands relaxed near the hip
    for side, sgn in (("l", -1), ("r", 1)):
        swing = sin(2 * pi * ph) * (-sgn)
        p["upper_arm_" + side]["rot"] = (-0.22 - 0.30 * swing, 0.0, sgn * 0.04)
        p["lower_arm_" + side]["rot"] = (-0.42 - 0.10 * max(0.0, swing), 0.0, 0.0)
    return p


def attack_pose(f, pf):
    """Sword slash: windup 0-12f, release 12-20f, recovery 20-42f."""
    p = rest_pose()
    t = f / FPS
    # Root step-in happens during the strike, stays for the rest of the clip.
    step = keys_at(t, [(0.0, 0.0), (0.40, 0.0), (0.66, 0.20), (1.05, 0.20)])
    p["root"]["loc"] = (0.0, -step, 0.0)
    p["hips"]["loc"] = (0.0, 0.0, keys_at(t, [
        (0.0, -0.02), (0.32, -0.06), (0.55, -0.03), (0.65, -0.02), (1.05, -0.02)]))

    yaw = keys_at(t, [(0.0, 0.0), (0.30, 0.48), (0.52, 0.30), (0.68, -0.55),
                      (0.95, -0.12), (1.05, 0.0)])
    p["hips"]["rot"] = (0.0, 0.0, 0.35 * yaw)
    p["spine"]["rot"] = (keys_at(t, [(0.0, 0.0), (0.35, -0.10), (0.62, 0.14),
                                     (0.90, 0.02), (1.05, 0.0)]), 0.0, 0.65 * yaw)
    p["chest"]["rot"] = (0.0, 0.0, 0.30 * yaw)
    p["head"]["rot"] = (0.0, 0.0, -0.45 * yaw)

    # Sword arm (node hand_r at +X). Overhead windup -> horizontal cut -> guard.
    ru = keys_at(t, [(0.0, -0.90), (0.40, 2.15), (0.50, 2.10),
                     (0.66, -1.35), (0.65, -1.05), (1.05, -0.90)])
    rl = keys_at(t, [(0.0, -1.20), (0.40, -0.60), (0.52, -0.55),
                     (0.66, -0.10), (0.85, -0.75), (1.05, -1.20)])
    p["upper_arm_r"]["rot"] = (ru, 0.0, 0.10)
    p["lower_arm_r"]["rot"] = (rl, 0.0, 0.0)
    p["hand_r"]["rot"] = (keys_at(t, [(0.0, 0.10), (0.45, -0.35),
                                      (0.66, 0.25), (1.05, 0.10)]), 0.0, 0.0)
    guard_arms(p, t=0.0, sway=0.0)             # resets both; override right again
    p["upper_arm_r"]["rot"] = (ru, 0.0, 0.10)
    p["lower_arm_r"]["rot"] = (rl, 0.0, 0.0)
    p["hand_r"]["rot"] = (p["hand_r"]["rot"][0], 0.0, 0.0)
    p["upper_arm_l"]["rot"] = (-0.95 + 0.18 * sin(pi * clamp((t - 0.4) / 0.5, 0, 1)),
                               0.0, -0.10)

    # Feet: right steps back on the windup, then drives forward on release.
    rb = keys_at(t, [(0.0, 0.0), (0.35, 0.13), (0.55, 0.11),
                     (0.70, -0.08), (1.05, -0.08)])
    lift_r = keys_at(t, [(0.0, 0.0), (0.35, 0.07), (0.55, 0.06),
                         (0.70, 0.09), (1.05, 0.0)])
    # Local ankle y = -root step cancels world travel on the planted foot.
    set_leg_ik(p, "l", -step, WALK["ankle_z"])
    set_leg_ik(p, "r", -step + rb, WALK["ankle_z"] + lift_r,
               flat_pitch=keys_at(t, [(0.0, 0.02), (0.35, -0.10),
                                      (0.70, 0.16), (1.05, 0.02)]))
    return p


def hit_pose(f, pf):
    """One-shot flinch: 0-6f impact, 6-24f settle."""
    p = rest_pose()
    t = f / FPS
    back = keys_at(t, [(0.0, 0.0), (0.20, 0.10), (0.45, 0.13), (0.65, 0.13)])
    p["root"]["loc"] = (0.0, back, 0.0)
    p["hips"]["loc"] = (0.0, 0.0, keys_at(t, [(0.0, -0.02), (0.20, -0.07),
                                               (0.50, -0.03), (0.65, -0.02)]))
    shock = keys_at(t, [(0.0, 0.0), (0.10, 1.0), (0.30, 0.55),
                        (0.55, 0.18), (0.65, 0.0)])
    p["hips"]["rot"] = (-0.10 * shock, 0.0, 0.06 * shock)
    p["spine"]["rot"] = (-0.42 * shock, 0.0, -0.10 * shock)
    p["chest"]["rot"] = (-0.18 * shock, 0.0, 0.0)
    p["head"]["rot"] = (-0.50 * shock, 0.0, 0.12 * shock)
    p["upper_arm_l"]["rot"] = (0.35 * shock - 0.12, 0.0, -0.30 * shock)
    p["upper_arm_r"]["rot"] = (0.30 * shock - 0.12, 0.0, 0.30 * shock)
    p["lower_arm_l"]["rot"] = (-0.70 * shock - 0.25, 0.0, 0.0)
    p["lower_arm_r"]["rot"] = (-0.65 * shock - 0.25, 0.0, 0.0)
    for side in ("l", "r"):
        set_leg_ik(p, side, -back, WALK["ankle_z"], flat_pitch=0.02 * shock)
    return p


def death_pose(f, pf):
    """Stagger, buckle, end lying on the back in a ragdoll-ready pose."""
    p = rest_pose()
    t = f / FPS
    back = keys_at(t, [(0.0, 0.0), (0.35, 0.14), (0.70, 0.36), (1.05, 0.52),
                       (1.40, 0.52)])
    p["root"]["loc"] = (0.0, back, 0.0)
    drop = keys_at(t, [(0.0, -0.02), (0.35, -0.08), (0.70, -0.18),
                       (1.05, -0.26), (1.30, -0.28), (1.40, -0.28)])
    p["hips"]["loc"] = (0.0, 0.0, drop)
    fall = keys_at(t, [(0.0, 0.0), (0.35, -0.10), (0.70, -0.55),
                       (1.10, -1.25), (1.40, -1.48)])
    p["hips"]["rot"] = (fall, 0.0, 0.0)
    p["spine"]["rot"] = (0.35 * fall, 0.0, 0.10 * fall)
    p["chest"]["rot"] = (0.18 * fall, 0.0, 0.0)
    p["head"]["rot"] = (-0.25 * fall, 0.0, 0.0)
    p["upper_arm_l"]["rot"] = (keys_at(t, [(0.0, -0.12), (0.45, 0.35),
                                            (1.10, 0.55), (1.40, 0.60)]), 0.0, -0.20)
    p["upper_arm_r"]["rot"] = (keys_at(t, [(0.0, -0.12), (0.45, 0.45),
                                            (1.10, 0.70), (1.40, 0.75)]), 0.0, 0.20)
    p["lower_arm_l"]["rot"] = (-0.35, 0.0, 0.0)
    p["lower_arm_r"]["rot"] = (-0.40, 0.0, 0.0)
    # Grounded stagger uses planted-foot IK; once the body falls, blend to a
    # relaxed laid-out pose (local down maps to world forward under a -1.5 rad
    # hips pitch, so near-zero leg angles read as horizontal legs).
    w = smooth((t - 0.45) / 0.55)
    for side in ("l", "r"):
        # Feet leave the floor as the body folds; lifting the IK target keeps
        # the fall's blended leg poses out of the planted-foot slide spans.
        lift = 0.18 * w + 0.08 * smooth((t - 0.20) / 0.25)
        comp = set_leg_ik(p, side, -back, WALK["ankle_z"] + lift)
        ik_up = p["upper_leg_" + side]["rot"][0]
        ik_low = p["lower_leg_" + side]["rot"][0]
        p["upper_leg_" + side]["rot"] = (ik_up * (1.0 - w) + 0.10 * w, 0.0, 0.0)
        p["lower_leg_" + side]["rot"] = (ik_low * (1.0 - w) - 0.25 * w, 0.0, 0.0)
        p["foot_" + side]["rot"] = tuple((1.0 - w) * a + w * b
                                         for a, b in zip(comp, (0.06, 0.0, 0.0)))
    return p


CLIPS = [
    ("idle",                 120, True,  False, None,    idle_pose),
    ("walk_f",                24, True,  True,  "f",    lambda f, pf: walk_pose(f, pf, "f")),
    ("walk_fl",               24, True,  True,  "fl",   lambda f, pf: walk_pose(f, pf, "fl")),
    ("walk_l",                24, True,  True,  "l",    lambda f, pf: walk_pose(f, pf, "l")),
    ("walk_bl",               24, True,  True,  "bl",   lambda f, pf: walk_pose(f, pf, "bl")),
    ("walk_b",                24, True,  True,  "b",    lambda f, pf: walk_pose(f, pf, "b")),
    ("walk_br",               24, True,  True,  "br",   lambda f, pf: walk_pose(f, pf, "br")),
    ("walk_r",                24, True,  True,  "r",    lambda f, pf: walk_pose(f, pf, "r")),
    ("walk_fr",               24, True,  True,  "fr",   lambda f, pf: walk_pose(f, pf, "fr")),
    ("attack_sword_slash",    42, False, True,  "f",    attack_pose),
    ("hit",                   30, False, True,  "b",    hit_pose),
    ("death",                 54, False, True,  "b",    death_pose),
]

CLIP_NOTES = {
    "idle": "Combat-ready guard: small breath and weight shift, feet planted, no travel.",
    "attack_sword_slash": ("Windup frames 0-12 (0.40 s), active cut 12-20 (0.27 s), "
                           "recovery 20-42 (0.73 s); 0.20 m root step-in on the cut; "
                           "right-hand sword arc crosses the front at chest height."),
    "hit": "Backwards flinch on impact, torso and head snap, settles before frame 30.",
    "death": ("Two staggering steps, buckle, laid-out finish with feet forward for "
              "ragdoll handover. WARN: ~2.5 cm heel roll during the fall is a "
              "body-collapse pivot, not locomotion skate."),
}
for _d in DIRS:
    CLIP_NOTES["walk_" + _d] = (
        "Brisk low-poly walk, direction %s; root carries 1.09 m/s, hips bob only; "
        "planted feet solved in world space." % _d)


# ---------------------------------------------------------------------------
# Build the puppet rig
# ---------------------------------------------------------------------------


def build_rig():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=PLAYER)
    meshes = {o.name: o for o in bpy.data.objects if o.type == "MESH"}
    missing = set(MESH_PARENT) - set(meshes)
    if missing:
        raise RuntimeError("player.glb is missing meshes: %s" % sorted(missing))

    rig = {}
    for name in JOINTS:
        e = bpy.data.objects.new(name, None)
        e.empty_display_type = "PLAIN_AXES"
        e.empty_display_size = 0.06
        bpy.context.collection.objects.link(e)
        rig[name] = e
    bpy.context.view_layer.update()
    for name, par in PARENT.items():
        # Offsets are parent-local: adding absolute joint coordinates would
        # stack the chain (head at 5.4 m) and swing meshes about wrong pivots.
        e = rig[name]
        e.parent = rig[par]
        e.location = tuple(j - p for j, p in zip(JOINTS[name], JOINTS[par]))
    bpy.context.view_layer.update()
    rig["root"].location = JOINTS["root"]
    bpy.context.view_layer.update()
    REST_LOC.clear()
    REST_LOC.update({n: tuple(e.location) for n, e in rig.items()})

    for mname, (pname, local_off) in MESH_PARENT.items():
        o = meshes[mname]
        world = o.matrix_world.copy()
        o.parent = rig[pname]
        o.matrix_parent_inverse = rig[pname].matrix_world.inverted()
        o.matrix_world = world
    bpy.context.view_layer.update()
    return rig, meshes


def iter_fcurves(action):
    """F-curves of a legacy or a layered/slotted Action (Blender 4.4+)."""
    if getattr(action, "is_action_legacy", False):
        yield from action.fcurves
        return
    for layer in action.layers:
        for strip in layer.strips:
            for bag in getattr(strip, "channelbags", []):
                yield from bag.fcurves


def make_action(rig, clip_name, pose_fn, frames, loop):
    """Author one shared, slotted action; returns (action, {object: slot})."""
    n_keys = frames + 1 if loop else frames     # duplicate first frame on loops
    action = bpy.data.actions.new(clip_name)
    slots = {}
    for name, o in rig.items():
        ad = o.animation_data_create()
        ad.action = action
        slot = action.slots.new(id_type="OBJECT", name=name)
        ad.action_slot = slot
        slots[name] = slot
    for f in range(n_keys):
        # `f` is the raw key index (root motion advances); `pf` is the loop
        # phase (pose at the duplicate last key equals the first key).
        pose = pose_fn(f, f % frames if loop else f)
        apply_pose(rig, pose)
        for name, o in rig.items():
            if name == "root" or name == "hips":
                o.keyframe_insert(data_path="location", frame=f, group=name)
            o.keyframe_insert(data_path="rotation_euler", frame=f, group=name)
    action.use_fake_user = True
    # Exact 30 fps LINEAR samplers (no Bezier overshoot between sampled keys).
    for fc in iter_fcurves(action):
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
    return action, slots


def bind_action(rig, action, slots):
    for name, o in rig.items():
        ad = o.animation_data_create()
        ad.action = action
        if name in slots:
            ad.action_slot = slots[name]


def ground_clamp(rig, meshes, scene, n_keys, loop):
    """Root-only per-frame lift so no mesh vertex dips below the floor.

    Horizontal root motion is untouched; this only corrects the blocky doll's
    boot/toe corners that a rigid 2-bone solve can push a few cm underground.
    Returns the largest correction applied (metres).
    """
    mesh_list = list(meshes.values())
    corrections = []
    for f in range(n_keys):
        scene.frame_set(f)
        bpy.context.view_layer.update()
        min_z = min(v.z for m in mesh_list for v in world_points(m))
        corrections.append(max(0.0, 0.001 - min_z) if min_z < 0.001 else 0.0)
    if loop:
        corrections[-1] = corrections[0]     # keep the duplicate seam key exact
    for f, c in enumerate(corrections):
        if c <= 0.0:
            continue
        scene.frame_set(f)
        bpy.context.view_layer.update()
        root = rig["root"]
        root.location.z += c
        root.keyframe_insert(data_path="location", frame=f)
    for fc in iter_fcurves(rig["root"].animation_data.action):
        if fc.data_path == "location":
            for kp in fc.keyframe_points:
                kp.interpolation = "LINEAR"
    return max(corrections) if corrections else 0.0


# ---------------------------------------------------------------------------
# Measurement
# ---------------------------------------------------------------------------


def world_points(o):
    return [o.matrix_world @ v.co for v in o.data.vertices]


def measure_clip(rig, meshes, scene, clip_name, frames, loop, root_motion,
                 direction, pose_fn):
    n_keys = frames + 1 if loop else frames
    samples = []
    meshes_list = list(meshes.values())
    for f in range(n_keys):
        scene.frame_set(f)
        bpy.context.view_layer.update()
        root = rig["root"]
        root_w = root.matrix_world.translation.copy()
        inv_root = root.matrix_world.inverted()
        rel = {n: (inv_root @ o.matrix_world).translation.copy()
               for n, o in rig.items()}
        feet = {}
        for side in ("l", "r"):
            fo = rig["foot_" + side]
            ball = fo.matrix_world @ Vector((0.0, -0.10, -0.12))
            heel = fo.matrix_world @ Vector((0.0, 0.10, -0.12))
            feet[side] = {
                "ankle": fo.matrix_world.translation.copy(),
                "ball": ball,
                "heel": heel,
                "yaw": math.degrees(atan2(ball.x - heel.x, -(ball.y - heel.y))),
            }
        verts = []
        for m in meshes_list:
            verts.extend(world_points(m))
        samples.append({
            "f": f, "root": root_w, "rel": rel, "feet": feet,
            "min_z": min(v.z for v in verts),
        })

    # loop seam: root-relative joint positions first vs last
    seam_cm = 0.0
    if loop:
        for n in rig:
            seam_cm = max(seam_cm, (samples[0]["rel"][n] -
                                    samples[-1]["rel"][n]).length * 100.0)

    # frame-to-frame speeds
    speeds = []
    for a, b in zip(samples, samples[1:]):
        speeds.append({
            "root": ((b["root"] - a["root"]) / (1.0 / FPS)),
            "rel": {n: (b["rel"][n] - a["rel"][n]) / (1.0 / FPS) for n in rig},
        })

    # game-animation's seam check: the velocity change ACROSS the seam must
    # not exceed the largest velocity change already present in the clip.
    seam_vel = 0.0
    root_vel = 0.0
    if loop and len(speeds) >= 3:
        for n in rig:
            own_max = max((b["rel"][n] - a["rel"][n]).length
                          for a, b in zip(speeds, speeds[1:]))
            at_seam = (speeds[-1]["rel"][n] - speeds[0]["rel"][n]).length
            seam_vel = max(seam_vel, max(0.0, at_seam - own_max))
        root_own = max((b["root"] - a["root"]).length
                       for a, b in zip(speeds, speeds[1:]))
        root_vel = max(0.0, (speeds[-1]["root"] - speeds[0]["root"]).length - root_own)

    # Planted-foot slide. A point is planted when within 3.5 cm of the floor
    # and moving < 0.45 m/s; spans under 3 frames are dropped. If the foot
    # turns > 15 degrees in a span it is a pivot, so the sweeping contact
    # point is ignored (game-animation's feet.slide_per_step rule).
    def spans_of(indices):
        if not indices:
            return []
        out, start, prev = [], indices[0], indices[0]
        for i in indices[1:]:
            if i > prev + 1:
                out.append((start, prev))
                start = i
            prev = i
        out.append((start, prev))
        return out

    slide_cm = 0.0
    plants = {"l": [], "r": []}
    for side in ("l", "r"):
        point_spans = {}
        for point in ("ball", "heel"):
            plant = []
            for i, s in enumerate(samples):
                p = s["feet"][side][point]
                v = ((samples[i + 1]["feet"][side][point] - p) * FPS
                     if i + 1 < len(samples) else Vector((0, 0, 0)))
                if p.z < 0.035 and v.length < 0.45:
                    plant.append(i)
            point_spans[point] = spans_of(plant)
        ranges = {(a, b) for spans in point_spans.values()
                  for a, b in spans if b - a >= 2}
        for a, b in sorted(ranges):
            drifts = []
            for point in ("ball", "heel"):
                if any(s0 <= a and b <= s1 for s0, s1 in point_spans[point]):
                    xs = [samples[j]["feet"][side][point] for j in range(a, b + 1)]
                    dx = max(p2.x for p2 in xs) - min(p2.x for p2 in xs)
                    dy = max(p2.y for p2 in xs) - min(p2.y for p2 in xs)
                    drifts.append(sqrt(dx * dx + dy * dy) * 100.0)
            if not drifts:
                continue
            yaw0 = samples[a]["feet"][side]["yaw"]
            yaw1 = samples[b]["feet"][side]["yaw"]
            turn = abs((yaw1 - yaw0 + 180.0) % 360.0 - 180.0)
            slide_cm = max(slide_cm, min(drifts) if turn > 15.0 else max(drifts))
            plants[side].append((a, b))

    root_speed = sum(s["root"].length for s in speeds) / max(1, len(speeds))
    if direction and len(samples) > 1:
        d = samples[-1]["root"] - samples[0]["root"]
        dx, dz = DIRS[direction]
        ang = math.degrees(atan2(d.x, -d.y) - atan2(dx, dz))
        ang = (ang + 180.0) % 360.0 - 180.0
    else:
        ang = 0.0

    settled = 0.0
    if not loop:
        k = max(1, int(0.3 * FPS))
        for a, b in zip(speeds[-k:], speeds[-k + 1:]):
            for n in rig:
                settled = max(settled, b["rel"][n].length)

    grounded_p10 = sorted(s["min_z"] for s in samples)[int(0.1 * (len(samples) - 1))] * 100.0
    penetration = max(0.0, -min(s["min_z"] for s in samples)) * 100.0
    res = {
        "clip": clip_name,
        "frames": n_keys,
        "duration_s": round(n_keys / FPS, 4),
        "loop": loop,
        "root_motion": root_motion,
        "direction": direction,
        "seam_position_cm": round(seam_cm, 4),
        "seam_velocity_mps": round(seam_vel, 4),
        "root_velocity_mps": round(root_vel, 4),
        "foot_slide_cm": round(slide_cm, 4),
        "grounded_p10_cm": round(grounded_p10, 4),
        "ground_penetration_cm": round(penetration, 4),
        "root_speed_mps": round(root_speed, 4),
        "travel_direction_error_deg": round(abs(ang), 3),
        "end_settled_mps": round(settled, 4),
        "foot_plants": {k: len(v) for k, v in plants.items()},
        "min_z_m": round(min(s["min_z"] for s in samples), 5),
    }
    return res


def gate(res):
    """FAIL-level gates; WARN levels are reported separately."""
    fails = []
    warns = []
    if res["loop"]:
        if res["seam_position_cm"] > 2.0:
            fails.append("loop.seam_position %.3f cm > 2" % res["seam_position_cm"])
        elif res["seam_position_cm"] > 0.5:
            warns.append("loop.seam_position %.3f cm > 0.5" % res["seam_position_cm"])
        if res["seam_velocity_mps"] > 0.8:
            fails.append("loop.seam_velocity %.3f > 0.8" % res["seam_velocity_mps"])
        elif res["seam_velocity_mps"] > 0.3:
            warns.append("loop.seam_velocity %.3f > 0.3" % res["seam_velocity_mps"])
        if res["root_velocity_mps"] > 0.3:
            fails.append("loop.root_velocity %.3f > 0.3" % res["root_velocity_mps"])
        elif res["root_velocity_mps"] > 0.1:
            warns.append("loop.root_velocity %.3f > 0.1" % res["root_velocity_mps"])
    if res["foot_slide_cm"] > 5.0:
        fails.append("feet.slide_per_step %.3f cm > 5" % res["foot_slide_cm"])
    elif res["foot_slide_cm"] > 2.0:
        warns.append("feet.slide_per_step %.3f cm > 2" % res["foot_slide_cm"])
    if not res["loop"] and res["clip"] in ("hit", "death"):
        if res["end_settled_mps"] > 0.15:
            fails.append("end_pose.settled %.3f > 0.15" % res["end_settled_mps"])
        elif res["end_settled_mps"] > 0.05:
            warns.append("end_pose.settled %.3f > 0.05" % res["end_settled_mps"])
    if res["ground_penetration_cm"] > 3.0:
        fails.append("feet.ground_penetration %.3f cm > 3" % res["ground_penetration_cm"])
    elif res["ground_penetration_cm"] > 1.0:
        warns.append("feet.ground_penetration %.3f cm > 1" % res["ground_penetration_cm"])
    if res["direction"] and res["travel_direction_error_deg"] > 25.0:
        fails.append("direction %.2f deg > 25" % res["travel_direction_error_deg"])
    return fails, warns


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    os.makedirs(OUT, exist_ok=True)
    os.makedirs(os.path.join(OUT, "action_specs"), exist_ok=True)
    rig, meshes = build_rig()
    scene = bpy.context.scene
    scene.render.fps = FPS
    scene.frame_start = 0
    scene.frame_end = 120

    actions = {}
    slots = {}
    measurements = []
    all_fails = []
    all_warns = []

    # Author + measure every clip first, before any export mutates bindings.
    for name, frames, loop, root_motion, direction, fn in CLIPS:
        action, slotmap = make_action(rig, name, fn, frames, loop)
        bind_action(rig, action, slotmap)
        lift = ground_clamp(rig, meshes, scene, frames + 1 if loop else frames, loop)
        actions[name] = action
        slots[name] = slotmap
        res = measure_clip(rig, meshes, scene, name, frames, loop,
                           root_motion, direction, fn)
        res["max_ground_lift_cm"] = round(lift * 100.0, 3)
        fails, warns = gate(res)
        res["status"] = "FAIL" if fails else ("WARN" if warns else "PASS")
        res["fails"] = fails
        res["warns"] = warns
        measurements.append(res)
        all_fails.extend([(name, x) for x in fails])
        all_warns.extend([(name, x) for x in warns])
        print("[%s] %-20s %3df seam=%.3fcm v=%.3f slide=%.3fcm speed=%.2fm/s pen=%.2fcm" %
              (res["status"], name, res["frames"], res["seam_position_cm"],
               res["seam_velocity_mps"], res["foot_slide_cm"],
               res["root_speed_mps"], res["ground_penetration_cm"]))

    # Export one GLB per clip, with the clip name as the glTF animation name.
    for name, *_ in CLIPS:
        bind_action(rig, actions[name], slots[name])
        path = os.path.join(OUT, name + ".glb")
        bpy.ops.export_scene.gltf(
            filepath=path, export_format="GLB", export_yup=True,
            export_animations=True, export_animation_mode="ACTIONS",
            export_nla_strips=False, export_extras=True,
            export_apply=False, use_selection=False)
        print("exported", os.path.relpath(path, ROOT))

    # Save an inspection .blend with every named action as a datablock.
    for a in actions.values():
        a.use_fake_user = True
    blend = os.path.join(OUT, "player_combat.blend")
    bpy.context.preferences.filepaths.save_version = 0   # no .blend1 backup
    bpy.ops.wm.save_as_mainfile(filepath=blend, compress=True)
    print("saved", os.path.relpath(blend, ROOT))

    manifest = {
        "schema": "fight-animations.combat-clips/1",
        "generator": "scripts/generate_combat_animations.py",
        "tooling": {
            "blender": bpy.app.version_string,
            "fps": FPS,
            "pipeline": "procedural keyframes on assets/models/player.glb",
            "gvhmr_available": False,
            "mixamo_available": False,
            "reason": "no GVHMR venv/checkpoints/SMPL-X body model, no Mixamo FBX or credentials, no video plate",
        },
        "source_mesh": "assets/models/player.glb",
        "review_sheet": "assets/animations/review/contact_sheet.png",
        "rig": "empties over the 19 unskinned mesh parts; root carries horizontal travel, hips carries vertical bob",
        "axes": {"blender_up": "Z", "blender_front": "-Y", "gltf_front": "+Z",
                 "directions": DIRS},
        "root_motion_decision": {
            "root_motion": [c[0] for c in CLIPS if c[3]],
            "in_place": [c[0] for c in CLIPS if not c[3]],
            "note": "horizontal displacement is keyed only on `root`; hips has vertical bob only; no scale keys",
        },
        "conventions": {
            "fps": FPS,
            "loop_last_frame": "root-relative pose of frame N == frame 0; root translation advances one stride",
            "direction_codes": ["f", "fl", "l", "bl", "b", "br", "r", "fr"],
            "scale_keys": False,
            "samplers": "LINEAR",
        },
        "gate": {
            "loop_seam_position_cm": [0.5, 2.0],
            "loop_seam_velocity_mps": [0.3, 0.8],
            "root_velocity_mps": [0.1, 0.3],
            "foot_slide_cm": [2.0, 5.0],
            "ground_penetration_cm": [1.0, 3.0],
            "end_settled_mps": [0.05, 0.15],
            "note": "[warn, fail]; game-animation references/validation.md",
        },
        "clips": [],
        "summary": {
            "count": len(measurements),
            "pass": sum(1 for m in measurements if m["status"] == "PASS"),
            "warn": sum(1 for m in measurements if m["status"] == "WARN"),
            "fail": sum(1 for m in measurements if m["status"] == "FAIL"),
        },
        "fails": [{"clip": c, "check": x} for c, x in all_fails],
        "warnings": [{"clip": c, "check": x} for c, x in all_warns],
    }
    for name, frames, loop, root_motion, direction, fn in CLIPS:
        res = next(m for m in measurements if m["clip"] == name)
        res = dict(res)
        res["path"] = "assets/animations/%s.glb" % name
        res["action"] = name
        res["notes"] = CLIP_NOTES[name]
        res["source"] = "procedural (no mocap; GVHMR/Mixamo unavailable)"
        res["license"] = "CC0 (authored here on the project's player mesh)"
        manifest["clips"].append(res)
    mpath = os.path.join(OUT, "manifest.json")
    with open(mpath, "w") as fh:
        json.dump(manifest, fh, indent=2)
    print("wrote", os.path.relpath(mpath, ROOT))
    print("SUMMARY pass=%d warn=%d fail=%d" %
          (manifest["summary"]["pass"], manifest["summary"]["warn"],
           manifest["summary"]["fail"]))
    return 1 if all_fails else 0


if __name__ == "__main__":
    sys.exit(main())
