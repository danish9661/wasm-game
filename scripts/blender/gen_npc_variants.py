"""Starfall asset expansion — family 2: NPC & creature variants.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_npc_variants.py

Ten new silhouettes spanning 0.6–2.5 u (the 1.55 u player is the reference).
All are flat-shaded, palette-only and re-based to z = 0 / (0, 0). The mini-boss
`npc_forest_warden` is the hero: bevel + weighted normals + UV pack.
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import new_scene, box, cyl, cone, ico, contact_sheet
from expansion_common import P, collection, finalize, write_records

new_scene()
RECORDS = []
SETS = []


def stick(p, name, loc, radius, height, material, vertices=5, tilt=(0, 0, 0)):
    o = cyl(name, loc, radius, height, material, p, vertices)
    o.rotation_euler = tilt
    return o


def quad(p, name, z, key_body, key_head, scale_body=(1.35, 0.80, 0.85),
         body_y=0.24, body_r=0.20, head_y=-0.34, head_z=0.34, head_r=0.15):
    ico(name + "_body", (0, body_y, z * 0.62), body_r, P[key_body], p, 0,
        scale=scale_body)
    ico(name + "_head", (0, head_y, head_z), head_r, P[key_head], p, 0,
        scale=(1.0, 1.05, 0.9))
    for i, (x, y) in enumerate(((-0.16, -0.20), (0.16, -0.20),
                                (-0.16, 0.26), (0.16, 0.26))):
        stick(p, "%s_leg_%d" % (name, i), (x, y, z * 0.30), 0.055,
              z * 0.60, P[key_body], 4,
              (0, 0, (-0.12 if x < 0 else 0.12)))


# ---------------------------------------------------------------------------
# Humanoid NPCs
# ---------------------------------------------------------------------------
def build_npc_merchant():
    p = collection("npc_merchant")
    for i, x in enumerate((-0.16, 0.16)):
        stick(p, "leg_%d" % i, (x, 0, 0.23), 0.095, 0.46, P["wood_dark"], 5)
    box("tunic", (0, 0, 0.76), (0.62, 0.40, 0.66), P["player"], p)
    box("belt", (0, 0, 0.53), (0.66, 0.43, 0.09), P["leather"] if "leather" in P else P["wood_dark"], p)
    for i, x in enumerate((-0.39, 0.39)):
        stick(p, "arm_%d" % i, (x, 0, 0.82), 0.085, 0.52, P["cloth"], 5,
              (0, 0, -0.18 if i else 0.18))
    ico("head", (0, -0.02, 1.29), 0.19, P["skin"], p, 0)
    cone("hood", (0, 0.02, 1.45), 0.27, 0.09, 0.36, P["cloth"], p, 6)
    box("pack", (0, 0.31, 0.90), (0.60, 0.28, 0.66), P["wood_dark"], p)
    box("pack_flap", (0, 0.46, 0.96), (0.48, 0.03, 0.32), P["cloth"], p)
    box("satchel", (0.43, 0.10, 0.52), (0.22, 0.18, 0.26), P["wood"], p)
    ico("coin", (0.44, -0.02, 0.66), 0.055, P["gold"], p, 0)
    return p


def build_npc_guard():
    p = collection("npc_guard")
    for i, x in enumerate((-0.17, 0.17)):
        stick(p, "greave_%d" % i, (x, 0, 0.30), 0.10, 0.60, P["iron"], 6)
        box("boot_%d" % i, (x, -0.07, 0.07), (0.20, 0.30, 0.14), P["wood_dark"], p)
    box("cuirass", (0, 0, 0.93), (0.66, 0.40, 0.72), P["steel"], p)
    ico("breastplate", (0, -0.19, 0.99), 0.24, P["iron"], p, 0,
        scale=(1.0, 0.55, 1.1))
    for i, x in enumerate((-0.40, 0.40)):
        ico("pauldron_%d" % i, (x, 0, 1.20), 0.16, P["steel"], p, 0)
        stick(p, "arm_%d" % i, (x, 0, 0.86), 0.09, 0.56, P["iron"], 5)
    ico("head", (0, -0.02, 1.47), 0.18, P["skin"], p, 0)
    cone("helm", (0, 0, 1.62), 0.22, 0.15, 0.30, P["steel"], p, 6)
    box("crest", (0, 0.02, 1.82), (0.06, 0.28, 0.20), P["enemy"], p)
    box("shield", (-0.48, -0.14, 0.92), (0.10, 0.40, 0.72), P["wood"], p)
    box("shield_boss", (-0.55, -0.14, 0.92), (0.03, 0.18, 0.18), P["gold"], p)
    stick(p, "spear", (0.52, -0.06, 0.90), 0.035, 1.75, P["wood_light"], 5)
    cone("spearhead", (0.52, -0.06, 1.82), 0.09, 0.0, 0.28, P["steel"], p, 5)
    return p


def build_npc_villager_child():
    p = collection("npc_villager_child")
    for i, x in enumerate((-0.12, 0.12)):
        stick(p, "leg_%d" % i, (x, 0, 0.15), 0.07, 0.30, P["wood_dark"], 5)
        box("shoe_%d" % i, (x, -0.05, 0.05), (0.15, 0.22, 0.10), P["leather"] if "leather" in P else P["wood_dark"], p)
    box("smock", (0, 0, 0.46), (0.46, 0.32, 0.42), P["player"], p)
    for i, x in enumerate((-0.29, 0.29)):
        stick(p, "arm_%d" % i, (x, 0, 0.50), 0.06, 0.34, P["skin"], 5,
              (0, 0, -0.12 if i else 0.12))
    ico("head", (0, -0.02, 0.81), 0.22, P["skin"], p, 0)
    ico("hair", (0, 0.05, 0.94), 0.21, P["hair"], p, 0, scale=(1.0, 1.0, 0.55))
    ico("eye_l", (-0.08, -0.19, 0.83), 0.035, P["stone_dark"], p, 0)
    ico("eye_r", (0.08, -0.19, 0.83), 0.035, P["stone_dark"], p, 0)
    box("patch", (0.24, -0.10, 0.40), (0.12, 0.04, 0.12), P["cloth"], p)
    return p


def build_npc_elder():
    p = collection("npc_elder")
    cone("robe", (0, 0, 0.49), 0.30, 0.19, 0.98, P["cloth"], p, 6)
    for i, x in enumerate((-0.24, 0.24)):
        stick(p, "arm_%d" % i, (x, 0.02, 0.92), 0.075, 0.54, P["cloth"], 5,
              (0.10, 0, -0.20 if i else 0.14))
    ico("head", (0, -0.08, 1.16), 0.18, P["skin"], p, 0)
    ico("brow", (0, -0.13, 1.28), 0.17, P["bone"], p, 0, scale=(1.0, 0.6, 0.5))
    cone("beard", (0, -0.16, 0.96), 0.15, 0.04, 0.42, P["bone"], p, 6)
    cone("hat", (0, 0.02, 1.36), 0.28, 0.08, 0.32, P["wood_dark"], p, 6)
    stick(p, "staff", (0.40, -0.04, 0.82), 0.038, 1.64, P["wood_light"], 5)
    ico("staff_orb", (0.40, -0.04, 1.70), 0.12, P["arcane"], p, 0)
    box("satchel", (-0.34, 0.14, 0.66), (0.18, 0.16, 0.22), P["wood"], p)
    return p


# ---------------------------------------------------------------------------
# Creatures
# ---------------------------------------------------------------------------
def build_npc_wolf_pup():
    p = collection("npc_wolf_pup")
    quad(p, "pup", 0.34, "wood_light", "bone", body_r=0.19,
         scale_body=(1.35, 0.78, 0.85), head_y=-0.32, head_z=0.36, head_r=0.15)
    for i, x in enumerate((-0.09, 0.09)):
        cone("ear_%d" % i, (x, -0.38, 0.54), 0.065, 0.0, 0.20,
             P["wood_dark"], p, 4)
    ico("nose", (0, -0.47, 0.32), 0.05, P["stone_dark"], p, 0)
    tail = cone("tail", (0, 0.40, 0.45), 0.08, 0.0, 0.34, P["wood"], p, 5)
    tail.rotation_euler = (0.9, 0, 0)
    return p


def build_npc_dire_wolf():
    p = collection("npc_dire_wolf")
    quad(p, "dire", 0.68, "wood", "wood_light", body_r=0.32,
         scale_body=(1.55, 0.85, 0.90), head_y=-0.52, head_z=0.82, head_r=0.24)
    box("snout", (0, -0.72, 0.72), (0.22, 0.32, 0.18), P["bone"], p)
    for i, x in enumerate((-0.15, 0.15)):
        cone("ear_%d" % i, (x, -0.52, 1.08), 0.09, 0.0, 0.26,
             P["wood_dark"], p, 5)
        cone("fang_%d" % i, (x, -0.84, 0.63), 0.045, 0.0, 0.14,
             P["bone"], p, 4)
    for i, (x, z) in enumerate(((-0.16, 0.98), (0, 1.04), (0.16, 0.98))):
        ico("mane_%d" % i, (x, -0.28, z), 0.15, P["leaf_dark"], p, 0,
            scale=(1.0, 0.7, 1.2))
    tail = cone("tail", (0, 0.66, 0.88), 0.13, 0.0, 0.56, P["wood"], p, 5)
    tail.rotation_euler = (0.85, 0, 0)
    return p


def build_npc_swamp_toad():
    p = collection("npc_swamp_toad")
    ico("body", (0, 0.04, 0.32), 0.36, P["swamp"], p, 0,
        scale=(1.25, 1.05, 0.80))
    ico("throat", (0, -0.27, 0.20), 0.24, P["swamp_dark"], p, 0,
        scale=(1.1, 0.7, 0.55))
    for i, x in enumerate((-0.22, 0.22)):
        ico("hind_leg_%d" % i, (x, 0.28, 0.14), 0.15, P["swamp_dark"], p, 0,
            scale=(1.0, 1.3, 0.7))
        stick(p, "front_leg_%d" % i, (x * 0.8, -0.30, 0.10), 0.065, 0.20,
              P["swamp"], 4)
        ico("eye_%d" % i, (x * 0.72, -0.25, 0.58), 0.13, P["swamp"], p, 0)
        ico("pupil_%d" % i, (x * 0.72, -0.35, 0.59), 0.07, P["gold"], p, 0)
    box("mouth", (0, -0.34, 0.16), (0.42, 0.04, 0.05), P["wood_dark"], p)
    for i, (x, y) in enumerate(((-0.28, -0.05), (0.26, 0.02), (0, 0.24))):
        ico("wart_%d" % i, (x, y, 0.55), 0.07, P["leaf_light"], p, 0)
    return p


def build_npc_ice_elemental():
    p = collection("npc_ice_elemental")
    cone("lower_shard", (0, 0, 0.52), 0.30, 0.0, 1.04, P["ice"], p, 6)
    ico("torso", (0, 0, 1.20), 0.38, P["ice"], p, 0, scale=(1.05, 0.85, 1.1))
    cone("head_spire", (0, -0.04, 1.74), 0.20, 0.0, 0.62, P["ice"], p, 5)
    for i, side in enumerate((-1, 1)):
        arm = cone("arm_%d" % i, (side * 0.47, 0, 1.30), 0.14, 0.02, 0.78,
                   P["ice"], p, 5)
        arm.rotation_euler = (0, 0, side * 0.72)
        ico("shoulder_%d" % i, (side * 0.31, 0, 1.42), 0.16, P["snow"], p, 0)
    ico("core", (0, -0.28, 1.22), 0.18, P["arcane"], p, 0)
    for i, (x, y, z) in enumerate(((-0.52, 0.30, 0.80), (0.50, 0.28, 1.70),
                                   (0.12, 0.40, 1.05))):
        shard = cone("orbit_%d" % i, (x, y, z), 0.07, 0.0, 0.34,
                     P["arcane_light"], p, 4)
        shard.rotation_euler = (0.2, 0, 0.4)
    return p


def build_npc_desert_scorpion():
    p = collection("npc_desert_scorpion")
    ico("carapace", (0, 0.06, 0.32), 0.33, P["sand_dark"], p, 0,
        scale=(1.15, 1.45, 0.62))
    ico("head", (0, -0.40, 0.28), 0.20, P["sand"], p, 0,
        scale=(1.1, 0.9, 0.7))
    for i in range(4):
        for side in (-1, 1):
            y = -0.22 + i * 0.20
            leg = cyl("leg_%d_%d" % (i, side), (side * 0.38, y, 0.16),
                      0.032, 0.42, P["sand_dark"], p, 4)
            leg.rotation_euler = (0, 0, side * 0.92)
    for side in (-1, 1):
        stick(p, "claw_arm_%d" % side, (side * 0.34, -0.56, 0.24), 0.055,
              0.30, P["sand"], 4, (0, 0, side * 0.45))
        box("pincer_%d" % side, (side * 0.48, -0.74, 0.26),
            (0.22, 0.24, 0.12), P["sand"], p)
        cone("pincer_tip_%d" % side, (side * 0.48, -0.90, 0.27), 0.06, 0.0,
             0.18, P["stone_light"], p, 4)
    for i, (y, z) in enumerate(((-0.14, 0.70), (0.06, 0.84), (0.26, 0.92))):
        ico("tail_seg_%d" % i, (0, y, z), 0.13 - i * 0.02, P["sand_dark"], p, 0)
    cone("sting", (0, 0.40, 0.94), 0.08, 0.0, 0.30, P["bone"], p, 5,
         rotation=None)
    return p


def build_npc_forest_warden():
    """Hero mini-boss: branching antlers, a front-facing leaf cloak and staff.

    2.45 u overall (antler tips) with a ~2.2 u body; both are recorded in the
    brief so the silhouette is not mistaken for a 1.58x-scale humanoid.
    """
    p = collection("npc_forest_warden")
    for i, x in enumerate((-0.18, 0.18)):
        stick(p, "leg_%d" % i, (x, 0, 0.425), 0.11, 0.85, P["wood_dark"], 6)
    box("torso", (0, 0, 1.28), (0.72, 0.42, 0.92), P["leaf_dark"], p)
    ico("chest_rune", (0, -0.22, 1.34), 0.20, P["arcane"], p, 0,
        scale=(1.0, 0.45, 1.0))
    for i, x in enumerate((-0.46, 0.46)):
        ico("pauldron_%d" % i, (x, 0, 1.60), 0.19, P["leaf"], p, 0)
        stick(p, "arm_%d" % i, (x, 0, 1.18), 0.10, 0.64, P["wood"], 5)
    box("cloak", (0, 0.30, 1.02), (0.84, 0.10, 1.30), P["leaf_dark"], p)
    # Faceted lobes overlap the chest, so the cloak reads from the eye view.
    for i, (x, y, z) in enumerate(((-0.28, -0.27, 1.36),
                                   (0.28, -0.27, 1.36))):
        ico("cloak_lobe_%d" % i, (x, y, z), 0.22,
            P["leaf" if i != 1 else "leaf_light"], p, 0,
            scale=(1.0, 0.35, 0.75))
    ico("head", (0, -0.03, 1.98), 0.22, P["skin"], p, 0)
    box("mask", (0, -0.21, 1.94), (0.30, 0.06, 0.22), P["wood"], p)
    for side in (-1, 1):
        # A beam that leans outward, with a tine branching from its tip: the
        # visible crotch is what makes this read as an antler, not a peg.
        stick(p, "antler_%d" % side, (side * 0.11, 0, 2.02), 0.06, 0.60,
              P["wood_light"], 5, (0, side * 0.75, 0))
        ico("antler_crotch_%d" % side, (side * 0.31, 0, 2.24), 0.06,
            P["wood_light"], p, 0)
        stick(p, "antler_tine_%d" % side, (side * 0.31, 0, 2.24), 0.035,
              0.28, P["wood_light"], 4, (0, side * 1.25, 0))
        stick(p, "antler_branch_%d" % side, (side * 0.20, 0, 2.12), 0.03,
              0.22, P["wood_light"], 4, (0, -side * 0.90, 0))
    stick(p, "staff", (0.56, -0.06, 0.98), 0.045, 1.95, P["wood"], 6)
    ico("staff_rune", (0.56, -0.06, 2.02), 0.10, P["arcane"], p, 0)
    cone("staff_tip", (0.56, -0.06, 2.13), 0.09, 0.0, 0.20,
         P["arcane_light"], p, 5)
    return p


ASSETS = [
    ("npc_merchant",         build_npc_merchant,         (0.95, 0.85, 1.65), "market vendor", "wide pack silhouette"),
    ("npc_guard",            build_npc_guard,            (1.05, 0.65, 1.95), "town watch", "shield + spear"),
    ("npc_villager_child",   build_npc_villager_child,   (0.70, 0.50, 1.05), "village child", "oversized head"),
    ("npc_elder",            build_npc_elder,            (0.90, 0.70, 1.82), "village elder", "hunched + staff"),
    ("npc_wolf_pup",         build_npc_wolf_pup,         (0.45, 0.80, 0.62), "forest", "small quadruped"),
    ("npc_dire_wolf",        build_npc_dire_wolf,        (0.75, 1.25, 1.32), "forest", "maned predator"),
    ("npc_swamp_toad",       build_npc_swamp_toad,       (0.90, 0.80, 0.78), "swamp", "squat broad body"),
    ("npc_ice_elemental",    build_npc_ice_elemental,    (1.10, 0.95, 2.20), "tundra", "floating crystalline"),
    ("npc_desert_scorpion",  build_npc_desert_scorpion,  (1.10, 1.45, 1.12), "desert", "low + raised sting"),
    ("npc_forest_warden",    build_npc_forest_warden,    (1.28, 0.69, 2.32), "forest", "HERO mini-boss"),
]

for name, builder, target, role, notes in ASSETS:
    col = builder()
    finalize(name, col, "npc_variants", target, "%s; %s" % (role, notes),
             hero=(name == "npc_forest_warden"), bevel_width=0.006,
             bevel_parts=("torso", "staff", "pauldron", "mask") if name == "npc_forest_warden" else None,
             records=RECORDS)
    SETS.append((name, col))

write_records("npc_variants", RECORDS)
try:
    contact_sheet(SETS, "npc_variants.png", width=1900, height=360)
except Exception as exc:
    print("PREVIEW SKIPPED", exc)
