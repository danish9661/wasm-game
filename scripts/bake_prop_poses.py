"""Bake animated world props / interior elements into per-frame static GLBs.

    blender -b --python scripts/bake_prop_poses.py
    blender -b --python scripts/bake_prop_poses.py -- --no-render

Why: the shipping renderer draws one static GLB per sprite.  To make campfires,
torches, portals, banners, trees, water and interior fittings *move*, we author
a short loop for each and bake every frame into a standalone, normalised mesh,
exactly like scripts/bake_pose_frames.py does for the player.  A frame swap is
then just a model swap -- no skinning, no morph targets, no runtime code.

Contract per frame (same as the player pose bakes, but a tighter 400-tri budget):

  * one merged mesh, flat shaded, palette colours only, no textures;
  * base at Y = 0 and X/Z centre = (0, 0) in glTF space;
  * ALL frames of a clip use one shared normalisation offset, so pivot and size
    never jump between frames (the static anchor of each prop sits at the origin);
  * under 400 triangles per frame.

Outputs assets/models/animprop_<clip>_<NN>.glb, assets/models/prop_poses.json,
Workbench PNGs in /tmp/opencode/prop_sheet_frames and the contact sheet
assets/previews/prop_pose_bake.png (assembled with scripts/assemble_prop_sheet.py).
"""

import json
import math
import os
import sys

import bpy
from mathutils import Euler, Matrix, Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MODELS = os.path.join(ROOT, "assets", "models")
PREVIEWS = os.path.join(ROOT, "assets", "previews")
FRAME_DIR = "/tmp/opencode/prop_sheet_frames"

# ART_DIRECTION.md palette.  Only lightness variants of existing hues are added
# (leaf/stone/cloth/water shades); no new hues are invented.
PALETTE = {
    "wood": "#6B4423", "wood_light": "#8B5C33", "leather": "#592F1A",
    "stone": "#7A7F87", "stone_dark": "#5A6068", "stone_light": "#A6AEB6",
    "iron": "#B8C0C8", "steel": "#C8D0D8", "gold": "#D9A62E",
    "leaf": "#3A7A33", "leaf_dark": "#2C6028", "leaf_light": "#5B9C4C",
    "cloth": "#39527A", "cloth_light": "#4E6C99",
    "water": "#3A7CA5", "water_light": "#5D9BC2",
    "arcane": "#6B3FA0", "arcane_dim": "#4A2B72", "fire": "#FF6A2A",
}
MAT_KEYS = list(PALETTE)
TRI_BUDGET = 400
BASE_TOL = 0.05
CENTER_TOL_MIN = 0.25
CENTER_TOL_REL = 0.08


def rgb(hex_color):
    h = hex_color.lstrip("#")
    return tuple(int(h[i:i + 2], 16) / 255.0 for i in (0, 2, 4))


def make_materials():
    mats = {}
    for key, color in PALETTE.items():
        m = bpy.data.materials.new("prop_%s" % key)
        m.use_nodes = True
        bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        bsdf.inputs["Base Color"].default_value = (*rgb(color), 1.0)
        bsdf.inputs["Roughness"].default_value = 0.85
        bsdf.inputs["Metallic"].default_value = 0.0
        m.diffuse_color = (*rgb(color), 1.0)  # Workbench MATERIAL colour
        mats[key] = m
    return mats


# ---------------------------------------------------------------------------
# Geometry helpers -- all coordinates Blender Z-up (glTF Y-up on export).
# ---------------------------------------------------------------------------
class MeshData:
    """Accumulates (vertices, triangle-source faces, material keys)."""

    def __init__(self):
        self.verts = []
        self.faces = []
        self.mats = []

    def add(self, verts, faces, mat):
        base = len(self.verts)
        self.verts.extend(tuple(v) for v in verts)
        self.faces.extend(tuple(base + i for i in f) for f in faces)
        self.mats.extend([MAT_KEYS.index(mat)] * len(faces))

    def tris(self):
        return sum(len(f) - 2 for f in self.faces)

    def bounds(self):
        lo = [min(v[i] for v in self.verts) for i in range(3)]
        hi = [max(v[i] for v in self.verts) for i in range(3)]
        return lo, hi


def box(center, size, rot=None, pivot=None):
    """Axis-aligned box, optionally Euler-rotated about `pivot`."""
    c = Vector(center)
    hx, hy, hz = (s / 2.0 for s in size)
    offsets = [Vector((x * hx, y * hy, z * hz))
               for x, y, z in ((-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1),
                               (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1))]
    R = Euler(rot, "XYZ").to_matrix() if rot else Matrix.Identity(3)
    p = Vector(pivot) if pivot is not None else c
    verts = [p + R @ (c + o - p) for o in offsets]
    faces = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4),
             (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
    return verts, faces


def prism(p0, p1, radius, sides=6):
    """Cylinder / truncated cone along an arbitrary axis (caps are n-gons)."""
    a, b = Vector(p0), Vector(p1)
    axis = (b - a).normalized()
    ref = Vector((0, 0, 1)) if abs(axis.z) < 0.9 else Vector((0, 1, 0))
    u = axis.cross(ref).normalized()
    v = axis.cross(u).normalized()
    ring = [u * (radius * math.cos(2 * math.pi * i / sides))
            + v * (radius * math.sin(2 * math.pi * i / sides))
            for i in range(sides)]
    verts = [a + r for r in ring] + [b + r for r in ring]
    faces = [(i, (i + 1) % sides, sides + (i + 1) % sides, sides + i)
             for i in range(sides)]
    faces += [tuple(reversed(range(sides))), tuple(range(sides, 2 * sides))]
    return verts, faces


def cone(p0, p1, radius, sides=6):
    """Cone with a single apex at p1 and an n-gon base at p0."""
    a, b = Vector(p0), Vector(p1)
    axis = (b - a).normalized()
    ref = Vector((0, 0, 1)) if abs(axis.z) < 0.9 else Vector((0, 1, 0))
    u = axis.cross(ref).normalized()
    v = axis.cross(u).normalized()
    ring = [a + u * (radius * math.cos(2 * math.pi * i / sides))
            + v * (radius * math.sin(2 * math.pi * i / sides))
            for i in range(sides)]
    apex = len(ring)
    verts = ring + [b]
    faces = [(i, (i + 1) % sides, apex) for i in range(sides)]
    faces.append(tuple(reversed(range(sides))))
    return verts, faces


PHI = (1.0 + math.sqrt(5.0)) / 2.0
_ICO_V = [(-1, PHI, 0), (1, PHI, 0), (-1, -PHI, 0), (1, -PHI, 0),
          (0, -1, PHI), (0, 1, PHI), (0, -1, -PHI), (0, 1, -PHI),
          (PHI, 0, -1), (PHI, 0, 1), (-PHI, 0, -1), (-PHI, 0, 1)]
_ICO_F = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
          (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
          (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
          (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1)]


def ico(center, radius, scale=(1, 1, 1), rot=None, pivot=None):
    """Subdivision-0 icosahedron (20 tris), optionally stretched / rotated."""
    R = Euler(rot, "XYZ").to_matrix() if rot else Matrix.Identity(3)
    s = Vector(scale)
    n = math.sqrt(PHI * PHI + 1.0)
    c = Vector(center)
    p = Vector(pivot) if pivot is not None else c
    verts = [p + R @ (c + Vector((v[0] * s.x, v[1] * s.y, v[2] * s.z))
                      * (radius / n) - p) for v in _ICO_V]
    return verts, [tuple(f) for f in _ICO_F]


def quad_grid(nx, ny, fn):
    """nx x ny quad grid; fn(i, j, nx, ny) -> (vertex, material key)."""
    verts, mats = [], []
    for j in range(ny + 1):
        for i in range(nx + 1):
            v, mat = fn(i, j, nx, ny)
            verts.append(v)
            mats.append(mat)
    faces = []
    for j in range(ny):
        for i in range(nx):
            a = j * (nx + 1) + i
            faces.append((a, a + 1, a + nx + 2, a + nx + 1))
    return verts, faces, mats


def add_grid(m, verts, faces, mats):
    base = len(m.verts)
    m.verts.extend(tuple(v) for v in verts)
    m.faces.extend(tuple(base + i for i in f) for f in faces)
    m.mats.extend(MAT_KEYS.index(k) for k in mats)


# ---------------------------------------------------------------------------
# Animated elements.  phase = 2*pi*i/n, so each clip is a seamless loop.
# ---------------------------------------------------------------------------
def build_campfire(m, phase, i, n):
    # Static anchor: ash bed (min Z = 0), six-ring of stones, four crossed logs.
    m.add(*prism((0, 0, 0), (0, 0, 0.04), 0.52, 6), "stone_dark")
    for k in range(6):
        a = 2 * math.pi * k / 6
        m.add(*ico((0.42 * math.cos(a), 0.42 * math.sin(a), 0.18), 0.17),
              "stone")
    for k in range(4):
        a = math.pi * k / 4
        m.add(*prism((-0.32 * math.cos(a), -0.32 * math.sin(a), 0.30),
                     (0.32 * math.cos(a), 0.32 * math.sin(a), 0.46),
                     0.075, 5), "wood")
    # Outer tongue flickers tall/short and leans; inner tongue chases it.
    flick = 1.0 + 0.20 * math.sin(phase) + 0.09 * math.sin(3 * phase + 0.7)
    lean = 0.09 * math.sin(phase + 0.5)
    h = 0.62 * flick
    m.add(*cone((lean * 0.3, 0, 0.42), (lean, 0.04 * math.cos(phase), 0.42 + h),
                0.24 * (1.0 + 0.10 * math.cos(phase)), 6), "fire")
    h2 = 0.38 * (1.0 + 0.25 * math.sin(phase + 1.2))
    m.add(*cone((-lean * 0.2, 0, 0.43), (-lean * 0.5, 0, 0.43 + h2),
                0.12, 5), "gold")
    # Ember pulse: rising and swelling with a second harmonic.
    ez = 0.28 + 0.07 * math.sin(2 * phase + 0.3)
    m.add(*ico((0.10 * math.sin(2 * phase), -0.08, ez),
               0.07 * (1.0 + 0.28 * math.sin(2 * phase + 0.3))), "gold")


def build_campfire_ember(m, phase, i, n):
    m.add(*prism((0, 0, 0), (0, 0, 0.04), 0.44, 6), "stone_dark")
    for k in range(5):
        a = 2 * math.pi * k / 5
        pulse = 1.0 + 0.22 * math.sin(2 * phase + k * 0.9)
        m.add(*ico((0.23 * math.cos(a), 0.23 * math.sin(a), 0.08),
                   0.105 * pulse, scale=(1, 1, 0.7)),
              "fire" if k % 2 else "gold")
    for k in range(3):
        a = 2 * math.pi * k / 3 + phase
        z = 0.22 + 0.16 * (0.5 + 0.5 * math.sin(2 * phase + k))
        m.add(*ico((0.16 * math.cos(a), 0.16 * math.sin(a), z), 0.045), "gold")


def _flame_pair(m, base_z, phase, outer_r=0.15, outer_h=0.44):
    flick = 1.0 + 0.22 * math.sin(phase) + 0.07 * math.sin(3 * phase)
    lean = 0.06 * math.sin(phase + 0.4)
    m.add(*cone((0, 0, base_z), (lean, 0, base_z + outer_h * flick),
                outer_r * (1 + 0.12 * math.cos(phase)), 6), "fire")
    m.add(*cone((0, 0, base_z - 0.02), (-lean * 0.4, 0,
                                        base_z + outer_h * 0.55 * flick),
                0.07, 5), "gold")


def build_torch(m, phase, i, n):
    m.add(*prism((0, 0, 0), (0, 0, 0.70), 0.055, 6), "wood")
    m.add(*prism((0, 0, 0.60), (0, 0, 0.76), 0.10, 6), "leather")
    _flame_pair(m, 0.74, phase, 0.15, 0.44)


def build_brazier(m, phase, i, n):
    for k in range(3):
        a = 2 * math.pi * k / 3
        m.add(*prism((0.30 * math.cos(a), 0.30 * math.sin(a), 0),
                     (0.30 * math.cos(a), 0.30 * math.sin(a), 0.60),
                     0.06, 5), "iron")
    m.add(*prism((0, 0, 0.55), (0, 0, 0.83), 0.38, 8), "iron")
    glow = 1.0 + 0.18 * math.sin(2 * phase)
    m.add(*cone((0, 0, 0.82), (0, 0, 0.82 + 0.22 * glow), 0.30, 8), "gold")
    _flame_pair(m, 0.86, phase, 0.22, 0.50)


def build_lantern(m, phase, i, n):
    m.add(*prism((0, 0, 0), (0, 0, 0.10), 0.20, 8), "iron")
    m.add(*prism((0, 0, 0.10), (0, 0, 0.52), 0.16, 8), "fire")
    for k in range(4):
        a = math.pi * k / 2 + math.pi / 4
        m.add(*box((0.16 * math.cos(a), 0.16 * math.sin(a), 0.31),
                   (0.05, 0.05, 0.48)), "iron")
    m.add(*cone((0, 0, 0.52), (0, 0, 0.70), 0.24, 8), "iron")
    for x in (-0.13, 0.13):
        m.add(*box((x, 0, 0.78), (0.04, 0.04, 0.24)), "iron")
    m.add(*box((0, 0, 0.90), (0.30, 0.04, 0.04)), "iron")
    # Light pulse: a hot core swelling behind the glass.
    pulse = 1.0 + 0.30 * math.sin(2 * phase + 0.4)
    m.add(*ico((0, 0, 0.31), 0.10 * pulse, scale=(0.8, 0.8, 1.5)), "gold")


def build_portal(m, phase, i, n):
    m.add(*box((0, 0, 0.07), (2.0, 0.85, 0.14)), "stone_dark")
    m.add(*box((0, 0, 0.15), (1.7, 0.65, 0.10)), "stone_light")
    for x in (-0.74, 0.74):
        m.add(*box((x, 0, 1.0), (0.32, 0.42, 1.90)), "stone")
        m.add(*box((x, 0, 0.22), (0.42, 0.50, 0.16)), "stone_dark")
    for k in range(7):
        a = math.pi - math.pi * k / 6
        x, z = 0.74 * math.cos(a), 1.95 + 0.74 * math.sin(a)
        m.add(*box((x, 0, z), (0.30, 0.44, 0.26),
                   rot=(0, -(a + math.pi / 2), 0)), "stone")
    m.add(*box((0, 0, 1.0), (1.18, 0.05, 1.75)), "arcane_dim")
    pulse = 1.0 + 0.22 * math.sin(3 * phase)
    m.add(*ico((0, 0, 1.08), 0.20 * pulse, scale=(0.7, 0.4, 1.3)), "arcane")
    # Swirl: six runes orbit the core in the portal plane, with depth wobble.
    for k in range(6):
        a = 2 * math.pi * k / 6 + phase
        r = 0.40 + 0.06 * math.sin(phase + k)
        m.add(*ico((r * math.cos(a), 0.10 * math.sin(3 * phase + k),
                    1.08 + r * 1.35 * math.sin(a)), 0.075),
              "gold" if k % 2 else "arcane")


def build_banner(m, phase, i, n):
    m.add(*prism((0, 0, 0), (0, 0, 2.20), 0.055, 6), "wood")
    m.add(*box((0, 0, 2.08), (0.95, 0.07, 0.07)), "leather")
    m.add(*box((0, 0, 0.06), (0.38, 0.38, 0.12)), "stone")
    m.add(*ico((0, 0, 2.25), 0.10), "gold")

    def cell(ii, jj, nx, ny):
        t = jj / ny
        x = -0.36 + 0.72 * ii / nx
        z = 2.02 - 1.04 * t
        # Wave travels down the cloth; taper the free bottom edge.
        y = -0.05 - 0.10 * math.sin(2 * math.pi * (ii / nx) * 0.9
                                    + 2 * math.pi * t * 0.55 - phase)
        xx = x * (1.0 - 0.30 * t) + 0.02 * math.sin(phase)
        return (xx, y, z), "cloth_light" if (ii + jj) % 3 == 0 else "cloth"

    v, f, mats = quad_grid(4, 4, cell)
    add_grid(m, v, f, mats)
    # Gilded emblem sits on the cloth, moving with its surface.
    ey = -0.05 - 0.10 * math.sin(math.pi * 0.9 - phase)
    m.add(*box((0, ey - 0.03, 1.62 + 0.02 * math.sin(phase)),
               (0.32, 0.03, 0.08)), "gold")
    m.add(*box((0, ey - 0.03, 1.62 + 0.02 * math.sin(phase)),
               (0.08, 0.03, 0.32)), "gold")


def build_tree(m, phase, i, n):
    m.add(*prism((0, 0, 0), (0, 0, 1.44), 0.15, 6), "wood")
    # Left/right clusters sway in opposition with a shared phase: the canopy
    # visibly rocks while its combined footprint stays centred on the trunk.
    clusters = [
        ((0, 0, 1.95), 0.62, "leaf", 0.0, 0.035, 1.0),
        ((-0.40, 0.06, 1.76), 0.48, "leaf_dark", 0.0, 0.11, 1.0),
        ((0.40, -0.10, 1.80), 0.50, "leaf_light", 0.0, 0.11, -1.0),
        ((0.06, 0.26, 2.32), 0.46, "leaf", 0.0, 0.05, 1.0),
    ]
    for (cx, cy, cz), r, mat, off, amp, direction in clusters:
        sway = direction * amp * math.sin(phase + off)
        m.add(*ico((cx + sway, cy + direction * 0.5 * amp * math.cos(phase + off), cz),
                   r, rot=(0.04 * math.cos(phase + off), 0, 0.05 * sway)), mat)


def build_pine(m, phase, i, n):
    m.add(*prism((0, 0, 0), (0, 0, 0.84), 0.11, 6), "wood")
    tiers = [
        (1.02, 0.62, 0.08, 0.92, "leaf_dark", 0.0, 0.03),
        (1.62, 0.50, 0.06, 0.88, "leaf", 0.8, 0.06),
        (2.25, 0.35, 0.0, 1.00, "leaf_light", 1.6, 0.09),
    ]
    for z, r0, r1, h, mat, off, amp in tiers:
        sway = amp * math.sin(phase + off)
        top = (sway, 0.5 * sway, z + h)
        # Frustum by two rings, tilted with the sway.
        a, b = Vector((0, 0, z)), Vector(top)
        axis = (b - a).normalized()
        u = axis.cross(Vector((0, 1, 0))).normalized()
        v = axis.cross(u).normalized()
        ring0 = [a + u * (r0 * math.cos(2 * math.pi * k / 8))
                 + v * (r0 * math.sin(2 * math.pi * k / 8)) for k in range(8)]
        ring1 = [b + u * (r1 * math.cos(2 * math.pi * k / 8))
                 + v * (r1 * math.sin(2 * math.pi * k / 8)) for k in range(8)]
        verts = ring0 + ring1
        faces = [(k, (k + 1) % 8, 8 + (k + 1) % 8, 8 + k) for k in range(8)]
        faces.append(tuple(reversed(range(8))))
        if r1 > 0:
            faces.append(tuple(range(8, 16)))
        m.add(verts, faces, mat)


def build_water(m, phase, i, n):
    m.add(*box((0, 0, 0.02), (1.8, 1.8, 0.04)), "water")

    def cell(ii, jj, nx, ny):
        x = -0.9 + 1.8 * ii / nx
        y = -0.9 + 1.8 * jj / ny
        r = math.hypot(x, y)
        falloff = max(0.0, 1.0 - r / 1.15)
        crest = 0.5 + 0.5 * math.sin(2 * math.pi * r / 0.65 - phase)
        z = 0.04 + 0.09 * crest * falloff
        return (x, y, z), "water_light" if crest > 0.62 else "water"

    v, f, mats = quad_grid(6, 6, cell)
    add_grid(m, v, f, mats)


def build_door(m, phase, i, n):
    # Deep jambs keep the static frame dominant in the X/Z footprint so the
    # swinging leaf cannot push the shared origin off-centre.
    for x in (-0.41, 0.41):
        m.add(*box((x, 0, 1.0), (0.08, 0.50, 2.0)), "wood")
    m.add(*box((0, 0, 1.95), (0.90, 0.50, 0.10)), "wood")
    m.add(*box((0, 0, 0.04), (0.90, 0.50, 0.08)), "stone_dark")
    # Leaf rotates about the hinge at x=-0.37, opening out and closing again.
    angle = 0.55 * (1.0 - math.cos(phase))
    hinge = (-0.37, 0.0, 0.0)
    rot = (0, 0, -angle)
    m.add(*box((0, 0, 0.925), (0.74, 0.09, 1.85), rot=rot, pivot=hinge),
          "wood_light")
    for z in (0.55, 1.35):
        m.add(*box((0, -0.052, z), (0.56, 0.02, 0.50),
                   rot=rot, pivot=hinge), "wood")
    m.add(*ico((0.27, -0.07, 0.95), 0.045, rot=rot, pivot=hinge), "gold")


def build_window(m, phase, i, n):
    m.add(*box((0, 0.02, 0.45), (0.60, 0.04, 0.70)), "stone_dark")
    lit = 0.5 + 0.5 * math.sin(2 * phase + 0.6)
    m.add(*box((0, 0, 0.45), (0.60 * (0.95 + 0.05 * lit), 0.04,
                              0.70 * (0.95 + 0.05 * lit))),
          "fire" if lit > 0.45 else "gold")
    for x in (-0.35, 0.35):
        m.add(*box((x, 0, 0.45), (0.10, 0.14, 0.90)), "wood")
    for z in (0.05, 0.85):
        m.add(*box((0, 0, z), (0.80, 0.14, 0.10)), "wood")
    m.add(*box((0, -0.045, 0.45), (0.04, 0.02, 0.70)), "wood")
    m.add(*box((0, -0.045, 0.45), (0.60, 0.02, 0.04)), "wood")
    # Flickering glow behind the panes (swells and sinks).
    pulse = 1.0 + 0.34 * math.sin(2 * phase + 0.6)
    m.add(*ico((0, -0.03, 0.45 + 0.05 * math.sin(phase)),
               0.13 * pulse, scale=(1.4, 0.4, 1.7)), "gold")


# name -> (frame count, fps, builder, description for the doc)
CLIPS = {
    "campfire": (8, 8.0, build_campfire,
                 "flickering flame: two cones vary height/radius/lean, ember bobs"),
    "campfire_ember": (8, 6.0, build_campfire_ember,
                       "five coals pulse in size, three sparks rise"),
    "torch": (6, 8.0, build_torch,
              "flame pair flickers above a fixed wrapped handle"),
    "brazier": (6, 8.0, build_brazier,
                "coal glow pulses and flame pair flickers in an iron bowl"),
    "lantern": (6, 6.0, build_lantern,
                "hot core swells behind the glass: a light pulse"),
    "portal": (8, 8.0, build_portal,
               "six runes orbit the core and the heart pulses"),
    "banner": (8, 6.0, build_banner,
               "4x4 cloth grid travels in a sine wave down the hanging banner"),
    "tree": (8, 4.0, build_tree,
             "four canopy clusters sway and tilt with staggered phases"),
    "pine": (8, 4.0, build_pine,
             "three conical tiers lean, amplitude increasing toward the tip"),
    "water": (8, 6.0, build_water,
              "6x6 grid with concentric sine crests; crest quads use a lighter hue"),
    "door": (6, 6.0, build_door,
             "leaf swings 0..63 deg and back about a fixed hinge"),
    "window": (6, 6.0, build_window,
               "glass hue alternates and a glow ico swells: lit-window flicker"),
}


# ---------------------------------------------------------------------------
# Export / render
# ---------------------------------------------------------------------------
def mesh_object(data, mats):
    mesh = bpy.data.meshes.new("animprop_mesh")
    mesh.from_pydata(data.verts, [], data.faces)
    mesh.update()
    for key in MAT_KEYS:
        mesh.materials.append(mats[key])
    for poly, mi in zip(mesh.polygons, data.mats):
        poly.material_index = mi
        poly.use_smooth = False
    obj = bpy.data.objects.new("animprop", mesh)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def export_glb(obj, path):
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.export_scene.gltf(
        filepath=path, export_format="GLB", export_yup=True,
        export_animations=False, export_apply=False, export_extras=False,
        use_selection=True)


def setup_render():
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.render.resolution_x = 200
    sc.render.resolution_y = 200
    sc.render.image_settings.file_format = "PNG"
    sc.display.shading.light = "STUDIO"
    sc.display.shading.color_type = "MATERIAL"
    sc.display.shading.show_shadows = True
    sc.display.shading.background_type = "VIEWPORT"
    sc.display.shading.background_color = (0.30, 0.32, 0.35)


def render_frame(lo, hi, path):
    sc = bpy.context.scene
    target = Vector((0, 0, (lo[2] + hi[2]) / 2.0))
    span = max(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2], 1.05)
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = span * 1.45
    cam = bpy.data.objects.new("cam", cam_data)
    sc.collection.objects.link(cam)
    cam.location = target + Vector((3.0, -3.0, 2.5))  # yaw 45, pitch ~30
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    sc.camera = cam
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam, do_unlink=True)
    bpy.data.cameras.remove(cam_data, do_unlink=True)


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    do_render = "--no-render" not in args

    os.makedirs(MODELS, exist_ok=True)
    os.makedirs(PREVIEWS, exist_ok=True)
    if do_render:
        os.makedirs(FRAME_DIR, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    mats = make_materials()
    if do_render:
        setup_render()

    manifest = {}
    max_tris = 0
    total = 0
    heights = []

    for name, (count, fps, builder, _desc) in CLIPS.items():
        # Pass 1: build every frame, then derive ONE normalisation offset from
        # the union AABB so every frame shares the same pivot and size.
        frames = [MeshData() for _ in range(count)]
        for i, data in enumerate(frames):
            builder(data, 2 * math.pi * i / count, i, count)
        lo_all = [min(d.bounds()[0][k] for d in frames) for k in range(3)]
        hi_all = [max(d.bounds()[1][k] for d in frames) for k in range(3)]
        offset = Vector((-(lo_all[0] + hi_all[0]) / 2.0,
                         -(lo_all[1] + hi_all[1]) / 2.0, -lo_all[2]))

        files = []
        for i, data in enumerate(frames):
            tris = data.tris()
            if tris >= TRI_BUDGET:
                raise ValueError("%s frame %d: %d tris >= %d budget"
                                 % (name, i, tris, TRI_BUDGET))
            max_tris = max(max_tris, tris)

            shifted = MeshData()
            shifted.verts = [tuple(Vector(v) + offset) for v in data.verts]
            shifted.faces, shifted.mats = data.faces, data.mats
            lo, hi = shifted.bounds()
            if abs(lo[2]) > BASE_TOL:
                raise ValueError("%s frame %d: base Z %.4f" % (name, i, lo[2]))
            tol = max(CENTER_TOL_MIN,
                      CENTER_TOL_REL * max(hi[0] - lo[0], hi[1] - lo[1]))
            if abs((lo[0] + hi[0]) / 2) > tol or abs((lo[1] + hi[1]) / 2) > tol:
                raise ValueError("%s frame %d: off-centre X %.3f Y %.3f (tol %.3f)"
                                 % (name, i, (lo[0] + hi[0]) / 2,
                                    (lo[1] + hi[1]) / 2, tol))

            obj = mesh_object(shifted, mats)
            filename = "animprop_%s_%02d.glb" % (name, i)
            export_glb(obj, os.path.join(MODELS, filename))
            if do_render:
                render_frame(lo, hi, os.path.join(
                    FRAME_DIR, filename.replace(".glb", ".png")))
            bpy.data.objects.remove(obj, do_unlink=True)
            files.append(filename)
            total += 1
            heights.append(hi[2] - lo[2])

        manifest[name] = {"fps": fps, "frames": files, "loop": True}
        print("[bake] %-15s %2d frames @ %g fps, %d tris, offset=(%+.3f,%+.3f,%+.3f)"
              % (name, count, fps, frames[0].tris(), *offset))

    with open(os.path.join(MODELS, "prop_poses.json"), "w") as fh:
        json.dump(manifest, fh, indent=2)
        fh.write("\n")

    print("[bake] %d clips, %d GLBs; max %d tris; heights %.2f..%.2f u"
          % (len(manifest), total, max_tris, min(heights), max(heights)))
    if do_render:
        print("[bake] raw frames in %s" % FRAME_DIR)
    return 0


if __name__ == "__main__":
    sys.exit(main())
