# Third animation wave — locomotion, boss phase 2, NPC work

34 clips on the same bound-derived pivot rigs as `../wave2/`. Limb and torso
arcs are published at **1.25×** the earlier key poses (`manifest.json` →
`conventions.pose_amplify`) so the motion reads at diorama size; root travel
and planted-contact targets are unchanged and every gate is re-measured after
scaling. 30 fps, LINEAR, no scale keys, one named Action per GLB.

| Clip | Model | Group | Loop | Duration (s) | Root offset (m) | Gate |
|---|---|---|---|---|---|---|
| `slime_idle` | slime | enemy | yes | 4.033 | 0.000 | PASS |
| `slime_death` | slime | enemy | no | 2.000 | 0.408 | PASS |
| `goblin_idle` | goblin | enemy | yes | 4.033 | 0.000 | PASS |
| `goblin_walk` | goblin | enemy | yes | 0.833 | 0.000 | PASS |
| `goblin_run` | goblin | enemy | yes | 0.633 | 0.000 | PASS |
| `skeleton_idle` | skeleton | enemy | yes | 4.033 | 0.000 | PASS |
| `skeleton_walk` | skeleton | enemy | yes | 0.833 | 0.000 | PASS |
| `skeleton_run` | skeleton | enemy | yes | 0.633 | 0.000 | PASS |
| `wolf_idle` | npc_dire_wolf | enemy | yes | 4.033 | 0.000 | PASS |
| `wolf_walk` | npc_dire_wolf | enemy | yes | 0.833 | 0.000 | PASS |
| `wolf_run` | npc_dire_wolf | enemy | yes | 0.633 | 0.000 | PASS |
| `archer_idle` | archer | enemy | yes | 4.033 | 0.000 | PASS |
| `archer_walk` | archer | enemy | yes | 0.833 | 0.000 | PASS |
| `archer_run` | archer | enemy | yes | 0.633 | 0.000 | PASS |
| `brute_idle` | brute | enemy | yes | 4.033 | 0.000 | PASS |
| `brute_walk` | brute | enemy | yes | 0.833 | 0.000 | PASS |
| `brute_run` | brute | enemy | yes | 0.633 | 0.000 | PASS |
| `spider_idle` | spider | enemy | yes | 4.033 | 0.000 | PASS |
| `archer_death` | archer | enemy | no | 2.000 | 0.275 | PASS |
| `brute_death` | brute | enemy | no | 2.000 | 0.369 | PASS |
| `scorpion_queen_enrage` | scorpion_queen | boss | no | 2.000 | 0.029 | PASS |
| `scorpion_queen_ultimate` | scorpion_queen | boss | no | 1.800 | 0.030 | PASS |
| `toad_king_enrage` | toad_king | boss | no | 2.000 | 0.029 | PASS |
| `toad_king_ultimate` | toad_king | boss | no | 1.600 | 0.056 | PASS |
| `stormcaller_enrage` | stormcaller | boss | no | 2.000 | 0.112 | PASS |
| `stormcaller_ultimate` | stormcaller | boss | no | 1.600 | 0.050 | PASS |
| `ocean_leviathan_enrage` | ocean_leviathan | boss | no | 2.000 | 0.029 | PASS |
| `ocean_leviathan_ultimate` | ocean_leviathan | boss | no | 1.800 | 0.034 | PASS |
| `colossus_enrage` | colossus | boss | no | 2.000 | 0.039 | PASS |
| `colossus_ultimate` | colossus | boss | no | 1.800 | 0.094 | PASS |
| `villager_gather` | npc_villager_child | npc | yes | 1.633 | 0.000 | PASS |
| `guard_watch` | guard | npc | yes | 3.233 | 0.000 | PASS |
| `merchant_count` | npc_merchant | npc | yes | 2.033 | 0.000 | PASS |
| `golem_work` | golem | npc | yes | 1.633 | 0.000 | PASS |

**Locomotion note.** The `*_walk` / `*_run` cycles are in-place gait loops:
the engine applies world travel (root offset 0 is reported in the manifest),
which keeps planted soles exact on these single-pivot rigs. Idle and work
loops use the same foot lock, so their contacts do not skate.

Boss phase 2 pairs: `*_enrage` is a held transition pose for the state change;
`*_ultimate` is the amplified phase-1 attack (~1.35× limb arcs, 1.4× travel).

## Verification

- **34/34 clips PASS**, 0 WARN, 0 FAIL on the published (amplified) poses.
- `scripts/verify_wave3_animations.py` re-derives contact tips from source GLB
  bounds, re-imports every GLB in a fresh Blender process and re-runs the
  gates: **34/34 passed**.

```bash
blender -b --python scripts/generate_wave3_animations.py
blender -b --python scripts/verify_wave3_animations.py
```
