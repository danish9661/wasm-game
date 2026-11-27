"""Rebuild the worst character-family models with the game-asset-modeling workflow.

    ~/.local/blender/blender -b --python scripts/blender/rebuild_characters.py -- MODEL [MODEL...]

Every part is authored with bmesh primitives, selectively bevelled (weighted
edges), finished with weighted normals, UV-packed and exported flat-shaded.
Filenames and the base-at-zero / centred-pivot contract are preserved.

For models with pre-existing animation rigs (spider, scorpion_queen, colossus,
npc_merchant) the original mesh NODE NAMES are kept exactly: wave2/3/4 bind to
those names, so detail is merged into the existing parts instead of added as
new nodes.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from char_rebuild_lib import Builder, export, new_palette, normalise  # noqa: E402

P = math.pi if False else __import__("math").pi  # keep import single-line friendly

ALL = ["wolf", "wraith", "bat", "scorpion_queen", "colossus", "spider",
       "npc_desert_scorpion", "npc_merchant", "sea_turtle", "npc_wolf_pup",
       "scarecrow", "npc_farmer", "npc_fisher", "npc_bard", "npc_sailor",
       "npc_traveling_scholar", "npc_innkeeper"]


# ---------------------------------------------------------------------------
# Unbound models: one merged mesh, full freedom of form.
# ---------------------------------------------------------------------------
def wolf(b):
    b.cyl((0, 0.02, 0.62), 0.28, -0.34, 0.34, "wood", n=8, r1=0.24,
          axis=(0, 1, 0), bevel=True)
    b.ico((0, -0.26, 0.64), 0.27, "wood", (1.0, 0.85, 1.05))
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = sx * 0.18, sy * 0.26
            b.limb((x, y, 0.48), (x + sx * 0.02, y, 0.11), 0.095, 0.075, "wood", 6)
            b.box_at((x + sx * 0.02, y - 0.03, 0.055), (0.14, 0.20, 0.11), "leather")
    b.limb((0, -0.34, 0.72), (0, -0.52, 0.88), 0.17, 0.14, "wood", 6)
    b.ico((0, -0.60, 0.93), 0.22, "wood", (1.0, 1.0, 0.95), bevel=False)
    b.limb((0, -0.72, 0.87), (0, -0.94, 0.81), 0.12, 0.08, "sand", 6)
    b.ico((0, -0.96, 0.82), 0.06, "leather")
    for sx in (-1, 1):
        b.cyl((sx * 0.13, -0.58, 1.05), 0.10, 0.0, 0.24, "wood", n=4, r1=0.006)
        b.ico((sx * 0.10, -0.77, 0.97), 0.04, "bone")
    b.limb((0, 0.34, 0.72), (0, 0.66, 0.92), 0.11, 0.035, "wood", 6)


def wraith(b):
    b.cyl((0, 0, 0.66), 0.34, 0, 1.28, "arcane", n=8, r1=0.16, bevel=True)
    for i in range(4):
        a = P / 4 + i * P / 2
        x, y = 0.30 * __import__("math").cos(a), 0.30 * __import__("math").sin(a)
        b.cyl((x, y, 0.16), 0.09, 0.0, 0.26, "arcane", n=4, r1=0.006)
    b.ico((0, 0, 1.20), 0.27, "arcane", (1.35, 0.72, 0.72))
    for sx in (-1, 1):
        b.limb((sx * 0.28, 0, 1.22), (sx * 0.46, -0.10, 0.84), 0.10, 0.05, "cloth", 6)
        b.ico((sx * 0.48, -0.12, 0.80), 0.075, "bone")
        b.cyl((sx * 0.50, 0.02, 0.92), 0.08, 0.0, 0.24, "arcane", n=4, r1=0.006)
    b.cyl((0, 0.02, 1.46), 0.26, 0, 0.32, "arcane", n=6, r1=0.10, bevel=True)
    b.ico((0, -0.14, 1.46), 0.19, "bone", (1.0, 0.62, 1.0))
    for sx in (-1, 1):
        b.box_at((sx * 0.085, -0.255, 1.49), (0.075, 0.025, 0.085), "eye")


def bat(b):
    b.ico((0, 0, 0.52), 0.20, "arcane", (0.82, 0.92, 1.22))
    b.ico((0, -0.08, 0.76), 0.17, "arcane", bevel=True)
    for sx in (-1, 1):
        b.cyl((sx * 0.10, -0.06, 0.94), 0.075, 0.0, 0.22, "arcane", n=4, r1=0.006)
        b.ico((sx * 0.075, -0.21, 0.79), 0.038, "bone")
        poly = [(sx * 0.13, 0.06), (sx * 0.44, 0.13), (sx * 0.36, -0.02),
                (sx * 0.47, -0.20), (sx * 0.18, -0.11)]
        b.prism(poly, (0, 0, 0.035), "leather")
        b.box_at((sx * 0.28, 0.0, 0.63), (0.31, 0.025, 0.03), "arcane")
        b.box_at((sx * 0.07, 0.03, 0.06), (0.07, 0.10, 0.12), "leather")
    b.cyl((0, 0.16, 0.54), 0.06, 0.0, 0.20, "arcane", n=4, r1=0.006)


def npc_desert_scorpion(b):
    b.ico((0, 0.12, 0.56), 0.38, "sand", (1.25, 1.0, 0.80), bevel=True)
    b.ico((0, -0.34, 0.52), 0.23, "sand_dark")
    for sx in (-1, 1):
        b.ico((sx * 0.12, -0.50, 0.59), 0.045, "fire")
        b.limb((sx * 0.30, -0.36, 0.52), (sx * 0.62, -0.50, 0.48),
               0.08, 0.06, "sand_dark", 6)
        b.cyl((sx * 0.72, -0.58, 0.58), 0.10, 0, 0.30, "steel", n=4, r1=0.006)
        b.cyl((sx * 0.72, -0.58, 0.42), 0.10, 0, 0.30, "steel", n=4, r1=0.006)
    for sx in (-1, 1):
        for i, y in enumerate((-0.20, 0.10, 0.40)):
            b.limb((sx * 0.34, y, 0.48), (sx * 0.72, y + 0.04, 0.07),
                   0.055, 0.035, "sand_dark", 4)
    b.limb((0, 0.44, 0.72), (0, 0.70, 1.08), 0.16, 0.13, "sand", 6)
    b.limb((0, 0.70, 1.08), (0, 0.44, 1.32), 0.13, 0.10, "sand", 6)
    b.limb((0, 0.44, 1.32), (0, -0.04, 1.36), 0.10, 0.06, "sand_dark", 6)
    b.cyl((0, -0.10, 1.36), 0.075, 0, 0.24, "fire", n=4, r1=0.006)


def sea_turtle(b):
    b.cyl((0, 0, 0.24), 0.43, 0.0, 0.10, "wood", n=8, bevel=True)
    b.ico((0, 0, 0.45), 0.38, "leaf", (1.18, 1.05, 0.78), bevel=True)
    for x, y in ((0, 0), (-0.20, -0.12), (0.20, -0.12), (-0.16, 0.16), (0.16, 0.16)):
        b.box_at((x, y, 0.68), (0.15, 0.15, 0.035), "leaf_dark")
    b.ico((0, -0.48, 0.40), 0.19, "leaf", (1.0, 1.2, 0.9))
    for sx in (-1, 1):
        b.ico((sx * 0.085, -0.60, 0.46), 0.04, "bone")
        b.limb((sx * 0.26, -0.26, 0.28), (sx * 0.52, -0.42, 0.10),
               0.10, 0.045, "leaf_dark", 6)
        b.limb((sx * 0.26, 0.30, 0.28), (sx * 0.50, 0.46, 0.10),
               0.10, 0.045, "leaf_dark", 6)
    b.cyl((0, 0.46, 0.32), 0.07, 0, 0.16, "leaf_dark", n=4, r1=0.006)


def npc_wolf_pup(b):
    b.ico((0, 0.06, 0.33), 0.23, "wood", (1.38, 0.92, 0.92))
    b.ico((0, -0.28, 0.55), 0.25, "wood", bevel=True)
    b.ico((0, -0.49, 0.49), 0.13, "sand")
    b.ico((0, -0.58, 0.51), 0.055, "leather")
    for sx in (-1, 1):
        b.cyl((sx * 0.15, -0.28, 0.78), 0.10, 0, 0.20, "wood", n=4, r1=0.006)
        b.ico((sx * 0.11, -0.46, 0.58), 0.04, "bone")
        for y in (-0.12, 0.24):
            b.limb((sx * 0.14, y, 0.30), (sx * 0.16, y, 0.07), 0.06, 0.045, "wood", 4)
    b.limb((0, 0.24, 0.42), (0, 0.50, 0.58), 0.08, 0.03, "wood", 4)
    b.ico((0, 0.52, 0.59), 0.055, "sand")


# ---------------------------------------------------------------------------
# Animation-bound models: keep every existing mesh node name exactly.
# ---------------------------------------------------------------------------
def spider(parts):
    p = lambda n: parts.setdefault(n, Builder(*CTX))
    p("thorax").ico((0, -0.22, 0.50), 0.20, "stone", bevel=True)
    p("abdomen").ico((0, 0.24, 0.60), 0.33, "leather", (1, 1.2, 0.85), bevel=True)
    for n, sx in (("eye_l.006", -1), ("eye_r.006", 1)):
        p(n).ico((sx * 0.09, -0.38, 0.57), 0.045, "bone")
    for n, sx in (("fang_1", -1), ("fang_2", 1)):
        p(n).cyl((sx * 0.07, -0.38, 0.40), 0.05, 0, 0.14, "bone", n=4, r1=0.006)
    for side, s in (("l", -1), ("r", 1)):
        for i, y in enumerate((-0.30, -0.08, 0.14, 0.36)):
            up, low = "upper_%s%d" % (side, i), "lower_%s%d" % (side, i)
            p(up).limb((s * 0.18, y, 0.52), (s * 0.40, y, 0.46), 0.045, 0.035, "stone", 4)
            p(low).limb((s * 0.40, y, 0.46), (s * 0.64, y + (0.05 if i > 1 else -0.05), 0.05),
                        0.035, 0.02, "stone", 4)


def scorpion_queen(parts):
    p = lambda n: parts.setdefault(n, Builder(*CTX))
    body = p("body.004")
    body.ico((0, 0.20, 0.76), 0.46, "arcane", (1.32, 1.0, 0.82), bevel=True)
    body.box_at((0, 0.02, 1.00), (0.42, 0.34, 0.08), "gold")
    for sx in (-1, 1):
        p("crown_l" if sx < 0 else "crown_r").cyl(
            (sx * 0.14, -0.42, 0.94), 0.07, 0, 0.26, "gold", n=4, r1=0.006)
    p("head.005").ico((0, -0.44, 0.72), 0.26, "enemy", bevel=True)
    for n, sx in (("eye_l.010", -1), ("eye_r.010", 1)):
        p(n).box_at((sx * 0.12, -0.65, 0.78), (0.07, 0.03, 0.07), "fire")
    for side, s in (("l", -1), ("r", 1)):
        p("arm_" + side + ".004").limb((s * 0.40, -0.40, 0.62),
                                       (s * 0.74, -0.62, 0.56), 0.10, 0.07, "stone", 6)
        p("pincer_%s_up" % side).cyl((s * 0.86, -0.70, 0.66), 0.12, 0, 0.34,
                                     "steel", n=4, r1=0.006)
        p("pincer_%s_lo" % side).cyl((s * 0.86, -0.70, 0.48), 0.12, 0, 0.34,
                                     "steel", n=4, r1=0.006)
    p("tail_1").limb((0, 0.56, 0.88), (0, 0.80, 1.45), 0.18, 0.15, "arcane", 6)
    p("tail_2").limb((0, 0.80, 1.45), (0, 0.44, 1.90), 0.15, 0.11, "arcane", 6)
    p("tail_3").limb((0, 0.44, 1.90), (0, -0.04, 2.02), 0.11, 0.07, "enemy", 6)
    p("stinger").cyl((0, -0.08, 2.02), 0.09, 0, 0.24, "fire", n=4, r1=0.006)
    for side, s in (("l", -1), ("r", 1)):
        for i, y in enumerate((-0.24, 0.12, 0.48)):
            p("leg_%s%d" % (side, i)).limb((s * 0.42, y, 0.60),
                                           (s * 0.82, y + 0.04, 0.06),
                                           0.07, 0.035, "stone", 4)


def colossus(parts):
    p = lambda n: parts.setdefault(n, Builder(*CTX))
    p("pelvis.001").ico((0, 0, 1.26), 0.42, "stone", (1.25, 0.75, 0.55))
    torso = p("torso.004")
    torso.cyl((0, 0, 1.82), 0.50, 0, 0.76, "stone", n=8, r1=0.61, bevel=True)
    for n, sx in (("crack_l", -1), ("crack_r", 1)):
        p(n).box_at((sx * 0.22, -0.50, 1.88), (0.05, 0.03, 0.42), "fire")
    head = p("head.008")
    head.ico((0, -0.02, 2.50), 0.31, "stone", (1.12, 0.92, 1.0), bevel=True)
    head.box_at((0, -0.30, 2.54), (0.44, 0.05, 0.12), "charcoal")
    for n, sx in (("eye_l.014", -1), ("eye_r.014", 1)):
        p(n).box_at((sx * 0.13, -0.34, 2.56), (0.09, 0.03, 0.08), "fire")
    for i, x in enumerate((-0.18, 0.0, 0.18)):
        p("crown_%d" % i).cyl((x, -0.02, 2.76), 0.075, 0, 0.26, "gold", n=4, r1=0.006)
    for side, s in (("l", -1), ("r", 1)):
        sh = p("shoulder_" + side + ".002")
        sh.ico((s * 0.72, 0, 2.20), 0.35, "iron", bevel=True)
        sh.cyl((s * 0.80, 0, 2.42), 0.13, 0, 0.26, "steel", n=4, r1=0.006)
        p("arm_" + side + ".006").limb((s * 0.80, 0, 2.18),
                                       (s * 0.88, 0, 1.36), 0.24, 0.19, "stone", 6)
    p("fist_l.001").ico((-0.88, -0.04, 1.22), 0.27, "stone")
    maul = p("fist_r.001")
    maul.ico((0.88, -0.04, 1.22), 0.27, "stone")
    maul.box_at((0.94, -0.30, 1.22), (0.48, 0.30, 0.32), "iron", bevel=True)
    maul.box_at((0.92, -0.18, 1.22), (0.09, 0.34, 0.09), "wood")
    for side, s in (("l", -1), ("r", 1)):
        p("leg_" + side + ".006").limb((s * 0.34, 0, 1.20),
                                       (s * 0.34, 0, 0.28), 0.26, 0.20, "stone", 6)
        p("foot_" + side + ".004").box_at((s * 0.34, -0.06, 0.14), (0.44, 0.58, 0.28), "stone")


def npc_merchant(parts):
    p = lambda n: parts.setdefault(n, Builder(*CTX))
    p("tunic").cyl((0, 0, 0.92), 0.29, 0, 0.56, "cloth", n=8, r1=0.34, bevel=True)
    p("belt").cyl((0, 0, 0.72), 0.32, 0, 0.08, "leather", n=8)
    pack = p("pack")
    pack.ico((0.32, 0.34, 1.32), 0.30, "wood", (1.0, 0.85, 1.0), bevel=True)
    pack.box_at((0.32, 0.60, 1.32), (0.42, 0.04, 0.42), "leather")
    pack.box_at((0.10, 0.34, 1.32), (0.04, 0.32, 0.42), "leather")
    pack.box_at((0.54, 0.34, 1.32), (0.04, 0.32, 0.42), "leather")
    p("pack_flap").box_at((0.32, 0.62, 1.50), (0.48, 0.06, 0.30), "leather")
    satchel = p("satchel")
    satchel.ico((-0.42, 0.02, 0.66), 0.21, "leather", (1.0, 0.8, 1.1))
    satchel.box_at((-0.22, -0.02, 0.94), (0.06, 0.05, 0.58), "leather")
    p("coin").ico((-0.44, -0.20, 0.76), 0.14, "gold")
    head = p("head")
    head.ico((0, -0.02, 1.42), 0.22, "skin", bevel=True)
    for sx in (-1, 1):
        head.box_at((sx * 0.085, -0.21, 1.46), (0.05, 0.02, 0.055), "eye")
    head.ico((0, -0.23, 1.38), 0.05, "skin")
    head.box_at((0, -0.22, 1.30), (0.09, 0.02, 0.025), "leather")
    p("hood").cyl((0, 0.06, 1.62), 0.28, 0, 0.26, "sand_dark", n=6, r1=0.10)
    for n, sx in (("arm_0", -1), ("arm_1", 1)):
        arm = p(n)
        arm.limb((sx * 0.30, 0, 1.12), (sx * 0.37, -0.06, 0.76), 0.11, 0.08, "cloth", 6)
        arm.ico((sx * 0.38, -0.08, 0.72), 0.075, "skin")
    for n, sx in (("leg_0", -1), ("leg_1", 1)):
        leg = p(n)
        leg.limb((sx * 0.13, 0, 0.68), (sx * 0.13, 0, 0.13), 0.10, 0.08, "trousers", 6)
        leg.box_at((sx * 0.13, -0.04, 0.065), (0.15, 0.22, 0.13), "leather")


def scarecrow(b):
    b.cyl((0, 0, 0.60), 0.07, 0, 1.20, "wood", n=6)
    b.cyl((0, 0, 1.20), 0.055, -0.46, 0.46, "wood", n=6, axis=(1, 0, 0))
    b.cyl((0, 0, 1.00), 0.30, 0, 0.60, "cloth", n=8, r1=0.24, bevel=True)
    for i in range(4):
        a = i * P / 2 + P / 4
        x, y = 0.26 * __import__("math").cos(a), 0.26 * __import__("math").sin(a)
        b.cyl((x, y, 0.72), 0.07, 0, 0.20, "cloth", n=4, r1=0.006)
    b.ico((0, 0, 1.48), 0.19, "sand", bevel=True)
    for sx in (-1, 1):
        b.box_at((sx * 0.085, -0.17, 1.51), (0.06, 0.025, 0.07), "eye")
    b.box_at((0, -0.17, 1.38), (0.13, 0.02, 0.025), "leather")
    b.cyl((0, 0, 1.66), 0.28, 0, 0.04, "sand_dark", n=6)
    b.cyl((0, 0, 1.70), 0.20, 0, 0.24, "sand_dark", n=6, r1=0.06)
    for sx in (-1, 1):
        b.limb((sx * 0.22, 0, 1.24), (sx * 0.52, 0.02, 1.10), 0.09, 0.06, "cloth", 6)
        for j in range(2):
            b.cyl((sx * (0.50 + j * 0.05), 0.02, 1.10), 0.04, 0, 0.18,
                  "sand", n=4, r1=0.006)
    for j in range(3):
        b.cyl((0, 0, 1.58 + j * 0.02), 0.05, 0, 0.20 + j * 0.04,
              "sand", n=4, r1=0.006)
    b.limb((0.50, 0.04, 0.10), (0.64, 0.04, 1.86), 0.055, 0.035, "wood", 6)


def npc_fisher(b):
    b.cyl((0, 0, 0.92), 0.28, 0, 0.62, "cloth", n=8, r1=0.34, bevel=True)
    b.cyl((0, 0, 0.70), 0.31, 0, 0.08, "leather", n=8)
    b.ico((0, -0.02, 1.42), 0.22, "skin", bevel=True)
    for sx in (-1, 1):
        b.box_at((sx * 0.085, -0.21, 1.46), (0.05, 0.02, 0.055), "eye")
    b.cyl((0, 0, 1.60), 0.30, 0, 0.04, "sand", n=8)
    b.cyl((0, 0, 1.64), 0.18, 0, 0.22, "sand", n=6, r1=0.14)
    b.limb((0.40, -0.06, 0.84), (0.68, -0.10, 1.84), 0.045, 0.025, "wood", 6)
    b.ico((0.70, -0.10, 1.90), 0.13, "fire")
    b.cyl((0.70, -0.10, 1.90), 0.145, 0, 0.05, "snow", n=6)
    b.cyl((-0.44, 0.02, 0.24), 0.17, 0, 0.32, "wood", n=6)
    for sx in (-1, 1):
        b.limb((sx * 0.30, 0, 1.12), (sx * 0.38, -0.06, 0.76), 0.11, 0.08, "cloth", 6)
        b.ico((sx * 0.39, -0.08, 0.72), 0.075, "skin")
        b.limb((sx * 0.13, 0, 0.66), (sx * 0.13, 0, 0.13), 0.10, 0.08, "trousers", 6)
        b.box_at((sx * 0.13, -0.04, 0.065), (0.15, 0.22, 0.13), "leather")


def npc_bard(b):
    b.cyl((0, 0, 0.92), 0.28, 0, 0.58, "cloth", n=8, r1=0.34, bevel=True)
    b.box_at((0, 0.26, 1.00), (0.50, 0.06, 0.68), "enemy")
    b.box_at((0, 0, 0.70), (0.48, 0.36, 0.09), "gold")
    b.ico((0, -0.02, 1.42), 0.22, "skin", bevel=True)
    for sx in (-1, 1):
        b.box_at((sx * 0.085, -0.21, 1.46), (0.05, 0.02, 0.055), "eye")
    b.ico((0, -0.23, 1.38), 0.05, "skin")
    b.cyl((0, 0.02, 1.62), 0.28, 0, 0.05, "wood", n=6)
    b.cyl((0.14, 0.04, 1.74), 0.075, 0, 0.24, "player", n=4, r1=0.006)
    b.ico((-0.30, -0.26, 1.04), 0.20, "wood", (1.0, 0.65, 1.2))
    b.limb((-0.30, -0.26, 1.12), (-0.02, -0.30, 1.66), 0.045, 0.028, "wood", 6)
    for dx in (-0.08, 0.0, 0.08):
        b.box_at((-0.30 + dx, -0.38, 1.04), (0.012, 0.012, 0.46), "bone")
    for sx in (-1, 1):
        b.limb((sx * 0.30, 0, 1.12), (sx * 0.38, -0.06, 0.76), 0.11, 0.08, "cloth", 6)
        b.ico((sx * 0.39, -0.08, 0.72), 0.075, "skin")
        b.limb((sx * 0.13, 0, 0.66), (sx * 0.13, 0, 0.13), 0.10, 0.08, "trousers", 6)
        b.box_at((sx * 0.13, -0.04, 0.065), (0.15, 0.22, 0.13), "leather")


def npc_sailor(b):
    b.cyl((0, 0, 0.92), 0.29, 0, 0.60, "cloth", n=8, r1=0.34, bevel=True)
    b.box_at((0, -0.21, 1.00), (0.50, 0.04, 0.50), "bone")
    b.box_at((0, 0, 0.70), (0.48, 0.36, 0.09), "leather")
    b.ico((0, -0.02, 1.42), 0.22, "skin", bevel=True)
    for sx in (-1, 1):
        b.box_at((sx * 0.085, -0.21, 1.46), (0.05, 0.02, 0.055), "eye")
    b.ico((0, -0.23, 1.38), 0.05, "skin")
    b.cyl((0, 0.02, 1.60), 0.22, 0, 0.10, "enemy", n=6)
    b.cyl((0.22, 0.10, 1.56), 0.05, 0, 0.24, "enemy", n=4, r1=0.006)
    b.limb((0.42, -0.10, 0.72), (0.58, -0.22, 1.48), 0.035, 0.025, "steel", 6)
    b.box_at((0.48, -0.18, 0.76), (0.22, 0.05, 0.05), "gold")
    b.box_at((0.42, -0.10, 0.62), (0.06, 0.06, 0.16), "wood")
    b.ico((-0.42, 0.10, 0.72), 0.18, "sand")
    for sx in (-1, 1):
        b.limb((sx * 0.30, 0, 1.12), (sx * 0.38, -0.06, 0.76), 0.11, 0.08, "cloth", 6)
        b.ico((sx * 0.39, -0.08, 0.72), 0.075, "skin")
        b.limb((sx * 0.13, 0, 0.66), (sx * 0.13, 0, 0.13), 0.10, 0.08, "trousers", 6)
        b.box_at((sx * 0.13, -0.04, 0.065), (0.15, 0.22, 0.13), "leather")


def npc_traveling_scholar(b):
    b.cyl((0, 0, 0.92), 0.29, 0, 0.62, "cloth", n=8, r1=0.34, bevel=True)
    b.box_at((0, -0.20, 1.00), (0.42, 0.04, 0.50), "bone")
    b.ico((0, -0.02, 1.42), 0.22, "skin", bevel=True)
    for sx in (-1, 1):
        b.box_at((sx * 0.085, -0.21, 1.48), (0.055, 0.025, 0.055), "eye")
        b.box_at((sx * 0.085, -0.205, 1.56), (0.09, 0.02, 0.03), "hair")
    b.ico((0, -0.23, 1.38), 0.05, "skin")
    b.cyl((0, 0.02, 1.64), 0.26, 0, 0.24, "cloth", n=6, r1=0.11)
    for sx in (-1, 1):
        b.box_at((sx * 0.12, -0.32, 1.00), (0.20, 0.04, 0.28), "bone")
    b.box_at((0, -0.33, 1.00), (0.045, 0.06, 0.30), "enemy")
    b.ico((-0.40, 0.12, 0.72), 0.18, "wood")
    for dx in (-0.06, 0.06):
        b.cyl((-0.40 + dx, 0.16, 1.02), 0.06, 0, 0.34, "bone", n=6)
    for sx in (-1, 1):
        b.limb((sx * 0.30, 0, 1.12), (sx * 0.38, -0.06, 0.76), 0.11, 0.08, "cloth", 6)
        b.ico((sx * 0.39, -0.08, 0.72), 0.075, "skin")
        b.limb((sx * 0.13, 0, 0.66), (sx * 0.13, 0, 0.13), 0.10, 0.08, "trousers", 6)
        b.box_at((sx * 0.13, -0.04, 0.065), (0.15, 0.22, 0.13), "leather")


def npc_farmer(parts):
    p = lambda n: parts.setdefault(n, Builder(*CTX))
    p("torso.005").cyl((0, 0, 0.90), 0.27, 0, 0.54, "cloth", n=8, r1=0.33, bevel=True)
    vest = p("vest")
    vest.cyl((0, -0.19, 0.90), 0.27, 0, 0.48, "sand", n=6, r1=0.21)
    vest.box_at((0, -0.24, 1.18), (0.24, 0.04, 0.20), "sand")
    for sx in (-1, 1):
        vest.box_at((sx * 0.15, -0.22, 1.28), (0.05, 0.04, 0.26), "sand")
    p("belt.001").cyl((0, 0, 0.70), 0.31, 0, 0.08, "leather", n=8)
    p("seed_sack").ico((-0.42, 0.12, 0.72), 0.20, "sand", (1, 0.9, 1.1))
    head = p("head.005")
    head.ico((0, -0.02, 1.42), 0.22, "skin", bevel=True)
    for sx in (-1, 1):
        head.box_at((sx * 0.085, -0.21, 1.46), (0.055, 0.025, 0.055), "eye")
    head.box_at((0, -0.22, 1.32), (0.10, 0.02, 0.025), "leather")
    p("hat_brim.002").cyl((0, 0, 1.62), 0.30, 0, 0.04, "sand", n=6)
    p("hat.002").cyl((0, 0, 1.64), 0.19, 0, 0.26, "sand_dark", n=4, r1=0.15)
    for n, sx in (("arm_0.005", -1), ("arm_1.005", 1)):
        arm = p(n)
        arm.limb((sx * 0.30, 0, 1.12), (sx * 0.38, -0.06, 0.76), 0.11, 0.08, "cloth", 6)
        arm.ico((sx * 0.39, -0.08, 0.72), 0.075, "skin")
    p("hoe").limb((0.42, -0.08, 0.90), (0.54, -0.08, 1.62), 0.045, 0.03, "wood", 6)
    p("hoe_blade").box_at((0.62, -0.08, 1.70), (0.26, 0.06, 0.10), "iron")
    for n, sx in (("leg_0.006", -1), ("leg_1.006", 1)):
        p(n).limb((sx * 0.13, 0, 0.66), (sx * 0.13, 0, 0.15), 0.10, 0.08, "trousers", 6)
    for n, sx in (("boot_0.005", -1), ("boot_1.005", 1)):
        p(n).box_at((sx * 0.13, -0.04, 0.075), (0.16, 0.24, 0.15), "leather")


def npc_innkeeper(parts):
    p = lambda n: parts.setdefault(n, Builder(*CTX))
    p("belly").ico((0, 0, 0.92), 0.31, "cloth", (1.05, 0.85, 1.0))
    apron = p("apron.001")
    apron.cyl((0, -0.20, 0.86), 0.26, 0, 0.48, "bone", n=6, r1=0.20)
    apron.box_at((0, -0.26, 1.12), (0.24, 0.04, 0.22), "bone")
    apron.box_at((0.42, 0.12, 1.48), (0.34, 0.05, 0.44), "wood")
    apron.box_at((0.26, 0.12, 1.48), (0.06, 0.05, 0.06), "gold")
    p("belt.001").cyl((0, 0, 0.70), 0.31, 0, 0.08, "leather", n=8)
    head = p("head.002")
    head.ico((0, -0.02, 1.42), 0.22, "skin", bevel=True)
    for sx in (-1, 1):
        head.box_at((sx * 0.085, -0.21, 1.47), (0.055, 0.025, 0.055), "eye")
    p("mustache").box_at((0, -0.22, 1.33), (0.20, 0.025, 0.05), "hair")
    p("cap.002").cyl((0, 0.02, 1.60), 0.24, 0, 0.22, "cloth", n=6, r1=0.10)
    for n, sx in (("arm_0.002", -1), ("arm_1.002", 1)):
        arm = p(n)
        arm.limb((sx * 0.30, 0, 1.12), (sx * 0.42, -0.12, 0.86), 0.10, 0.075, "cloth", 4)
        arm.ico((sx * 0.44, -0.14, 0.82), 0.075, "skin")
    for i, x in enumerate((-0.06, 0.06)):
        p("mug_%d" % i).cyl((0.46 + x, -0.18, 1.06), 0.07, 0, 0.16, "bone", n=6)
        p("foam_%d" % i).ico((0.46 + x, -0.18, 1.15), 0.055, "snow")
    p("tray").box_at((0.46, -0.18, 0.96), (0.32, 0.26, 0.04), "wood")
    for n, sx in (("leg_0.001", -1), ("leg_1.001", 1)):
        leg = p(n)
        leg.limb((sx * 0.13, 0, 0.66), (sx * 0.13, 0, 0.15), 0.10, 0.08, "trousers", 4)
        leg.box_at((sx * 0.13, -0.04, 0.075), (0.16, 0.24, 0.15), "leather")


BOUND = {"spider": spider, "scorpion_queen": scorpion_queen,
         "colossus": colossus, "npc_merchant": npc_merchant,
         "npc_farmer": npc_farmer, "npc_innkeeper": npc_innkeeper}
SINGLE = {"wolf": wolf, "wraith": wraith, "bat": bat,
          "npc_desert_scorpion": npc_desert_scorpion, "sea_turtle": sea_turtle,
          "npc_wolf_pup": npc_wolf_pup, "scarecrow": scarecrow,
          "npc_fisher": npc_fisher, "npc_bard": npc_bard,
          "npc_sailor": npc_sailor, "npc_traveling_scholar": npc_traveling_scholar}

CTX = None


def build(name):
    global CTX
    idx, mats = new_palette()
    CTX = (idx, mats)
    if name in SINGLE:
        b = Builder(idx, mats)
        SINGLE[name](b)
        obj = b.finish(name, bevel_width=0.006)
        export(obj, name)
    else:
        parts = {}
        BOUND[name](parts)
        objs = []
        for n, b in parts.items():
            print("  FINISH", n, flush=True)
            objs.append(b.finish(n, bevel_width=0.006, object_name=n, pivot=False))
        normalise(objs)
        export(objs, name)
    expected = set()
    print("BUILT", name, flush=True)


def main():
    names = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else ALL
    for name in names:
        if name not in ALL:
            raise SystemExit("unknown model: " + name)
        build(name)
    print("REBUILD_DONE", len(names), flush=True)


if __name__ == "__main__":
    main()
