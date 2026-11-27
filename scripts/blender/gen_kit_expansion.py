"""Starfall asset expansion — family 3: modular kit + two complete interiors.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_kit_expansion.py

Eight modular pieces snap to the 1 u grid (1 u frontage, 2.5 u storey,
0.9 x 2.0 u door, 0.8 x 0.9 u window). The very same piece builders assemble
`blacksmith_hall_interior` and `temple_hall_interior`, so openings, wall
thickness and storey height stay consistent across the kit.
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import new_scene, box, cyl, cone, ico, contact_sheet
from expansion_common import P, collection, finalize, write_records
import family_repairs as R

new_scene()
RM = R.materials()
RECORDS = []
SETS = []
STOREY = 2.5
DOOR_W, DOOR_H = 0.9, 2.0
WINDOW_W, WINDOW_H = 0.8, 0.9
SILL = 1.1  # window base lifted at runtime: 1.1 -> 2.0, inside the 2.5 u storey


class Frame:
    """Local-to-world placement for a kit piece (central origin, base z=0).

    ``z0`` lifts an opening to its sill height; ``rot`` rotates a wall to run
    along Z; ``prefix`` keeps part names unique in assembled interiors.
    """

    def __init__(self, col, cx=0.0, cy=0.0, rot=0.0, z0=0.0, prefix=""):
        self.col, self.cx, self.cy = col, cx, cy
        self.c, self.s = math.cos(rot), math.sin(rot)
        self.rot, self.z0, self.prefix = rot, z0, prefix

    def _loc(self, p):
        x, y, z = p
        return (self.cx + x * self.c - y * self.s,
                self.cy + x * self.s + y * self.c, self.z0 + z)

    def _name(self, name):
        return (self.prefix + "_" + name) if self.prefix else name

    def box(self, name, p, scale, material, ry=0.0):
        o = box(self._name(name), self._loc(p), scale, material, self.col)
        o.rotation_euler = (0, ry, self.rot)
        return o

    def cyl(self, name, p, radius, depth, material, vertices=6, rot=(0, 0, 0)):
        o = cyl(self._name(name), self._loc(p), radius, depth, material,
                self.col, vertices)
        o.rotation_euler = (rot[0], rot[1], rot[2] + self.rot)
        return o

    def cone(self, name, p, r1, r2, depth, material, vertices=5, rot=(0, 0, 0)):
        o = cone(self._name(name), self._loc(p), r1, r2, depth, material,
                 self.col, vertices)
        o.rotation_euler = (rot[0], rot[1], rot[2] + self.rot)
        return o

    def ico(self, name, p, radius, material, scale=None):
        o = ico(self._name(name), self._loc(p), radius, material, self.col,
                0, scale)
        o.rotation_euler.z = self.rot
        return o


def place(col, builder, cx=0, cy=0, rot=0, z0=0, prefix=""):
    builder(Frame(col, cx, cy, rot, z0, prefix))


# ---------------------------------------------------------------------------
# Modular pieces — the single source of truth for both interiors
# ---------------------------------------------------------------------------
def build_floor(f):
    """Exactly one 1x1 tile: four 0.25-deep planks, top at 0.1."""
    for i in range(4):
        y = -0.375 + i * 0.25
        f.box("plank_%d" % i, (0, y, 0.05), (1.0, 0.25, 0.10),
              P["wood"] if i % 2 == 0 else P["wood_light"])


def build_wall_wood(f):
    """1 u frontage, 0.15 thick, 2.5 u tall — timber frame wainscot."""
    f.box("wainscot", (0, 0, 0.40), (1.0, 0.15, 0.80), P["wood"])
    f.box("rail", (0, 0, 0.825), (1.0, 0.15, 0.05), P["wood_dark"])
    f.box("plaster", (0, 0, 1.675), (1.0, 0.15, 1.65), P["sand"])


def build_wall_stone(f):
    """1 u frontage, 0.18 thick, 2.5 u tall — three stone courses."""
    f.box("course_low", (0, 0, 0.35), (1.0, 0.18, 0.70), P["stone_dark"])
    f.box("course_mid", (0, 0, 1.25), (1.0, 0.18, 1.10), P["stone"])
    f.box("course_top", (0, 0, 2.15), (1.0, 0.18, 0.70), P["stone_light"])


def build_wall_wood_window(f):
    """Same 2.5 u wall, but with a real 0.8 x 0.9 opening at 1.1-2.0 u."""
    f.box("wainscot", (0, 0, 0.40), (1.0, 0.15, 0.80), P["wood"])
    f.box("rail", (0, 0, 0.825), (1.0, 0.15, 0.05), P["wood_dark"])
    f.box("sill_band", (0, 0, 0.975), (1.0, 0.15, 0.25), P["sand"])
    for i, x in enumerate((-0.45, 0.45)):
        f.box("jamb_%d" % i, (x, 0, 1.55), (0.10, 0.15, 0.90), P["sand"])
    f.box("head_band", (0, 0, 2.25), (1.0, 0.15, 0.50), P["sand"])


def build_wall_stone_window(f):
    """Stone equivalent: solid below 1.1 and above 2.0, open in between."""
    f.box("course_low", (0, 0, 0.55), (1.0, 0.18, 1.10), P["stone_dark"])
    f.box("course_top", (0, 0, 2.25), (1.0, 0.18, 0.50), P["stone_light"])
    for i, x in enumerate((-0.45, 0.45)):
        f.box("side_%d" % i, (x, 0, 1.55), (0.10, 0.18, 0.90), P["stone"])


def build_door(f):
    """0.9 wide x 2.0 tall, centred on X, base at 0. Knob adds ~0.02 depth."""
    for i, x in enumerate((-0.41, 0.41)):
        f.box("jamb_%d" % i, (x, 0, 1.0), (0.08, 0.12, DOOR_H), P["wood"])
    f.box("lintel", (0, 0, 1.95), (DOOR_W, 0.12, 0.10), P["wood"])
    f.box("leaf", (0, 0, 0.925), (0.74, 0.09, 1.85), P["wood_dark"])
    f.box("panel_top", (0, -0.052, 1.35), (0.56, 0.01, 0.50), P["wood"])
    f.box("panel_bottom", (0, -0.052, 0.55), (0.56, 0.01, 0.50), P["wood"])
    f.ico("knob", (0.27, -0.06, 0.95), 0.045, P["gold"])


def build_window(f):
    """0.8 wide x 0.9 tall, 0.10 deep, centred on X, base at 0."""
    f.box("glass", (0, 0, 0.45), (0.60, 0.04, 0.70), P["water"])
    for i, x in enumerate((-0.35, 0.35)):
        f.box("jamb_%d" % i, (x, 0, 0.45), (0.10, 0.10, WINDOW_H), P["wood"])
    for i, z in enumerate((0.05, 0.85)):
        f.box("rail_%d" % i, (0, 0, z), (WINDOW_W, 0.10, 0.10), P["wood"])
    f.box("mullion_v", (0, -0.045, 0.45), (0.04, 0.01, 0.70), P["wood"])
    f.box("mullion_h", (0, -0.045, 0.45), (0.60, 0.01, 0.04), P["wood"])


def build_stairs(f):
    """1 u wide, six 0.25 u treads running 1.5 u and rising 1.5 u."""
    for i in range(6):
        h = (i + 1) * 0.25
        y = -0.625 + i * 0.25
        f.box("riser_%d" % i, (0, y, (h - 0.03) / 2),
              (1.0, 0.25, h - 0.03), P["wood"])
        f.box("tread_%d" % i, (0, y, h - 0.015), (1.0, 0.25, 0.03),
              P["wood_light"])


def build_roof_gable(f):
    """One tiled roof cell: 1x1 footprint, ridge along Y, 0.55 u rise."""
    angle = math.atan2(0.55, 0.5)
    for side in (-1, 1):
        f.box("slope_%d" % side, (side * 0.255, 0, 0.275),
              (0.60, 1.0, 0.06), P["stone_dark"], ry=side * angle)
        f.box("eave_%d" % side, (side * 0.47, 0, 0.035),
              (0.08, 1.0, 0.07), P["wood"])
    f.box("ridge", (0, 0, 0.56), (0.10, 1.0, 0.08), P["stone_light"])
    f.box("tie_beam", (0, 0, 0.04), (0.94, 1.0, 0.06), P["wood"])


def build_pillar(f):
    """0.3 sq column, 2.5 u tall — matches one storey."""
    f.box("base", (0, 0, 0.075), (0.30, 0.30, 0.15), P["stone_dark"])
    f.cyl("shaft", (0, 0, 1.30), 0.11, 2.30, P["stone"], 6)
    f.box("band", (0, 0, 2.18), (0.26, 0.26, 0.08), P["stone_light"])
    f.box("capital", (0, 0, 2.45), (0.30, 0.30, 0.10), P["stone_light"])


def kit_wall_wood_qa(f):
    """Authoritative detailed standalone kit wall (interiors use the cheap one)."""
    R.build_kit_wall_wood(f.col, RM)


def kit_wall_stone_qa(f):
    R.build_kit_wall_stone(f.col, RM)


KIT = [
    ("kit_floor",       build_floor,      (1.00, 1.00, 0.10), "one tile", "4 planks"),
    ("kit_wall_wood",   kit_wall_wood_qa, (1.00, 0.15, 2.50), "1 u frontage", "bevelled planks + brace"),
    ("kit_wall_stone",  kit_wall_stone_qa, (1.00, 0.18, 2.50), "1 u frontage", "staggered stone courses"),
    ("kit_door",        build_door,       (0.90, 0.12, 2.00), "0.9 x 2.0", " centred on X"),
    ("kit_window",      build_window,     (0.80, 0.10, 0.90), "0.8 x 0.9", "raise to 1.1 sill"),
    ("kit_stairs",      build_stairs,     (1.00, 1.50, 1.50), "1 u wide", "6 treads"),
    ("kit_roof_gable",  build_roof_gable, (1.00, 1.00, 0.65), "one roof cell", "ridge along Y"),
    ("kit_pillar",      build_pillar,     (0.30, 0.30, 2.50), "one storey", "column"),
]


# ---------------------------------------------------------------------------
# Interior props (non-kit dressing, assembled into the same collections)
# ---------------------------------------------------------------------------
def forge(f):
    f.box("forge_base", (0, 0, 0.10), (1.6, 0.7, 0.20), P["stone_dark"])
    f.box("forge_back", (0, 0.28, 0.85), (1.6, 0.18, 1.50), P["stone"])
    for x in (-0.68, 0.68):
        f.box("forge_pier_%d" % x, (x, 0.02, 0.70), (0.24, 0.66, 1.20), P["stone"])
    f.box("forge_hood", (0, 0.20, 1.42), (1.6, 0.60, 0.18), P["stone_dark"])
    f.box("forge_mouth", (0, 0.32, 0.52), (1.00, 0.03, 0.80), P["wood_dark"])
    f.cone("forge_flame", (0, 0.22, 0.62), 0.34, 0.0, 0.82, P["fire"], 5)
    f.ico("forge_ember", (-0.18, 0.22, 0.36), 0.12, P["gold"])


def anvil(f):
    f.cyl("anvil_stump", (0, 0, 0.24), 0.22, 0.48, P["wood"], 6)
    f.box("anvil_base", (0, 0, 0.53), (0.42, 0.30, 0.10), P["iron"])
    f.box("anvil_waist", (0, 0, 0.65), (0.24, 0.18, 0.16), P["iron"])
    f.box("anvil_top", (0, 0, 0.78), (0.52, 0.22, 0.10), P["steel"])
    f.cone("anvil_horn", (0.40, 0, 0.78), 0.11, 0.0, 0.30, P["steel"], 5,
           rot=(0, -math.pi / 2, 0))


def bellows(f):
    f.box("bellows_board", (0, 0, 0.30), (0.60, 0.42, 0.08), P["wood"])
    f.box("bellows_fold", (0, 0, 0.44), (0.56, 0.38, 0.18), P["leather"] if "leather" in P else P["wood_dark"])
    f.box("bellows_top", (0, 0, 0.56), (0.62, 0.44, 0.08), P["wood"])
    f.cyl("bellows_nozzle", (0.36, -0.10, 0.44), 0.06, 0.34, P["iron"], 5,
          rot=(0, math.pi / 2, 0))


def tool_rack(f):
    f.box("rack_back", (0, 0, 1.00), (1.2, 0.10, 0.60), P["wood"])
    for i, x in enumerate((-0.42, 0, 0.42)):
        f.box("tool_%d" % i, (x, -0.10, 1.05), (0.08, 0.08, 0.68),
              P["steel"] if i != 1 else P["iron"])


def coal_pile(f):
    for i, (x, y, r) in enumerate(((-0.20, 0, 0.24), (0.18, 0.10, 0.20),
                                   (0.02, -0.20, 0.16))):
        f.ico("coal_%d" % i, (x, y, r), r,
              P["charcoal"] if "charcoal" in P else P["stone_dark"])


def trough(f):
    f.box("trough_base", (0, 0, 0.12), (1.2, 0.50, 0.24), P["wood_dark"])
    f.box("trough_water", (0, 0, 0.25), (1.0, 0.36, 0.04), P["water"])


def altar(f):
    """Tiered plinth, raised glowing idol and a single clear fire bowl."""
    f.box("altar_base", (0, 0, 0.15), (1.4, 0.8, 0.30), P["stone_dark"])
    f.box("altar_plinth", (0, 0, 0.41), (1.1, 0.7, 0.22), P["stone_light"])
    f.box("altar_shaft", (0, 0, 0.82), (1.0, 0.6, 0.62), P["stone"])
    f.box("altar_slab", (0, 0, 1.19), (1.5, 0.9, 0.16), P["stone_light"])
    f.ico("altar_idol", (0, -0.02, 1.45), 0.22, P["arcane"])
    f.cone("altar_flame", (0.42, 0, 1.34), 0.20, 0.0, 0.46, P["fire"], 5)


def brazier(f):
    f.cyl("brazier_leg", (0, 0, 0.40), 0.06, 0.80, P["iron"], 6)
    f.cyl("brazier_bowl", (0, 0, 0.86), 0.24, 0.14, P["stone_dark"], 6)
    f.cone("brazier_flame", (0, 0, 1.10), 0.16, 0.0, 0.42, P["fire"], 5)


def banner(f):
    f.box("banner_rod", (0, 0, 2.10), (0.9, 0.06, 0.06), P["gold"])
    f.box("banner_cloth", (0, 0, 1.62), (0.72, 0.04, 0.92), P["player"])


def steps(f):
    for i in range(2):
        f.box("step_%d" % i, (0, -0.375 + i * 0.25, (0.1 * (i + 1)) / 2),
              (1.0, 0.25, 0.1 * (i + 1)), P["stone"])


# ---------------------------------------------------------------------------
# Assembled rooms — every wall/opening comes from the KIT builders above
# ---------------------------------------------------------------------------
def wall_centres(h):
    """Centres of 1 u segments along a boundary at +/- h.

    A 5-tile span (h = 2.5) uses integer centres -2..2; a 4-tile span
    (h = 2.0) uses half-integer centres -1.5..1.5. Both fill the edge exactly.
    """
    if abs(h - round(h)) < 1e-6:
        n = int(round(h))
        return [i + 0.5 for i in range(-n, n)]
    n = int(h - 0.5)
    return [float(i) for i in range(-n, n + 1)]


def walls_around(col, builder, hx, hy, skip_south_center=False,
                 window_builder=None, window_positions=()):
    """Place 1 u wall segments on the four boundary grid lines.

    Segments listed in ``window_positions`` use ``window_builder``, which cuts
    a real 0.8 x 0.9 opening for the matching ``kit_window`` piece.
    """
    for y in (-hy, hy):
        for x in wall_centres(hx):
            if skip_south_center and y == -hy and x == 0:
                continue
            which = window_builder if (x, y) in window_positions else builder
            place(col, which, x, y, prefix="wall_x%s_%s" % (x, y))
    for x in (-hx, hx):
        for y in wall_centres(hy):
            which = window_builder if (x, y) in window_positions else builder
            place(col, which, x, y, rot=math.pi / 2,
                  prefix="wall_y%s_%s" % (x, y))


def build_blacksmith_hall_interior():
    p = collection("blacksmith_hall_interior")
    hx, hy = 2.5, 2.0  # 5 x 4 tiles, boundaries on the grid lines
    Frame(p).box("floor_slab", (0, 0, 0.05), (5.0, 4.0, 0.10), P["wood"])
    windows = ((-1, hy), (1, hy))
    walls_around(p, build_wall_wood, hx, hy, skip_south_center=True,
                 window_builder=build_wall_wood_window,
                 window_positions=windows)
    place(p, build_door, 0, -hy, prefix="door")
    Frame(p).box("door_transom", (0, -hy, 2.25), (0.9, 0.15, 0.50), P["wood"])
    for i, (x, y) in enumerate(windows):
        place(p, build_window, x, y, z0=SILL, prefix="window_%d" % i)
    place(p, forge, 0, 1.35)
    place(p, anvil, -0.80, 0.35)
    place(p, bellows, 0.85, 0.15)
    place(p, tool_rack, 2.32, 0.90, rot=math.pi / 2)
    place(p, coal_pile, 1.20, 0.35)
    place(p, trough, -1.45, 1.05)
    return p


def build_temple_hall_interior():
    p = collection("temple_hall_interior")
    h = 2.5  # 5 x 5 tiles
    Frame(p).box("floor_slab", (0, 0, 0.05), (5.0, 5.0, 0.10), P["stone_light"])
    windows = ((-1.5, h), (1.5, h))
    walls_around(p, build_wall_stone, h, h, skip_south_center=True,
                 window_builder=build_wall_stone_window,
                 window_positions=windows)
    place(p, build_door, 0, -h, prefix="door")
    Frame(p).box("door_transom", (0, -h, 2.25), (0.9, 0.18, 0.50), P["stone"])
    for i, (x, y) in enumerate(windows):
        place(p, build_window, x, y - 0.10, z0=SILL,
              prefix="window_%d" % i)
    for i, (x, y) in enumerate(((-1.5, -1.0), (1.5, -1.0),
                                (-1.5, 1.0), (1.5, 1.0))):
        place(p, build_pillar, x, y, prefix="pillar_%d" % i)
    place(p, steps, 0, -1.90)
    place(p, altar, 0, 1.70)
    place(p, brazier, -1.85, 1.20)
    place(p, brazier, 1.85, 1.20)
    place(p, banner, -2.32, 0.20, rot=math.pi / 2)
    return p


# Explicit export list: verify_models.py uses this to group the whole family
# under one category instead of falling back to per-name heuristics.
EXPORTS = [
    "kit_floor", "kit_wall_wood", "kit_wall_stone", "kit_door", "kit_window",
    "kit_stairs", "kit_roof_gable", "kit_pillar",
    "blacksmith_hall_interior", "temple_hall_interior",
]


def kit_builder(name, builder):
    """Wrap a Frame-based piece builder into a zero-arg asset builder."""
    def build():
        col = collection(name)
        place(col, builder)
        return col
    return build


ASSETS = [
    (name, kit_builder(name, builder), target, "grid kit", notes)
    for name, builder, target, _grid, notes in KIT
] + [
    ("blacksmith_hall_interior", build_blacksmith_hall_interior,
     (5.15, 4.15, 2.50), "interior", "hero; 5x4 tiles, forge + 0.9x2.0 door"),
    ("temple_hall_interior", build_temple_hall_interior,
     (5.20, 5.20, 2.50), "interior", "hero; 5x5 tiles, colonnade + altar"),
]

for name, builder, target, family, notes in ASSETS:
    col = builder()
    is_hero = name.endswith("_interior")
    finalize(name, col, "kit_expansion", target, "%s; %s" % (family, notes),
             hero=is_hero, bevel_width=0.005,
             bevel_parts=("anvil_top", "altar_slab", "door_leaf") if is_hero else None,
             records=RECORDS)
    SETS.append((name, col))

write_records("kit_expansion", RECORDS)
try:
    contact_sheet(SETS, "kit_expansion.png", width=1900, height=360)
except Exception as exc:
    print("PREVIEW SKIPPED", exc)
