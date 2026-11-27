# Visual QA — nature & biome props (family: `nature_biome`)

Audit and repair of the nature/biome GLBs in `assets/models/` after the user
reported that models still look bad in game. Work was confined to this worktree
(branch `visual-nature`); nothing was merged into `main`.

## Scope

43 models: tree/pine/apple_tree/dead_tree/mangrove/mangrove_roots, foliage
(bush, berry_bush, fern, grass_tuft, flower, reed, swamp_reeds, cactus,
mushroom, mushroom_cluster, herb_patch, wheat, carrot, potato, vines, lilypad),
rocks/minerals (rock, rock_pile, rubble, dune_rock, ore, crystal, ice_crystal,
ice_spire, obsidian_spire, basalt_columns, volcanic_bomb, lava_vent), ocean
(coral, coral_fan, coral_brain, shell_cluster, driftwood, wreck_plank,
frozen_log) and hay bale/haystack. The authoritative list is
`scripts/blender/nature_family.json`.

## Method — gameplay-scale contact sheets

`scripts/blender/render_nature_cells.py` renders every family GLB in headless
Blender at **true gameplay scale** (1u = 1 tile), beside a **1.55u player
silhouette** (the actual `player.glb` flattened to a neutral grey), on a 1u
checkerboard, with the game camera (orthographic, yaw 45°, pitch 30°) and sun +
cool fill. Collider meshes are hidden; emissive is capped at 0.45 in the audit
harness so base hues stay legible (the engine tone-maps the stronger in-game
glow). `scripts/build_nature_sheet.py` composes the cells into labelled sheets
with triangle counts and heights from `metrics.json`.

Evidence (kept in the repo):

| Stage | Paths |
|---|---|
| Before | `assets/previews/nature_qa/before/nature_sheet_01.png`, `nature_sheet_02.png` |
| After round 1 | `assets/previews/nature_qa/after/nature_sheet_01.png`, `nature_sheet_02.png` (superseded by final) |
| Final | `assets/previews/nature_qa/after/nature_sheet_01.png`, `nature_sheet_02.png` |

### In-game screenshot review

`/tmp/opencode/deployed_play.png` shows the 3D village: trees read as green
cones on brown cylinders, rocks as small grey blobs, and several large white
cubes (those are the building family, not this family). The cone silhouette,
blob reading and tiny scale confirmed the need for this audit.

## Fresh-eyes critic — round 1 (before)

Independent subagent, given only the before sheets, `docs/ART_DIRECTION.md` and
a short brief (`/tmp/opencode/nature_qa/critic_brief.md`). Ranking:

| # | Model | Severity | Failure |
|---|---|---|---|
| 1 | hay_bale | BLOCKER | pale cube with ribbon = wrapped parcel, not straw |
| 2 | dead_tree | BLOCKER | one post + kindling; no branching hierarchy |
| 3 | mangrove_roots | BLOCKER | blue/teal stump (water hue in wood), stick legs |
| 4 | ice_spire | BLOCKER | 4-sided traffic cones on a square display slab |
| 5 | pine | MAJOR | exactly three stacked cones |
| 6 | cactus | MAJOR | three flat-capped green cylinders, no ribs/spines |
| 7 | volcanic_bomb | MAJOR | bright orange box with dark patches |
| 8 | coral_brain | MAJOR | half yellow / half pink crumpled wrapper |
| 9 | vines | MAJOR | flat green crosses, 1–2 px strokes |
| 10 | potato | MAJOR | dirt clump with weeds; no tubers visible |

## Round-1 fixes

All ten were rebuilt with the game-asset-modeling workflow: briefs under
`assets/src/props/prop_<id>/`, bmesh parts from `assets/src/_family_nature.py`
and `assets/src/_family_hero.py`, 1-segment bevels, `gam.uv_pack`, weighted
normals via `gam.finish`, Khronos validation and Babylon review via
`gam_cli.py all`, then the GLB copied over `assets/models/<name>.glb` (same
filename, base z = 0, footprint centred).

- **hay_bale** → squat elliptical loft with raised straw bands and straw nubs.
- **dead_tree** → tapered trunk, root flare, two generations of forked branches and twigs.
- **mangrove_roots** → five two-segment arched prop roots, wood stump, swamp-green foliage crown, no blue.
- **ice_spire** → faceted icy prisms with shifted tips and scattered chips; square slab removed.
- **pine** → overlapping offset needle lobes with branch stubs (no plain cones).
- **cactus** → tapered ribbed column, rounded tapered arms, spine nubs.
- **volcanic_bomb** → angular dark stone with emissive lava cracks and ash chips.
- **coral_brain** → sand dome with raised ridges/grooves (no pink/yellow).
- **vines** → twisted climbing strand with broad leaves on a small support.
- **potato** → four visible half-buried tubers with eyes and modest shoots.

## Fresh-eyes critic — round 2 (after round 1)

Independent subagent, given after + before sheets and the style guide
(`/tmp/opencode/nature_qa/critic_brief_round2.md`). Remaining ranking:

| # | Model | Severity | Failure |
|---|---|---|---|
| 1 | coral_brain | BLOCKER | smooth mustard egg; ridges invisible (regressed) |
| 2 | wheat | BLOCKER | cream pad with toothpicks, no grain heads |
| 3 | coral_fan | MAJOR | pink sticks with yellow cups (off-palette) |
| 4 | flower | MAJOR | cream petals, bright yellow centre |
| 5 | shell_cluster | MAJOR | pink pentagons on a cream pad |
| 6 | hay_bale | MAJOR | reads as a yellow barrel, not a bale |
| 7 | swamp_reeds | MAJOR | green slab with hairline stems and blue shards |
| 8 | basalt_columns | MAJOR | five identical prisms on a pale slab |
| 9 | carrot | MAJOR | brown tile with upside-down orange cones |
| 10 | lilypad | MAJOR | flat green coin, no notch or veins |

It also noted pine still cone-ish and 3.04u (over the tree cap), cactus still
plain, volcanic_bomb glow too subtle, vines reading as a stump, potato as a
walnut, and grass_tuft/fern/reed/herb_patch/rubble as sub-60px specks.

## Round-2 fixes

`scripts/blender/rebuild_nature_wave2.py` rebuilt 22 models in one headless run,
using the same bmesh/bevel/UV-pack/weighted-normal pipeline via `gam.finish`;
each has a brief under `assets/src/props/prop_<id>/`, passed Khronos validation
and Babylon review, and was copied over its `assets/models/<name>.glb`.

Rebuilt: coral_brain, wheat, coral_fan, flower, shell_cluster, hay_bale,
swamp_reeds, basalt_columns, carrot, lilypad, pine, volcanic_bomb, vines,
potato, ice_crystal, coral, dune_rock, grass_tuft, fern, reed, herb_patch,
rubble.

Key changes: faceted ridged brain coral; real grain heads on 5 wheat stalks
(no pad); fan blades in fire-orange/arcane (no pink); gold/bone flower petals;
sand/bone spiral shells; chamfered rectangular bale with leather bands and
straw facets; thick cattail reeds on a small mud patch; varied tilted basalt
columns; buried tapered carrot roots with fronds; notched lilypad with veins
and rim; pine lowered to 3.00u (tree cap) with wider offset needle lobes; six
lava cracks; taller twisted vines; five distinct potato tubers; ice_crystal
recoloured (purple leak removed); coral branches recoloured; sand-faceted
dune_rock; thicker, readable grass/fern/reed/herb/rubble detail.

## Fresh-eyes critic — round 3 (final gate)

Two independent critics reviewed the post-wave-2 sheets (session restarts made
the temporary `/tmp` sheets unavailable to them, so they reviewed the copies in
`assets/previews/nature_qa/after/`). Neither had any stake in the work.

- Critic A: worst remaining = bush (BLOCKER, featureless green lump), then
  mangrove_roots, potato, crystal, tree, rubble, lily_pad, driftwood.
- Critic B: worst remaining = mushroom_cluster (BLOCKER: 768 tris, off-palette
  pink/yellow), haystack (BLOCKER: pale cream, lowest contrast), then flower,
  mushroom, hay_bale, bush, apple_tree, coral_fan.

The union of their findings was addressed in a final wave (`--only` mode of
`scripts/blender/rebuild_nature_wave2.py`): **tree, bush, mushroom,
mushroom_cluster, crystal, haystack, rubble, potato** were rebuilt through the
same brief → bmesh → bevel → UV-pack → weighted-normal → Khronos → Babylon
pipeline and copied over their `assets/models/*.glb`. In particular:

- `mushroom_cluster` 768 → 420 tris, pink/yellow caps → arcane/fire caps with bone spots.
- `haystack` cream pyramid → gold/sand faceted straw stack.
- `bush` single lump → four leaf lobes, stems, underside and sprigs.
- `tree` lollipop → rounded faceted crown with three boughs; 2.97u.
- `crystal` pale pink → arcane purple with emissive core.
- `rubble`, `potato` given larger/more legible part structure.

A third independent critic then flagged four remaining blockers: `pine`
(still read as stacked cones), `rubble` (unreadable 0.23u speck), `mushroom`
and `mushroom_cluster` (cream stems / off-palette caps). A final targeted pass
rebuilt exactly those plus `coral_brain` and `shell_cluster`:

- `pine` → visible forked boughs carrying five irregular foliage clumps; **no cones at all**, 2.92u.
- `rubble` → one large broken stone plus chips, 0.37u tall (was 0.23u).
- `mushroom` / `mushroom_cluster` → wood-brown stems and fire/enemy-red caps with bone spots (no cream stems, no pink or yellow caps).
- `coral_brain` → arcane-purple body with fire-red/bone ridges (no mustard blob).
- `shell_cluster` → sand/gold spiral shells (no cream).

Final sheet review (`assets/previews/nature_qa/after/nature_sheet_0*.png`,
titled **FINAL clean**) shows no plain cube, no featureless blob, no off-palette
pink/cream and no tree that is just cones. The remaining observations are
minor/major-at-worst: `apple_tree` has a small single apple and 2.46u height,
`hay_bale` reads as a banded bale rather than a rounded one, `lily_pad`,
`driftwood`, `wreck_plank` and `flower` are small ground props, and
`volcanic_bomb`'s lava cracks are subtle at 60px. **No finding above MAJOR
remains.**

## Validation

All checks were rerun after the final rebuild:

```
python3 scripts/verify_models.py --quiet
python3 scripts/check_metrics.py
```

Both pass: **254 GLBs, 254 passed, 0 failed**; player 1.55u, house 3.10u,
all base Y ≈ 0. Every rebuilt GLB also passed the skill's
`gam_cli.py validate` (Khronos glTF validator: 0 errors) and `gam_cli.py review`
(Babylon, 0 engine console errors); `validate.json`/`review.json` are stored
next to each brief and under `assets/review/`.

## Reproduce

```bash
~/.local/blender/blender -b --python scripts/blender/render_nature_cells.py -- \
  --out /tmp/opencode/nature_qa/cells
python3 scripts/build_nature_sheet.py \
  --cells /tmp/opencode/nature_qa/cells \
  --out /tmp/opencode/nature_qa/sheets --title "Nature / Biome props"
python3 scripts/verify_models.py --quiet && python3 scripts/check_metrics.py
```
