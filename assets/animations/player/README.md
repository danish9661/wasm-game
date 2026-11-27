# Final player animation set

50 clips, authored from scratch for the verified **1.551 u** player in
`assets/models/player.glb`. This directory is the authoritative player set;
the earlier root-level humanoid clips are superseded. The second wave added the
8-direction walk/run aliases and six interaction clips; the third wave adds the
8-direction sprint aliases plus jump, climb, dodge, sleep, cook, craft and
door_open. All are bound to the same measured rig.

| Clip | Loop | Root offset (m) | Duration (s) |
|---|---|---|---|
| `idle` | yes | 0.00 | 4.033 |
| `walk` | yes | +0.84 forward (1.05 m/s) | 0.833 |
| `run` | yes | +0.96 forward (1.60 m/s) | 0.633 |
| `walk_f` / `walk_l` / `walk_b` / `walk_r` | yes | 0.84 along the named axis | 0.833 |
| `walk_fl` / `walk_fr` / `walk_bl` / `walk_br` | yes | 0.594 on each axis (0.84 total) | 0.833 |
| `run_f` / `run_l` / `run_b` / `run_r` | yes | 0.96 along the named axis | 0.633 |
| `run_fl` / `run_fr` / `run_bl` / `run_br` | yes | 0.679 on each axis (0.96 total) | 0.633 |
| `sprint_f` / `sprint_l` / `sprint_b` / `sprint_r` | yes | 0.96 along the named axis (1.80 m/s) | 0.567 |
| `sprint_fl` / `sprint_fr` / `sprint_bl` / `sprint_br` | yes | 0.679 on each axis (0.96 total) | 0.567 |
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
| `harvest` | no | +0.08 at apex, returns to 0 | 1.800 |
| `build` | no | +0.09 at apex, returns to 0 | 1.800 |
| `mine` | no | +0.12 held | 1.600 |
| `open_chest` | no | +0.14 at apex, returns to 0 | 1.800 |
| `eat` | no | 0.00 | 2.000 |
| `drink` | no | 0.00 | 1.800 |
| `jump` | no | +0.28 forward, 0.44 m peak rise on root | 2.200 |
| `climb` | yes | +0.35 up per cycle (uniform ground clamp) | 1.233 |
| `dodge` | no | 0.42 m lateral at apex, returns to 0 | 1.600 |
| `sleep` | yes | 0.00 | 4.033 |
| `cook` | yes | 0.00 | 2.033 |
| `craft` | yes | 0.00 | 1.633 |
| `door_open` | no | +0.26 forward, held through settle | 2.200 |

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
reach with a crouched hip; sprint uses the same 0.48 m stride over a shorter
16-frame cycle (1.80 m/s). One-shots keep their step-in inside the same reach;
the heavy attack uses 0.25 m rather than an unreachable 0.38 m lunge.

## Numeric proof

`manifest.json` → `gates_warn_fail` uses the game-animation `[warn, fail]`
thresholds. Latest generation:

- **50 clips: 49 PASS, 1 WARN, 0 FAIL.** The one WARN is the pre-existing
  `death` clip (3.71 cm heel roll during the collapse, documented in its note).
- Loop seam position: **0.000 cm**; seam/root velocity excess: **0.000 m/s**
  on all 26 locomotion loops, including every diagonal.
- Foot slide: walk 0.61 cm, run 0.67 cm, sprint 0.67 cm (0.64 cm diagonals),
  all interaction clips 0.00 cm; new dodge 0.77 cm, door_open 1.50 cm, and
  jump/climb/sleep/cook/craft 0.00 cm.
- Ground penetration: **0.00 cm** on every clip.
- `scripts/verify_player_final.py` re-opens all 50 exported GLBs at 30 fps and
  re-measures: **50/50 passed**, source SHA unchanged.

## Files

- `<clip>.glb`: one named 30 fps animation per file, LINEAR samplers, no scale keys.
- `manifest.json`: duration, loop flag, root-motion offsets in metres, all gates,
  derived rig dimensions, source SHA-256 and per-clip notes.
- `review/contact_sheet.png`: rendered frame strips for the original combat /
  interaction review set (the review scripts cover a fixed 13-clip selection).

## Reproduce

```bash
blender -b --python scripts/generate_player_final.py     # rebuild + manifest
blender -b --python scripts/verify_player_final.py       # reload + verify
blender -b --python scripts/render_player_final.py       # frame stills
python3 scripts/assemble_player_sheet.py                 # contact sheet
```
