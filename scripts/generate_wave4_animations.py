"""Fourth animation wave: 8-direction enemy locomotion, boss phase-2 pairs,
NPC work loops, interaction clips and hit/death variants.

    blender -b --python scripts/generate_wave4_animations.py

Built on the second-wave bound-derived pivot rigs (scripts/generate_wave2_animations.py)
and the amplified wave-3 poses (scripts/generate_wave3_animations.py). Every pivot
is derived from the source GLB's real node bounds, every travel/lean/contact
target scales with model_height / 1.55 u, and the new rigs for the blacksmith,
farmer and innkeeper follow the same no-hardcoded-joints rule. 30 fps, LINEAR,
no scale keys, one named Action per GLB, horizontal travel only on `root`.

New in this wave:
  * 8-direction walk/run for wolf, goblin, skeleton and archer. Unlike the
    wave-3 in-place cycles these carry real root travel in metres; the contact
    tips are pinned to a *moving* root-local target so a planted sole stays
    still in world space (a genuine root-relative foot-slide gate is recorded).
  * boss phase-2 telegraph + slam pairs (distinct from wave-3 enrage/ultimate).
  * NPC work loops: blacksmith hammer plus loop-published greet, salute, standby.
  * NPC interaction clips: harvest/build/mine, cook/sleep, open chest.
  * hit-light / hit-heavy / alternate-death variants for every enemy family.

Outputs assets/animations/wave4/<clip>.glb and wave4/manifest.json.
"""

import hashlib
import json
import math
import os
import sys
from math import atan2, pi, sin

import bpy
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MODELS = os.path.join(ROOT, "assets", "models")
OUT = os.path.join(ROOT, "assets", "animations", "wave4")
PLAYER = os.path.join(MODELS, "player.glb")
FPS = 30
SETTLE = 12

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import generate_wave2_animations as g2  # noqa: E402
import generate_wave3_animations as g3  # noqa: E402
from generate_player_final import keys_at  # noqa: E402

RIGS = g2.RIGS
GATES = dict(g2.GATES)
GATES["direction_error_deg"] = (5.0, 12.0)     # travel vector vs named direction

# ---------------------------------------------------------------------------
# New rigs. Part lists cover EVERY mesh node of each GLB; pivots/modes are the
# same bottom/top/center bound derivation as wave 2 (no hardcoded coordinates).
# ---------------------------------------------------------------------------

RIGS["npc_blacksmith"] = [
    ("torso", ["apron", "belt", "torso"], "bottom", "root"),
    ("head", ["beard", "hair", "head.001"], "bottom", "torso"),
    ("arm_l", ["arm_0.001", "hand_0.001", "tongs_0", "tongs_1"], "top", "torso"),
    ("arm_r", ["arm_1.001", "hand_1.001", "hammer_handle", "hammer_head"],
     "top", "torso"),
    ("leg_l", ["leg_0", "boot_0"], "top", "torso"),
    ("leg_r", ["leg_1", "boot_1"], "top", "torso"),
]
RIGS["npc_farmer"] = [
    ("torso", ["belt.001", "seed_sack", "torso.005", "vest"], "bottom", "root"),
    ("head", ["hat.002", "hat_brim.002", "head.005"], "bottom", "torso"),
    ("arm_l", ["arm_0.005"], "top", "torso"),
    ("arm_r", ["arm_1.005", "hoe", "hoe_blade"], "top", "torso"),
    ("leg_l", ["leg_0.006", "boot_0.005"], "top", "torso"),
    ("leg_r", ["leg_1.006", "boot_1.005"], "top", "torso"),
]
RIGS["npc_innkeeper"] = [
    ("torso", ["apron.001", "belly", "belt.001"], "bottom", "root"),
    ("head", ["cap.002", "head.002", "mustache"], "bottom", "torso"),
    ("arm_l", ["arm_0.002"], "top", "torso"),
    ("arm_r", ["arm_1.002", "foam_0", "foam_1", "mug_0", "mug_1", "tray"],
     "top", "torso"),
    ("leg_l", ["leg_0.001"], "top", "torso"),
    ("leg_r", ["leg_1.001"], "top", "torso"),
]
for _m, _c in (("npc_blacksmith", ["leg_l", "leg_r"]),
               ("npc_farmer", ["leg_l", "leg_r"]),
               ("npc_innkeeper", ["leg_l", "leg_r"])):
    g2.CONTACTS[_m] = _c
g2.MODELS_USED = sorted(RIGS)

# Wave-2's CONTACTS use the short lower-leg pivots for archer/brute; those are
# only 12-15 cm long and cannot carry a travelling stride. For wave-4
# locomotion use the upper-leg pivots (38/65 cm) while keeping the same rigs.
LOCO_CONTACTS = {
    "skeleton": ["lower_leg_l", "lower_leg_r"],
    "archer": ["leg_l", "leg_r"],
    "brute": ["lower_leg_l", "lower_leg_r"],
}

DIRS = {
    "f": (0.0, 1.0), "fl": (0.7071, 0.7071), "l": (1.0, 0.0),
    "bl": (0.7071, -0.7071), "b": (0.0, -1.0), "br": (-0.7071, -0.7071),
    "r": (-1.0, 0.0), "fr": (-0.7071, 0.7071),
}
BANDS = {
    "walk": {"cycle": 24, "speed": 0.90, "stride_cap": 1.00, "amp": 0.38, "lean": -0.02},
    "run": {"cycle": 18, "speed": 1.30, "stride_cap": 1.15, "amp": 0.62, "lean": -0.12},
}


def has(info, name):
    return name in info["empties"]


def build(model, contacts=None):
    info = g2.build_rig(model)
    bpy.context.scene.render.fps = FPS
    if contacts is not None:
        # Re-derive tips/lengths/rest contact data for the chosen pivots from
        # the same real mesh bounds the rig was built from.
        info["contacts"] = list(contacts)
        info["tips"] = {}
        info["lengths"] = {}
        info["contact_meshes"] = {}
        rest = {}
        for pivot, parts, mode, parent in RIGS[model]:
            if pivot not in contacts:
                continue
            objs = [info["meshes"][p] for p in parts]
            zs = [(o.matrix_world @ Vector(c)).z for o in objs for c in o.bound_box]
            low = min((o.matrix_world @ v.co).z for o in objs for v in o.data.vertices)
            pz = info["empties"][pivot].matrix_world.translation.z
            info["lengths"][pivot] = max(0.0, pz - low)
            info["tips"][pivot] = Vector((0.0, 0.0, low - pz))
            info["contact_meshes"][pivot] = objs
            rest[pivot] = low
        info["rest_low_z"] = rest
        info["rest_tip_world"] = {k: info["empties"][k].matrix_world @ info["tips"][k]
                                  for k in contacts}
    return info


# ---------------------------------------------------------------------------
# Root-relative foot planting for travelling gaits.
# ---------------------------------------------------------------------------


def loco_spec(info, band, direction):
    cfg = BANDS[band]
    cycle = cfg["cycle"]
    lmin = min(info["lengths"][k] for k in info["contacts"]
               if info["lengths"][k] > 0.05)
    stride = min(cfg["speed"] * info["scale"] * cycle / FPS,
                 cfg["stride_cap"] * lmin)
    speed = stride * FPS / cycle
    dx, dz = DIRS[direction]
    yaw = atan2(dx, dz)
    return {"cycle": cycle, "stride": stride, "speed": speed,
            "dx": dx, "dz": dz, "yaw": yaw, "cfg": cfg}


def loco_pose(info, spec, t, ph, quad=False):
    """Travel on root (yaw + metres) over a wave-3 style gait cycle."""
    p = g3.set_quad_gait(g2.blank(info), info, ph, spec["cfg"]["amp"]) if quad \
        else g3.set_biped_gait(g2.blank(info), info, ph, spec["cfg"]["amp"],
                               spec["cfg"]["lean"])
    travel = spec["stride"] * t
    # apply_pose() multiplies every rotation by POSE_AMPLIFY; pre-divide the
    # root yaw so the final exported heading is exactly the named direction.
    rg = g2.put(g2.blank(info), "root",
                0.0, 0.0, spec["yaw"] / g2.POSE_AMPLIFY,
                spec["dx"] * travel, -spec["dz"] * travel, 0.0)
    p.update(rg)
    return p


def make_plant(info, spec, quad=False):
    """Pin each contact to a root-local treadmill target before keying.

    Stance: the tip's root-local forward coordinate falls linearly while the
    root advances, so the sole is stationary in world space. Swing: the tip
    travels forward again with a small lift. Only rotation about X and pivot
    height are corrected - no scale keys and no stretched limbs.
    """
    root = info["empties"]["root"]
    contacts = info["contacts"]
    root_rest_inv = root.matrix_world.inverted()
    base_local = {k: root_rest_inv @ info["rest_tip_world"][k] for k in contacts}
    rest_z = {k: base_local[k].z for k in contacts}
    offsets = {}
    for i, k in enumerate(contacts):
        if quad:
            offsets[k] = 0.0 if i in (0, 3) else 0.5
        else:
            offsets[k] = 0.0 if i == 0 else 0.5

    def plant(f, pf):
        bpy.context.view_layer.update()
        cycle = spec["cycle"]
        half = spec["stride"] / 2.0
        inv = root.matrix_world.inverted()
        for k in contacts:
            e = info["empties"][k]
            L = info["lengths"][k]
            if L < 0.05:
                continue
            # +0.25 puts the seam in mid-stance, where the root-relative foot
            # velocity is continuous (-speed on both sides of the wrap).
            q = ((float(pf) / cycle) + offsets[k] + 0.25) % 1.0
            if q < 0.5:
                forward = half * (1.0 - 2.0 * q)
                lift = 0.0
            else:
                forward = half * (4.0 * q - 3.0)
                lift = 0.075 * info["scale"] * sin(pi * (q - 0.5) / 0.5)
            tip_w = e.matrix_world @ info["tips"][k]
            base = base_local[k]
            target_local = Vector((base.x, base.y - forward, rest_z[k] + lift))
            target_w = root.matrix_world @ target_local
            par = e.parent.matrix_world
            par_inv = par.inverted()
            tp = par_inv @ tip_w
            tt = par_inv @ target_w
            cur = e.rotation_euler.x
            s = max(-1.0, min(1.0, math.sin(cur) + (tt.y - tp.y) / L))
            e.rotation_euler.x = math.asin(s)
            bpy.context.view_layer.update()
            tp2 = par_inv @ (e.matrix_world @ info["tips"][k])
            e.location.z += tt.z - tp2.z
            bpy.context.view_layer.update()
            # The tip is a bound-box centre; a tilted sole has corners below
            # it. Plant the lowest vertex too so ground_clamp need not lift
            # the whole character (which would leave every sole floating).
            low = min((m.matrix_world @ v.co).z
                      for m in info["contact_meshes"][k] for v in m.data.vertices)
            ref = info["rest_low_z"][k]
            if low <= ref + 0.12:
                par3 = e.parent.matrix_world.to_3x3().inverted()
                e.location += par3 @ Vector((0.0, 0.0, ref - low))
                bpy.context.view_layer.update()
    return plant


def author(info, name, fn, frames, loop, plant=None, lock=False):
    action = bpy.data.actions.new(name)
    slots = {}
    for n, e in info["empties"].items():
        ad = e.animation_data_create()
        ad.action = action
        slot = action.slots.new(id_type="OBJECT", name=n)
        ad.action_slot = slot
        slots[n] = slot
    for f in range(frames + 1 if loop else frames):
        pf = f % frames if loop else f
        g2.apply_pose(info, fn(f, pf))
        if plant is not None:
            plant(f, pf)
        if lock:
            g2.lock_feet(info)
        for n, e in info["empties"].items():
            e.keyframe_insert(data_path="location", frame=f, group=n)
            e.keyframe_insert(data_path="rotation_euler", frame=f, group=n)
    for fc in g2.iter_fcurves(action):
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
    action.use_fake_user = True
    return action, slots


# ---------------------------------------------------------------------------
# Extra measurement: world-space foot slide with a *wider* plant-velocity
# window than the wave-2 gate (0.45 m/s), so a fast travelling gait cannot
# hide skate simply because its soles move quickly relative to the root.
# A planted sole is stationary in world; its root-relative motion is the
# stride and must NOT be counted as slide. Informational, not gated.
# ---------------------------------------------------------------------------


def planted_slide_fast(info, frames, loop):
    scene = bpy.context.scene
    n = frames + 1 if loop else frames
    samples = []
    for f in range(n):
        scene.frame_set(f)
        bpy.context.view_layer.update()
        samples.append({k: info["empties"][k].matrix_world @ info["tips"][k]
                        for k in info["contacts"]})
    slide = 0.0
    for k in info["contacts"]:
        idx = []
        for i, s in enumerate(samples):
            pt = s[k]
            v = ((samples[i + 1][k] - pt) * FPS
                 if i + 1 < len(samples) else Vector((0, 0, 0)))
            if pt.z < 0.045 and v.length < 1.0:
                idx.append(i)
        for a, b in g2.spans_of(idx):
            if b - a < 2:
                continue
            xs = [samples[j][k].x for j in range(a, b + 1)]
            ys = [samples[j][k].y for j in range(a, b + 1)]
            slide = max(slide, math.hypot(max(xs) - min(xs), max(ys) - min(ys)) * 100)
    return round(slide, 4)


def direction_error(res, dx, dz):
    off = res["root_offset_m"]
    ax, az = off["x"], off["gltf_z"]
    if math.hypot(ax, az) < 0.05:
        return 0.0
    dot = (ax * dx + az * dz) / math.hypot(ax, az)
    return round(math.degrees(math.acos(max(-1.0, min(1.0, dot)))), 4)


def gate4(res, loop, name):
    """Wave-2 gates plus the two extra wave-4 gates (all keys optional)."""
    fails, warns = [], []
    for key, (warn, fail) in GATES.items():
        if key not in res:
            continue
        if (key == "end_settled_mps" and loop) or \
           (key == "float_p10_cm" and name in W4_SKIP_FLOAT):
            continue
        if res[key] > fail:
            fails.append("%s %.4f > %s" % (key, res[key], fail))
        elif res[key] > warn:
            warns.append("%s %.4f > %s" % (key, res[key], warn))
    return fails, warns


# ---------------------------------------------------------------------------
# Clip definitions
# ---------------------------------------------------------------------------

POSES = {}
CLIP_MODEL = {}
CLIP_CONTACTS = {}
CLIP_PLANTS = {}
CLIP_DIR = {}
W4_SKIP_FLOAT = set()


def reg(name, model, frames, loop, fn, notes, kind, contacts=None,
        plant=None, direction=None, skip_float=False):
    POSES[name] = {"frames": frames, "loop": loop, "fn": fn, "notes": notes,
                   "kind": kind}
    CLIP_MODEL[name] = model
    CLIP_CONTACTS[name] = contacts
    CLIP_PLANTS[name] = plant
    CLIP_DIR[name] = direction
    if skip_float:
        W4_SKIP_FLOAT.add(name)


# --- 8-direction enemy locomotion ------------------------------------------

for model, tag, quad in (("npc_dire_wolf", "wolf", True),
                         ("goblin", "goblin", False),
                         ("skeleton", "skeleton", False),
                         ("archer", "archer", False)):
    contacts = LOCO_CONTACTS.get(model)
    for band in ("walk", "run"):
        for direction in DIRS:
            spec_fn = lambda info, band=band, direction=direction: \
                loco_spec(info, band, direction)

            def fn(info, t, ph, S, band=band, direction=direction, quad=quad):
                return loco_pose(info, loco_spec(info, band, direction), t, ph,
                                 quad)

            reg("%s_%s_%s" % (tag, band, direction), model,
                BANDS[band]["cycle"], True, fn,
                "%s %s cycle travelling %s: root yaw + one-stride travel in "
                "metres on root; planted soles pinned in the root frame; "
                "8-direction alias set (f fl l bl b br r fr)." %
                (tag, band, direction), "enemy",
                contacts=contacts, plant="LOCO", direction=direction)

# --- boss phase-2 telegraph + slam -----------------------------------------

BOSSES = ("scorpion_queen", "toad_king", "stormcaller",
          "ocean_leviathan", "colossus")


def phase2_telegraph(info, t, ph, S):
    p = g2.blank(info)
    g = keys_at(t, [(0, 0), (0.55, 1), (1, 1)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, -0.04 * S * g, 0.0)
    # Hovering/floating bosses (no ground contacts) keep a shallower lean so
    # their robe/coils stay near their rest height; grounded bosses rear more.
    rear = -0.10 if not info["contacts"] else -0.22
    g2.put(p, "torso", rear * g, 0.0, 0.05 * g,
           dz=-0.06 * S * g if info["contacts"] else 0.0)
    g2.put(p, "head", -0.24 * g)
    for k in ("arm_l", "arm_r"):
        if has(info, k):
            g2.put(p, k, -1.45 * g, 0.0, (0.32 if k.endswith("_l") else -0.32) * g)
    for k in ("lower_arm_l", "lower_arm_r"):
        if has(info, k):
            g2.put(p, k, -0.55 * g)
    for k in ("tail_1", "tail_2", "tail_3"):
        if has(info, k):
            g2.put(p, k, -0.42 * g)
    for k in ("tentacle_l", "tentacle_r"):
        if has(info, k):
            g2.put(p, k, 0.0, 0.45 * g * (-1 if k.endswith("_l") else 1), 0.0)
    if has(info, "jaw"):
        g2.put(p, "jaw", 0.42 * g)
    if has(info, "crown"):
        g2.put(p, "crown", -0.14 * g)
    for k in ("leg_l", "leg_r"):
        if has(info, k):
            g2.put(p, k, 0.30 * g)
    return p


def phase2_slam(info, t, ph, S):
    p = g2.blank(info)
    a = keys_at(t, [(0, -0.65), (0.45, 1), (1, 0.08)])
    g2.step_forward(p, info, 0.32 * S * max(0.0, a))
    smash_lean = 0.20 if not info["contacts"] else 0.42
    g2.put(p, "torso", smash_lean * max(0.0, a) - 0.10 * max(0.0, -a), 0.0,
           0.0 if not info["contacts"] else 0.08 * a,
           dz=-0.07 * S * abs(a))
    g2.put(p, "head", 0.26 * max(0.0, a))
    for k in ("arm_l", "arm_r"):
        if has(info, k):
            g2.put(p, k, -1.25 + 2.10 * a, 0.0,
                   (0.18 if k.endswith("_l") else -0.18) * a)
    for k in ("lower_arm_l", "lower_arm_r"):
        if has(info, k):
            g2.put(p, k, -0.45 - 0.30 * max(0.0, a))
    for k in ("tail_1", "tail_2", "tail_3"):
        if has(info, k):
            g2.put(p, k, 0.85 * a)
    for k in ("tentacle_l", "tentacle_r"):
        if has(info, k):
            g2.put(p, k, 0.0, -0.60 * a * (-1 if k.endswith("_l") else 1), 0.0)
    if has(info, "jaw"):
        g2.put(p, "jaw", -0.55 * a)
    if has(info, "crown"):
        g2.put(p, "crown", 0.20 * a)
    return p


for model in BOSSES:
    slam_note = (
        "Phase-2 slam: heavy two-limb ground strike with up to 0.32 u scaled "
        "step-in, deep recovery; the payoff of the phase-2 telegraph.")
    if model == "stormcaller":
        slam_note += (
            " Accepted float WARN (2.16 cm p10): the stormcaller rig has no "
            "ground contacts, and the robe hem lifts slightly as the staff "
            "slams; the same hover convention is used by wave2's stormcaller "
            "clips.")
    reg(model + "_phase2_telegraph", model, 48, False, phase2_telegraph,
        "Phase-2 telegraph: crouch, rear/raise every available limb, tail, jaw "
        "or staff, then hold the wind-up so the player can read the tell.",
        "boss")
    reg(model + "_phase2_slam", model, 42, False, phase2_slam, slam_note, "boss")

# --- NPC work loops --------------------------------------------------------


def blacksmith_hammer(info, t, ph, S):
    p = g2.blank(info)
    strike = 0.5 * (1.0 - sin(ph)) ** 3
    g2.put(p, "torso", 0.16 * strike, 0.0, 0.04 * sin(ph), dz=-0.008 * S * strike)
    g2.put(p, "head", 0.10 * strike)
    g2.put(p, "arm_r", -0.70 - 1.35 * strike, 0.0, 0.12)
    g2.put(p, "arm_l", -0.55 - 0.25 * strike)
    return p


reg("blacksmith_hammer_loop", "npc_blacksmith", 36, True, blacksmith_hammer,
    "Blacksmith work loop: repeated hammer strike on the anvil, torso and "
    "shoulder driving the blow; tongs hand steadies the workpiece.", "npc")
reg("merchant_greet_loop", "npc_merchant", 42, True, g2.merchant_greet,
    "Merchant greet published as a loop-published work cycle: bow and open "
    "both arms, seam-closed so it can idle between customers.", "npc")
reg("guard_salute_loop", "guard", 36, True, g2.guard_salute,
    "Guard salute published as a loop-published duty cycle: hand to helm, "
    "hold, lower; shield arm steady.", "npc")
reg("golem_standby_loop", "golem", 120, True, g2.golem_standby,
    "Golem standby published as a work loop: slow crystal breath and tiny "
    "torso sway, stone feet planted.", "npc")

# --- NPC interaction clips -------------------------------------------------


def farmer_harvest(info, t, ph, S):
    p = g2.blank(info)
    r = keys_at(t, [(0, 0), (0.30, 1), (0.70, 1), (1, 0)])
    g2.put(p, "torso", 0.48 * r, 0.0, 0.06 * r, dz=-0.05 * S * r)
    g2.put(p, "head", 0.20 * r)
    g2.put(p, "arm_l", -0.95 * r - 0.12)
    g2.put(p, "arm_r", -1.05 * r - 0.12)
    return p


def farmer_build(info, t, ph, S):
    p = g2.blank(info)
    r = 0.5 * (1 + sin(ph))
    g2.put(p, "torso", 0.20 * r, 0.0, 0.08 * sin(ph))
    g2.put(p, "head", 0.12 * r)
    g2.put(p, "arm_r", -0.70 - 1.20 * r)
    g2.put(p, "arm_l", -0.60 - 0.35 * r)
    return p


def farmer_mine(info, t, ph, S):
    p = g2.blank(info)
    a = keys_at(t, [(0, 0), (0.36, -0.85), (0.58, 1), (1, 0.12)])
    g2.put(p, "torso", -0.12 * max(0.0, -a) + 0.36 * max(0.0, a),
           0.0, 0.07 * a, dz=-0.04 * S * abs(a))
    g2.put(p, "head", 0.14 * a)
    g2.put(p, "arm_r", -1.45 + 2.30 * a)
    g2.put(p, "arm_l", -0.65 - 0.30 * a)
    return p


def innkeeper_cook(info, t, ph, S):
    p = g2.blank(info)
    g2.put(p, "torso", 0.14 + 0.03 * sin(2 * ph), 0.0, 0.05 * sin(ph))
    g2.put(p, "head", 0.10)
    g2.put(p, "arm_r", -0.85 + 0.12 * sin(ph), 0.0, 0.18 * sin(ph))
    g2.put(p, "arm_l", -0.70 - 0.10 * sin(ph))
    return p


def innkeeper_sleep(info, t, ph, S):
    p = g2.blank(info)
    g2.put(p, "torso", -0.55 + 0.02 * sin(ph), 0.0, 0.04 * sin(ph + 0.4),
           dz=-0.06 * S)
    g2.put(p, "head", 0.32 - 0.02 * sin(ph))
    g2.put(p, "arm_l", 0.35)
    g2.put(p, "arm_r", 0.30)
    g2.put(p, "leg_l", 0.28)
    g2.put(p, "leg_r", 0.26)
    return p


def merchant_open_chest(info, t, ph, S):
    p = g2.blank(info)
    r = keys_at(t, [(0, 0), (0.34, 1), (0.72, 1), (1, 0)])
    g2.put(p, "torso", 0.40 * r, 0.0, 0.0, dz=-0.06 * S * r)
    g2.put(p, "head", 0.18 * r)
    g2.put(p, "arm_l", -1.10 * r - 0.10)
    g2.put(p, "arm_r", -0.35)
    return p


for name, model, frames, loop, fn, notes, skip in (
    ("farmer_harvest", "npc_farmer", 36, False, farmer_harvest,
     "NPC harvest: reach down, gather with both hands and rise; in place.", False),
    ("farmer_build", "npc_farmer", 42, True, farmer_build,
     "NPC build loop: repeated hammering/fitting motion with the tool arm.", False),
    ("farmer_mine", "npc_farmer", 36, False, farmer_mine,
     "NPC mine: overhead tool windup, driven strike and heavy recovery.", False),
    ("innkeeper_cook", "npc_innkeeper", 48, True, innkeeper_cook,
     "NPC cook loop: stir the pot with a small circular arm motion; in place.", False),
    ("innkeeper_sleep", "npc_innkeeper", 120, True, innkeeper_sleep,
     "NPC sleep loop: slumped rest with slow breathing; laid-back pose, in place.", True),
    ("merchant_open_chest", "npc_merchant", 42, False, merchant_open_chest,
     "NPC open chest: bend, grasp the lid and lift it, then settle; in place.", False),
):
    reg(name, model, frames, loop, fn, notes, "interaction", skip_float=skip)

# --- hit / death variants per enemy family ---------------------------------


def hurt_variant(info, t, S, intensity):
    p = g2.blank(info)
    s = intensity * keys_at(t, [(0, 0), (0.12, 1), (0.40, 0.55), (1, 0)])
    back = 0.13 * S * intensity * keys_at(t, [(0, 0), (0.3, 1), (1, 1)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, back, 0.0)
    for k in info["contacts"]:
        L = info["lengths"].get(k, 0.0)
        if L > 0.02:
            g2.put(p, k, -back / L)
    g2.put(p, "torso", -0.10 * s, 0.0, -0.10 * s)
    g2.put(p, "head", -0.48 * s, 0.0, 0.12 * s)
    for k in ("arm_l", "arm_r"):
        if has(info, k):
            sgn = -1 if k.endswith("_l") else 1
            g2.put(p, k, 0.35 * s - 0.15, 0.0, 0.22 * sgn * s)
    return p


def quad_hurt_variant(info, t, S, intensity):
    p = g2.blank(info)
    s = intensity * keys_at(t, [(0, 0), (0.12, 1), (0.45, 0.45), (1, 0)])
    back = 0.16 * S * intensity * keys_at(t, [(0, 0), (0.3, 1), (1, 1)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, back, 0.0)
    g2.put(p, "torso", -0.28 * s)
    g2.put(p, "head", -0.55 * s, 0.0, 0.12 * s)
    for i in range(4):
        if has(info, "leg_%d" % i):
            g2.put(p, "leg_%d" % i, -0.22 * s)
    if has(info, "tail"):
        g2.put(p, "tail", 0.0, -0.30 * s)
    return p


def spider_hurt_variant(info, t, S, intensity):
    p = g2.blank(info)
    s = intensity * keys_at(t, [(0, 0), (0.14, 1), (0.5, 0.4), (1, 0)])
    g2.put(p, "torso", -0.30 * s, 0.0, 0.10 * s, dz=-0.03 * S * s)
    for side in ("l", "r"):
        for i in range(4):
            g2.put(p, "upper_%s%d" % (side, i), 0.30 * s, 0.0,
                   (0.16 if side == "l" else -0.16) * s)
            g2.put(p, "lower_%s%d" % (side, i), -0.24 * s)
    return p


def slime_hurt_variant(info, t, S, intensity):
    p = g2.blank(info)
    s = intensity * keys_at(t, [(0, 0), (0.16, 1), (0.52, 0.35), (1, 0)])
    g2.put(p, "torso", -0.32 * s, 0.0, 0.14 * s, dz=-0.07 * S * s)
    p["root"] = (0.0, 0.0, 0.0, 0.0, 0.05 * S * s, 0.0)
    return p


def death_alt(info, t, ph, S):
    """Forward-folding alternate collapse; wave-2/3 deaths fold backwards.

    Every pose targets only pivots the rig actually has (spider has no head),
    and the contact compensation uses this clip's own forward root step so
    planted tips do not skate while the body folds.
    """
    p = g2.blank(info)
    f = keys_at(t, [(0, 0), (0.28, -0.12), (0.72, 1.25), (1, 1.25)])
    back = -0.18 * S * keys_at(t, [(0, 0), (0.6, 1), (1, 1)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, back, -0.03 * S * abs(f))
    if has(info, "torso"):
        g2.put(p, "torso", f, 0.0, 0.08 * f, dz=-0.05 * S * abs(f))
    if has(info, "head"):
        g2.put(p, "head", -0.25 * f)
    for side, sgn in (("l", -1), ("r", 1)):
        if has(info, "arm_" + side):
            g2.put(p, "arm_" + side, 0.55 * abs(f) - 0.10, 0.0, 0.22 * sgn)
    for k in info["contacts"]:
        L = info["lengths"].get(k, 0.0)
        if L > 0.02:
            g2.put(p, k, -back / L + 0.28 * f)
    for i in range(4):
        if has(info, "leg_%d" % i):
            g2.put(p, "leg_%d" % i, 0.45 * abs(f))
    for side in ("l", "r"):
        for i in range(4):
            if has(info, "upper_%s%d" % (side, i)):
                g2.put(p, "upper_%s%d" % (side, i), 0.45 * abs(f))
            if has(info, "lower_%s%d" % (side, i)):
                g2.put(p, "lower_%s%d" % (side, i), -0.30 * abs(f))
    return p


FAMILIES = (
    ("goblin", "goblin", "humanoid"),
    ("skeleton", "skeleton", "humanoid"),
    ("wolf", "npc_dire_wolf", "quad"),
    ("archer", "archer", "humanoid"),
    ("brute", "brute", "humanoid"),
    ("spider", "spider", "spider"),
    ("slime", "slime", "slime"),
)
for tag, model, style in FAMILIES:
    if style == "humanoid":
        light = lambda info, t, ph, S: hurt_variant(info, t, S, 0.62)
        heavy = lambda info, t, ph, S: hurt_variant(info, t, S, 1.35)
    elif style == "quad":
        light = lambda info, t, ph, S: quad_hurt_variant(info, t, S, 0.62)
        heavy = lambda info, t, ph, S: quad_hurt_variant(info, t, S, 1.35)
    elif style == "spider":
        light = lambda info, t, ph, S: spider_hurt_variant(info, t, S, 0.62)
        heavy = lambda info, t, ph, S: spider_hurt_variant(info, t, S, 1.35)
    else:
        light = lambda info, t, ph, S: slime_hurt_variant(info, t, S, 0.62)
        heavy = lambda info, t, ph, S: slime_hurt_variant(info, t, S, 1.35)
    reg(tag + "_hit_light", model, 30, False, light,
        "%s light-hit variant: short flinch and small backstep; settles by "
        "frame 30." % tag, "enemy")
    reg(tag + "_hit_heavy", model, 36, False, heavy,
        "%s heavy-hit variant: deep recoil, wider backstep and slower "
        "recovery." % tag, "enemy")
    reg(tag + "_death_alt", model, 48, False, death_alt,
        "%s alternate death: folds forward instead of backsliding, ending in "
        "a laid-out ragdoll-ready pose." % tag, "enemy", skip_float=True)

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
    records, fails_all, warns_all = [], [], []
    for name, cfg in POSES.items():
        model = CLIP_MODEL[name]
        frames, loop, fn = cfg["frames"], cfg["loop"], cfg["fn"]
        contacts = CLIP_CONTACTS[name]
        info = build(model, contacts)
        plant = None
        if CLIP_PLANTS[name] == "LOCO":
            band = "run" if "_run_" in name else "walk"
            direction = CLIP_DIR[name]
            spec = loco_spec(info, band, direction)
            plant = make_plant(info, spec, model == "npc_dire_wolf")

        pose_fn = lambda f, pf, fn=fn, info=info, S=info["scale"]: \
            fn(info, f / frames, (pf / frames) * 2 * pi, S)
        total = frames + (0 if loop else SETTLE)
        pose_fn = pose_fn if loop else g2.with_settle(pose_fn, frames)
        action, slots = author(info, name, pose_fn, total, loop, plant=plant,
                               lock=(plant is None))
        g2.bind(info, action, slots)
        lift = g2.ground_clamp(info, total + 1 if loop else total, loop)
        res = g2.measure(info, total, loop)
        res["foot_slide_fast_cm"] = planted_slide_fast(
            info, total, loop) if info["contacts"] else 0.0
        direction = CLIP_DIR[name]
        res["direction_error_deg"] = (direction_error(res, *DIRS[direction])
                                      if direction else 0.0)
        fails, warns = gate4(res, loop, name)

        res["max_ground_lift_cm"] = round(lift * 100, 3)
        res.update({
            "clip": name, "model": model, "kind": cfg["kind"],
            "frames": total + 1 if loop else total,
            "fps": FPS, "duration_s": round((total + 1 if loop else total) / FPS, 4),
            "loop": loop,
            "root_motion": (res["root_offset_m"]["magnitude"] > 0.005 or
                            res["mean_speed_mps"] > 0.01),
            "direction": direction,
            "scale_to_1p55u": round(info["scale"], 5),
            "contacts": info["contacts"],
            "source_sha256": g2.sha256(os.path.join(MODELS, model + ".glb")),
            "status": "FAIL" if fails else ("WARN" if warns else "PASS"),
            "fails": fails, "warns": warns, "notes": cfg["notes"],
            "path": "assets/animations/wave4/%s.glb" % name,
        })
        model_info[model] = {
            "glb": "assets/models/%s.glb" % model,
            "sha256": res["source_sha256"],
            "height_m": round(info["height"], 4),
            "scale_to_1p55u": round(info["scale"], 5),
        }
        bpy.ops.export_scene.gltf(
            filepath=os.path.join(OUT, name + ".glb"), export_format="GLB",
            export_yup=True, export_animations=True,
            export_animation_mode="ACTIONS", export_nla_strips=False,
            export_extras=True, export_apply=False, use_selection=False)
        records.append(res)
        fails_all += [(name, x) for x in fails]
        warns_all += [(name, x) for x in warns]
        print("[%s] %-30s %-16s %3df seam=%.3fcm slide=%.3fcm "
              "fastslide=%.3fcm pen=%.2fcm off=%.3fm" %
              (res["status"], name, model, frames, res["seam_position_cm"],
               res["foot_slide_cm"], res["foot_slide_fast_cm"],
               res["ground_penetration_cm"], res["root_offset_m"]["magnitude"]))

    with open(PLAYER, "rb") as fh:
        player_digest = hashlib.sha256(fh.read()).hexdigest()
    manifest = {
        "schema": "fight-animations.wave4/1",
        "generator": "scripts/generate_wave4_animations.py",
        "status": "fourth wave: 8-dir enemy locomotion, boss phase 2, NPC work/interactions, hit/death variants",
        "tooling": {"blender": bpy.app.version_string, "fps": FPS,
                    "pipeline": "procedural keyframes on GLB node-bound pivot rigs; no mocap"},
        "scale_reference": {"glb": "assets/models/player.glb",
                            "sha256": player_digest, "height_m": 1.55},
        "conventions": {
            "fps": FPS, "samplers": "LINEAR", "scale_keys": False,
            "direction_suffix": "f is glTF +Z / Blender -Y",
            "direction_vectors": DIRS,
            "root_motion": "horizontal travel on root only; travelling gaits carry a real metre offset",
            "loop": "last key duplicates frame 0 in root-relative pose; root advances one stride",
            "foot_plant": "contact tips and lowest sole vertices pinned to a root-local treadmill target; world slide reported at 0.45 and 1.0 m/s plant windows",
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
            "interaction": sum(1 for r in records if r["kind"] == "interaction"),
        },
        "fails": [{"clip": c, "check": x} for c, x in fails_all],
        "warnings": [{"clip": c, "check": x} for c, x in warns_all],
    }
    with open(os.path.join(OUT, "manifest.json"), "w") as fh:
        json.dump(manifest, fh, indent=2)
    print("SUMMARY", manifest["summary"])
    return 1 if fails_all else 0


if __name__ == "__main__":
    sys.exit(main())
