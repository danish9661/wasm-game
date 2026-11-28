# Visual QA — Characters / NPCs / Enemies / Bosses

Branch: `visual-characters`. Audit date: 2026-11-28.
Scope: **52 character-family GLBs** (Character + Npc Variants categories, every
`npc_*`, plus creature-like `fire_elemental`, `stone_elemental`, `forest_deer`,
`river_otter`, `sea_turtle`, `scarecrow`). Frozen list:
`assets/qa/characters/family.json`.

## Evidence

| Stage | Path |
|---|---|
| Before — gameplay scale (~66 px per 1.55 u human) | `assets/qa/characters/before/sheet_gameplay.png` |
| Before — zoom scale (~155 px per human) | `assets/qa/characters/before/sheet_zoom.png` |
| After round 1 — gameplay / zoom | `assets/qa/characters/after/sheet_gameplay.png`, `assets/qa/characters/after/sheet_zoom.png` |
| Final (after round 2) — gameplay | `assets/qa/characters/after2/sheet_gameplay.png` |
| Final (after round 2) — zoom | `assets/qa/characters/after2/sheet_zoom.png` |
| Per-model renders | `assets/qa/characters/{before,after,after2}/{gameplay,zoom}/<name>.png` |
| Babylon (target engine) review sheets | `assets/qa/characters/review/<name>_sheet.png` |
| Babylon console/error log | `assets/qa/characters/review/<name>/review.json` |
| Briefs used for review | `assets/qa/characters/briefs/<name>.json` |

Sheets were rendered headlessly with `scripts/blender/render_character_audit.py`
and composed by `scripts/compose_character_sheet.py`: orthographic yaw 45° /
pitch 30°, 1 u = 1 tile, base on z = 0, flat-shaded palette materials, and a
**1.55 u grey human silhouette at the same scale in every cell**. The supplied
in-game screenshot (`/tmp/opencode/deployed_play.png`) showed townsfolk as flat
orange blocks and several props as white cubes; the audit sheets confirm the
character GLBs are the right reference frame for judging gameplay readability.

## Round 1 — fresh-eyes critique (before)

A fresh subagent received only the before sheets, `docs/ART_DIRECTION.md` and a
short brief. Ranked worst ten:

| # | Model | Dominant problems | Severity |
|---:|---|---|---|
| 1 | `wolf` | unreadable at 60 px, flat/boxy silhouette, missing detail — near-black smudge | BLOCKER |
| 2 | `wraith` | featureless purple cone, no head/arms/face | BLOCKER |
| 3 | `bat` | giant flat wings, tiny body, wrong proportions | BLOCKER |
| 4 | `scorpion_queen` | jumble of purple segments and loose white shards, unreadable | BLOCKER |
| 5 | `colossus` | stack of plain grey cubes, no face or weapon | MAJOR |
| 6 | `spider` | low slab on thin sticks — reads as a table | MAJOR |
| 7 | `npc_desert_scorpion` | tiny sand lump, tail/pincers invisible | MAJOR |
| 8 | `npc_merchant` | blue box + brown box + tiny cone head, no arms/face | MAJOR |
| 9 | `sea_turtle` | green pebble; head and legs invisible at 0.58 u | MAJOR |
| 10 | `npc_wolf_pup` | near-black speck, no readable head or features | MAJOR |

Family pattern: box/cone torsos with tiny faceted heads and no facial detail on
the human NPCs; small creatures either hug the ground or use near-black values.
The critic explicitly protected `stormcaller`, `ocean_leviathan`, `toad_king`,
`guard`, `skeleton` and `player`, plus `armored_goblin`, `imp`, `golem` and
`stone_elemental`. None of those were modified.

## Fixes applied

All ten were rebuilt by `scripts/blender/rebuild_characters.py` using the
game-asset-modeling workflow: bmesh parts (`gam.bm_box`, `gam.bm_cylinder`,
icospheres, prisms), **selective bevels** driven by `bevel_weight_edge`,
**weighted normals** (`gam.shade_hard_surface`), a deterministic non-overlapping
**UV pack** (`pack_uv`), flat Principled palette materials, and glTF Y-up export
with base at z = 0 and a centred pivot. Every model stays under the 500-tri
asset budget.

| Model | Before → after tris | What changed |
|---|---:|---|
| `wolf` | 488 → 340 | Warm `#6B4423` coat, tapered torso, four separated legs/paws, angular snout, pale eyes, pointed ears and raised tail |
| `wraith` | 148 → 264 | Hooded tapered shroud, pale skull mask with dark eye hollows, two arms/hands and ragged hems and wisps |
| `bat` | 148 → 196 | Wings reduced to notched body-width membranes, round head with upright ears and bright eyes |
| `scorpion_queen` | 286 → 332 | One dominant carapace, two symmetric steel pincers, a single clean three-segment tail with a fire stinger; loose shards removed |
| `colossus` | 254 → 396 | Bevelled tapered torso, angled iron pauldrons, visored head with fire eyes and a gold crown, one arm carrying a clear iron maul |
| `spider` | 284 → 296 | Rounded segmented abdomen lifted off the floor, eight splayed jointed legs, pale forward eye pair |
| `npc_desert_scorpion` | 264 → 312 | Enlarged to 1.55 u, arched tail with stinger, two raised steel pincers and six splayed legs |
| `npc_merchant` | 184 → 380 | Rounded satchel and strap replace the square pack, exposed arms and hands, skin-toned face under the hood, gold coin |
| `sea_turtle` | 250 → 260 | Domed faceted shell with scutes, distinct head with eyes, four flippers and a tail |
| `npc_wolf_pup` | 128 → 224 | Warm brown coat, oversized head/ears, pale muzzle and tail tip |

Animation-bound models (`scorpion_queen`, `colossus`, `spider`, `npc_merchant`)
keep **every original mesh node name** so the wave2/3/4 bound rigs still bind.
Their animation clips and manifests were regenerated after the GLB hash change;
the player GLB was left untouched (its animation hash is unchanged).

## Validation

- Khronos glTF-Validator: **0 errors, 0 warnings** on all 18 rebuilt GLBs
  (`scripts/validate_character_rebuilds.py`).
- Babylon harness: **0 engine console errors** on all 18; front, three-quarter,
  iso, iso-context and CAM-feed views rendered.
- `python3 scripts/verify_models.py --quiet` — 254 GLBs, 254 passed, 0 failed.
- `python3 scripts/check_metrics.py` — OK (254 models, player 1.55 u, house
  3.10 u, all base Y ≈ 0).
- Rebound animations: wave2 30/30, wave3 34/34, wave4 105/105,
  player-final 50/50 verified after regeneration.

## Round 2 — fresh-eyes critique (after round-1 fixes)

A second independent subagent received only the round-1 **after** sheets, the
art direction and a short brief. No BLOCKER remained, but **eight MAJOR**
findings were raised, all on the shared "box torso + post legs + blank wedge
head" humanoid template:

| # | Model | Dominant problems | Severity |
|---:|---|---|---|
| 1 | `scarecrow` | T-sign silhouette, no straw/face/fabric shape | MAJOR |
| 2 | `npc_farmer` | huge flat apron box, blank face, hairline hoe | MAJOR |
| 3 | `npc_fisher` | identical box body, rod invisible | MAJOR |
| 4 | `npc_bard` | lute hidden behind box torso | MAJOR |
| 5 | `npc_merchant` | pack hidden behind shoulder, coin too small | MAJOR |
| 6 | `npc_sailor` | generic body, sword a grey sliver | MAJOR |
| 7 | `npc_traveling_scholar` | blank face, tiny book | MAJOR |
| 8 | `npc_innkeeper` | plain white apron box, blank face | MAJOR |

The same subagent explicitly accepted `archer`, `armored_goblin`, `bat`,
`brute`, `colossus`, `goblin`, `guard`, `imp`, `ocean_leviathan`, `ogre`,
`player`, `raider`, `scorpion_queen`, `skeleton`, `slime`, `spider`,
`stormcaller`, `toad_king`, `wolf`, `wraith`, `wraith_knight`,
`fire_elemental`, `forest_deer`, `npc_blacksmith`, `npc_captain`,
`npc_desert_scorpion`, `npc_dire_wolf`, `npc_elder`, `npc_forest_warden`,
`npc_guard`, `npc_hunter`, `npc_ice_elemental`, `npc_mermaid`, `npc_priest`,
`npc_wolf_pup`, `river_otter` and `sea_turtle`. Its MINOR notes (hairline canes,
small stars, similar purple cones, rock-humanoid accents) were polish, not
ship-blockers.

### Round-2 fixes

All eight were rebuilt with the same bmesh/bevel/weighted-normal/UV-pack
workflow and remain under the 500-tri budget:

| Model | Tris after | What changed |
|---|---:|---|
| `scarecrow` | 356 | Tapered ragged coat wedge and hem tufts, stitched eyes/mouth, straw at neck and wrists, crooked staff breaking the T outline |
| `npc_farmer` | 396 | Trapezoid apron/vest with bib and straps, raised hoe with a readable blade at head height, face marks |
| `npc_fisher` | 372 | Thick rod angled across the body with a bright float above the head, tapered coat, hat and face |
| `npc_bard` | 368 | Lute slung across the chest with the neck protruding past the shoulder; feathered hat and face |
| `npc_merchant` | 416 | Tall framed pack raised above one shoulder, hip pouch, enlarged gold coin, face under hood |
| `npc_sailor` | 356 | Tapered jacket, cutlass held out from the body, bandana and face |
| `npc_traveling_scholar` | 388 | Open two-page book facing the camera, brow/eye marks, hood and scrollsatchel |
| `npc_innkeeper` | 384 | Tapered apron, visible eyes and moustache, hanging wooden sign, tray with two mugs |

`npc_farmer` and `npc_innkeeper` keep their exact wave4 rig node names
(including `hoe`, `hoe_blade`, `foam_0/1`, `mug_0/1`, `tray`); `npc_merchant`
keeps its wave2/3/4 names. Wave animations were regenerated after the hashes
changed and all verifiers pass again.

## Round 3 — final fresh-eyes gate

A third independent subagent judged only the final (`after2`) sheets, the art
direction and a short brief. Verdict: **NO BLOCKER OR MAJOR REMAINS**. It
confirmed that every enemy has a distinct outline, bosses are visibly larger
than the 1.55 u human (`colossus` 3.02 u, `stormcaller` 2.65 u,
`ocean_leviathan` 2.34 u, `toad_king` 2.23 u, `scorpion_queen` 2.22 u), and no
character is a featureless capsule or a stack of plain boxes.

Only MINOR polish notes remain, deliberately deferred as cosmetic:
`npc_blacksmith` (no held tool), `bat` (dark at 60 px), `fire_elemental`
(disconnected small parts), and `golem` / `stone_elemental` (similar chunky rock
silhouettes). Everything else was explicitly accepted as ship-ready.

## Final state

18 character-family models were rebuilt and verified. The eight models the
first critic protected (`stormcaller`, `ocean_leviathan`, `toad_king`, `guard`,
`skeleton`, `player`, `armored_goblin`, `imp`) plus `golem` and
`stone_elemental` were left untouched. All original filenames, base-at-zero /
centred pivot contracts, palette-only flat shading and the 500-tri budget are
preserved.

## Contract notes

- Filenames unchanged; base Y ≈ 0; footprint centred; flat shading; palette-only
  materials; no textures; triangle budget respected.
- `assets/spec.json` had a missing comma after `wave4_note` (invalid JSON), which
  blocked the skill validator and Babylon harness. Fixed in place; no semantic
  change to the spec.

## Remaining known issues (after models-v6)

- `npc_blacksmith` still has no clearly readable held tool at gameplay scale.
  Its GLB SHA-256 is locked in `assets/animations/wave4/manifest.json` and the
  wave-4 rig enumerates its exact mesh nodes (`tongs_0`, `tongs_1`, `apron`, …);
  adding a hammer would force regeneration of all 105 wave-4 clips. Deferred
  deliberately rather than risking the animation contract.
- The other four Round-3 MINOR notes are closed in wave 6: `bat` now has a
  lighter wing/bone palette and gold eyes, `fire_elemental` is one connected
  flame body, and `stone_elemental` was rebuilt as a tall obsidian/arcane
  figure so it no longer echoes the broad grey `golem`.
- No BLOCKER or MAJOR finding remains open.
