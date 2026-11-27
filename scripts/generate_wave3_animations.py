"""Third animation wave: enemy locomotion/idles/deaths, boss phase 2, NPC work.

    blender -b --python scripts/generate_wave3_animations.py

Built on the second-wave bound-derived pivot rigs (scripts/generate_wave2_animations.py):
every pivot is derived from the source GLB's real node bounds and every travel,
lean and contact target scales with model_height / 1.55 u. 30 fps, LINEAR, no
scale keys, horizontal travel only on `root`, one named Action per GLB, and the
same numeric [warn, fail] gates as the player and second-wave sets.

Outputs assets/animations/wave3/<clip>.glb and wave3/manifest.json.
"""

import hashlib
import json
import os
import sys
from math import pi, sin

import bpy

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MODELS = os.path.join(ROOT, "assets", "models")
OUT = os.path.join(ROOT, "assets", "animations", "wave3")
FPS = 30

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import generate_wave2_animations as g2  # noqa: E402
from generate_player_final import keys_at  # noqa: E402

RIGS = g2.RIGS
GATES = g2.GATES
SETTLE = 12


# ---------------------------------------------------------------------------
# Pose helpers (canonical pivot names are defined in wave2's RIGS)
# ---------------------------------------------------------------------------


def has(info, name):
    return name in info["empties"]


def set_biped_gait(p, info, ph, amp, lean):
    """In-place walk/run cycle: legs counter-swing, arms oppose, torso bobs."""
    swing = sin(ph)
    g2.put(p, "torso", lean, 0.0, 0.05 * swing,
           dz=0.010 * (1.0 + amp) * (1 - sin(2 * ph)) / 2)
    g2.put(p, "head", 0.0, 0.0, -0.05 * swing)
    for side, sgn in (("l", -1), ("r", 1)):
        leg = "leg_" + side
        if has(info, leg):
            g2.put(p, leg, amp * swing * sgn)
            low = "lower_leg_" + side
            if has(info, low):
                g2.put(p, low, -0.30 * max(0.0, -sgn * swing))
        arm = "arm_" + side
        if has(info, arm):
            g2.put(p, arm, -0.25 - 0.40 * swing * (-sgn),
                   0.0, 0.05 * sgn)
            low = "lower_arm_" + side
            if has(info, low):
                g2.put(p, low, -0.35)
    return p


def set_quad_gait(p, info, ph, amp):
    swing = sin(ph)
    g2.put(p, "torso", 0.0, 0.0, 0.04 * swing)
    g2.put(p, "head", -0.04, 0.0, 0.05 * swing)
    for i in (0, 1, 2, 3):
        g2.put(p, "leg_%d" % i, amp * swing * (1 if i % 2 == 0 else -1))
    if has(info, "tail"):
        g2.put(p, "tail", 0.0, 0.25 * sin(ph + 0.8), 0.0)
    return p


def slump(info, t, S, fall=-1.30):
    """Ragdoll-ready collapse on a pivot rig; no scale keys."""
    p = g2.blank(info)
    f = keys_at(t, [(0, 0), (0.28, -0.12), (0.72, fall), (1, fall)])
    back = 0.26 * S * keys_at(t, [(0, 0), (0.6, 1), (1, 1)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, back, -0.03 * S * abs(f))
    g2.put(p, "torso", f, 0.0, 0.08 * f, dz=-0.05 * S * abs(f))
    g2.put(p, "head", -0.25 * f)
    for side, sgn in (("l", -1), ("r", 1)):
        if has(info, "arm_" + side):
            g2.put(p, "arm_" + side, 0.55 * abs(f) - 0.10, 0.0, 0.22 * sgn)
        leg = "leg_" + side
        if has(info, leg):
            g2.put(p, leg, -back / max(0.05, info["lengths"][leg]) + 0.28 * f)
    for i in range(4):
        if has(info, "leg_%d" % i):
            g2.put(p, "leg_%d" % i, 0.45 * abs(f))
    return p


def bigger(fn, factor=1.35, travel=1.4):
    """Amplify a second-wave attack for a boss phase-2 ultimate."""
    def pose(info, t, ph, S):
        p = fn(info, t, ph, S)
        for k, v in p.items():
            if k == "root":
                p[k] = (v[0], v[1], v[2], v[3], v[4] * travel, v[5])
            else:
                p[k] = (v[0] * factor, v[1] * factor, v[2] * factor,
                        v[3], v[4], v[5] * 1.2)
        return p
    return pose


# ---------------------------------------------------------------------------
# Clip definitions
# ---------------------------------------------------------------------------

POSES = {}
CLIP_MODEL = {}


def reg(name, model, frames, loop, fn, notes, kind, skip_float=False, lock=None):
    POSES[name] = {"frames": frames, "loop": loop, "fn": fn, "notes": notes,
                   "kind": kind, "skip_float": skip_float, "lock": lock}
    CLIP_MODEL[name] = model


def idle_pose(info, t, ph, S, quadruped=False):
    p = g2.blank(info)
    g2.put(p, "torso", 0.018 * sin(ph), 0.0, 0.015 * sin(ph + 0.6),
           dz=0.006 * S * sin(2 * ph))
    g2.put(p, "head", -0.01 * sin(ph), 0.0, 0.04 * sin(ph + 1.1))
    if quadruped:
        for i in range(4):
            g2.put(p, "leg_%d" % i, 0.03 * sin(ph + i * 0.5))
        if has(info, "tail"):
            g2.put(p, "tail", 0.0, 0.15 * sin(ph))
    else:
        for side, sgn in (("l", -1), ("r", 1)):
            if has(info, "arm_" + side):
                g2.put(p, "arm_" + side, -0.15 + 0.03 * sgn * sin(ph))
            if has(info, "leg_" + side):
                g2.put(p, "leg_" + side, 0.03 * sgn * sin(ph))
    return p


def walk_pose(info, t, ph, S):
    return set_biped_gait(g2.blank(info), info, ph, 0.38, -0.02)


def run_pose(info, t, ph, S):
    return set_biped_gait(g2.blank(info), info, ph, 0.62, -0.12)


quad_walk = lambda info, t, ph, S: set_quad_gait(g2.blank(info), info, ph, 0.32)
quad_run = lambda info, t, ph, S: set_quad_gait(g2.blank(info), info, ph, 0.52)

# --- enemy locomotion / idle / death --------------------------------------

reg("slime_idle", "slime", 120, True,
    lambda info, t, ph, S: g2.put(g2.blank(info), "torso",
                                  0.05 * sin(ph), 0.0, 0.07 * sin(2 * ph),
                                  dz=0.008 * S * sin(ph)),
    "Idle slime: slow body sway and a shallow breathing bob; puddle planted.",
    "enemy", lock=True)
def slime_death(info, t, ph, S):
    f = keys_at(t, [(0, 0), (0.7, 1), (1, 1)])
    p = g2.blank(info)
    g2.put(p, "torso", -1.15 * f, 0.0, 0.12 * f, dz=-0.06 * S * f)
    g2.put(p, "root", 0.0, 0.0, 0.0, 0.0, 0.12 * S * f, 0.0)
    return p


reg("slime_death", "slime", 48, False, slime_death,
    "Slime slumps flat in place with a small backslide; laid-out finish.",
    "enemy", skip_float=True)

for model, tag, quad in (("goblin", "goblin", False), ("skeleton", "skeleton", False),
                         ("npc_dire_wolf", "wolf", True), ("archer", "archer", False),
                         ("brute", "brute", False)):
    reg(tag + "_idle", model, 120, True,
        (lambda info, t, ph, S, q=quad: idle_pose(info, t, ph, S, q)),
        "%s idle: weight shift, glance and breathing; feet planted, in place." % tag,
        "enemy", lock=True)
    reg(tag + "_walk", model, 24, True,
        (lambda info, t, ph, S, q=quad:
         (set_quad_gait(g2.blank(info), info, ph, 0.32) if q
          else set_biped_gait(g2.blank(info), info, ph, 0.38, -0.02))),
        "%s walk cycle in place: alternating leg swing and torso counter-rotation." % tag,
        "enemy")
    reg(tag + "_run", model, 18, True,
        (lambda info, t, ph, S, q=quad:
         (set_quad_gait(g2.blank(info), info, ph, 0.52) if q
          else set_biped_gait(g2.blank(info), info, ph, 0.62, -0.12))),
        "%s run cycle in place: deeper lean and wider limb drive." % tag, "enemy")

reg("spider_idle", "spider", 120, True,
    lambda info, t, ph, S: g2.put(g2.blank(info), "torso",
                                  0.02 * sin(ph), 0.0, 0.03 * sin(ph + 0.7),
                                  dz=0.008 * S * sin(2 * ph)),
    "Spider idle: small abdomen bob and leg flex; all eight tips planted.",
    "enemy", lock=True)

for name, model in (("archer_death", "archer"), ("brute_death", "brute")):
    reg(name, model, 48, False,
        lambda info, t, ph, S: slump(info, t, S),
        "%s staggers, buckles and ends in a laid-out ragdoll-ready pose." % model,
        "enemy", skip_float=True)

# --- boss phase 2 ----------------------------------------------------------


def enrage(info, t, ph, S):
    p = g2.blank(info)
    g = keys_at(t, [(0, 0), (0.40, 1), (1, 1)])
    p["root"] = (0.0, 0.0, 0.0, 0.0, 0.0, 0.02 * S * g)
    g2.put(p, "torso", -0.18 * g, 0.0, 0.06 * g)
    g2.put(p, "head", -0.20 * g)
    for side, sgn in (("l", -1), ("r", 1)):
        if has(info, "arm_" + side):
            g2.put(p, "arm_" + side, -1.30 * g, 0.0, 0.30 * sgn * g)
    for key in ("tail_1", "tail_2", "tail_3", "tentacle_l", "tentacle_r", "crown"):
        if has(info, key):
            g2.put(p, key, -0.35 * g if key.startswith("tail") else 0.0,
                   0.45 * g * (-1 if key.endswith("_l") else 1), 0.0)
    for side in ("l", "r"):
        if has(info, "leg_" + side):
            g2.put(p, "leg_" + side, 0.28 * g)
    return p


BOSSES = (
    ("scorpion_queen", g2.scorpion_attack, 42),
    ("toad_king", g2.toad_attack, 36),
    ("stormcaller", g2.storm_attack, 36),
    ("ocean_leviathan", g2.leviathan_attack, 42),
    ("colossus", g2.colossus_attack, 42),
)
for model, attack_fn, frames in BOSSES:
    reg(model + "_enrage", model, 48, False, enrage,
        "Phase-2 enrage: rear up, raise limbs/tail and hold the pose for the "
        "state transition; in place.", "boss")
    reg(model + "_ultimate", model, frames, False, bigger(attack_fn),
        "Amplified phase-2 ultimate: ~1.35x limb arcs and 1.4x root travel "
        "of the phase-1 attack.", "boss")

# --- NPC work loops --------------------------------------------------------


def villager_gather(info, t, ph, S):
    p = g2.blank(info)
    reach = 0.5 * (1 - sin(ph))
    g2.put(p, "torso", 0.32 * reach, 0.0, 0.05 * sin(ph))
    g2.put(p, "head", 0.12 * reach)
    for side in ("l", "r"):
        g2.put(p, "arm_" + side, -0.85 * reach - 0.15)
    return p


def guard_watch(info, t, ph, S):
    p = g2.blank(info)
    g2.put(p, "torso", 0.0, 0.0, 0.10 * sin(ph))
    g2.put(p, "head", -0.02, 0.0, 0.22 * sin(ph + 0.5))
    g2.put(p, "arm_l", -0.85)
    g2.put(p, "arm_r", -0.25)
    return p


def merchant_count(info, t, ph, S):
    p = g2.blank(info)
    count = 0.5 * (1 + sin(ph))
    g2.put(p, "torso", 0.14 * count)
    g2.put(p, "head", 0.16 * count)
    g2.put(p, "arm_r", -0.70 * count - 0.15)
    g2.put(p, "arm_l", -0.18)
    return p


def golem_work(info, t, ph, S):
    p = g2.blank(info)
    press = 0.5 * (1 - sin(ph))
    g2.put(p, "torso", 0.10 * press, 0.0, 0.03 * sin(ph))
    g2.put(p, "head", 0.06 * press)
    for side in ("l", "r"):
        g2.put(p, "arm_" + side, -0.60 - 0.75 * press)
    return p


reg("villager_gather", "npc_villager_child", 48, True, villager_gather,
    "Villager work loop: crouch and gather with both hands, repeat.",
    "npc", lock=True)
reg("guard_watch", "guard", 96, True, guard_watch,
    "Guard watch loop: slow scan left/right, shield held ready.",
    "npc", lock=True)
reg("merchant_count", "npc_merchant", 60, True, merchant_count,
    "Merchant work loop: count coins at the belt and nod.",
    "npc", lock=True)
reg("golem_work", "golem", 48, True, golem_work,
    "Golem work loop: both stone fists press down in a slow rhythm.",
    "npc", lock=True)

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
    records, fails_all, warns_all = [], [], []
    for name, cfg in POSES.items():
        model = CLIP_MODEL[name]
        frames, loop, fn = cfg["frames"], cfg["loop"], cfg["fn"]
        info = g2.build_rig(model)
        bpy.context.scene.render.fps = FPS
        model_info[model] = {
            "glb": "assets/models/%s.glb" % model,
            "sha256": g2.sha256(os.path.join(MODELS, model + ".glb")),
            "height_m": round(info["height"], 4),
            "scale_to_1p55u": round(info["scale"], 5),
        }
        S = info["scale"]
        pose_fn = lambda f, pf, fn=fn, S=S: fn(info, f / frames,
                                              (pf / frames) * 2 * pi, S)
        total = frames + (0 if loop else SETTLE)
        pose_fn = pose_fn if loop else g2.with_settle(pose_fn, frames)
        lock = cfg["lock"] if cfg["lock"] is not None else not loop
        action, slots = g2.author(info, name, pose_fn, total, loop, lock=lock)
        g2.bind(info, action, slots)
        lift = g2.ground_clamp(info, total + 1 if loop else total, loop)
        res = g2.measure(info, total, loop)
        fails, warns = g2.gate(res, loop, name)
        res["max_ground_lift_cm"] = round(lift * 100, 3)
        res.update({
            "clip": name, "model": model, "kind": cfg["kind"],
            "frames": total + 1 if loop else total,
            "fps": FPS, "duration_s": round((total + 1 if loop else total) / FPS, 4),
            "loop": loop,
            "root_motion": (res["root_offset_m"]["magnitude"] > 0.005 or
                            res["mean_speed_mps"] > 0.01),
            "scale_to_1p55u": round(S, 5), "contacts": info["contacts"],
            "source_sha256": model_info[model]["sha256"],
            "status": "FAIL" if fails else ("WARN" if warns else "PASS"),
            "fails": fails, "warns": warns, "notes": cfg["notes"],
            "path": "assets/animations/wave3/%s.glb" % name,
        })
        bpy.ops.export_scene.gltf(
            filepath=os.path.join(OUT, name + ".glb"), export_format="GLB",
            export_yup=True, export_animations=True,
            export_animation_mode="ACTIONS", export_nla_strips=False,
            export_extras=True, export_apply=False, use_selection=False)
        records.append(res)
        fails_all += [(name, x) for x in fails]
        warns_all += [(name, x) for x in warns]
        print("[%s] %-30s %-16s %3df seam=%.3fcm slide=%.3fcm pen=%.2fcm "
              "off=%.2fm" % (res["status"], name, model, frames,
                             res["seam_position_cm"], res["foot_slide_cm"],
                             res["ground_penetration_cm"],
                             res["root_offset_m"]["magnitude"]))

    with open(os.path.join(MODELS, "player.glb"), "rb") as fh:
        player_digest = hashlib.sha256(fh.read()).hexdigest()
    manifest = {
        "schema": "fight-animations.wave3/1",
        "generator": "scripts/generate_wave3_animations.py",
        "status": "third wave: amplified enemy locomotion/idles/deaths, boss phase 2, NPC work loops",
        "tooling": {"blender": bpy.app.version_string, "fps": FPS,
                    "pipeline": "procedural keyframes on wave2 bound-derived pivot rigs"},
        "scale_reference": {"glb": "assets/models/player.glb",
                            "sha256": player_digest, "height_m": 1.55},
        "conventions": {
            "fps": FPS, "samplers": "LINEAR", "scale_keys": False,
            "root_motion": "horizontal travel on root only; in-place cycles report 0",
            "loop": "last key duplicates frame 0 in root-relative pose",
            "pose_amplify": g2.POSE_AMPLIFY,
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
