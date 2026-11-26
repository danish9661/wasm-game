# Final player animation set

13 clips, authored from scratch for the verified **1.551 u** player in
`assets/models/player.glb`. This directory is the authoritative player set;
the earlier root-level humanoid clips are superseded.

| Clip | Loop | Root offset (m) | Duration (s) |
|---|---|---|---|
| `idle` | yes | 0.00 | 4.033 |
| `walk` | yes | +0.84 forward (1.05 m/s) | 0.833 |
| `run` | yes | +0.96 forward (1.60 m/s) | 0.633 |
| `attack_1` | no | +0.14 | 1.600 |
| `attack_2` | no | +0.16 | 1.600 |
| `attack_3` | no | +0.22 | 1.800 |
| `heavy_attack` | no | +0.25 | 2.200 |
| `block` | no | 0.00 | 2.000 |
| `dodge_roll` | no | +0.65 forward | 1.800 |
| `hit_light` | no | -0.13 back | 1.400 |
| `hit_heavy` | no | -0.20 back | 1.600 |
| `death` | no | -0.56 back (0.63 total) | 2.400 |
| `interact` | no | +0.12 at apex, returns to 0 | 1.800 |

The attack chain and heavy attack publish explicit windup / cut / recovery
windows in `manifest.json` → `clips[].notes`, and the timing contract is in
`../action_specs/combat.json`.

## Dimension binding

`scripts/generate_player_final.py` derives hip, knee, ankle, shoulder, elbow and
wrist pivots from the **actual mesh bounds** in `player.glb` (not hardcoded),
and `manifest.json` records the source SHA-256 plus derived leg/arm lengths.
If the final player GLB changes, rerun the generator: it re-binds and the
verifier fails if the SHA no longer matches the manifest.

Stride and root travel are computed from those dimensions:

```
stride = speed * cycle_frames / (2 * 30 fps)
```

so walk/run use 0.42 / 0.48 m strides that stay inside the derived 0.35 m leg
reach with a crouched hip. One-shots keep their step-in inside the same reach;
the heavy attack uses 0.25 m rather than an unreachable 0.38 m lunge.

## Numeric proof

`manifest.json` → `gates_warn_fail` uses the game-animation `[warn, fail]`
thresholds. Latest generation:

- **13/13 clips PASS**, 0 WARN, 0 FAIL.
- Loop seam position: **0.000 cm**; seam/root velocity excess: **0.000 m/s**.
- Foot slide: walk 0.71 cm, run 0.77 cm, heavy attack 1.84 cm (all below 2 cm).
- Ground penetration: **0.00 cm** on every clip; 10th-percentile sole height
  0.10–0.94 cm (no floating).
- `scripts/verify_player_final.py` re-opens all 13 exported GLBs at 30 fps and
  re-measures: **13/13 passed**, source SHA unchanged.

## Files

- `<clip>.glb`: one named 30 fps animation per file, LINEAR samplers, no scale keys.
- `manifest.json`: duration, loop flag, root-motion offsets in metres, all gates,
  derived rig dimensions, source SHA-256 and per-clip notes.
- `review/contact_sheet.png`: rendered frame strips for every clip.

## Reproduce

```bash
blender -b --python scripts/generate_player_final.py     # rebuild + manifest
blender -b --python scripts/verify_player_final.py       # reload + verify
blender -b --python scripts/render_player_final.py       # frame stills
python3 scripts/assemble_player_sheet.py                 # contact sheet
```
