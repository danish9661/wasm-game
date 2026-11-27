# Visual QA — buildings, structures, interiors & kit pieces

Branch: `visual-buildings`. Work stayed in this worktree; nothing merged to `main`.

## 1. Scope and method

Family under audit: all 69 models in the `Building` metric category, the 38
`Structure` models, and the 8 `Kit Expansion` pieces (**115 models** total).

### Gameplay-scale contact sheets

`scripts/blender/render_family_sheet.py` (headless Blender 5.1) renders each
model in its own cell at a **fixed gameplay scale**: 1 Blender unit = 1 tile,
orthographic camera, yaw 45°, pitch 30°. A plain **1.55 u human silhouette**
stands beside every model, with a 1×1 u reference tile under it. Each cell also
carries a 60 px inset for the legibility test. Tall landmarks keep the same
relative scale but widen the camera field; the zoom factor is printed on the
cell label.

`scripts/make_family_sheet.py` assembles four labeled 6×5 sheets.

| Stage | Sheets |
|---|---|
| Before fixes | `assets/previews/family_gameplay_before_01.png` … `_04.png` |
| After fixes | `assets/previews/family_gameplay_after_01.png` … `_04.png` |

Reproduce:

```bash
~/.local/blender/blender -b --python scripts/blender/render_family_sheet.py -- --out /tmp/opencode/qa/after
python3 scripts/make_family_sheet.py --cells /tmp/opencode/qa/after --stage after
```

## 2. Root cause of the white/gray cubes in the deployed village

The large plain white cubes in `/tmp/opencode/deployed_play.png` were **not**
missing textures — they were the `house.glb` collision proxy.

`assets/src/props/prop_house/build_prop_house.py` uses the skill's
`gam.finish(..., collider_kind="box")`, which exports a `prop_house_col` mesh
node with **no material**, alongside the detailed `prop_house_lod0`. The web
renderer (`web/src/model.rs::walk_node`) walks *every* glTF node and draws
every triangle primitive, falling back to white when a primitive has no
material. The untextured box therefore painted over the detailed house. The
same defect existed on `altar.glb` and `prop_star_crown_shrine.glb`.

Fix: `scripts/strip_collider_meshes.py` keeps the collider **node**, its name
and its `ig_collider` extra, but removes only its `mesh` reference. The runtime
renderer no longer draws the proxy; gameplay collision remains discoverable by
name. The three skill build scripts now call the stripper immediately after
`gam.export`, so a rebuild cannot reintroduce the cubes.

> Handoff (outside this family): `tree.glb` still carries the same white box
> proxy (`prop_tree_col`). Nature-family owners should apply the same stripper.

## 3. Fresh-eyes critic — before

An independent subagent saw only the four before sheets, `ART_DIRECTION.md` and
a short brief. Its ranked worst ten:

| # | Model | Severity | Main failures |
|---|---|---|---|
| 1 | `house` | BLOCKER | solid grey cube; no roof/door/windows; unreadable at 60 px |
| 2 | `prop_star_crown_shrine` | BLOCKER | grey monolith; no gold crown or arcane purple |
| 3 | `altar` | BLOCKER | featureless cuboid; no runes, gold or emissive accent |
| 4 | `house_interior` | MAJOR | hollow grey box; no plank floor, windows or hearth visible |
| 5 | `interior_wall` | MAJOR | flat grey/brown slab; no framing or courses |
| 6 | `kit_wall_stone` | MAJOR | flat grey panel; no masonry or mortar detail |
| 7 | `kit_wall_wood` | MAJOR | two-tone flat panel; no plank seams or bracing |
| 8 | `statue` | MAJOR | stacked grey blocks with a gold dot; no humanoid read |
| 9 | `interior_floor` | MAJOR | single flat brown quad; vanishes at 60 px |
| 10 | `spike` | MAJOR | one-pixel needle; almost no visual mass |

The critic also called out `kit_pillar`, `wall`, `castle_keep`, `interior_rug`,
`dungeon_floor_grate`, `torch` and `torch_sconce` as blank/near-invisible — but
rated them below the top ten (MINOR or functional flatness).

## 4. What changed

### Collider removals (all three were BLOCKER)

* `house.glb`, `altar.glb`, `prop_star_crown_shrine.glb`: collider proxy no
  longer rendered; the detailed skill-built LOD is now visible in game.

### Bmesh rebuilds (`scripts/blender/family_repairs.py`)

All parts are authored as bmesh boxes/cones/icos, bevelled on the
silhouette-defining group, flat-shaded with weighted normals applied, and
packed into unique UV0 atlas cells. Hidden faces are skipped to stay inside the
500-triangle budget. Filenames, base-at-zero and bottom-centre pivots are
unchanged.

| Model | Before → after tris | Change |
|---|---:|---|
| `house_interior` | 1216 → 1240 | Dollhouse cutaway (no east wall, low south wall), 8 plank strips, corner posts, framed blue windows; hearth/table/bed/shelf now visible |
| `interior_wall` | 36 → 152 | Wainscot, two framed panels, studs, rail and capping beam |
| `kit_wall_stone` | 36 → 236 | Staggered 5-course masonry with mortar backing and varied face depths |
| `kit_wall_wood` | 36 → 170 | Four plank strips, dark seams, diagonal brace, rail and cap |
| `statue` | 102 → 404 | Tapered cuirass, head/brow, arms, belt, tabard, sword and small crown — readable figure |
| `interior_floor` | 48 → 164 | 12 planks in 4 palette tones with real gaps on a dark subfloor |
| `spike` | 52 → 302 | Wide octagonal plinth, thick tapered stake, iron bands, red accent and four braces |

Source-of-truth generators now call the same repair builders:
`gen_interiors.py`, `gen_kit_expansion.py` and `gen_structures.py`. The
standalone repair export path is
`scripts/blender/gen_zz_family_repairs.py` (the `zz` prefix keeps the project's
metric-category scanner from assigning a new category to `kit_*` names).

## 5. Fresh-eyes critic — after

The same independent fresh-eyes protocol was repeated on the four **after**
sheets. Verdicts: **all ten flagged models FIXED** — the house shows a gabled
slate roof, chimney, door and two blue windows; the shrine has a gold crown and
purple crystal; the altar is tiered with a gold ring; the interiors, walls,
floor, statue and spike all read at 60 px. The critic found **no remaining
MAJOR or BLOCKER finding anywhere in the family**. Flat items that remain flat
(floors, rugs, kit walls, gates) are flat by function, and the apparent
cropping of very tall landmarks is preview framing, not a model defect.

## 6. Validation

| Gate | Result |
|---|---|
| `python3 scripts/verify_models.py` | **254 / 254 GLBs pass**, 0 failures; metrics + `ASSET_METRICS.md` regenerated |
| `python3 scripts/check_metrics.py` | OK — player 1.55 u, house 3.10 u, all base Y ≈ 0 |
| Khronos glTF validator (`tools/khronos_validate.mjs`) | 0 errors, 0 warnings on all 10 modified GLBs |
| Babylon review (`tools/review/shoot.mjs`) | 0 console errors / warnings on all 10 modified GLBs; sheets in `assets/review/qa_<id>.png` |

Notes: the project's `assets/spec.json` has a pre-existing missing comma after
`wave4_note`; the Babylon review used a temporary corrected copy
(`/tmp/opencode/qa/spec_review.json`) and did not modify the project file.
`verify_models.py` still counts the (now unreferenced) collider mesh data in
the GLB binary, so triangle totals for the three collider-fixed models are
unchanged; the rendered LOD triangle counts are 1226 / 398 / 456 respectively.

## 7. Status

The white cubes are gone, the top-ten findings are closed, and a second
independent critique found no finding above MAJOR. No merge into `main`.
