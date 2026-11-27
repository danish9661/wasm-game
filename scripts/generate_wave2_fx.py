"""Deterministic second-wave visual-effect GLBs.

    blender -b --python scripts/generate_wave2_fx.py

Twelve one-shot / looping FX are authored as small sets of named, flat-shaded
low-poly primitives and animated purely through node transforms. No textures,
no rigs, no external files beyond the two clip manifests used to validate sync.

Conventions (same family as generate_world_animations.py):
  * 30 fps, LINEAR F-curve samplers, one named Action per GLB, export_yup=True.
  * Keys exist at frames 0 .. (frames-1); sampler times therefore span
    0 .. (frames-1)/30 seconds. duration_s in the manifest is frames/30.
  * Loops sample one complete period at N keys (no duplicate terminal key).
  * Scale keys are allowed and intentional for FX: they produce pop-in,
    expansion and fade-out without alpha animation. Character rigs forbid them.
  * Everything is deterministic: sin/cos plus seeded key patches, no random.

Outputs assets/animations/wave2/fx/<effect>.glb and wave2/fx_manifest.json.
"""

import json
import os
import sys
from math import cos, pi, sin

import bpy

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "assets", "animations", "wave2", "fx")
MANIFEST = os.path.join(ROOT, "assets", "animations", "wave2", "fx_manifest.json")
WAVE2_MANIFEST = os.path.join(ROOT, "assets", "animations", "wave2", "manifest.json")
PLAYER_MANIFEST = os.path.join(ROOT, "assets", "animations", "player", "manifest.json")
FPS = 30

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from generate_combat_animations import iter_fcurves  # noqa: E402

# The only hues used by this set (ART_DIRECTION.md / assets/spec.json).
PALETTE = {
    "arcane": "#6B3FA0",
    "enemy": "#C0392B",
    "gold": "#D9A62E",
    "steel": "#C8D0D8",
    "fire": "#FF6A2A",
    "water": "#3A7CA5",
    "grass": "#5B8C3A",
    "snow": "#E8F0FA",
}


# ---------------------------------------------------------------------------
# Generic helpers
# ---------------------------------------------------------------------------


def clamp(v, lo=0.0, hi=1.0):
    return max(lo, min(hi, v))


def keys_at(t, keys):
    """Linear ramp through authored (time, value) key patches."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * ((t - t0) / (t1 - t0))
    return keys[-1][1]


def patch(seed, i):
    """Deterministic pseudo-random value in [-1, 1] (golden-ratio hashing)."""
    return sin(seed * 1.618033988749895 + i * 2.399963229728653)


def unit(seed, i):
    return 0.5 + 0.5 * patch(seed, i)


def srgb_to_linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def rgba(hex_colour):
    h = hex_colour.lstrip("#")
    return tuple(srgb_to_linear(int(h[i:i + 2], 16) / 255.0) for i in (0, 2, 4)) + (1.0,)


def material(name, colour, metallic=False):
    m = bpy.data.materials.new("fx_" + name)
    m.use_nodes = True
    bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bsdf.inputs["Base Color"].default_value = rgba(PALETTE[colour])
    bsdf.inputs["Roughness"].default_value = 0.85
    bsdf.inputs["Metallic"].default_value = 1.0 if metallic else 0.0
    m.diffuse_color = rgba(PALETTE[colour])
    return m


MATS = {}


def build_mats():
    """Rebuild materials after each factory reset (old datablocks are gone)."""
    global MATS
    MATS = {name: material(name, name, name in ("steel", "gold")) for name in PALETTE}


def finish(o, mat):
    o.data.materials.append(mat)
    for poly in o.data.polygons:
        poly.use_smooth = False
    return o


def cube(name, size, loc, mat):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    o = bpy.context.object
    o.name = name
    o.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    return finish(o, mat)


def cone(name, radius, depth, sides, loc, mat, radius2=0.0):
    bpy.ops.mesh.primitive_cone_add(vertices=sides, radius1=radius,
                                    radius2=radius2, depth=depth, location=loc)
    o = bpy.context.object
    o.name = name
    return finish(o, mat)


def cylinder(name, radius, depth, sides, loc, mat):
    bpy.ops.mesh.primitive_cylinder_add(vertices=sides, radius=radius,
                                        depth=depth, location=loc)
    o = bpy.context.object
    o.name = name
    return finish(o, mat)


def ico(name, radius, loc, mat, subdiv=0):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=subdiv, radius=radius,
                                          location=loc)
    o = bpy.context.object
    o.name = name
    return finish(o, MATS[mat] if isinstance(mat, str) else mat)


def clean():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.render.fps = FPS
    build_mats()


def make_action(objects, name, frames, key_fn):
    """One slotted Action, three channels per object, keys 0 .. frames-1."""
    action = bpy.data.actions.new(name)
    for o in objects:
        ad = o.animation_data_create()
        ad.action = action
        ad.action_slot = action.slots.new(id_type="OBJECT", name=o.name)
    for f in range(frames):
        key_fn(f, f)
        for o in objects:
            o.keyframe_insert(data_path="location", frame=f, group=o.name)
            o.keyframe_insert(data_path="rotation_euler", frame=f, group=o.name)
            o.keyframe_insert(data_path="scale", frame=f, group=o.name)
    for fc in iter_fcurves(action):
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
    action.use_fake_user = True
    return action


def export(name, path):
    bpy.ops.export_scene.gltf(
        filepath=path, export_format="GLB", export_yup=True,
        export_animations=True, export_animation_mode="ACTIONS",
        export_nla_strips=False, export_extras=True, export_apply=False,
        use_selection=False)


def triangles(objects):
    return sum(len(p.vertices) - 2 for o in objects for p in o.data.polygons)


# ---------------------------------------------------------------------------
# Effects. Each builder returns (objects, key_fn, palette_names, note).
# Every key_fn gives every object a time-varying location, rotation and scale
# so the exporter cannot drop a channel as static.
# ---------------------------------------------------------------------------


def boss_telegraph_ring(frames):
    objs, parts = [], []
    for i in range(12):
        a = 2 * pi * i / 12
        o = cube("ring_seg_%02d" % i, (0.22, 0.05, 0.024),
                 (0.52 * cos(a), 0.52 * sin(a), 0.026),
                 MATS["arcane" if i % 2 == 0 else "enemy"])
        objs.append(o)
        parts.append((o, a, 0.52, 0))
    for i in range(6):
        a = 2 * pi * (i + 0.5) / 6
        o = ico("rune_%02d" % i, 0.055,
                (0.26 * cos(a), 0.26 * sin(a), 0.05), MATS["gold"])
        objs.append(o)
        parts.append((o, a, 0.26, 1))

    def key(f, pf):
        t = f / (frames - 1)
        grow = keys_at(t, [(0, 0.30), (0.55, 1.15), (0.78, 1.0), (1, 0.02)])
        pulse = 1.0 + 0.06 * sin(12 * pi * t)
        for i, (o, a, r0, kind) in enumerate(parts):
            if kind == 0:
                r = r0 * grow * pulse
                o.location = (r * cos(a), r * sin(a), 0.026 + 0.004 * sin(2 * pi * t + i))
                o.rotation_euler = (0.0, 0.0, a + pi / 2 + 0.012 * sin(4 * pi * t + i))
                s = max(0.001, grow)
                o.scale = (s, s, s)
            else:
                r = r0 * grow
                o.location = (r * cos(a), r * sin(a), 0.05 + 0.04 * grow)
                o.rotation_euler = (0.0, 0.0, 0.6 * t + 0.1 * sin(2 * pi * t + i))
                s = max(0.001, 0.9 * grow * (1.0 + 0.14 * sin(8 * pi * t + a)))
                o.scale = (s, s, s)

    return objs, key, ["arcane", "enemy", "gold"], "Expanding ground ring; scales pop in and fade out."


def boss_impact_burst(frames):
    objs, spikes, dust = [], [], []
    for i in range(10):
        a = 2 * pi * i / 10 + 0.12 * patch(3, i)
        o = cone("spike_%02d" % i, 0.075, 0.34, 4, (0.1 * cos(a), 0.1 * sin(a), 0.17),
                 MATS["enemy" if i % 2 == 0 else "fire"])
        objs.append(o)
        spikes.append((o, a))
    for i in range(8):
        a = 2 * pi * (i + 0.5) / 8
        o = ico("dust_%02d" % i, 0.065, (0.22 * cos(a), 0.22 * sin(a), 0.05),
                MATS["steel" if i % 2 else "gold"])
        objs.append(o)
        dust.append((o, a))
    disc = cylinder("shock_disc", 0.34, 0.022, 8, (0.0, 0.0, 0.025), MATS["gold"])
    objs.append(disc)

    def key(f, pf):
        t = f / (frames - 1)
        out = keys_at(t, [(0, 0.2), (0.25, 1.0), (0.6, 1.2), (1, 0.02)])
        for i, (o, a) in enumerate(spikes):
            r = 0.1 + 0.62 * out
            o.location = (r * cos(a), r * sin(a), 0.10 + 0.03 * sin(2 * pi * t + i))
            o.rotation_euler = (0.0, pi / 2, a + 0.08 * sin(3 * pi * t + i))
            s = max(0.001, out)
            o.scale = (s, s, s)
        for i, (o, a) in enumerate(dust):
            ph = (i / 8 + 0.8 * t) % 1.0
            r = 0.22 + 0.55 * ph
            o.location = (r * cos(a), r * sin(a), 0.05 + 0.32 * sin(pi * ph))
            o.rotation_euler = (0.2 * t, 0.3 * t, a + 2 * pi * ph)
            s = max(0.001, 1.2 * (1.0 - ph))
            o.scale = (s, s, s)
        s = max(0.001, out)
        disc.location = (0.0, 0.0, 0.025 + 0.002 * sin(2 * pi * t))
        disc.rotation_euler = (0.0, 0.0, 0.4 * t)
        disc.scale = (s, s, s)

    return objs, key, ["enemy", "fire", "gold", "steel"], "Radial spikes, expanding shock disc and kicked-up dust."


def slash_arc(frames):
    objs, segs = [], []
    n = 9
    for i in range(n):
        o = cube("arc_%02d" % i, (0.17, 0.045, 0.075), (0.0, 0.0, 0.0),
                 MATS["snow" if i in (0, n - 1) else "steel"])
        objs.append(o)
        segs.append(o)
    tips = []
    for i in range(2):
        o = ico("tip_%02d" % i, 0.05, (0.0, 0.0, 0.0), MATS["snow"])
        objs.append(o)
        tips.append(o)

    def key(f, pf):
        t = f / (frames - 1)
        sweep = -0.65 + 1.3 * t
        for i, o in enumerate(segs):
            base = -1.05 + 2.1 * i / (n - 1)
            delay = 0.05 * i
            local = clamp((t - delay) / max(0.2, 1.0 - delay))
            env = keys_at(local, [(0, 0.001), (0.18, 1.0), (0.62, 0.9), (1, 0.001)])
            a = base + sweep
            o.location = (0.58 * sin(a), 0.02 * sin(4 * pi * t + i), 0.55 * cos(a))
            o.rotation_euler = (0.0, a + 0.05 * sin(2 * pi * t + i), 0.0)
            s = max(0.001, env)
            o.scale = (s, s, s)
        for i, o in enumerate(tips):
            a = (-1.1 if i == 0 else 1.1) + sweep
            o.location = (0.58 * sin(a), 0.02 * cos(4 * pi * t + i), 0.55 * cos(a))
            o.rotation_euler = (0.0, a, 0.3 * t)
            s = max(0.001, keys_at(t, [(0, 1), (0.55, 0.8), (1, 0.001)]))
            o.scale = (s, s, s)

    return objs, key, ["steel", "snow"], "Steel crescent sweep, staggered segment pop-in, snow-bright tips."


def hit_flash(frames):
    objs, parts = [], []
    for i in range(4):
        size = (0.055, 0.03, 0.62) if i < 2 else (0.62, 0.03, 0.055)
        o = cube("beam_%02d" % i, size, (0.0, 0.0, 0.8), MATS["gold" if i < 2 else "snow"])
        objs.append(o)
        parts.append((o, i))
    for i in range(4):
        a = pi / 4 + i * pi / 2
        o = cube("spark_%02d" % i, (0.025, 0.025, 0.24), (0.0, 0.0, 0.8),
                 MATS["fire" if i % 2 == 0 else "gold"])
        objs.append(o)
        parts.append((o, i + 4))
    core = ico("core", 0.095, (0.0, 0.0, 0.8), MATS["snow"], subdiv=0)
    objs.append(core)

    def key(f, pf):
        t = f / (frames - 1)
        env = keys_at(t, [(0, 1.0), (0.2, 0.4), (0.35, 1.0), (0.6, 0.22), (1, 0.001)])
        spin = 0.55 * t
        for i, (o, _) in enumerate(parts[:4]):
            a = spin + (0.0 if i < 2 else pi / 2)
            o.location = (0.012 * sin(5 * pi * t + i), 0.01 * cos(4 * pi * t + i),
                          0.8 + 0.01 * sin(2 * pi * t))
            o.rotation_euler = (0.0, a, 0.05 * sin(6 * pi * t + i))
            s = max(0.001, env)
            o.scale = (s, s, s)
        for j, (o, k) in enumerate(parts[4:]):
            a = pi / 4 + j * pi / 2 + spin
            o.location = (0.02 * sin(6 * pi * t + k), 0.015 * cos(4 * pi * t + k), 0.8)
            o.rotation_euler = (0.0, a, 0.08 * sin(8 * pi * t + k))
            s = max(0.001, env * 0.9)
            o.scale = (s, s, s)
        core.location = (0.0, 0.0, 0.8 + 0.015 * sin(4 * pi * t))
        core.rotation_euler = (0.0, spin, 0.2 * t)
        s = max(0.001, env * 1.1)
        core.scale = (s, s, s)

    return objs, key, ["gold", "fire", "snow"], "Bright cross/star flash with a hot centre and spin decay."


def death_puff(frames):
    objs, dust, shards = [], [], []
    for i in range(10):
        a = 2 * pi * i / 10
        o = ico("dust_%02d" % i, 0.085, (0.28 * cos(a), 0.28 * sin(a), 0.05),
                MATS["grass" if i % 3 else "snow"])
        objs.append(o)
        dust.append((o, a))
    for i in range(8):
        a = 2 * pi * (i + 0.5) / 8
        o = cube("shard_%02d" % i, (0.055, 0.045, 0.11),
                 (0.14 * cos(a), 0.14 * sin(a), 0.8 + 0.12 * unit(9, i)),
                 MATS["steel" if i % 2 else "snow"])
        objs.append(o)
        shards.append((o, a, 0.8 + 0.12 * unit(9, i)))

    def key(f, pf):
        t = f / (frames - 1)
        fade = keys_at(t, [(0, 0.9), (0.45, 1.0), (0.75, 0.7), (1, 0.001)])
        for i, (o, a) in enumerate(dust):
            ph = (i / 10 + 0.55 * t) % 1.0
            r = 0.28 + 0.72 * ph
            o.location = (r * cos(a), r * sin(a), 0.05 + 0.09 * sin(pi * ph))
            o.rotation_euler = (0.1 * t, 0.2 * t, a + 2 * pi * ph)
            s = max(0.001, fade * (1.0 - 0.75 * ph))
            o.scale = (s, s, s)
        for i, (o, a, h0) in enumerate(shards):
            local = clamp((t - 0.08 * (i % 4)) / max(0.2, 1.0 - 0.08 * (i % 4)))
            z = max(0.03, h0 * (1.0 - local ** 1.6))
            o.location = (0.14 * cos(a) + 0.25 * local * cos(a),
                          0.14 * sin(a) + 0.25 * local * sin(a), z)
            o.rotation_euler = (1.6 * local + 0.05 * sin(2 * pi * t + i),
                                0.5 * local, a + 2.5 * local)
            s = max(0.001, 1.0 - 0.85 * clamp((t - 0.6) / 0.4))
            o.scale = (s, s, s)

    return objs, key, ["grass", "snow", "steel"], "Low rolling dust ring plus falling stone/metal shards."


def hop_dust(frames):
    objs, puffs, specks = [], [], []
    for i in range(8):
        a = 2 * pi * i / 8
        o = ico("puff_%02d" % i, 0.055, (0.18 * cos(a), 0.18 * sin(a), 0.03),
                MATS["grass" if i % 2 == 0 else "snow"])
        objs.append(o)
        puffs.append((o, a))
    for i in range(4):
        a = 2 * pi * (i + 0.5) / 4
        o = cube("speck_%02d" % i, (0.035, 0.035, 0.035),
                 (0.16 * cos(a), 0.16 * sin(a), 0.04), MATS["snow"])
        objs.append(o)
        specks.append((o, a))

    def key(f, pf):
        t = f / frames  # one full period sampled at N keys (loop)
        spin = 2 * pi * t
        for i, (o, a) in enumerate(puffs):
            ph = (i / 8 + t) % 1.0
            r = 0.18 + 0.3 * ph
            o.location = (r * cos(a), r * sin(a), 0.03 + 0.11 * sin(pi * ph))
            o.rotation_euler = (0.1 * sin(spin + i), 0.2 * sin(spin + i), a + spin)
            s = max(0.02, 0.9 * (1.0 - ph))
            o.scale = (s, s, s)
        for i, (o, a) in enumerate(specks):
            ph = (i / 4 + t) % 1.0
            r = 0.16 + 0.18 * ph
            o.location = (r * cos(a), r * sin(a), 0.04 + 0.16 * sin(pi * ph))
            o.rotation_euler = (0.3 * sin(spin + i), 0.4 * cos(spin + i), spin)
            s = max(0.02, 0.8 * (1.0 - ph))
            o.scale = (s, s, s)

    return objs, key, ["grass", "snow"], "Small periodic footfall dust ring; seamless 24-frame loop."


def staff_lightning(frames):
    objs, segs = [], []
    main = []
    for i in range(7):
        o = cube("bolt_%02d" % i, (0.055, 0.2, 0.055), (0.0, 0.0, 0.0),
                 MATS["arcane" if i % 2 else "steel"])
        objs.append(o)
        segs.append(o)
        main.append(o)
    for branch, src in enumerate((3, 4)):
        for j in range(3):
            o = cube("fork_%d_%d" % (branch, j), (0.04, 0.16, 0.04), (0.0, 0.0, 0.0),
                     MATS["snow" if j % 2 else "arcane"])
            objs.append(o)
            segs.append((o, branch, j, src))

    def key(f, pf):
        t = f / (frames - 1)
        flick = keys_at(t, [(0, 1.0), (0.12, 0.05), (0.2, 1.0), (0.3, 0.1),
                            (0.42, 0.85), (0.55, 0.0), (1, 0.0)])
        for i, o in enumerate(main):
            a = 0.12 * patch(11, i)
            o.location = (a + 0.02 * sin(14 * pi * t + i), 0.05 * patch(23, i),
                          0.12 + 0.19 * i + 0.015 * sin(10 * pi * t + i))
            o.rotation_euler = (0.12 * sin(12 * pi * t + i), 0.1 * patch(5, i),
                                0.18 * sin(9 * pi * t + i))
            s = max(0.001, flick * (1.0 - 0.05 * i))
            o.scale = (s, s, s)
        for o, branch, j, src in segs[7:]:
            side = 1.0 if branch == 0 else -1.0
            o.location = (0.12 * patch(11, src) + side * (0.1 + 0.09 * j)
                          + 0.02 * sin(16 * pi * t + j),
                          0.05 * patch(23, src),
                          0.12 + 0.19 * src - 0.11 * j + 0.015 * cos(12 * pi * t + j))
            o.rotation_euler = (0.2 * sin(14 * pi * t + j), side * 0.35,
                                0.25 * sin(10 * pi * t + j))
            s = max(0.001, flick * 0.85 * (1.0 - 0.15 * j))
            o.scale = (s, s, s)

    return objs, key, ["arcane", "steel", "snow"], "Forked arcane bolt chain with flicker key patches."


def tongue_splash(frames):
    objs, fan, foam = [], [], []
    for i in range(9):
        a = -0.95 + 1.9 * i / 8
        o = cone("drop_%02d" % i, 0.05, 0.19, 5, (0.12 * sin(a), 0.0, 0.08),
                 MATS["water" if i % 3 else "snow"])
        objs.append(o)
        fan.append((o, a))
    for i in range(4):
        a = -1.0 + 2.0 * i / 3
        o = ico("foam_%02d" % i, 0.06, (0.3 * sin(a), 0.0, 0.04), MATS["snow"])
        objs.append(o)
        foam.append((o, a))

    def key(f, pf):
        t = f / (frames - 1)
        fade = keys_at(t, [(0, 1.0), (0.55, 1.0), (0.8, 0.5), (1, 0.001)])
        for i, (o, a) in enumerate(fan):
            spread = 0.6 + 0.5 * unit(13, i)
            x = 0.12 * sin(a) + 0.95 * t * sin(a) * spread
            z = max(0.03, 0.08 + 0.95 * t * cos(a) * spread - 0.8 * t * t)
            o.location = (x, 0.03 * sin(4 * pi * t + i), z)
            o.rotation_euler = (0.15 * sin(2 * pi * t + i), pi / 2 - 0.35 * a,
                                a + 0.1 * sin(6 * pi * t + i))
            s = max(0.001, fade)
            o.scale = (s, s, s)
        for i, (o, a) in enumerate(foam):
            ph = (i / 4 + 0.7 * t) % 1.0
            o.location = (0.3 * sin(a) + 0.35 * ph * sin(a),
                          0.08 * sin(2 * pi * ph + i), 0.04 + 0.12 * sin(pi * ph))
            o.rotation_euler = (0.2 * t, 0.3 * t, a + 2 * pi * ph)
            s = max(0.001, fade * (1.0 - 0.8 * ph))
            o.scale = (s, s, s)

    return objs, key, ["water", "snow"], "Water splash fan with ballistic droplets and foam beads."


def stinger_drip(frames):
    objs, drops, splashes = [], [], []
    for i in range(9):
        x = 0.32 * patch(17, i)
        y = 0.24 * patch(29, i)
        o = ico("venom_%02d" % i, 0.045, (x, y, 1.15),
                MATS["grass" if i % 3 else "arcane"])
        objs.append(o)
        drops.append((o, x, y))
    for i in range(4):
        a = 2 * pi * i / 4
        o = cube("splash_%02d" % i, (0.09, 0.035, 0.035),
                 (0.12 * cos(a), 0.12 * sin(a), 0.03), MATS["grass"])
        objs.append(o)
        splashes.append((o, a))

    def key(f, pf):
        t = f / (frames - 1)
        for i, (o, x, y) in enumerate(drops):
            delay = 0.02 * (i % 5)
            local = clamp((t - delay) / max(0.2, 1.0 - delay))
            z = 0.025 + 1.12 * (1.0 - local ** 1.5)
            o.location = (x + 0.05 * sin(6 * pi * t + i),
                          y + 0.04 * cos(5 * pi * t + i), z)
            o.rotation_euler = (0.3 * sin(4 * pi * t + i), 0.4 * t, 2 * pi * t + i)
            s = max(0.001, 1.0 - 0.45 * clamp((t - 0.8) / 0.2))
            o.scale = (s, s, s)
        for i, (o, a) in enumerate(splashes):
            local = clamp((t - 0.55) / 0.45)
            r = 0.12 + 0.3 * local
            o.location = (r * cos(a), r * sin(a), 0.03 + 0.04 * sin(pi * local))
            o.rotation_euler = (0.1 * t, 0.2 * t, a + 0.5 * sin(4 * pi * t + i))
            s = max(0.001, 0.9 * sin(pi * clamp(local * 1.2)) + 0.01)
            o.scale = (s, s, s)

    return objs, key, ["grass", "arcane"], "Venom droplets arc down with staggered ground splashes."


def tentacle_wake(frames):
    objs, lines, foam = [], [], []
    for i in range(12):
        x = -0.95 + 1.9 * i / 11
        o = cube("wake_%02d" % i, (0.30, 0.045, 0.028), (x, 0.0, 0.03),
                 MATS["water" if i % 3 else "snow"])
        objs.append(o)
        lines.append((o, x))
    for i in range(5):
        x = -0.8 + 0.4 * i
        o = ico("crest_%02d" % i, 0.07, (x, 0.0, 0.06), MATS["snow"])
        objs.append(o)
        foam.append((o, x))

    def key(f, pf):
        t = f / (frames - 1)
        sweep = -0.55 + 1.1 * t
        env = keys_at(t, [(0, 0.2), (0.25, 1.0), (0.75, 1.0), (1, 0.02)])
        for i, (o, x) in enumerate(lines):
            wave = max(0.0, sin(2 * pi * (x / 2.0 + 0.5 * t)))
            o.location = (x + sweep, 0.03 * sin(4 * pi * t + i), 0.03 + 0.1 * wave)
            o.rotation_euler = (0.05 * sin(2 * pi * t + i), 0.06 * wave,
                                0.12 * cos(2 * pi * (x / 2.0 + t)))
            s = max(0.001, env * (0.7 + 0.3 * wave))
            o.scale = (s, s, s)
        for i, (o, x) in enumerate(foam):
            wave = max(0.0, sin(2 * pi * (x / 2.0 + 0.5 * t)))
            o.location = (x + sweep, 0.05 * sin(4 * pi * t + i), 0.06 + 0.15 * wave)
            o.rotation_euler = (0.2 * t, 0.3 * wave, 2 * pi * t + i)
            s = max(0.001, env * (0.5 + 0.6 * wave))
            o.scale = (s, s, s)

    return objs, key, ["water", "snow"], "Foam/water wake sweeping sideways with travelling crests."


def mine_sparks(frames):
    objs, sparks = [], []
    for i in range(14):
        a = 2 * pi * i / 14 + 0.18 * patch(31, i)
        o = cone("spark_%02d" % i, 0.028, 0.13, 4, (0.0, 0.0, 0.35),
                 MATS["gold" if i % 3 == 0 else ("steel" if i % 3 == 1 else "fire")])
        objs.append(o)
        sparks.append((o, a))

    def key(f, pf):
        t = f / (frames - 1)
        fade = keys_at(t, [(0, 1.0), (0.45, 0.85), (1, 0.001)])
        for i, (o, a) in enumerate(sparks):
            speed = 0.55 + 0.45 * unit(37, i)
            r = 0.8 * t * speed
            z = 0.35 + 0.65 * sin(pi * t) - 0.3 * t
            o.location = (r * cos(a), r * sin(a), max(0.03, z))
            o.rotation_euler = (0.0, pi / 2, a + 3 * pi * t + 0.1 * sin(8 * pi * t + i))
            s = max(0.001, fade)
            o.scale = (s, s, s)

    return objs, key, ["gold", "steel", "fire"], "Gold/steel sparks bursting from a pick strike."


def build_dust(frames):
    objs, puffs, chips = [], [], []
    for i in range(7):
        a = 2 * pi * i / 7
        o = ico("puff_%02d" % i, 0.075, (0.12 * cos(a), 0.12 * sin(a), 0.04),
                MATS["grass" if i % 2 == 0 else "snow"])
        objs.append(o)
        puffs.append((o, a))
    for i in range(4):
        a = 2 * pi * (i + 0.5) / 4
        o = cube("chip_%02d" % i, (0.045, 0.04, 0.045),
                 (0.1 * cos(a), 0.1 * sin(a), 0.3), MATS["gold" if i == 0 else "grass"])
        objs.append(o)
        chips.append((o, a))

    def key(f, pf):
        t = f / (frames - 1)
        fade = keys_at(t, [(0, 0.7), (0.35, 1.0), (1, 0.001)])
        for i, (o, a) in enumerate(puffs):
            ph = (i / 7 + 0.75 * t) % 1.0
            r = 0.12 + 0.38 * ph
            o.location = (r * cos(a), r * sin(a), 0.04 + 0.13 * sin(pi * ph))
            o.rotation_euler = (0.15 * t, 0.2 * t, a + 2 * pi * ph)
            s = max(0.001, fade * (1.0 - 0.85 * ph))
            o.scale = (s, s, s)
        for i, (o, a) in enumerate(chips):
            z = max(0.03, 0.3 - 0.28 * t + 0.03 * sin(4 * pi * t + i))
            o.location = (0.1 * cos(a) + 0.18 * t * cos(a),
                          0.1 * sin(a) + 0.18 * t * sin(a), z)
            o.rotation_euler = (2 * pi * t, 1.5 * t, a + 2 * pi * t)
            s = max(0.001, 1.0 - 0.9 * clamp((t - 0.6) / 0.4))
            o.scale = (s, s, s)

    return objs, key, ["grass", "snow", "gold"], "Small dust puff and a few chips from placing a block."


# ---------------------------------------------------------------------------
# Effect catalogue: name, frames, loop, builder, sync. Sync is stated against
# clip names verified in wave2/manifest.json and player/manifest.json.
# ---------------------------------------------------------------------------

TELEGRAPHS = ["scorpion_queen_telegraph", "toad_king_telegraph",
              "stormcaller_telegraph", "ocean_leviathan_telegraph",
              "colossus_telegraph"]
ATTACKS = ["goblin_attack", "skeleton_attack", "wolf_attack", "spider_attack",
           "scorpion_queen_attack", "toad_king_attack", "stormcaller_attack",
           "ocean_leviathan_attack", "colossus_attack"]
HURTS = ["goblin_hurt", "skeleton_hurt", "wolf_hurt"]
DEATHS = ["goblin_death", "skeleton_death", "wolf_death"]
SLASHES = ["goblin_attack", "skeleton_attack", "wolf_attack", "spider_attack"]

CATALOGUE = [
    ("fx_boss_telegraph_ring", 48, False, boss_telegraph_ring,
     {"clips": TELEGRAPHS, "frame": 0, "manifest": "assets/animations/wave2/manifest.json",
      "note": "Start at the first frame of every boss telegraph."}),
    ("fx_boss_impact_burst", 30, False, boss_impact_burst,
     {"clips": ATTACKS, "frame": 15, "manifest": "assets/animations/wave2/manifest.json",
      "note": "Trigger at the strike frame of *_attack clips (~frame 15); the FX peaks at its own local frame 0."}),
    ("fx_slash_arc", 18, False, slash_arc,
     {"clips": SLASHES, "frame": 15, "manifest": "assets/animations/wave2/manifest.json",
      "note": "Sword/claw attacks; sweep trail starts at the chop/bite contact frame."}),
    ("fx_hit_flash", 15, False, hit_flash,
     {"clips": HURTS, "frame": 4, "manifest": "assets/animations/wave2/manifest.json",
      "note": "Peak recoil frame of every *_hurt clip."}),
    ("fx_death_puff", 45, False, death_puff,
     {"clips": DEATHS, "frame": 24, "manifest": "assets/animations/wave2/manifest.json",
      "note": "Trigger as the body folds down at frame 24 of *_death clips."}),
    ("fx_hop_dust", 24, True, hop_dust,
     {"clips": ["slime_hop", "spider_scuttle"], "frame": 12,
      "manifest": "assets/animations/wave2/manifest.json",
      "note": "Footfall/landing (slime_hop apex-to-land at frame 12; spider_scuttle footfalls repeat every 6 f)."}),
    ("fx_staff_lightning", 24, False, staff_lightning,
     {"clip": "stormcaller_attack", "frame": 12,
      "manifest": "assets/animations/wave2/manifest.json",
      "note": "Staff slam / bolt release frame."}),
    ("fx_tongue_splash", 30, False, tongue_splash,
     {"clip": "toad_king_attack", "frame": 10,
      "manifest": "assets/animations/wave2/manifest.json",
      "note": "Tongue/lunge contact frame."}),
    ("fx_stinger_drip", 36, False, stinger_drip,
     {"clip": "scorpion_queen_attack", "frame": 12,
      "manifest": "assets/animations/wave2/manifest.json",
      "note": "Tail stinger strike frame."}),
    ("fx_tentacle_wake", 36, False, tentacle_wake,
     {"clip": "ocean_leviathan_attack", "frame": 12,
      "manifest": "assets/animations/wave2/manifest.json",
      "note": "Tentacle sweep contact frame."}),
    ("fx_mine_sparks", 30, False, mine_sparks,
     {"clip": "mine", "frames": [10, 21],
      "manifest": "assets/animations/player/manifest.json",
      "note": "Both two-handed pick strikes in the player mine clip."}),
    ("fx_build_dust", 24, False, build_dust,
     {"clip": "build", "frame": 8,
      "manifest": "assets/animations/player/manifest.json",
      "note": "Block placement / pat contact frame."}),
]


def validate_sync():
    """Only ever reference clips that actually exist in the manifests."""
    known = {}
    for path, rel in ((WAVE2_MANIFEST, "assets/animations/wave2/manifest.json"),
                      (PLAYER_MANIFEST, "assets/animations/player/manifest.json")):
        with open(path) as fh:
            data = json.load(fh)
        known[rel] = {c["clip"] for c in data["clips"]}
    for name, _, _, _, sync in CATALOGUE:
        clips = [sync["clip"]] if "clip" in sync else sync["clips"]
        for clip in clips:
            if clip not in known[sync["manifest"]]:
                raise RuntimeError("%s sync clip %r missing from %s" %
                                   (name, clip, sync["manifest"]))


def main():
    validate_sync()
    os.makedirs(OUT, exist_ok=True)
    records = []
    for name, frames, loop, builder, sync in CATALOGUE:
        clean()
        objs, key_fn, colours, note = builder(frames)
        make_action(objs, name, frames, key_fn)
        path = os.path.join(OUT, name + ".glb")
        export(name, path)
        records.append({
            "name": name,
            "path": "assets/animations/wave2/fx/" + name + ".glb",
            "duration_s": round(frames / FPS, 4),
            "fps": FPS,
            "loop": loop,
            "frames": frames,
            "primitive_count": len(objs),
            "triangle_count": triangles(objs),
            "palette": colours,
            "colours": {c: PALETTE[c] for c in colours},
            "sync": sync,
            "notes": note,
        })
        print("[FX ] %-24s frames=%3d prims=%2d tris=%3d" %
              (name, frames, len(objs), triangles(objs)))

    manifest = {
        "schema": "fight-animations.wave2-fx/1",
        "generator": "scripts/generate_wave2_fx.py",
        "verifier": "scripts/verify_wave2_fx.py",
        "tooling": {
            "blender": bpy.app.version_string,
            "fps": FPS,
            "pipeline": "node-keyed flat-shaded primitives; one slotted Action per GLB",
        },
        "conventions": {
            "fps": FPS,
            "samplers": "LINEAR",
            "export_yup": True,
            "one_action_per_glb": True,
            "scale_keys": "allowed and intentional for FX (pop-in, expansion, fade-out); forbidden on character rigs",
            "geometry": "named low-poly primitives only: cubes, 4-8 sided cones/cylinders, ico spheres subdiv 0-1; flat shaded; no textures",
            "determinism": "no randomness; math.sin/cos plus seeded key patches",
            "time": "keys at frames 0..(frames-1); sampler times span 0..(frames-1)/30 s; duration_s = frames/fps",
            "loop": "loops sample one full period at N keys with no duplicate terminal key",
            "sync": "frame/frames are source-clip frames at which to trigger the FX; the FX event itself starts at its own local frame 0",
            "sync_sources": ["assets/animations/wave2/manifest.json",
                             "assets/animations/player/manifest.json"],
        },
        "palette": PALETTE,
        "effects": records,
        "summary": {
            "count": len(records),
            "total_primitives": sum(r["primitive_count"] for r in records),
            "total_triangles": sum(r["triangle_count"] for r in records),
            "loops": sum(1 for r in records if r["loop"]),
            "total_duration_s": round(sum(r["duration_s"] for r in records), 4),
            "palette_used": sorted({c for r in records for c in r["palette"]}),
            "sync_clips": sorted({c for r in records
                                  for c in ([r["sync"]["clip"]] if "clip" in r["sync"]
                                            else r["sync"]["clips"])}),
        },
    }
    with open(MANIFEST, "w") as fh:
        json.dump(manifest, fh, indent=2)
    print("wrote", os.path.relpath(MANIFEST, ROOT), manifest["summary"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
