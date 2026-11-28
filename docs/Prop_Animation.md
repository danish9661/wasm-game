# Animated props and interior elements

This document records how the per-frame world-element animations in
`assets/models/animprop_<name>_<NN>.glb` were authored. The clip manifest is
`assets/models/prop_poses.json`, which uses the same schema as `poses.json`:

```json
"<clip>": { "fps": 8, "frames": ["animprop_<clip>_00.glb", ...], "loop": true }
```

All clips loop. Playback is a plain model swap on each frame — no skinning,
morph targets, shader tricks or runtime animation code. Nothing in `web/src`
was changed.

## Pipeline (Blender, headless)

Source of truth: `scripts/bake_prop_poses.py`.

```bash
blender -b --python scripts/bake_prop_poses.py
python3 scripts/assemble_prop_sheet.py \
    /tmp/opencode/prop_sheet_frames assets/previews/prop_pose_bake.png
```

The script builds each frame with plain `bpy.data.meshes.new(...).from_pydata`,
so there are no imported assets, textures or dependencies beyond `bpy`.
Every frame is one merged mesh with flat-shaded faces (`use_smooth = False`)
and Principled materials limited to the `ART_DIRECTION.md` palette (plus
lightness variants of the same hues: darker/lighter leaf, stone, cloth and
water). Roughness is 0.85, metallic 0, no image nodes.

The exporter call mirrors `scripts/bake_pose_frames.py`: `export_yup=True`,
`export_animations=False`, `use_selection=True`, so a Blender Z-up vertex
`(x, y, z)` becomes a glTF Y-up vertex `(x, z, -y)` and the ground plane stays
at glTF `Y = 0`.

### Shared pivot / size

For each clip the script builds all frames first, computes the **union** AABB
across them, and derives one normalisation offset:

```
dx = -(minX + maxX)/2,  dy = -(minY + maxY)/2,  dz = -minZ
```

That *same* offset is applied to every frame. Per-frame re-centring is
deliberately avoided: the static anchor (ash bed, pole, trunk, jambs, door
frame, water slab, …) sits at the origin and touches `z = 0`, so the union
offset is ~zero for most clips and every frame shares an identical pivot and
anchor size. Moving parts may change a frame's own AABB, but they never move
the origin. The script asserts, before export, that each frame has base
`Y ≈ 0`, X/Z centre within the verifier tolerance, and fewer than 400
triangles. The largest baked frame is 308 triangles.

### Poses and phase

Every clip uses `phase = 2*pi*i / N`, so frame 0 and the wrap-around frame
join seamlessly. Where a prop needs a less mechanical rhythm, a second
harmonic (`sin(3*phase)`, `sin(2*phase + k)`) or a per-part phase offset is
added. Frame rates are chosen for readability rather than physical speed:
fast flickers run at 8 fps, ambient sways at 4–6 fps.

## Element-by-element authoring notes

### campfire — `campfire` (8 frames) and `campfire_ember` (8 frames)

*Static:* a 6-sided ash bed from `z=0..0.04`, six subdivision-0 icosahedron
stones on a 0.42 u ring, and four 5-sided log prisms crossed at the centre.
*Motion:* the outer fire cone changes height (`0.62 * (1 + .20 sin p +
.09 sin(3p+.7))`) and radius, and its tip leans with `sin(p+.5)`; the inner
gold cone chases it with a different phase. A gold ember ico bobs in Z and
swells on the second harmonic. Triangle count: 242.

The separate `campfire_ember` clip is the glow-bed read on its own: an ash
disc, five coals whose icos are scaled by `1 + .22 sin(2p + k)` (alternating
fire/gold) and three small sparks orbiting at rising heights. 180 triangles.

### torch / brazier / lantern (6 frames each)

*Torch:* a fixed 6-sided handle and leather wrap; the same two-cone flame
pair as the campfire, smaller, flickering above `z=0.74`. 58 triangles.

*Brazier:* three 5-sided iron legs and an 8-sided bowl; the gold coal cone
pulses (`1 + .18 sin 2p`) and the flame pair flickers above it. 108 triangles.

*Lantern:* fixed iron base, 8-sided fire-coloured glass, four frame posts,
hood and handle. The *light pulse* is a gold icosahedron behind the glass
scaled by `1 + .30 sin(2p+.4)` and stretched vertically. 174 triangles.

### portal — 8 frames

A stone step/threshold, two pillars with feet, seven rotated arch blocks and
a dim arcane plane are static. The arcane heart ico pulses (`1 + .22 sin 3p`),
and six rune icos orbit it in the portal plane with a depth wobble
(`0.10 sin(3p + k)`), giving a swirl rather than a spin. 308 triangles — the
heaviest clip, still well under the 400 budget.

### banner — 8 frames

A fixed pole, crossbar, stone foot and gold finial. The cloth is a 4×4 quad
grid hanging from `z=2.02` to `z=0.98`; each vertex's depth is
`-0.05 - 0.10*sin(0.9*2pi*u + 0.55*2pi*v - phase)` and the bottom edge
tapers to 70% width. Gold emblem bars follow the cloth surface. 120 triangles.

### tree / pine — 8 frames each

*Tree:* a fixed 6-sided trunk plus four subdivision-0 canopy icos. Each
cluster sways in X/Y with its own phase offset (`0.05..0.12 u`) and tilts
slightly, so the silhouette changes without any translation of the trunk.
100 triangles.

*Pine:* a fixed trunk and three 8-sided conical tiers. Each tier is rebuilt
per frame as a tilted frustum leaning by `amp*sin(p+off)`, with the amplitude
increasing from 0.03 u at the base to 0.09 u at the tip. 98 triangles.

### water surface — 8 frames

A thin water slab pins the base at `z=0`; a 6×6 quad grid above it carries
concentric crests: `z = 0.04 + 0.09 * (0.5+0.5 sin(2*pi*r/0.65 - phase)) *
max(0, 1 - r/1.15)`. Quads whose crest exceeds 0.62 use a lighter water hue,
so the travelling ripple reads even at small on-screen sizes. 84 triangles.

### interior door — 6 frames

Deep wooden jambs, lintel and a stone threshold stay fixed (the extra depth
keeps the shared X/Z centre stable while the leaf swings). The leaf, its two
panels and the gold knob are generated in local space and rotated about the
hinge at `x=-0.37` by `angle = 0.55*(1 - cos(phase))` — 0° closed to ~63°
open and back. The shared union offset places the pivot at the origin. 104
triangles.

### lit window — 6 frames

A stone backing, two jambs, two rails and two mullions are fixed. The glass
box pulses a few percent in size and alternates its material between fire and
gold according to `lit = 0.5+0.5 sin(2p+.6)`, while a stretched gold glow ico
swells (`1 + .34 sin(2p+.6)`) and sinks behind the panes. 116 triangles.

## Verification

* `python3 scripts/verify_models.py --quiet` — parses every GLB directly and
  checks base Y, X/Z centring, non-empty meshes and triangle budget. All 528
  GLBs in `assets/models` pass (442 pre-existing + 86 new).
* `python3 scripts/check_metrics.py` — re-asserts the metrics invariants.
* `cargo test -p game` and
  `cargo check --target wasm32-unknown-unknown -p wasm_game` — confirm the
  asset drop-in did not disturb the engine or client builds.

The contact sheet `assets/previews/prop_pose_bake.png` shows one row per clip
and one column per frame, rendered headlessly in Blender Workbench at yaw
45° / pitch 30° with MATERIAL colour. It is the visual check that the motion
is actually visible and that the static anchor does not drift between frames.
