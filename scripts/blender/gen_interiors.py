"""Starfall — low-poly house interior kit generator.

Run with:
    ~/.local/blender/blender -b --python scripts/blender/gen_interiors.py

All assets are flat-shaded, palette-only (materials built from C), based at
z = 0 and centred on (0, 0). One collection per asset, exported as its own GLB.
`house_interior` is an assembled 4x4-tile example room made from the same
piece builders, showing how the kit snaps together on the 1 u grid.
"""
import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from common import *

new_scene()

# ---------------------------------------------------------------------------
# Palette materials — every colour comes from C, no textures.
# ---------------------------------------------------------------------------
M = {
    "wood":    mat("wood", C["wood"]),
    "leather": mat("leather", C["leather"]),
    "sand":    mat("sand", C["sand"]),
    "stone":   mat("stone", C["stone"]),
    "iron":    mat("iron", C["iron"], metallic=0.3),
    "steel":   mat("steel", C["steel"], metallic=0.6, rough=0.5),
    "gold":    mat("gold", C["gold"], metallic=0.55, rough=0.45),
    "cloth":   mat("cloth", C["cloth"]),
    "bone":    mat("bone", C["bone"]),
    "player":  mat("player", C["player"]),
    "water":   mat("water", C["water"], rough=0.35),
    "fire":    mat("fire", C["fire"], emission=C["fire"], strength=3.0),
    "ember":   mat("ember", C["gold"], emission=C["fire"], strength=2.0),
}


# ---------------------------------------------------------------------------
# Placement frame: build every piece at local (0, 0), then place at (cx, cy)
# with an optional Z rotation. Used for both standalone assets and the room.
# ---------------------------------------------------------------------------
class Piece:
    def __init__(self, col, cx=0.0, cy=0.0, rot=0.0):
        self.col = col
        self.cx, self.cy = cx, cy
        self.c, self.s = math.cos(rot), math.sin(rot)
        self.rot = rot

    def _loc(self, p):
        x, y, z = p
        return (self.cx + x * self.c - y * self.s,
                self.cy + x * self.s + y * self.c, z)

    def box(self, name, p, scale, material):
        o = box(name, self._loc(p), scale, material, self.col)
        o.rotation_euler.z += self.rot
        return o

    def cyl(self, name, p, radius, depth, material, vertices=8, rotation=None):
        o = cyl(name, self._loc(p), radius, depth, material, self.col, vertices)
        if rotation:
            o.rotation_euler = rotation
        o.rotation_euler.z += self.rot
        return o

    def cone(self, name, p, r1, r2, depth, material, vertices=8, rotation=None):
        o = cone(name, self._loc(p), r1, r2, depth, material, self.col,
                 vertices, rotation)
        o.rotation_euler.z += self.rot
        return o

    def ico(self, name, p, radius, material, subdivisions=0, scale=None):
        o = ico(name, self._loc(p), radius, material, self.col, subdivisions, scale)
        o.rotation_euler.z += self.rot
        return o


def tris(col):
    n = 0
    for o in col.objects:
        if o.type == "MESH":
            o.data.calc_loop_triangles()
            n += len(o.data.loop_triangles)
    return n


# ---------------------------------------------------------------------------
# Piece builders — all coordinates are local, base at z = 0.
# ---------------------------------------------------------------------------
def build_floor(p):
    """1.0 x 1.0 tile, 0.1 thick — four 0.25-deep planks."""
    for i in range(4):
        y = -0.375 + i * 0.25
        m = M["wood"] if i % 2 == 0 else M["leather"]
        p.box("plank_%d" % i, (0, y, 0.05), (1.0, 0.25, 0.1), m)


def build_wall(p):
    """1.0 wide x 2.5 tall x 0.15 thick — wainscot, rail, plaster."""
    p.box("wainscot", (0, 0, 0.40), (1.0, 0.15, 0.80), M["wood"])
    p.box("rail", (0, 0, 0.825), (1.0, 0.15, 0.05), M["leather"])
    p.box("plaster", (0, 0, 1.675), (1.0, 0.15, 1.65), M["bone"])


def build_door(p):
    """0.9 wide x 2.0 tall, ~0.12 deep (knob protrudes slightly)."""
    p.box("jamb_l", (-0.41, 0, 1.0), (0.08, 0.12, 2.0), M["wood"])
    p.box("jamb_r", (0.41, 0, 1.0), (0.08, 0.12, 2.0), M["wood"])
    p.box("lintel", (0, 0, 1.95), (0.9, 0.12, 0.1), M["wood"])
    p.box("leaf", (0, 0, 0.925), (0.74, 0.09, 1.85), M["leather"])
    p.box("panel_top", (0, -0.052, 1.35), (0.56, 0.01, 0.5), M["wood"])
    p.box("panel_bot", (0, -0.052, 0.55), (0.56, 0.01, 0.5), M["wood"])
    p.ico("knob", (0.27, -0.05, 0.95), 0.045, M["gold"], 0)


def build_window(p):
    """0.8 wide x 0.9 tall framed window, ~0.10 deep."""
    p.box("glass", (0, 0, 0.45), (0.6, 0.04, 0.7), M["water"])
    for x in (-0.35, 0.35):
        p.box("jamb", (x, 0, 0.45), (0.1, 0.1, 0.9), M["wood"])
    for z in (0.05, 0.85):
        p.box("rail", (0, 0, z), (0.8, 0.1, 0.1), M["wood"])
    p.box("mullion_v", (0, -0.045, 0.45), (0.04, 0.01, 0.7), M["wood"])
    p.box("mullion_h", (0, -0.045, 0.45), (0.6, 0.01, 0.04), M["wood"])


def build_stairs(p):
    """1.0 wide, 1.5 rise — six 0.25 steps, 1.5 deep overall."""
    for i in range(6):
        h = (i + 1) * 0.25
        y = -0.625 + i * 0.25
        p.box("step_%d" % i, (0, y, (h - 0.03) / 2),
              (1.0, 0.25, h - 0.03), M["wood"])
        p.box("tread_%d" % i, (0, y, h - 0.015), (1.0, 0.25, 0.03), M["sand"])


def build_fireplace(p):
    """1.2 tall stone hearth with an emissive firebox, ~1.0 x 0.5 footprint."""
    p.box("hearth", (0, 0, 0.05), (1.0, 0.5, 0.1), M["stone"])
    p.box("back", (0, 0.18, 0.60), (1.0, 0.14, 1.0), M["stone"])
    for x in (-0.41, 0.41):
        p.box("pillar", (x, 0, 0.55), (0.18, 0.5, 1.0), M["stone"])
    p.box("mantel", (0, 0, 1.10), (1.0, 0.5, 0.2), M["stone"])
    p.box("firebox", (0, 0.185, 0.50), (0.64, 0.02, 0.8), M["leather"])
    for i, x in enumerate((-0.12, 0.12)):
        p.cyl("log_%d" % i, (x, -0.05, 0.18), 0.07, 0.5, M["wood"], 6,
              rotation=(0, math.pi / 2, 0))
    p.cone("flame_a", (-0.12, -0.02, 0.50), 0.16, 0, 0.42, M["fire"], 6)
    p.cone("flame_b", (0.13, 0.02, 0.42), 0.13, 0, 0.34, M["ember"], 6)


def build_bed(p):
    """2.0 (X) x 1.0 (Y), 0.8 tall at the headboard."""
    for x in (-0.92, 0.92):
        for y in (-0.42, 0.42):
            p.box("leg", (x, y, 0.1), (0.1, 0.1, 0.2), M["leather"])
    p.box("frame", (0, 0, 0.26), (2.0, 1.0, 0.12), M["wood"])
    p.box("mattress", (0, 0, 0.41), (1.9, 0.9, 0.18), M["bone"])
    p.box("headboard", (-0.95, 0, 0.40), (0.1, 1.0, 0.8), M["wood"])
    p.box("pillow", (-0.62, 0, 0.55), (0.34, 0.66, 0.1), M["cloth"])
    p.box("blanket", (0.45, 0, 0.52), (1.0, 0.92, 0.06), M["player"])


def build_table(p):
    """1.2 x 0.8 table, 0.75 tall."""
    p.box("top", (0, 0, 0.71), (1.2, 0.8, 0.08), M["wood"])
    for x in (-0.5, 0.5):
        for y in (-0.3, 0.3):
            p.box("leg", (x, y, 0.335), (0.09, 0.09, 0.67), M["leather"])
    p.box("shelf", (0, 0, 0.2), (1.0, 0.6, 0.05), M["wood"])


def build_chair(p):
    """0.5 x 0.5 footprint, 0.5 seat, 0.9 back."""
    p.box("seat", (0, 0, 0.47), (0.5, 0.5, 0.06), M["wood"])
    for x in (-0.19, 0.19):
        for y in (-0.19, 0.19):
            p.box("leg", (x, y, 0.22), (0.07, 0.07, 0.44), M["leather"])
    for x in (-0.19, 0.19):
        p.box("back_post", (x, 0.19, 0.45), (0.07, 0.07, 0.9), M["wood"])
    p.box("backrest", (0, 0.19, 0.75), (0.5, 0.06, 0.3), M["wood"])


def build_shelf(p):
    """1.0 wide x 0.3 deep x 1.2 tall, with a few props."""
    p.box("back", (0, 0.125, 0.6), (1.0, 0.05, 1.2), M["wood"])
    for x in (-0.47, 0.47):
        p.box("side", (x, 0, 0.6), (0.06, 0.3, 1.2), M["wood"])
    for z in (0.04, 0.42, 0.8):
        p.box("shelf_%d" % int(z * 100), (0, 0, z), (0.94, 0.3, 0.05), M["wood"])
    p.box("top", (0, 0, 1.175), (1.0, 0.3, 0.05), M["wood"])
    for i, x in enumerate((-0.28, -0.1, 0.1)):
        p.box("book_%d" % i, (x, 0, 0.11), (0.1, 0.2, 0.1),
              M["cloth"] if i % 2 else M["player"])
    p.cyl("pot", (0.25, 0, 0.49), 0.09, 0.14, M["stone"], 6)
    p.box("jar", (-0.25, 0, 0.87), (0.14, 0.14, 0.12), M["sand"])


def build_chest(p):
    """0.8 x 0.5 x 0.7 wooden chest with iron bands."""
    p.box("body", (0, 0, 0.225), (0.8, 0.5, 0.45), M["wood"])
    p.box("lid", (0, 0, 0.575), (0.8, 0.5, 0.25), M["leather"])
    for x in (-0.3, 0.3):
        p.box("band", (x, 0, 0.35), (0.06, 0.5, 0.7), M["iron"])
    p.box("latch", (0, -0.23, 0.5), (0.12, 0.04, 0.12), M["gold"])


def build_rug(p):
    """1.0 x 1.0 flat rug, 0.02 thick."""
    p.box("base", (0, 0, 0.008), (1.0, 1.0, 0.016), M["cloth"])
    for y in (-0.47, 0.47):
        p.box("border", (0, y, 0.018), (1.0, 0.06, 0.004), M["player"])
    for x in (-0.47, 0.47):
        p.box("border", (x, 0, 0.018), (0.06, 0.88, 0.004), M["player"])
    d = p.box("diamond", (0, 0, 0.018), (0.3, 0.3, 0.004), M["gold"])
    d.rotation_euler.z += math.pi / 4


def build_lantern(p):
    """~0.4 tall caged lantern with an emissive flame core."""
    p.cyl("base", (0, 0, 0.025), 0.12, 0.05, M["iron"], 8)
    p.ico("core", (0, 0, 0.18), 0.1, M["fire"], 0)
    for x in (-0.09, 0.09):
        for y in (-0.09, 0.09):
            p.box("post", (x, y, 0.185), (0.03, 0.03, 0.27), M["iron"])
    p.cone("cap", (0, 0, 0.35), 0.14, 0.07, 0.06, M["iron"], 4)
    p.ico("finial", (0, 0, 0.39), 0.02, M["gold"], 0)


# ---------------------------------------------------------------------------
# Assembled example room — 4 x 4 tiles, walls on the 1 u grid lines.
# ---------------------------------------------------------------------------
def build_room():
    col = collection("house_interior")
    p = Piece(col)

    # One 4x4 floor slab (floor tiles are exactly 1u, so this is 4x4 of them).
    p.box("floor_slab", (0, 0, 0.05), (4.0, 4.0, 0.1), M["wood"])

    # Four walls, 4 x 1u segments each; the south wall has a door opening.
    for i in range(4):
        t = -1.5 + i
        build_wall(Piece(col, t, 2.0))            # north
        build_wall(Piece(col, t, -2.0)) if i != 2 else None  # south (door gap)
        build_wall(Piece(col, 2.0, t, math.pi / 2))  # east
        build_wall(Piece(col, -2.0, t, math.pi / 2))  # west

    # Door in the south wall segment centred at x = +0.5, plus its transom.
    build_door(Piece(col, 0.5, -2.0))
    p.box("door_transom", (0.5, -2.0, 2.25), (1.0, 0.15, 0.5), M["bone"])

    # Furniture — each placed with the same builder used for the solo asset.
    build_fireplace(Piece(col, 0.0, 1.60))
    build_bed(Piece(col, -0.90, 0.0))
    build_table(Piece(col, 0.90, -0.40))
    build_chair(Piece(col, 0.90, -1.45, math.pi))
    build_shelf(Piece(col, 1.68, 1.0, -math.pi / 2))
    return col


# ---------------------------------------------------------------------------
# Build, export, preview
# ---------------------------------------------------------------------------
ASSETS = [
    ("interior_floor", build_floor),
    ("interior_wall", build_wall),
    ("interior_door", build_door),
    ("interior_window", build_window),
    ("interior_stairs", build_stairs),
    ("interior_fireplace", build_fireplace),
    ("interior_bed", build_bed),
    ("interior_table", build_table),
    ("interior_chair", build_chair),
    ("interior_shelf", build_shelf),
    ("interior_chest", build_chest),
    ("interior_rug", build_rug),
    ("interior_lantern", build_lantern),
]

built = []
for name, fn in ASSETS:
    col = collection(name)
    fn(Piece(col))
    export_asset(name, col)
    print("TRIS", name, tris(col))
    built.append((name, col))

room = build_room()
export_asset("house_interior", room)
print("TRIS", "house_interior", tris(room))
built.append(("house_interior", room))

# Preview-only tweaks (all exports are already written): the room is 4u wide,
# wider than the sheet's 2.4u spacing, so nudge it clear of its neighbour, and
# drop the front wall/door below the floor for a dollhouse cutaway view of the
# furnished interior.
for o in room.objects:
    o.location.x += 0.7
    if abs(o.location.y + 2.0) < 0.01:
        o.location.z -= 4.0

# contact_sheet moves objects for the preview, so it must come last.
contact_sheet(built, "interiors.png", width=4200, height=760)
print("DONE interiors", len(built), "assets")
