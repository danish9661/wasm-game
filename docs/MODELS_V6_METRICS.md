# Starfall — models v6 metrics (QA-driven sixth wave)

Branch: `models-v6`. Generated from exported GLB geometry, not from targets.
Machine-readable source: `assets/models/expansion_models_v6.json`.

## Scope and method

Twenty **new** models close the gaps left by the four visual-QA audits:
three interior variants (library, guard post, market stall), five
biome-specific landmark pieces, seven NPC profession variants and five new
building silhouettes. Nine existing models receive targeted MINOR-polish
rebuilds where the critics' deferred notes could be fixed without invalidating
animation manifests (see below).

Every asset is authored by `scripts/blender/gen_models_v6.py` with:

- **bmesh forms** only — `create_cube`, `create_cone` (4–8 segments),
  `create_icosphere` (subdivision 0), transformed on a 1 u grid;
- a **selective silhouette bevel** (one 80°-limited segment, width 0.005 u) on
  the dominant parts, not a blanket bevel on micro-detail;
- **weighted normals**, then forced flat shading on every polygon;
- a deterministic planar UV project followed by **smart UV pack** in
  `expansion_common.hero_finish`;
- **palette-only** Principled materials (lightness variants of
  `docs/ART_DIRECTION.md`; no new hues, no textures);
- final vertex-level re-base so **base z = 0** and the footprint is exactly
  centred, with identity node transforms.

Reproduce:

```bash
~/.local/blender/blender -b --python scripts/blender/gen_models_v6.py
python3 scripts/verify_models.py --quiet && python3 scripts/check_metrics.py
```

## Player-relative scale

Reference: `player.glb` = **1.55 u** tall. Ratios are measured height / 1.55.
Building silhouettes land at 1.6–2.3× the player; the two tall landmarks reach
2.0× and 2.33×, in line with the existing `watchtower` / `town_hall` family.
Interiors are 4×4 u rooms with a 2.5 u storey; their 4.70 u footprint includes
the south door leaf and corner posts outside the wall grid.

| Model | Tris | W x D x H (u) | Base z | Centre X / Z | H / player | Mat |
|---|---:|---|---:|---:|---:|---:|
| `library_interior` | 1216 | 4.700 x 4.700 x 2.500 | 0.000 | +0.000 / +0.000 | 1.61x | 15 |
| `guard_post_interior` | 664 | 4.700 x 4.700 x 2.500 | 0.000 | +0.000 / +0.000 | 1.61x | 14 |
| `market_stall_interior` | 820 | 4.700 x 4.700 x 2.500 | 0.000 | +0.000 / +0.000 | 1.61x | 11 |
| `desert_obelisk_landmark` | 190 | 1.949 x 1.500 x 3.104 | 0.000 | +0.000 / +0.000 | 2.00x | 3 |
| `tundra_totem_landmark` | 208 | 1.400 x 1.300 x 2.450 | 0.000 | +0.000 / +0.000 | 1.58x | 5 |
| `swamp_shrine_landmark` | 248 | 1.700 x 1.520 x 2.460 | 0.000 | +0.000 / +0.000 | 1.59x | 8 |
| `volcanic_forge_landmark` | 234 | 1.840 x 1.371 x 1.960 | 0.000 | +0.000 / +0.000 | 1.26x | 4 |
| `ocean_lighthouse_landmark` | 298 | 2.003 x 1.548 x 3.615 | 0.000 | +0.000 / +0.000 | 2.33x | 7 |
| `npc_alchemist` | 432 | 1.093 x 0.570 x 2.140 | 0.000 | +0.000 / +0.000 | 1.38x | 8 |
| `npc_cartographer` | 368 | 1.155 x 0.692 x 1.960 | 0.000 | +0.000 / +0.000 | 1.26x | 8 |
| `npc_chef` | 384 | 1.155 x 0.575 x 2.080 | 0.000 | +0.000 / +0.000 | 1.34x | 9 |
| `npc_herbalist` | 416 | 1.198 x 0.510 x 1.980 | 0.000 | +0.000 / +0.000 | 1.28x | 9 |
| `npc_jeweler` | 384 | 1.120 x 0.520 x 1.790 | 0.000 | +0.000 / +0.000 | 1.16x | 7 |
| `npc_shepherd` | 376 | 1.400 x 0.560 x 2.200 | 0.000 | +0.000 / +0.000 | 1.42x | 7 |
| `npc_physician` | 412 | 1.059 x 0.480 x 1.790 | 0.000 | +0.000 / +0.000 | 1.16x | 9 |
| `granary` | 304 | 2.638 x 2.014 x 2.494 | 0.000 | +0.000 / +0.000 | 1.61x | 5 |
| `chapel` | 290 | 2.284 x 2.060 x 3.180 | 0.000 | +0.000 / +0.000 | 2.05x | 8 |
| `fisher_hut` | 328 | 2.600 x 2.300 x 2.589 | 0.000 | +0.000 / +0.000 | 1.67x | 7 |
| `gatehouse` | 344 | 3.240 x 2.110 x 3.120 | 0.000 | +0.000 / +0.000 | 2.01x | 7 |
| `observatory` | 244 | 1.600 x 1.790 x 2.625 | 0.000 | +0.000 / +0.000 | 1.69x | 8 |

### MINOR-polish rebuilds (existing filenames, existing pivots)

| Model | Tris | W x D x H (u) | Base z | Centre X / Z | H / player | Change |
|---|---:|---|---:|---:|---:|---|
| `bat` | 168 | 1.658 x 0.882 x 1.200 | 0.000 | +0.000 / +0.000 | 0.77x | lighter wing/bone palette, gold eyes and fangs read at 60 px |
| `fire_elemental` | 288 | 1.726 x 0.698 x 2.360 | 0.000 | +0.000 / +0.000 | 1.52x | one continuous flame core; arms, fists, legs and tongues seated on it |
| `stone_elemental` | 256 | 1.210 x 0.680 x 2.470 | 0.000 | +0.000 / +0.000 | 1.59x | tall obsidian body with arcane crown, clearly unlike the broad grey golem |
| `crossbow` | 118 | 1.074 x 0.800 x 1.170 | 0.000 | +0.000 / +0.000 | 0.76x | raised from 0.62 u; stock, limbs, string and a readable bolt |
| `arrow` | 146 | 0.274 x 0.316 x 1.070 | 0.000 | +0.000 / +0.000 | 0.69x | broad gold/sand flights instead of a hairline shaft |
| `coral_brain` | 152 | 0.939 x 0.750 x 0.622 | 0.000 | +0.000 / +0.000 | 0.40x | saturated arcane body with fire/bone ridges |
| `apple_tree` | 240 | 1.873 x 1.415 x 2.601 | 0.000 | +0.000 / +0.000 | 1.68x | four larger apples, two boughs, raised to the 2.5–3.0 u tree band |
| `hay_bale` | 112 | 1.063 x 0.880 x 0.782 | 0.000 | +0.000 / +0.000 | 0.51x | rounded elliptical bale with leather bands and straw tufts |
| `volcanic_bomb` | 188 | 0.880 x 0.809 x 0.880 | 0.000 | +0.000 / +0.000 | 0.57x | wide fire/gold lava cracks and seated embers |

## Budget and contract check

- New interior variants: **1216 / 664 / 820 tris** — all below the 1500-tri
  assembled/interior allowance.
- All 17 other new models: **190–432 tris** — all below the 500-tri cap.
- All nine polish rebuilds: **112–288 tris** — well below 500.
- Every one of the 29 exported GLBs: base z = 0.000, centre X/Z = +0.000,
  flat-shaded palette-only materials, no textures, stable filenames.
- `python3 scripts/verify_models.py --quiet`: **274 GLBs, 274 passed, 0 failed**
  (up from 254; 20 new filenames).
- `python3 scripts/check_metrics.py`: OK — 274 models, player 1.55 u,
  house 3.10 u, all base Y ≈ 0.

### Full-gate results

| Gate | Result |
|---|---|
| `cargo test -p game` | 207 passed, 0 failed |
| `cargo check --target wasm32-unknown-unknown -p wasm_game` | OK |
| `./build.sh` | OK (wasm-pack release build; GLB list regenerated to 274 entries) |
| `tests/smoke_3d.mjs` (`SMOKE_PORT=8879`) | 8 passed, 0 failed, 0 console errors |
| `verify_wave2_animations.py` | 30/30 passed |
| `verify_wave3_animations.py` | 34/34 passed |
| `verify_wave4_animations.py` | 105/105 passed |

No animation manifest or generated animation file changed: none of the
rebuilt models appears in the wave2/3/4 rig manifests.

## Deliberate exception

`npc_blacksmith` was **not** modified. Its GLB SHA-256 is hash-locked in
`assets/animations/wave4/manifest.json`, and the wave-4 rig enumerates every
mesh node (`tongs_0`, `tongs_1`, `apron`, …). Adding or rebuilding a held tool
would require regenerating all 105 bound wave-4 clips. It is listed in the
"remaining known issues" section of `docs/VISUAL_QA_CHARACTERS.md` instead of
being silently deferred.
