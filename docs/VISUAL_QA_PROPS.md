# Visual QA — props / weapons / tools / vehicles family

Status: **in progress** (before sheet captured; fixes pending).

## Scope

All 104 models listed in `scripts/qa/family_props.json`: melee/ranged weapons, shields,
camp and light props, storage and market props, tools, vehicles and wreckage, signs,
banners and decorative statuary. Characters, buildings, interiors, terrain and nature
are audited by their own families.

## Method

1. `scripts/blender/render_gameplay_sheet.py` imports every shipped GLB into a fresh
   headless Blender scene, places a **1.55 u human silhouette** beside it on a 1 u tile
   grid, and renders it with an orthographic camera at yaw 45° / pitch 30°
   (`docs/ART_DIRECTION.md`). A `60 px hero` inset shows the model at the size it has
   when the human is 60 px tall. Workbench shading is used so the flat palette colours
   are not blown out by emissive materials.
2. `scripts/assemble_gameplay_sheet.py` composes the labelled sheets:
   `assets/previews/qa_props/after/sheet_<group>.png` and the quick-scan
   `assets/previews/qa_props/after/sheet_overview_60px.png`.
3. A fresh-eyes critic (no access to source code, only sheets + art direction + brief)
   ranks the worst ten. The top offenders are rebuilt with the game-asset-modeling
   workflow (bmesh parts → bevels → weighted normals → UV pack → glTF/Khronos validate
   → Babylon review) and the sheets are re-rendered and re-critiqued.

## Before evidence

- Before sheets: `assets/previews/qa_props/before/sheet_weapons.png` (11),
  `sheet_camp_light_storage.png` (24), `sheet_market_tools.png` (22),
  `sheet_vehicles_wreck.png` (19), `sheet_decor_signs.png` (28),
  `sheet_overview_60px.png` (all 104 at 60 px). Final sheets use the same names
  under `assets/previews/qa_props/after/`.
- Individual final cells: `assets/previews/qa_props/cells/<group>/<name>.png`;
  the ten first-pass before cells are under `before/cells/`.
- In-game reference: `/tmp/opencode/deployed_play.png` — several plain light-grey cubes
  are visible in the village; `statue.glb` renders as an all-grey cuboid in the sheet and
  is the family model most consistent with that complaint. Other large cubes in that
  screenshot belong to the building/terrain families.

## Critic ranking (before)

Fresh-eyes critic (`general` subagent, no source access; brief in
`scripts/qa/CRITIC_BRIEF.md`), from the sheets above:

| Rank | Model | Severity | Finding |
|---:|---|---|---|
| 1 | `crate_box` | CRITICAL | Plain brown cube, one strap and two pegs; unreadable square at 60 px. |
| 2 | `arrow` | CRITICAL | 1.02 u pole dominated by a broad blue rectangular fletching — reads as a sign, not an arrow. |
| 3 | `fishing_rod` | CRITICAL | Hair-thin rod, cube reel, 2 px float — no held silhouette. |
| 4 | `rope_coil` | MAJOR | Flat tan asterisk of spokes with a black hole, not coiled rope. |
| 5 | `fishing_net` | MAJOR | Small cone on sticks reads as a shrine; no hoop or mesh. |
| 6 | `market_umbrella` | MAJOR | Desaturated salmon panels, stiff cone, grey cross base. |
| 7 | `coral_brain` | MAJOR | Pale/pink angular lump with no ridges or lobes. |
| 8 | `life_ring` | MAJOR | Four disconnected blocks instead of a torus. |
| 9 | `spear` | MAJOR | Hairline shaft and tiny point — a single vertical line at 60 px. |
| 10 | `farm_cart` | MAJOR | Open trough with invisible wheels — a flat brown box at 60 px. |

Additional call-outs: `interior_chest`, `shell_cluster`, `haystack`, `wreck_plank`,
`dock_plank` read as cubes/blobs or flat boards; `crossbow` is undersized (0.82 u).
Palette violations: pink shells/coral, salmon umbrella, blue arrow fletching.

## Fixes (top 10, in progress)

Each offender is rebuilt through the game-asset-modeling pipeline under
`assets/src/props/<id>/` (brief + bmesh build script), exported to
`assets/export/props/<id>.glb`, validated (Khronos included), reviewed in Babylon,
and copied to `assets/models/<original name>.glb` with the original filename and
the base-at-zero / centred pivot contract.

| Model | Rebuild |
|---|---|
| `crate_box` | Slatted plank walls, four corner posts, iron straps, lid planks, gold latch and keyhole. |
| `arrow` | Shortened to 0.78 u, two compact green flights, nock, gold collar, leaf-shaped steel head. |
| `fishing_rod` | Tapered bent rod, wrapped grip, faceted reel, taut line and a large red/white float. |
| `rope_coil` | Three stacked low-poly rope loops in two sand/brown tones plus a loose end. |
| `fishing_net` | Circular hoop with two net rings, faceted net bag, radial ribs and an angled handle. |
| `market_umbrella` | Eight saturated fire/orange and enemy-red panels with scallops and a weighted stone foot. |
| `coral_brain` | Sand/stone rounded lobes with three concentric brain-coral ridges. |
| `life_ring` | One thick 12-sided torus with alternating fire-orange / snow-white sectors and gold tabs. |
| `spear` | Thicker 8-sided shaft, wrapped grip, gold collar and a wide leaf blade. |
| `farm_cart` | Five-spoke wheels with hubs, axle, slatted side walls, shafts and a small load. |

## After evidence and re-critique

- Re-rendered sheets (same tooling, rebuilt GLBs): `assets/previews/qa_props/after/sheet_*.png`.
  The pre-fix sheets and the ten individual before cells are preserved in
  `assets/previews/qa_props/before/`.
- Babylon review sheets for every rebuild: `assets/review/<id>.png` (front, three-quarter,
  eye, iso, iso_context, CCTV feed, collision, UV density); all report `errors: 0`.
- **Fresh-eyes re-critique (second subagent)**: six of the ten fixes read correctly
  (`crate_box`, `rope_coil`, `market_umbrella`, `life_ring`, `farm_cart`, and mostly
  `fishing_net`); `coral_brain` and `spear` read partly. Four MAJOR findings remained:
  `campfire` (no visible orange flame), `arrow` (1-pixel line at 60 px),
  `fishing_net` (solid blue cone read as a bucket) and `haystack` (off-white heap,
  no straw). MINOR notes: `spear`, `fishing_rod`, `coral_brain`, `wheat`/`carrot`,
  `scarecrow`, sub-10 px debris.
- **Second fix pass**: `campfire` rebuilt with three large emissive `#FF6A2A`
  flame tongues over crossed logs and embers; `arrow` rebuilt with a thicker shaft,
  broad gold flights and a larger leaf point; `fishing_net` rebuilt as an open
  lattice (six ribs + three hoops, no solid bag); `haystack` rebuilt as a golden
  stacked cone with straw tufts and binding bands.
- **Third fresh-eyes verdict (FINAL)**: all four second-pass fixes read correctly;
  remaining notes are cosmetic MINOR only (`crossbow` undersized, `coral_brain`
  muted, `arrow` fletching thin, small food/debris props at 60 px). **No MAJOR or
  CRITICAL finding remains — SHIP.**

## Verification

`scripts/verify_models.py --quiet` and `scripts/check_metrics.py` both pass after
the replacements:

```
[verify_models] 254 GLBs: 254 passed, 0 failed
[check_metrics] OK: 254 models, player 1.55u, house 3.10u, all base Y ~= 0
```

Shipped replacement metrics (from `assets/models/metrics.json`):

| Model | Tris | Height (u) | Base Y | Centre X / Z | Check |
|---|---:|---:|---:|---:|---|
| `crate_box` | 372 | 0.695 | 0.000 | +0.000 / +0.000 | PASS |
| `arrow` | 172 | 0.880 | 0.000 | +0.009 / +0.000 | PASS |
| `fishing_rod` | 272 | 2.150 | 0.000 | +0.067 / -0.062 | PASS |
| `rope_coil` | 320 | 0.284 | 0.000 | +0.000 / +0.000 | PASS |
| `fishing_net` | 484 | 0.638 | 0.000 | +0.000 / +0.000 | PASS |
| `market_umbrella` | 416 | 2.321 | 0.000 | +0.000 / +0.000 | PASS |
| `coral_brain` | 484 | 0.658 | 0.000 | +0.000 / +0.000 | PASS |
| `life_ring` | 188 | 0.220 | 0.000 | +0.000 / +0.000 | PASS |
| `spear` | 288 | 1.260 | 0.010 | +0.000 / +0.000 | PASS |
| `campfire` | 384 | 0.816 | 0.000 | +0.000 / +0.000 | PASS |
| `haystack` | 380 | 1.097 | 0.000 | +0.000 / +0.000 | PASS |
| `farm_cart` | 432 | 0.890 | 0.000 | +0.000 / +0.000 | PASS |

All fourteen rebuilt assets pass the skill validator with `0 fail, 0 warn`, including
`khronos.errors: 0 errors, 0 warnings`. The GLB filenames in `assets/models/`
and the base-at-zero / centred-pivot contract are unchanged. `assets/spec.json`
had a missing comma (invalid JSON) that blocked the skill validator; it was fixed.

## Remaining known issues (after models-v6)

- Closed in wave 6: `crossbow` was rebuilt at 1.17 u (was 0.62 u) with a visible
  stock, limbs, string and bolt; `arrow` has broad gold/sand flights; and
  `coral_brain` uses saturated arcane/fire/bone palette values instead of a
  muted sand body.
- Still cosmetic: small food and debris props (e.g. `potato`, `carrot`,
  `wreck_debris`) remain sub-60 px by design. They are scene scatter, not hero
  reads, and were deliberately left unchanged to preserve their filenames and
  existing QA evidence.
- No BLOCKER, CRITICAL or MAJOR finding remains open.
