# Combat animation clips

12 named 30 fps clips on the existing `assets/models/player.glb` doll:
`idle`, eight 8-direction `walk_*` loops, `attack_sword_slash`, `hit`, `death`.

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

- `idle.glb`, `walk_*.glb`, `attack_sword_slash.glb`, `hit.glb`, `death.glb`:
  one GLB per clip, each with a single animation named after the clip.
- `player_combat.blend`: all named actions on the puppet rig, for inspection.
- `manifest.json`: per-clip measured seam, foot slide, grounding, speed,
  direction error, end-pose settling, fail/warn gates and clip notes.
- `action_specs/combat.json`: combat verbs with startup/active/recovery timing,
  contact rules and deterministic tests (`design-action-combat` conventions).
- `review/contact_sheet.png`: rendered frame strips for visual review.

## Conventions

- 30 fps, LINEAR samplers, no scale keys.
- `root` carries all horizontal travel (walk 1.09 m/s; slash/hit/death one-shot
  steps). `hips` carries vertical bob only. `idle` is in place.
- Loops duplicate frame 0 as the last key in root-relative pose; the root
  translation advances by one stride so the engine can wrap it.
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
python3 scripts/assemble_combat_sheet.py                    # contact sheet
```

Latest result: 12 clips generated, 0 FAIL, 1 WARN (`death`: ~2.5 cm heel roll
during the collapse, documented in its manifest note); 12/12 GLBs reloaded and
verified in a fresh Blender process.
