# Animation clips

> **Authoritative player set:** `player/` — a from-scratch redesign of 13 clips
> bound to the verified 1.551 u player, with source SHA-256, dimension-derived
> rig, per-clip root offsets in metres and numeric gates. The root-level
> humanoid clips below are retained as the earlier exploratory set and are
> **superseded** by `player/`. Weather FX and NPC work loops are in
> `world_manifest.json` (`fx_*`, `*_<work>` GLBs at this level).

Earlier exploratory set: 40 named 30 fps clips on the existing
`assets/models/player.glb` doll.

| Group | Clips |
|---|---|
| Idle | `idle` |
| Walk (8 directions) | `walk_f`, `walk_fl`, `walk_l`, `walk_bl`, `walk_b`, `walk_br`, `walk_r`, `walk_fr` |
| Run (8 directions) | `run_f`, `run_fl`, `run_l`, `run_bl`, `run_b`, `run_br`, `run_r`, `run_fr` |
| Sprint (8 directions) | `sprint_f`, `sprint_fl`, `sprint_l`, `sprint_bl`, `sprint_b`, `sprint_br`, `sprint_r`, `sprint_fr` |
| Combat | `attack_sword_slash`, `block`, `dodge`, `hit`, `death` |
| Traversal | `jump`, `climb` |
| Life / interaction | `bow`, `sleep`, `drink`, `eat`, `cook`, `craft`, `door_open`, `interact` |

## Why these are procedural, not mocap

The `mixamo-llm-mocap` video → GVHMR → Mixamo pipeline was checked first and
is **not available on this machine**:

- no GVHMR checkout/venv, no `inputs/checkpoints` (~5.3 GB) and no license-gated
  `SMPLX_NEUTRAL.npz` (GVHMR asserts on it at startup);
- no Mixamo FBX character or account/credentials;
- no video plate to estimate.

Blender **5.1.2** is available and was used headless. Tooling details and the
decision are recorded in `manifest.json` → `tooling`. Nothing here claims to be
captured from video or Mixamo.

## Files

- One GLB per clip, each with a single animation named after the clip.
- `player_combat.blend`: all named actions on the puppet rig, for inspection.
- `manifest.json`: per-clip measured seam, foot slide, grounding, speed,
  direction error, end-pose settling, fail/warn gates and clip notes.
- `action_specs/combat.json` and `action_specs/life.json`: verb timing, contact
  rules and deterministic tests (`design-action-combat` conventions).
- `review/contact_sheet.png` (combat/traversal) and
  `review/contact_sheet_life.png` (life/interaction): rendered frame strips.

## Conventions

- 30 fps, LINEAR samplers, no scale keys.
- `root` carries horizontal travel (walk 1.09, run 1.46, sprint 1.98 m/s) and
  the one-shot steps for slash/hit/death/jump/door_open/dodge/climb. `hips`
  carries vertical bob only. `idle`, `bow`, `sleep`, `drink`, `eat`, `cook`,
  `craft`, `block` and `interact` are in place.
- Loops duplicate frame 0 as the last key in root-relative pose; the root
  translation advances by one stride (or one climb rise) so the engine can wrap it.
- Direction suffixes: `f fl l bl b br r fr` with the glTF vectors in the
  manifest and `assets/spec.json` → `animation.directions_gltf`.
- Gates are `[warn, fail]` pairs in `assets/spec.json` → `animation.thresholds`
  (game-animation `references/validation.md` values).

## Reproduce / verify

```bash
blender -b --python scripts/generate_combat_animations.py   # rebuild + manifest
blender -b --python scripts/verify_combat_animations.py     # reload GLBs + re-measure
blender -b assets/animations/player_combat.blend \
  --python scripts/render_combat_review.py                  # frame stills
python3 scripts/assemble_combat_sheet.py                    # contact sheets
```

Latest result: 40 clips generated, **0 FAIL**, 1 WARN (`death`: ~2.5 cm heel
roll during the collapse, documented in its manifest note); **40/40 GLBs**
reloaded and verified in a fresh Blender process.
