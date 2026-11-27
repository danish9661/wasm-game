"""Starfall asset expansion — wave 3: dungeon, farm, town, ocean + NPCs.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_models_v3.py

Thirty new low-poly, flat-shaded, palette-only assets. Names deliberately do
not repeat anything already in ``assets/models``; every existing GLB was
checked before this set was authored (e.g. ``altar`` -> ``dungeon_altar``,
``bone_pile`` -> ``bone_pile_ribs`` / ``bone_pile_skulls``, ``well`` ->
``well_bucket``, ``coral`` -> ``coral_fan`` / ``coral_brain``).

Every asset is re-based to z = 0 / (0, 0) before export, measured against the
1.55 u player and recorded in ``assets/models/expansion_models_v3.json``.
``dungeon_cell_interior`` is the hero piece: selective bevel + weighted
normals + smart UV pack, using the 1500-triangle assembled-asset budget.
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import new_scene, box, cyl, cone, ico, contact_sheet
from expansion_common import P, collection, finalize, write_records

new_scene()
RECORDS = []
SETS = []
PLAYER = 1.55


# ---------------------------------------------------------------------------
# Thin wrappers around the common primitives (palette keys, not colours)
# ---------------------------------------------------------------------------
def slab(p, name, loc, scale, key):
    return box(name, loc, scale, P[key], p)


def rod(p, name, loc, radius, depth, key, verts=6, tilt=(0, 0, 0)):
    o = cyl(name, loc, radius, depth, P[key], p, verts)
    o.rotation_euler = tilt
    return o


def tip(p, name, loc, r1, r2, depth, key, verts=5, tilt=(0, 0, 0)):
    o = cone(name, loc, r1, r2, depth, P[key], p, verts)
    o.rotation_euler = tilt
    return o


def lump(p, name, loc, radius, key, scale=None):
    return ico(name, loc, radius, P[key], p, 0, scale)


def link(p, name, loc, size=0.10, key="iron"):
    """One chain link: a small, thin box (12 tris) in an alternating plane."""
    return slab(p, name, loc, (size, size * 0.32, size * 0.72), key)


# ===========================================================================
# Family A - dungeon interiors & props
# ===========================================================================
def build_dungeon_altar():
    """Tiered crypt altar: cold stone, offering bowl, skull and candlelight."""
    p = collection("dungeon_altar")
    slab(p, "base", (0, 0, 0.10), (1.30, 0.80, 0.20), "stone_dark")
    slab(p, "step", (0, 0, 0.27), (1.10, 0.66, 0.14), "stone")
    slab(p, "shaft", (0, 0, 0.58), (0.90, 0.54, 0.48), "stone_dark")
    slab(p, "slab", (0, 0, 0.87), (1.20, 0.72, 0.14), "stone_light")
    for i, x in enumerate((-0.44, 0.44)):
        rod(p, "pillar_%d" % i, (x, 0, 1.05), 0.075, 0.30, "stone", 6)
    lump(p, "skull", (0, -0.04, 1.05), 0.15, "bone", (1.0, 0.85, 0.8))
    for i, x in enumerate((-0.28, 0.28)):
        rod(p, "candle_%d" % i, (x, 0.16, 1.04), 0.045, 0.20, "bone", 5)
        tip(p, "flame_%d" % i, (x, 0.16, 1.20), 0.055, 0.0, 0.14, "fire", 5)
    rod(p, "offering_bowl", (0.32, -0.20, 0.96), 0.13, 0.07, "iron", 6)
    lump(p, "rune", (0, -0.36, 0.60), 0.11, "arcane", (1.0, 0.35, 1.0))
    return p


def build_sarcophagus():
    """Sealed stone coffin with a lid, end caps and a bleached bone motif."""
    p = collection("sarcophagus")
    slab(p, "plinth", (0, 0, 0.08), (1.90, 0.90, 0.16), "stone_dark")
    slab(p, "coffer", (0, 0, 0.34), (1.66, 0.76, 0.42), "stone")
    slab(p, "sarcophagus_lid", (0, 0, 0.62), (1.82, 0.84, 0.14), "stone_light")
    for i, x in enumerate((-0.76, 0.76)):
        slab(p, "end_cap_%d" % i, (x, 0, 0.46), (0.14, 0.84, 0.56), "stone_dark")
    for i, x in enumerate((-0.50, 0.50)):
        rod(p, "lid_band_%d" % i, (x, 0, 0.70), 0.035, 0.86, "iron", 4,
            (math.pi / 2, 0, 0))
    lump(p, "skull", (-0.20, -0.06, 0.73), 0.14, "bone", (1.0, 0.85, 0.7))
    rod(p, "cross_bone_a", (0.18, 0, 0.72), 0.035, 0.52, "bone", 4,
        (0, math.pi / 2, math.pi / 4))
    rod(p, "cross_bone_b", (0.18, 0, 0.72), 0.035, 0.52, "bone", 4,
        (0, math.pi / 2, -math.pi / 4))
    return p


def build_torch_sconce():
    """Wall-mounted iron bracket with a burning torch and a hanging chain."""
    p = collection("torch_sconce")
    slab(p, "wall_plate", (0, 0.14, 0.62), (0.28, 0.06, 0.42), "iron")
    slab(p, "bracket", (0, -0.02, 0.58), (0.10, 0.36, 0.10), "iron")
    rod(p, "cup", (0, -0.16, 0.62), 0.14, 0.12, "stone_dark", 6)
    rod(p, "torch", (0, -0.16, 0.86), 0.065, 0.48, "wood", 6,
        (0.16, 0, 0.06))
    tip(p, "flame", (0.02, -0.18, 1.20), 0.17, 0.0, 0.40, "fire", 6)
    lump(p, "ember", (-0.06, -0.10, 1.05), 0.06, "gold")
    for i, (x, z) in enumerate(((-0.10, 0.42), (0.10, 0.32))):
        link(p, "chain_%d" % i, (x, 0.12, z), 0.09)
    return p


def build_dungeon_chains():
    """A hanging length of iron chain with a hook and a floor anchor plate."""
    p = collection("dungeon_chains")
    slab(p, "anchor_plate", (0, 0, 0.04), (0.36, 0.30, 0.08), "stone_dark")
    rod(p, "anchor_ring", (0, 0, 0.11), 0.09, 0.05, "iron", 6)
    for i in range(9):
        z = 0.20 + i * 0.13
        link(p, "link_%d" % i, (0, 0, z), 0.15,
             "iron" if i % 2 else "steel")
    rod(p, "hook_shaft", (0, 0, 1.45), 0.045, 0.22, "iron", 5)
    tip(p, "hook", (0, 0, 1.28), 0.09, 0.0, 0.22, "steel", 5,
        (math.pi, 0, 0))
    lump(p, "hook_eye", (0, 0, 1.58), 0.08, "iron")
    return p


def build_dungeon_gate():
    """Portcullis: framed iron grate with pointed feet and a chain pair."""
    p = collection("dungeon_gate")
    for i, x in enumerate((-0.72, 0.72)):
        slab(p, "post_%d" % i, (x, 0, 1.05), (0.20, 0.24, 2.10), "stone_dark")
    slab(p, "lintel", (0, 0, 2.20), (1.64, 0.26, 0.20), "stone")
    for i, x in enumerate((-0.54, -0.27, 0, 0.27, 0.54)):
        rod(p, "bar_%d" % i, (x, 0, 1.05), 0.055, 2.02, "iron", 5)
        tip(p, "spike_%d" % i, (x, 0, 0.02), 0.085, 0.0, 0.24, "steel", 5,
            (math.pi, 0, 0))
    for i, z in enumerate((0.50, 1.02, 1.54, 2.02)):
        rod(p, "crossbar_%d" % i, (0, -0.05, z), 0.05, 1.34, "iron", 5,
            (0, math.pi / 2, 0))
    for i, x in enumerate((-0.50, 0.50)):
        lump(p, "ring_%d" % i, (x, -0.05, 2.14), 0.08, "iron")
    return p


def build_bone_pile_ribs():
    """Collapsed rib cage: paired ribs along a spine, skull and loose bones."""
    p = collection("bone_pile_ribs")
    lump(p, "mound", (0, 0, 0.06), 0.46, "stone_dark", (1.3, 1.05, 0.28))
    rod(p, "spine", (0, 0, 0.18), 0.05, 0.94, "bone", 5,
        (0, math.pi / 2, 0))
    for i, x in enumerate((-0.30, -0.10, 0.10, 0.30)):
        for side in (-1, 1):
            slab(p, "rib_%d_%d" % (i, side), (x, side * 0.20, 0.17),
                 (0.06, 0.36, 0.06), "bone")
    lump(p, "skull", (-0.46, 0.02, 0.22), 0.15, "bone", (1.1, 0.85, 0.8))
    for i, (x, y) in enumerate(((0.44, 0.16), (0.34, -0.22), (-0.10, -0.40))):
        rod(p, "loose_%d" % i, (x, y, 0.12), 0.04, 0.38, "bone", 4,
            (0, math.pi / 2, 0.5 if i % 2 else -0.4))
    return p


def build_bone_pile_skulls():
    """Heap of skulls and long bones — distinct silhouette from the rib pile."""
    p = collection("bone_pile_skulls")
    lump(p, "mound", (0, 0, 0.07), 0.52, "stone_dark", (1.2, 1.1, 0.3))
    for i, (x, y, z) in enumerate(((-0.24, 0.10, 0.24), (0.18, 0.18, 0.26),
                                   (0.30, -0.18, 0.20), (-0.10, -0.28, 0.18))):
        lump(p, "skull_%d" % i, (x, y, z), 0.16, "bone" if i % 2 else "sand",
             (1.0, 0.85, 0.8))
        slab(p, "jaw_%d" % i, (x, y - 0.12, z - 0.10), (0.18, 0.16, 0.06), "bone")
    for i, (x, y, a) in enumerate(((-0.46, -0.10, 0.4), (0.46, 0.06, -0.7),
                                   (0.02, 0.40, 1.2), (-0.30, 0.36, 0.9))):
        rod(p, "long_bone_%d" % i, (x, y, 0.14), 0.04, 0.52, "bone", 4,
            (0, math.pi / 2, a))
    return p


def build_dungeon_cell_interior():
    """Hero: 3x3 stone cell, barred window, door, chains and grave dressing."""
    p = collection("dungeon_cell_interior")
    slab(p, "floor_slab", (0, 0, 0.05), (3.0, 3.0, 0.10), "stone_dark")
    # Eight 1 u wall segments on the ±1.5 grid lines; three courses each.
    for side, fixed in (("x", -1.5), ("x", 1.5), ("y", -1.5), ("y", 1.5)):
        for i in (-1, 0, 1):
            x, y = (i, fixed) if side == "x" else (fixed, i)
            suffix = "%s%s_%s" % (side, i, fixed)
            rot = 0.0 if side == "x" else math.pi / 2
            base = slab(p, "wall_low_" + suffix, (x, y, 0.38), (1.0, 0.18, 0.76),
                        "stone_dark")
            mid = slab(p, "wall_mid_" + suffix, (x, y, 1.20), (1.0, 0.18, 0.88),
                       "stone")
            top = slab(p, "wall_top_" + suffix, (x, y, 2.14), (1.0, 0.18, 0.72),
                       "stone_light")
            for o in (base, mid, top):
                o.rotation_euler.z = rot
    # Door in the south wall (the centre segment is left as wall behind it).
    for i, x in enumerate((-0.46, 0.46)):
        slab(p, "door_jamb_%d" % i, (x, -1.44, 1.0), (0.08, 0.10, 2.0), "wood")
    slab(p, "door_leaf", (0, -1.50, 0.95), (0.84, 0.10, 1.90), "wood_dark")
    slab(p, "door_lintel", (0, -1.44, 1.96), (0.94, 0.12, 0.10), "iron")
    lump(p, "door_knob", (0.26, -1.58, 0.92), 0.045, "gold")
    # Barred window on the north wall at the 1.1 u sill.
    slab(p, "window_sill", (0, 1.40, 1.07), (0.92, 0.22, 0.10), "stone_light")
    for i, x in enumerate((-0.32, 0, 0.32)):
        rod(p, "window_bar_%d" % i, (x, 1.43, 1.52), 0.035, 0.90, "iron", 5)
    slab(p, "window_head", (0, 1.40, 2.00), (0.92, 0.22, 0.08), "stone_light")
    # Dressing: hanging chains, a coffin, a torch and a small bone scatter.
    for i, z in enumerate((0.45, 0.62, 0.79, 0.96)):
        link(p, "cell_chain_%d" % i, (-1.12, 1.14, z), 0.16)
    lump(p, "chain_anchor", (-1.12, 1.14, 2.22), 0.10, "iron")
    slab(p, "coffer", (0.86, 0.62, 0.28), (1.30, 0.70, 0.42), "stone")
    slab(p, "sarcophagus_lid", (0.86, 0.62, 0.55), (1.46, 0.78, 0.12),
         "stone_light")
    lump(p, "cell_skull", (-0.78, -0.62, 0.20), 0.14, "bone")
    for i, (x, y, a) in enumerate(((-0.50, -0.86, 0.6), (0.42, -0.70, -0.4))):
        rod(p, "cell_bone_%d" % i, (x, y, 0.15), 0.04, 0.42, "bone", 4,
            (0, math.pi / 2, a))
    rod(p, "cell_torch", (1.16, -1.16, 0.92), 0.05, 0.70, "wood", 5)
    tip(p, "cell_flame", (1.16, -1.16, 1.36), 0.13, 0.0, 0.32, "fire", 5)
    return p


DUNGEON = [
    ("dungeon_altar", build_dungeon_altar, (1.25, 0.80, 1.30),
     "crypt prop", "tiered altar + candles"),
    ("sarcophagus", build_sarcophagus, (1.90, 0.90, 0.80),
     "crypt prop", "sealed stone coffin"),
    ("torch_sconce", build_torch_sconce, (0.34, 0.50, 1.45),
     "wall prop", "iron bracket + flame"),
    ("dungeon_chains", build_dungeon_chains, (0.25, 0.24, 1.66),
     "hanging prop", "nine links + hook"),
    ("dungeon_gate", build_dungeon_gate, (1.64, 0.26, 2.40),
     "portcullis", "spiked iron grate"),
    ("bone_pile_ribs", build_bone_pile_ribs, (1.30, 1.00, 0.45),
     "scatter", "rib cage + skull"),
    ("bone_pile_skulls", build_bone_pile_skulls, (1.20, 1.05, 0.45),
     "scatter", "skull heap"),
    ("dungeon_cell_interior", build_dungeon_cell_interior, (3.20, 3.20, 2.50),
     "interior", "HERO 3x3 cell; 2.5 u storey"),
]


# ===========================================================================
# Family B - farm & crops
# ===========================================================================
def build_wheat():
    """Standing wheat bundle: five stalks with grain heads and broad leaves."""
    p = collection("wheat")
    lump(p, "soil", (0, 0, 0.05), 0.40, "sand", (1.15, 1.05, 0.24))
    for i, (x, y, h) in enumerate(((-0.22, 0.02, 0.72), (-0.08, -0.14, 0.86),
                                   (0.10, 0.06, 0.94), (0.24, -0.02, 0.78),
                                   (0.02, 0.20, 0.68))):
        rod(p, "stalk_%d" % i, (x, y, h / 2 + 0.08), 0.018, h, "grass", 4,
            (0, 0, (x) * 0.18))
        tip(p, "grain_%d" % i, (x * 1.05, y, h + 0.16), 0.075, 0.0, 0.30,
            "sand", 5)
        if i in (1, 2, 4):
            lump(p, "leaf_%d" % i, (x + 0.10, y, h * 0.48), 0.16,
                 "leaf_light" if i == 2 else "leaf", (1.0, 0.5, 0.12))
    return p


def build_carrot():
    """Three harvested carrots with feathery tops, laid on a soil patch."""
    p = collection("carrot")
    lump(p, "soil", (0, 0, 0.05), 0.42, "wood_dark", (1.2, 1.0, 0.22))
    for i, (x, y) in enumerate(((-0.20, 0.04), (0.16, 0.12), (0.04, -0.20))):
        tip(p, "root_%d" % i, (x, y, 0.20), 0.09, 0.015, 0.34, "player", 5,
            (math.pi, 0, 0.10 * (i - 1)))
        for j, dx in enumerate((-0.07, 0.07)):
            tip(p, "top_%d_%d" % (i, j), (x + dx, y, 0.46), 0.045, 0.0,
                0.24, "leaf", 4, (0, 0, dx * 2.5))
    return p


def build_potato():
    """Mounded hill with four exposed tubers and two creeping vines."""
    p = collection("potato")
    lump(p, "hill", (0, 0, 0.12), 0.50, "wood_dark", (1.2, 1.0, 0.42))
    for i, (x, y, r) in enumerate(((-0.24, 0.06, 0.15), (0.12, 0.18, 0.17),
                                   (0.26, -0.12, 0.13), (-0.06, -0.24, 0.14))):
        lump(p, "tuber_%d" % i, (x, y, 0.12), r,
             "sand" if i % 2 else "wood_light", (1.25, 0.9, 0.8))
    for i, x in enumerate((-0.18, 0.20)):
        rod(p, "vine_%d" % i, (x, 0, 0.34), 0.02, 0.46, "leaf_dark", 4)
        lump(p, "vine_leaf_%d" % i, (x + 0.10, 0.04, 0.44), 0.16,
             "leaf", (1.0, 0.55, 0.12))
    lump(p, "leaf_3", (-0.10, -0.12, 0.40), 0.15, "leaf_light",
         (1.0, 0.55, 0.12))
    return p


def build_fence_gate():
    """Farm gate: two heavy posts, five pickets and a diagonal latch brace."""
    p = collection("fence_gate")
    for i, x in enumerate((-0.55, 0.55)):
        slab(p, "post_%d" % i, (x, 0, 0.55), (0.14, 0.14, 1.10), "wood_dark")
        tip(p, "post_cap_%d" % i, (x, 0, 1.15), 0.10, 0.0, 0.14, "wood", 4)
    for i, z in enumerate((0.34, 0.88)):
        slab(p, "rail_%d" % i, (0, 0, z), (1.02, 0.08, 0.10), "wood")
    for i, x in enumerate((-0.40, -0.20, 0, 0.20, 0.40)):
        slab(p, "picket_%d" % i, (x, 0, 0.62), (0.11, 0.10, 1.02), "wood_light")
    brace = slab(p, "brace", (0, -0.08, 0.60), (1.20, 0.06, 0.10), "wood")
    brace.rotation_euler.z = 0.30
    slab(p, "latch", (0.40, -0.14, 0.70), (0.22, 0.06, 0.07), "iron")
    for i, x in enumerate((-0.55, 0.55)):
        rod(p, "hinge_%d" % i, (x, -0.08, 0.36), 0.05, 0.12, "iron", 4,
            (math.pi / 2, 0, 0))
    return p


def build_scarecrow():
    """Cross-framed guardian with a straw head, ragged coat and wide hat."""
    p = collection("scarecrow")
    rod(p, "post", (0, 0, 0.80), 0.055, 1.60, "wood", 6)
    slab(p, "crossbar", (0, 0, 1.20), (1.30, 0.08, 0.09), "wood")
    slab(p, "coat", (0, 0.02, 1.02), (0.72, 0.22, 0.62), "cloth")
    for i, x in enumerate((-0.56, 0.56)):
        tip(p, "sleeve_%d" % i, (x, 0, 1.12), 0.11, 0.06, 0.34, "player", 4,
            (0, math.pi / 2 if i else -math.pi / 2, 0))
        lump(p, "hand_%d" % i, (x * 1.18, 0, 1.10), 0.08, "sand")
    lump(p, "head", (0, 0, 1.62), 0.19, "sand", (1.0, 0.95, 1.05))
    rod(p, "hat_brim", (0, 0, 1.78), 0.30, 0.04, "wood_dark", 6)
    tip(p, "hat_crown", (0, 0, 1.90), 0.17, 0.12, 0.24, "wood_dark", 6)
    for i, x in enumerate((-0.08, 0.08)):
        lump(p, "eye_%d" % i, (x, -0.17, 1.65), 0.035, "charcoal" if "charcoal" in P else "stone_dark")
    for i, (x, y) in enumerate(((-0.30, 0.10), (0.30, -0.06), (0.10, 0.22))):
        tip(p, "straw_%d" % i, (x, y, 0.90), 0.06, 0.0, 0.26, "sand", 4,
            (0, 0, x))
    return p


def build_well_bucket():
    """Well variant: stone ring, hoisted bucket, winch posts and a small roof."""
    p = collection("well_bucket")
    for i in range(6):
        a = i * math.tau / 6.0
        slab(p, "stone_%d" % i,
             (math.cos(a) * 0.38, math.sin(a) * 0.34, 0.16),
             (0.26, 0.16, 0.32), "stone" if i % 2 else "stone_dark")
    rod(p, "water", (0, 0, 0.12), 0.28, 0.04, "water", 8)
    for i, x in enumerate((-0.34, 0.34)):
        slab(p, "winch_post_%d" % i, (x, 0, 0.72), (0.10, 0.10, 1.08),
             "wood_dark")
    rod(p, "winch", (0, 0, 1.20), 0.055, 0.76, "wood", 6,
        (0, math.pi / 2, 0))
    rod(p, "rope", (0.10, 0, 0.84), 0.015, 0.62, "sand", 4)
    rod(p, "bucket", (0.10, 0, 0.42), 0.15, 0.26, "wood", 6)
    rod(p, "bucket_rim", (0.10, 0, 0.55), 0.17, 0.04, "iron", 6)
    for side in (-1, 1):
        slab(p, "roof_%d" % side, (side * 0.22, 0, 1.44),
             (0.50, 0.86, 0.06), "stone_dark")
    slab(p, "ridge", (0, 0, 1.56), (0.08, 0.88, 0.08), "wood")
    return p


FARM = [
    ("wheat", build_wheat, (0.85, 0.80, 1.20), "crop", "grain bundle"),
    ("carrot", build_carrot, (0.90, 0.80, 0.60), "crop", "harvested roots"),
    ("potato", build_potato, (1.10, 1.00, 0.55), "crop", "mounded hill"),
    ("fence_gate", build_fence_gate, (1.30, 0.20, 1.30), "barrier", "picket gate"),
    ("scarecrow", build_scarecrow, (1.40, 0.70, 2.05), "field prop", "straw guardian"),
    ("well_bucket", build_well_bucket, (1.05, 0.90, 1.65), "well variant", "winch + bucket"),
]


# ===========================================================================
# Family C - town furniture
# ===========================================================================
def build_market_goods():
    """Market-stall dressing: crates, sacks, fruit baskets and a coin pile."""
    p = collection("market_goods")
    slab(p, "counter", (0, 0, 0.22), (1.60, 0.70, 0.12), "wood")
    slab(p, "crate", (-0.46, 0, 0.48), (0.46, 0.44, 0.42), "wood_light")
    slab(p, "crate_lid", (-0.46, 0, 0.70), (0.48, 0.46, 0.05), "wood_dark")
    for i, (x, y, r) in enumerate(((0.30, 0.12, 0.19), (0.58, -0.10, 0.17),
                                   (0.10, -0.16, 0.16))):
        lump(p, "sack_%d" % i, (x, y, 0.30 + r * 0.55), r,
             "sand" if i % 2 else "sand_dark", (1.0, 0.9, 1.15))
        rod(p, "tie_%d" % i, (x, y, 0.30 + r * 1.6), 0.04, 0.06, "wood", 5)
    rod(p, "basket", (0.54, 0.20, 0.38), 0.20, 0.22, "wood", 8)
    rod(p, "basket_rim", (0.54, 0.20, 0.50), 0.22, 0.04, "wood_dark", 8)
    for i, (x, y) in enumerate(((-0.04, 0.20), (0.10, 0.26), (0.04, 0.06))):
        lump(p, "fruit_%d" % i, (x, y, 0.34), 0.075,
             "enemy" if i == 1 else "player")
    slab(p, "cloth", (-0.10, -0.24, 0.30), (0.48, 0.30, 0.04), "cloth")
    for i, (x, y) in enumerate(((-0.20, 0.32), (0.02, 0.38))):
        lump(p, "coin_%d" % i, (x, y, 0.31), 0.045, "gold")
    return p


def build_bench():
    """Village bench: two planks, four legs and a pair of stretchers."""
    p = collection("bench")
    for i, y in enumerate((-0.12, 0.12)):
        slab(p, "seat_%d" % i, (0, y, 0.48), (1.50, 0.22, 0.07), "wood")
    for i, y in enumerate((-0.12, 0.12)):
        for j, x in enumerate((-0.60, 0.60)):
            slab(p, "leg_%d_%d" % (i, j), (x, y, 0.23), (0.10, 0.10, 0.46),
                 "wood_dark")
    slab(p, "back", (0, 0.26, 0.80), (1.50, 0.07, 0.40), "wood")
    slab(p, "back_rail", (0, 0.22, 1.02), (1.50, 0.08, 0.07), "wood_light")
    for i, x in enumerate((-0.60, 0.60)):
        slab(p, "stretcher_%d" % i, (x, 0, 0.18), (0.08, 0.42, 0.08),
             "wood_dark")
    return p


def build_street_lamp():
    """3.0 u street lamp: iron post, bracket, glazed box and a warm flame."""
    p = collection("street_lamp")
    slab(p, "foot", (0, 0, 0.08), (0.46, 0.46, 0.16), "stone_dark")
    rod(p, "post", (0, 0, 1.10), 0.07, 2.10, "iron", 6)
    rod(p, "collar", (0, 0, 2.10), 0.11, 0.12, "steel", 6)
    slab(p, "arm", (0.16, 0, 2.24), (0.40, 0.08, 0.08), "iron")
    slab(p, "lamp_base", (0.34, 0, 2.34), (0.30, 0.30, 0.06), "iron")
    for i, (x, y) in enumerate(((-0.13, -0.13), (0.13, -0.13),
                                (-0.13, 0.13), (0.13, 0.13))):
        rod(p, "frame_%d" % i, (0.34 + x, y, 2.58), 0.022, 0.46, "iron", 4)
    lump(p, "glass", (0.34, 0, 2.58), 0.16, "gold", (0.9, 0.9, 1.3))
    tip(p, "flame", (0.34, 0, 2.58), 0.08, 0.0, 0.24, "fire", 5)
    tip(p, "cap", (0.34, 0, 2.88), 0.23, 0.06, 0.18, "iron", 6)
    lump(p, "finial", (0.34, 0, 3.00), 0.06, "gold")
    for side in (-1, 1):
        slab(p, "brace_%d" % side, (side * 0.10, 0, 1.86),
             (0.05, 0.05, 0.30), "iron")
    return p


def build_signpost():
    """Wayfinding post with three arrow boards at staggered heights."""
    p = collection("signpost")
    rod(p, "post", (0, 0, 0.96), 0.075, 1.92, "wood_dark", 6)
    tip(p, "cap", (0, 0, 1.98), 0.10, 0.0, 0.14, "wood", 5)
    for i, (z, direction, length) in enumerate(((1.56, 1, 0.96),
                                                (1.22, -1, 0.82),
                                                (0.88, 1, 0.70))):
        slab(p, "board_%d" % i, (direction * length / 2, 0, z),
             (length, 0.06, 0.20), "wood_light")
        tip(p, "arrow_%d" % i,
            (direction * (length / 2 + 0.08), 0, z), 0.11, 0.0, 0.20,
            "wood", 4, (0, direction * math.pi / 2, 0))
        lump(p, "nail_%d" % i, (direction * 0.10, -0.05, z), 0.03, "iron")
    return p


def build_barrel_stacked():
    """Three stacked barrels plus a small crate — a town-yard variant."""
    p = collection("barrel_stacked")
    for i, (x, y, z, r) in enumerate(((-0.34, 0.02, 0.35, 0.30),
                                      (0.34, 0.04, 0.35, 0.30),
                                      (0, -0.02, 1.02, 0.26))):
        rod(p, "barrel_%d" % i, (x, y, z), r, 0.68 if i < 2 else 0.52,
            "wood", 8)
        for j, h in enumerate((-0.22, 0.22) if i < 2 else (-0.16, 0.16)):
            rod(p, "hoop_%d_%d" % (i, j), (x, y, z + h), r * 1.03, 0.06,
                "iron", 8)
    slab(p, "crate", (0.62, 0.20, 0.20), (0.40, 0.38, 0.38), "wood_light")
    rod(p, "rope", (-0.34, -0.30, 0.30), 0.02, 0.60, "sand", 4)
    return p


def build_barrel_open():
    """Open, water-filled barrel with loose staves and a leaning lid."""
    p = collection("barrel_open")
    rod(p, "body", (0, 0, 0.36), 0.32, 0.72, "wood", 8)
    rod(p, "water", (0, 0, 0.66), 0.27, 0.04, "water", 8)
    for i, h in enumerate((-0.26, 0.26)):
        rod(p, "hoop_%d" % i, (0, 0, 0.36 + h), 0.33, 0.06, "iron", 8)
    for i, a in enumerate((0.2, 1.9, 3.6)):
        slab(p, "stave_%d" % i,
             (math.cos(a) * 0.34, math.sin(a) * 0.34, 0.30),
             (0.06, 0.08, 0.58), "wood_light")
    lid = slab(p, "lid", (0.48, 0.16, 0.24), (0.52, 0.52, 0.06), "wood_dark")
    lid.rotation_euler = (0.20, 0.10, 0.14)
    rod(p, "handle", (0, 0, 0.74), 0.03, 0.16, "iron", 5)
    return p


TOWN = [
    ("market_goods", build_market_goods, (1.70, 0.80, 0.80),
     "stall dressing", "crates, sacks, fruit"),
    ("bench", build_bench, (1.50, 0.65, 1.10), "seat", "two-plank bench"),
    ("street_lamp", build_street_lamp, (0.60, 0.50, 3.05),
     "lighting", "3.0 u lamp"),
    ("signpost", build_signpost, (1.10, 0.20, 2.05), "wayfinding", "three arrows"),
    ("barrel_stacked", build_barrel_stacked, (1.20, 0.90, 1.40),
     "barrel variant", "three stacked"),
    ("barrel_open", build_barrel_open, (1.00, 0.85, 0.80),
     "barrel variant", "open + water"),
]


# ===========================================================================
# Family D - ocean props
# ===========================================================================
def build_anchor():
    """Iron anchor: ring, stock, shank and two curved flukes."""
    p = collection("anchor")
    rod(p, "shank", (0, 0, 0.72), 0.055, 1.24, "iron", 6)
    lump(p, "ring", (0, 0, 1.42), 0.14, "iron", (1.0, 0.30, 1.0))
    rod(p, "stock", (0, 0, 1.20), 0.045, 0.92, "wood_dark", 6,
        (0, math.pi / 2, 0))
    for i, x in enumerate((-0.44, 0.44)):
        lump(p, "stock_end_%d" % i, (x, 0, 1.20), 0.055, "iron")
        rod(p, "arm_%d" % i, (x * 0.55, 0, 0.26), 0.06, 0.70, "iron", 5,
            (0, x * 1.1, 0))
        tip(p, "fluke_%d" % i, (x * 1.15, 0, 0.34), 0.15, 0.0, 0.36,
            "steel", 5, (0, x * 4.5, 0))
    rod(p, "rope", (0.08, 0.10, 1.30), 0.02, 0.52, "sand", 4,
        (0, 0, 0.4))
    return p


def build_buoy():
    """Floating buoy: banded hull, lantern, antenna and two side floats."""
    p = collection("buoy")
    rod(p, "hull", (0, 0, 0.52), 0.30, 0.72, "enemy", 8)
    rod(p, "band", (0, 0, 0.62), 0.31, 0.12, "bone", 8)
    tip(p, "lower", (0, 0, 0.10), 0.30, 0.0, 0.22, "enemy", 8,
        (math.pi, 0, 0))
    tip(p, "shoulder", (0, 0, 0.96), 0.28, 0.08, 0.22, "enemy", 8)
    rod(p, "lantern_base", (0, 0, 1.12), 0.12, 0.08, "iron", 6)
    lump(p, "light", (0, 0, 1.26), 0.13, "gold")
    rod(p, "antenna", (0, 0, 1.52), 0.018, 0.42, "iron", 4)
    slab(p, "flag", (0.12, 0, 1.68), (0.24, 0.02, 0.14), "player")
    for i, x in enumerate((-0.36, 0.36)):
        lump(p, "float_%d" % i, (x, 0, 0.48), 0.13, "sand")
    return p


def build_shipwreck_mast():
    """Snapped mast and yard with a torn sail, rigging and barnacles."""
    p = collection("shipwreck_mast")
    rod(p, "mast", (0, 0, 1.05), 0.13, 2.10, "wood_dark", 8,
        (0.05, 0, 0.06))
    tip(p, "break", (0.10, 0, 2.16), 0.13, 0.05, 0.22, "wood_light", 8)
    rod(p, "yard", (0, 0, 1.66), 0.06, 1.40, "wood", 6,
        (0, math.pi / 2, 0))
    for i, x in enumerate((-0.45, 0.45)):
        slab(p, "sail_%d" % i, (x, -0.02, 1.18), (0.62, 0.04, 0.82),
             "cloth" if i else "bone")
        rod(p, "rigging_%d" % i, (x * 0.9, 0.06, 1.84), 0.014, 0.80,
            "sand", 4, (0, x * 1.2, 0))
    for i, (x, y, z) in enumerate(((-0.28, 0.12, 0.30), (0.26, -0.10, 0.52),
                                   (0.04, 0.22, 1.10))):
        lump(p, "barnacle_%d" % i, (x, y, z), 0.10, "stone_light")
    for i, x in enumerate((-0.30, 0.28)):
        slab(p, "splinter_%d" % i, (x, 0, 0.10), (0.42, 0.12, 0.16),
             "wood_light")
    return p


def build_dock_plank():
    """Floating dock section: three planks on beams with rope and barnacles."""
    p = collection("dock_plank")
    for i, y in enumerate((-0.24, 0, 0.24)):
        slab(p, "plank_%d" % i, (0, y, 0.14), (1.64, 0.22, 0.08),
             "wood" if i % 2 else "wood_light")
    for i, x in enumerate((-0.62, 0.62)):
        slab(p, "beam_%d" % i, (x, 0, 0.06), (0.14, 0.80, 0.12), "wood_dark")
    for i, (x, y) in enumerate(((-0.62, -0.24), (-0.62, 0.24),
                                (0.62, -0.24), (0.62, 0.24))):
        rod(p, "nail_%d" % i, (x, y, 0.19), 0.025, 0.03, "iron", 4)
    lump(p, "rope_loop", (-0.30, 0.34, 0.12), 0.11, "sand",
         (1.3, 1.0, 0.35))
    for i, (x, y) in enumerate(((0.32, 0.28), (-0.02, -0.30))):
        lump(p, "barnacle_%d" % i, (x, y, 0.20), 0.07, "stone_light")
    lump(p, "seaweed", (0.46, 0.10, 0.22), 0.14, "swamp", (1.0, 0.4, 0.35))
    return p


def build_coral_fan():
    """Flat sea fan: a central stem with six splayed branches and polyps."""
    p = collection("coral_fan")
    lump(p, "base", (0, 0, 0.08), 0.28, "stone_dark", (1.2, 1.0, 0.45))
    rod(p, "stem", (0, 0, 0.40), 0.06, 0.60, "coral", 5)
    for i, a in enumerate((-1.2, -0.75, -0.3, 0.3, 0.75, 1.2)):
        x = math.sin(a) * 0.40
        z = 0.52 + math.cos(a) * 0.22
        rod(p, "branch_%d" % i, (x * 0.6, 0, z * 0.72), 0.038, 0.58,
            "coral" if i % 2 else "player", 4, (0, a * 0.8, 0))
        lump(p, "polyp_%d" % i, (x, 0, z), 0.075,
             "player" if i % 2 else "coral")
    return p


def build_coral_brain():
    """Rounded brain coral: golden lobes with ridged, meandering grooves."""
    p = collection("coral_brain")
    lump(p, "base", (0, 0, 0.16), 0.42, "sand_dark", (1.15, 1.05, 0.55))
    for i, (x, y, z, r) in enumerate(((-0.20, 0.04, 0.34, 0.24),
                                      (0.18, 0.10, 0.36, 0.26),
                                      (0.04, -0.20, 0.30, 0.22),
                                      (-0.06, 0.22, 0.28, 0.19),
                                      (0.28, -0.12, 0.26, 0.18))):
        lump(p, "lobe_%d" % i, (x, y, z), r,
             "coral" if i % 2 else "gold", (1.1, 1.0, 0.85))
    for i, (x, y, z, a) in enumerate(((-0.20, -0.04, 0.52, 0.3),
                                      (0.16, 0.10, 0.56, -0.4),
                                      (0.04, -0.20, 0.46, 0.8),
                                      (-0.04, 0.20, 0.42, 0.1))):
        ridge = slab(p, "ridge_%d" % i, (x, y, z), (0.30, 0.05, 0.04),
                     "sand_dark")
        ridge.rotation_euler.z = a
    for i, x in enumerate((-0.30, 0.30)):
        lump(p, "polyp_%d" % i, (x, 0.08, 0.24), 0.07, "player")
    return p


OCEAN = [
    ("anchor", build_anchor, (1.10, 0.35, 1.60), "mooring", "ring + twin flukes"),
    ("buoy", build_buoy, (0.90, 0.70, 1.80), "navigation", "lantern buoy"),
    ("shipwreck_mast", build_shipwreck_mast, (1.60, 0.60, 2.30),
     "wreck", "snapped mast + sail"),
    ("dock_plank", build_dock_plank, (1.70, 0.85, 0.25), "docking", "plank section"),
    ("coral_fan", build_coral_fan, (1.05, 0.35, 1.10), "coral variant", "sea fan"),
    ("coral_brain", build_coral_brain, (1.10, 1.00, 0.70),
     "coral variant", "ridged brain"),
]


# ===========================================================================
# Family E - four more NPC variants (player = 1.55 u)
# ===========================================================================
def build_npc_blacksmith():
    """Broad aproned smith with a raised hammer and tongs (~1.80 u)."""
    p = collection("npc_blacksmith")
    for i, x in enumerate((-0.18, 0.18)):
        rod(p, "leg_%d" % i, (x, 0, 0.36), 0.11, 0.72, "wood_dark", 5)
        slab(p, "boot_%d" % i, (x, -0.07, 0.07), (0.22, 0.32, 0.14),
             "leather" if "leather" in P else "wood_dark")
    slab(p, "torso", (0, 0, 1.02), (0.78, 0.44, 0.72), "enemy")
    slab(p, "apron", (0, -0.24, 0.86), (0.64, 0.06, 0.72), "wood_dark")
    slab(p, "belt", (0, 0, 0.68), (0.80, 0.46, 0.10), "leather" if "leather" in P else "wood_dark")
    for i, x in enumerate((-0.48, 0.48)):
        rod(p, "arm_%d" % i, (x, 0, 1.05), 0.10, 0.60,
            "skin", 5, (0, -0.25 if i == 0 else 0.25, 0))
        lump(p, "hand_%d" % i, (x * 1.12, -0.04, 1.32), 0.10, "skin")
    lump(p, "head", (0, -0.02, 1.54), 0.20, "skin")
    lump(p, "hair", (0, 0.05, 1.68), 0.20, "hair", (1.0, 1.0, 0.55))
    lump(p, "beard", (0, -0.16, 1.38), 0.15, "hair", (0.9, 0.6, 1.0))
    rod(p, "hammer_handle", (0.62, -0.06, 1.42), 0.035, 0.62, "wood", 5,
        (0.10, 0, 0.28))
    slab(p, "hammer_head", (0.76, -0.10, 1.70), (0.28, 0.16, 0.16), "iron")
    for i, x in enumerate((-0.60, -0.50)):
        rod(p, "tongs_%d" % i, (x, -0.14, 0.86), 0.022, 0.52, "iron", 4,
            (0, 0, 0.10 if i else -0.10))
    return p


def build_npc_innkeeper():
    """Stout host with an apron, tray and foaming mug (~1.66 u)."""
    p = collection("npc_innkeeper")
    for i, x in enumerate((-0.16, 0.16)):
        rod(p, "leg_%d" % i, (x, 0, 0.30), 0.10, 0.60, "wood_dark", 5)
    lump(p, "belly", (0, 0, 0.88), 0.37, "player", (1.05, 0.85, 1.15))
    slab(p, "apron", (0, -0.30, 0.84), (0.56, 0.06, 0.76), "bone")
    slab(p, "belt", (0, 0, 0.62), (0.66, 0.42, 0.10), "leather" if "leather" in P else "wood_dark")
    for i, x in enumerate((-0.42, 0.42)):
        rod(p, "arm_%d" % i, (x, 0, 0.96), 0.09, 0.52, "player", 5,
            (0, -0.30 if i == 0 else 0.22, 0))
    lump(p, "head", (0, -0.02, 1.36), 0.19, "skin")
    lump(p, "cap", (0, 0.02, 1.50), 0.21, "cloth", (1.0, 1.05, 0.55))
    lump(p, "mustache", (0, -0.17, 1.29), 0.09, "hair", (1.4, 0.5, 0.5))
    slab(p, "tray", (0.50, -0.18, 1.22), (0.36, 0.28, 0.05), "wood_light")
    for i, x in enumerate((0.42, 0.58)):
        rod(p, "mug_%d" % i, (x, -0.18, 1.32), 0.075, 0.16, "wood", 6)
        lump(p, "foam_%d" % i, (x, -0.18, 1.41), 0.07, "bone")
    return p


def build_npc_child_urchin():
    """Small ragged child with an oversized patched cap and a walk-stick."""
    p = collection("npc_child_urchin")
    for i, x in enumerate((-0.11, 0.11)):
        rod(p, "leg_%d" % i, (x, 0, 0.17), 0.06, 0.34, "skin", 4)
        slab(p, "shoe_%d" % i, (x, -0.05, 0.05), (0.14, 0.22, 0.10),
             "wood_dark")
    slab(p, "smock", (0, 0, 0.48), (0.46, 0.32, 0.48), "cloth")
    for i, x in enumerate((-0.28, 0.28)):
        rod(p, "arm_%d" % i, (x, 0, 0.52), 0.055, 0.38, "skin", 4,
            (0, -0.14 if i == 0 else 0.14, 0))
    lump(p, "head", (0, -0.02, 0.90), 0.21, "skin")
    tip(p, "cap", (0.02, 0, 1.10), 0.24, 0.14, 0.30, "wood_dark", 6)
    for i, x in enumerate((-0.08, 0.08)):
        lump(p, "eye_%d" % i, (x, -0.19, 0.91), 0.03, "stone_dark")
    slab(p, "patch", (0.17, -0.16, 0.48), (0.14, 0.04, 0.14), "player")
    slab(p, "satchel", (-0.30, 0.12, 0.48), (0.22, 0.16, 0.26), "wood")
    slab(p, "strap", (-0.16, -0.12, 0.60), (0.05, 0.05, 0.42), "wood_dark")
    rod(p, "stick", (0.36, -0.02, 0.42), 0.022, 0.76, "wood_light", 4,
        (0, 0, 0.10))
    return p


def build_npc_traveling_scholar():
    """Scholar in a wide hat with satchel, book, scroll and quill (~1.70 u)."""
    p = collection("npc_traveling_scholar")
    for i, x in enumerate((-0.15, 0.15)):
        rod(p, "leg_%d" % i, (x, 0, 0.32), 0.085, 0.64, "wood_dark", 5)
    tip(p, "robe", (0, 0, 0.72), 0.31, 0.22, 0.68, "cloth", 6)
    slab(p, "torso", (0, 0, 1.12), (0.56, 0.36, 0.60), "cloth")
    for i, x in enumerate((-0.35, 0.35)):
        rod(p, "arm_%d" % i, (x, 0, 1.08), 0.075, 0.54,
            "cloth", 5, (0, -0.18 if i == 0 else 0.18, 0))
    lump(p, "head", (0, -0.02, 1.55), 0.18, "skin")
    lump(p, "beard", (0, -0.15, 1.41), 0.13, "bone", (0.9, 0.6, 1.0))
    rod(p, "hat_brim", (0, 0, 1.72), 0.30, 0.04, "wood_dark", 6)
    tip(p, "hat_crown", (0, 0, 1.84), 0.16, 0.13, 0.24, "wood_dark", 6)
    for i, x in enumerate((-0.08, 0.08)):
        lump(p, "lens_%d" % i, (x, -0.17, 1.56), 0.06, "steel")
    slab(p, "satchel", (-0.38, 0.14, 0.78), (0.28, 0.20, 0.32), "wood")
    slab(p, "book", (0.34, -0.16, 0.98), (0.28, 0.20, 0.06), "enemy")
    slab(p, "page", (0.34, -0.17, 1.02), (0.24, 0.17, 0.02), "bone")
    rod(p, "scroll", (0.40, 0.10, 1.14), 0.055, 0.28, "bone", 6,
        (math.pi / 2, 0, 0))
    tip(p, "quill", (-0.40, -0.10, 1.30), 0.04, 0.0, 0.36, "snow", 4,
        (0, 0, 0.35))
    return p


NPCS = [
    ("npc_blacksmith", build_npc_blacksmith, (1.25, 0.60, 1.85),
     "town", "apron + raised hammer"),
    ("npc_innkeeper", build_npc_innkeeper, (1.05, 0.70, 1.65),
     "tavern", "stout host + tray"),
    ("npc_child_urchin", build_npc_child_urchin, (0.75, 0.48, 1.25),
     "street", "ragged child + stick"),
    ("npc_traveling_scholar", build_npc_traveling_scholar, (1.05, 0.70, 1.95),
     "traveler", "wide hat + book"),
]


# ---------------------------------------------------------------------------
ASSETS = DUNGEON + FARM + TOWN + OCEAN + NPCS
EXPORTS = [name for name, _builder, _target, _family, _notes in ASSETS]

for name, builder, target, family, notes in ASSETS:
    col = builder()
    is_hero = name == "dungeon_cell_interior"
    finalize(name, col, "models_v3", target, "%s; %s" % (family, notes),
             hero=is_hero, bevel_width=0.005,
             bevel_parts=("floor_slab", "sarcophagus_lid") if is_hero else None,
             records=RECORDS)
    SETS.append((name, col))

write_records("models_v3", RECORDS)
print("WAVE3: %d assets, %.2f u player reference" % (len(RECORDS), PLAYER))
try:
    contact_sheet(SETS, "models_v3.png", width=2600, height=560)
except Exception as exc:  # previews are optional; never fail the build for them
    print("PREVIEW SKIPPED", exc)
