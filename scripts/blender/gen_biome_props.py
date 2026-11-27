"""Starfall asset expansion — family 1: biome-specific objects & props.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_biome_props.py

Twelve new low-poly, flat-shaded, palette-only assets: forest, desert, tundra,
swamp and ocean. Every asset is re-based to z = 0 / (0, 0) before export and
measured against the 1.55 u player. `mangrove` is the hero piece and goes
through the bevel + weighted-normal + UV-pack finishing path.
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import new_scene, box, cyl, cone, ico, contact_sheet
from expansion_common import P, collection, finalize, write_records

new_scene()
RECORDS = []
SETS = []


def shroom(p, stem, cap_mat, height=0.32, radius=0.18, vertices=5):
    cyl("%s_stem" % stem, (0, 0, height / 2), radius * 0.34, height,
        P["bone"], p, vertices)
    ico("%s_cap" % stem, (0, 0, height + radius * 0.18), radius, cap_mat, p, 0,
        scale=(1.0, 1.0, 0.55))


# ---------------------------------------------------------------------------
# Forest
# ---------------------------------------------------------------------------
def build_berry_bush():
    p = collection("berry_bush")
    ico("core", (0, 0, 0.36), 0.37, P["leaf"], p, 0, scale=(1.15, 1.05, 0.95))
    ico("left", (-0.30, 0.05, 0.29), 0.26, P["leaf_dark"], p, 0, scale=(1.0, 1.0, 0.9))
    ico("right", (0.29, -0.08, 0.31), 0.27, P["leaf_light"], p, 0, scale=(1.0, 1.0, 0.9))
    ico("back", (0.04, 0.29, 0.30), 0.25, P["leaf"], p, 0, scale=(1.0, 1.0, 0.9))
    for i, (x, y, z) in enumerate(((-0.27, -0.20, 0.44), (0.22, 0.24, 0.40),
                                   (0.04, -0.32, 0.35), (0.34, -0.02, 0.53),
                                   (-0.08, 0.16, 0.61))):
        ico("berry_%d" % i, (x, y, z), 0.055, P["enemy"], p, 0,
            scale=(1.0, 1.0, 0.85))
    return p


def build_dead_tree():
    p = collection("dead_tree")
    cyl("trunk", (0, 0, 0.88), 0.14, 1.76, P["wood_light"], p, 6)
    for i, (x, y, z, rot, length) in enumerate((
            (-0.28, 0.02, 1.72, (0, 0, -0.95), 0.74),
            (0.26, -0.05, 1.45, (0, 0, 0.85), 0.60),
            (0.02, 0.21, 2.02, (0.55, 0, 0.25), 0.48))):
        b = cyl("branch_%d" % i, (x, y, z), 0.065, length, P["wood_light"], p, 5)
        b.rotation_euler = rot
    for i, (x, y, a) in enumerate(((-0.38, 0.10, 0.5), (0.36, -0.12, -0.4),
                                   (-0.10, -0.38, 2.7))):
        root = cyl("root_%d" % i, (x, y, 0.14), 0.085, 0.55, P["wood_dark"], p, 5)
        root.rotation_euler = (0, a, a * 0.5)
    top = cone("snapped_top", (0.02, 0, 1.88), 0.11, 0.04, 0.30,
               P["wood_dark"], p, 5)
    top.rotation_euler = (0.12, 0, 0.08)
    return p


def build_mushroom_cluster():
    p = collection("mushroom_cluster")
    for i, (x, y, h, r) in enumerate(((-0.21, 0.04, 0.30, 0.19),
                                      (0.20, -0.10, 0.42, 0.22),
                                      (0.02, 0.21, 0.24, 0.14))):
        cyl("stem_%d" % i, (x, y, h / 2), r * 0.32, h, P["bone"], p, 5)
        ico("cap_%d" % i, (x, y, h + r * 0.16), r, P["enemy" if i != 1 else "player"],
            p, 0, scale=(1.0, 1.0, 0.55))
        ico("spot_%d" % i, (x + r * 0.3, y - r * 0.2, h + r * 0.48), 0.04,
            P["bone"], p, 0, scale=(1.0, 1.0, 0.35))
    return p


def build_herb_patch():
    p = collection("herb_patch")
    for i, (x, y, h) in enumerate(((-0.23, 0.06, 0.22), (0.18, 0.16, 0.28),
                                   (0.24, -0.16, 0.19), (-0.10, -0.23, 0.25),
                                   (0.02, 0.02, 0.30))):
        cyl("stalk_%d" % i, (x, y, h / 2), 0.025, h, P["grass"], p, 4)
        for j, side in enumerate((-1, 1)):
            leaf = ico("leaf_%d_%d" % (i, j), (x + side * 0.11, y, h * 0.72),
                       0.13, P["leaf_light" if j else "leaf"], p, 0,
                       scale=(1.0, 0.55, 0.18))
            leaf.rotation_euler = (0.25, side * 0.45, 0)
        cone("bud_%d" % i, (x, y, h + 0.045), 0.055, 0.0, 0.11,
             P["arcane_light"], p, 5)
    return p


def build_mangrove():
    """Hero prop: stilt roots below a broad, flat-topped swamp canopy."""
    p = collection("mangrove")
    trunk = cyl("trunk", (0.03, 0, 1.05), 0.17, 1.85, P["wood_dark"], p, 6)
    trunk.rotation_euler = (0, 0.04, 0)
    for i in range(4):
        a = i * math.tau / 4.0
        x, y = math.cos(a) * 0.60, math.sin(a) * 0.55
        root = cyl("stilt_root_%d" % i, (x * 0.55, y * 0.55, 0.52), 0.055,
                   1.30, P["wood"], p, 5)
        root.rotation_euler = (math.sin(a) * 0.47, -math.cos(a) * 0.48, 0)
        ico("root_foot_%d" % i, (x, y, 0.084), 0.14, P["wood_dark"], p, 0,
            scale=(1.1, 1.0, 0.6))
    for i, (x, y, z, r, key) in enumerate((
            (-0.70, 0.10, 2.08, 0.58, "leaf_dark"),
            (0.70, -0.12, 2.10, 0.58, "leaf"),
            (0.02, 0.42, 2.30, 0.52, "leaf_light"),
            (-0.24, -0.44, 2.02, 0.48, "leaf"),
            (0.34, 0.32, 2.20, 0.48, "leaf_dark"),
            (0.0, 0.05, 2.26, 0.42, "leaf"))):
        ico("canopy_%d" % i, (x, y, z), r, P[key], p, 0,
            scale=(1.15, 1.05, 0.65))
    for i, (x, y, h) in enumerate(((-0.38, -0.18, 0.55), (0.36, 0.22, 0.42))):
        cyl("pneumatophore_%d" % i, (x, y, h / 2), 0.055, h,
            P["wood_light"], p, 5)
    return p


# ---------------------------------------------------------------------------
# Desert / tundra
# ---------------------------------------------------------------------------
def build_dune_rock():
    p = collection("dune_rock")
    ico("main", (0, 0, 0.27), 0.50, P["sand"], p, 0, scale=(1.20, 0.95, 0.60))
    ico("ledge", (0.28, 0.06, 0.19), 0.31, P["sand_dark"], p, 0,
        scale=(1.10, 1.00, 0.55))
    ico("cap", (-0.20, -0.08, 0.42), 0.27, P["stone_light"], p, 0,
        scale=(1.10, 0.90, 0.48))
    for i, (x, y) in enumerate(((-0.42, 0.16), (0.40, -0.18))):
        ico("pebble_%d" % i, (x, y, 0.05), 0.09, P["sand_dark"], p, 0,
            scale=(1.0, 0.9, 0.6))
    return p


def build_ice_crystal():
    p = collection("ice_crystal")
    ico("snow_base", (0, 0, 0.10), 0.36, P["snow"], p, 0, scale=(1.1, 1.0, 0.45))
    cone("spire_main", (0, 0, 0.68), 0.21, 0.0, 1.16, P["ice"], p, 6)
    for i, (x, y, h, tilt) in enumerate(((-0.26, 0.05, 0.62, 0.22),
                                         (0.24, -0.08, 0.72, -0.18),
                                         (0.02, 0.25, 0.48, -0.12))):
        s = cone("shard_%d" % i, (x, y, h / 2 + 0.06), 0.12, 0.0, h,
                 P["ice" if i != 1 else "arcane_light"], p, 5)
        s.rotation_euler = (tilt, 0, tilt * 0.5)
    ico("glow_core", (0, 0.02, 0.30), 0.11, P["arcane"], p, 0)
    return p


def build_lava_vent():
    p = collection("lava_vent")
    ico("mouth", (0, 0, 0.08), 0.45, P["stone_dark"], p, 0, scale=(1.1, 1.05, 0.35))
    ico("pool", (0, 0, 0.19), 0.31, P["fire"], p, 0, scale=(1.0, 1.0, 0.30))
    for i in range(8):
        a = i * math.tau / 8.0
        rock = ico("rim_%d" % i, (math.cos(a) * 0.42, math.sin(a) * 0.40, 0.20),
                   0.15, P["stone" if i % 2 else "stone_dark"], p, 0,
                   scale=(1.0, 1.0, 0.85))
        rock.rotation_euler.z = a
    cone("flame", (0.02, 0, 0.44), 0.20, 0.0, 0.52, P["fire"], p, 6)
    ico("ember", (-0.10, 0.13, 0.37), 0.09, P["gold"], p, 0)
    box("crack_a", (0.46, 0.10, 0.06), (0.24, 0.07, 0.02), P["fire"], p)
    box("crack_b", (-0.38, -0.28, 0.06), (0.20, 0.06, 0.02), P["fire"], p)
    return p


# ---------------------------------------------------------------------------
# Swamp / ocean
# ---------------------------------------------------------------------------
def build_coral():
    p = collection("coral")
    ico("base", (0, 0, 0.10), 0.30, P["stone_dark"], p, 0, scale=(1.15, 1.0, 0.55))
    for i, (x, y, h, tilt, key) in enumerate((
            (-0.22, 0.08, 0.62, -0.20, "coral"),
            (0.20, -0.05, 0.78, 0.16, "player"),
            (0.02, 0.20, 0.50, 0.08, "coral"),
            (-0.05, -0.22, 0.44, -0.12, "player"))):
        branch = cyl("branch_%d" % i, (x, y, h / 2 + 0.10), 0.075, h,
                     P[key], p, 5)
        branch.rotation_euler = (tilt, 0, tilt * 0.5)
        ico("polyp_%d" % i, (x + tilt * h * 0.4, y, h + 0.10), 0.11,
            P["player" if i % 2 == 0 else "coral"], p, 0)
    for i, a in enumerate((0.4, 2.6, 4.5)):
        fan = ico("fan_%d" % i, (math.cos(a) * 0.26, math.sin(a) * 0.24, 0.28),
                  0.17, P["coral"], p, 0, scale=(0.35, 1.0, 0.75))
        fan.rotation_euler.z = a
    return p


def build_fossil():
    p = collection("fossil")
    slab = box("slab", (0, 0, 0.12), (1.20, 0.58, 0.24), P["stone_light"], p)
    slab.rotation_euler = (0, 0, 0.06)
    spine = cyl("spine", (0, 0, 0.27), 0.045, 0.92, P["bone"], p, 5)
    spine.rotation_euler = (0, math.pi / 2, 0)
    for i in range(5):
        x = -0.34 + i * 0.17
        for side in (-1, 1):
            rib = box("rib_%d_%d" % (i, side), (x, side * 0.16, 0.29),
                      (0.045, 0.34, 0.045), P["bone"], p)
            rib.rotation_euler = (side * 0.85, 0, 0)
    ico("skull", (0.50, 0, 0.30), 0.17, P["bone"], p, 0,
        scale=(1.25, 0.85, 0.70))
    for i, (x, y) in enumerate(((-0.50, 0.22), (-0.35, -0.20), (0.48, 0.20))):
        ico("chip_%d" % i, (x, y, 0.26), 0.06, P["stone_dark"], p, 0)
    return p


def build_wreck_plank():
    p = collection("wreck_plank")
    plank = box("plank", (0, 0, 0.12), (1.30, 0.30, 0.11), P["wood"], p)
    plank.rotation_euler = (0, 0, 0.07)
    for i, x in enumerate((-0.45, 0.42)):
        cyl("nail_%d" % i, (x, -0.02, 0.19), 0.035, 0.03, P["iron"], p, 6)
    broken = box("splinter", (0.70, 0.03, 0.12), (0.30, 0.10, 0.08),
                 P["wood_light"], p)
    broken.rotation_euler = (0, 0, -0.42)
    ico("barnacle_a", (-0.42, 0.12, 0.20), 0.08, P["stone_light"], p, 0)
    ico("barnacle_b", (0.20, -0.10, 0.19), 0.06, P["stone_light"], p, 0)
    seaweed = ico("seaweed", (-0.10, 0.16, 0.22), 0.18, P["swamp"], p, 0,
                  scale=(1.0, 0.35, 0.30))
    seaweed.rotation_euler = (0, 0, 0.3)
    return p


def build_fishing_net():
    p = collection("fishing_net")
    for i in range(8):
        a = i * math.tau / 8.0
        seg = box("hoop_%d" % i, (math.cos(a) * 0.38, math.sin(a) * 0.34, 0.46),
                  (0.14, 0.06, 0.055), P["wood_light"], p)
        seg.rotation_euler.z = a
        if i % 2 == 0:
            ico("float_%d" % i, (math.cos(a) * 0.40, math.sin(a) * 0.36, 0.50),
                0.075, P["player"], p, 0)
    cone("net_bag", (0, 0, 0.25), 0.36, 0.07, 0.42, P["cloth"], p, 8)
    rope = cyl("rope", (0.28, -0.30, 0.50), 0.025, 0.55, P["wood"], p, 4)
    rope.rotation_euler = (0, 0, 0.6)
    return p


ASSETS = [
    ("berry_bush",      build_berry_bush,      (0.85, 0.80, 0.75), "forest", "food resource"),
    ("dead_tree",       build_dead_tree,       (1.50, 1.40, 2.60), "forest", "bare snag"),
    ("mushroom_cluster", build_mushroom_cluster, (0.75, 0.70, 0.60), "forest", "three caps"),
    ("herb_patch",      build_herb_patch,      (0.70, 0.65, 0.30), "forest", "harvest herb"),
    ("mangrove",        build_mangrove,        (2.60, 1.75, 2.72), "swamp", "HERO stilt tree"),
    ("dune_rock",       build_dune_rock,       (1.05, 0.85, 0.55), "desert", "wind-worn"),
    ("ice_crystal",     build_ice_crystal,     (0.75, 0.70, 1.25), "tundra", "glowing shards"),
    ("lava_vent",       build_lava_vent,       (0.95, 0.90, 0.65), "volcanic", "emissive vent"),
    ("coral",           build_coral,           (0.85, 0.75, 0.95), "ocean", "staghorn"),
    ("fossil",          build_fossil,          (1.25, 0.65, 0.50), "desert", "rib slab"),
    ("wreck_plank",     build_wreck_plank,     (1.45, 0.40, 0.30), "ocean", "shipwreck debris"),
    ("fishing_net",     build_fishing_net,     (0.90, 0.80, 0.55), "ocean", "hoop net"),
]

for name, builder, target, biome, notes in ASSETS:
    col = builder()
    finalize(name, col, "biome_props", target, "%s; %s" % (biome, notes),
             hero=(name == "mangrove"), bevel_width=0.006, records=RECORDS)
    SETS.append((name, col))

write_records("biome_props", RECORDS)
try:
    contact_sheet(SETS, "biome_props.png", width=1900, height=340)
except Exception as exc:  # previews are optional; never fail the build for them
    print("PREVIEW SKIPPED", exc)
