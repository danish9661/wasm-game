# Starfall — 3D Art Direction (shared contract for all asset agents)

## Style: "Low-Poly Diorama"

Chunky, faceted, flat-shaded models with clean silhouettes and saturated accent
colours. Think a tabletop diorama: readable at a glance, no textures, no smooth
shading, no realism. Every asset must read clearly at ~60 px on screen.

## Camera & composition

- Orthographic camera, yaw **45°**, pitch **30°** (close to the original 2:1
  isometric look). No perspective distortion.
- Sun key light (warm, high), cool hemisphere fill, soft contact shadows.
- Emissive accents for fire, crystals, magic and the Star Crown.

## Palette (hex, sRGB)

| Element | Colour |
|---|---|
| Player tunic | `#FF8C26` |
| Skin | `#EAB88C` |
| Wood / dark leather | `#6B4423` / `#592F1A` |
| Stone / iron | `#7A7F87` / `#B8C0C8` |
| Steel / blade | `#C8D0D8` |
| Gold / crown / accents | `#D9A62E` |
| Leaves | `#3A7A33` |
| Grass / forest floor | `#5B8C3A` |
| Desert sand | `#D9B46A` |
| Snow | `#E8F0FA` |
| Swamp green | `#4A6741` |
| Water | `#3A7CA5` |
| Arcane purple | `#6B3FA0` |
| Fire / lava | `#FF6A2A` |
| Enemy red | `#C0392B` |

Use these as the base; vary lightness slightly for contrast, do not invent new
hues.

## Proportions & units

- **1 tile = 1.0 Blender unit**, ground plane at `z = 0`.
- Character height ≈ **1.6 u**; tree ≈ 2.5–3.0 u; house ≈ 3.0 u;
  held weapon ≈ 1.0–1.3 u.
- Base of every asset sits exactly on `z = 0`, centred on `(0, 0)`.
- glTF export is **Y-up** (Blender exporter handles the Z-up → Y-up conversion).

## Geometry rules

- Low-poly only: cubes, 4–8 sided cones/cylinders, ico spheres subdivision 0–1.
- **< 500 triangles per asset** (most should be far fewer).
- **Flat shading** (`polygon.use_smooth = False`) — no smooth normals.
- Materials: Principled BSDF, roughness ≈ 0.85, metallic only for steel/gold.
- One collection per asset, named exactly like the output GLB (`tree.glb`,
  `sword.glb`, …). Export each collection separately with `use_selection=True`.
- No external textures, no image files, no dependencies beyond `bpy`.

## File layout

- Shared helpers: `scripts/blender/common.py`
- Generators: `scripts/blender/gen_<set>.py`
- Output models: `assets/models/<name>.glb`
- Contact-sheet previews: `assets/previews/<set>.png`

## Runtime plan (renderer)

- Replace the 2D quad rasteriser with a wgpu glTF pipeline: instanced draw per
  mesh, one depth buffer instead of painter's-algorithm Y-sort.
- Terrain stays a tile grid but becomes a heightfield mesh with per-biome
  materials.
- Day/night = directional light + shader uniform, not per-vertex colours.
- Keeps existing gameplay/simulation code untouched: `game/src/sim.rs`,
  `quest`, `combat`, `building`, `world` stay as-is; only `render.rs` and
  `web/src/renderer.rs` change.
