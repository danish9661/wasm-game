# Second animation wave — enemies, bosses and NPCs

30 clips, 30 fps, LINEAR samplers, no scale keys, one named Action per GLB.
Every pivot rig is derived at generation time from the **actual mesh-node
bounds** of its source GLB; travel, leans and contact compensation scale with
`model_height / 1.55 u` (the verified player). No mocap, no hardcoded joints.

| Clip | Model | Group | Loop | Duration (s) | Root offset (m) | Gate |
|---|---|---|---|---|---|---|
| `slime_hop` | slime | enemy | yes | 0.833 | 0.000 | PASS |
| `goblin_attack` | goblin | enemy | no | 1.600 | 0.000 | PASS |
| `goblin_hurt` | goblin | enemy | no | 1.400 | 0.094 | PASS |
| `goblin_death` | goblin | enemy | no | 2.000 | 0.219 | PASS |
| `skeleton_attack` | skeleton | enemy | no | 1.600 | 0.000 | PASS |
| `skeleton_hurt` | skeleton | enemy | no | 1.400 | 0.140 | PASS |
| `skeleton_death` | skeleton | enemy | no | 2.000 | 0.329 | PASS |
| `wolf_attack` | npc_dire_wolf | enemy | no | 1.600 | 0.000 | PASS |
| `wolf_hurt` | npc_dire_wolf | enemy | no | 1.400 | 0.126 | PASS |
| `wolf_death` | npc_dire_wolf | enemy | no | 2.000 | 0.169 | PASS |
| `spider_scuttle` | spider | enemy | yes | 0.833 | 0.000 | PASS |
| `spider_attack` | spider | enemy | no | 1.600 | 0.013 | PASS |
| `archer_shoot` | archer | enemy | no | 1.600 | 0.000 | PASS |
| `brute_smash` | brute | enemy | no | 1.800 | 0.032 | PASS |
| `scorpion_queen_telegraph` | scorpion_queen | boss | no | 2.000 | 0.000 | PASS |
| `scorpion_queen_attack` | scorpion_queen | boss | no | 1.800 | 0.021 | PASS |
| `toad_king_telegraph` | toad_king | boss | no | 2.000 | 0.000 | PASS |
| `toad_king_attack` | toad_king | boss | no | 1.600 | 0.040 | PASS |
| `stormcaller_telegraph` | stormcaller | boss | no | 2.000 | 0.050 | PASS |
| `stormcaller_attack` | stormcaller | boss | no | 1.600 | 0.034 | PASS |
| `ocean_leviathan_telegraph` | ocean_leviathan | boss | no | 2.000 | 0.001 | PASS |
| `ocean_leviathan_attack` | ocean_leviathan | boss | no | 1.800 | 0.024 | PASS |
| `colossus_telegraph` | colossus | boss | no | 2.000 | 0.000 | PASS |
| `colossus_attack` | colossus | boss | no | 1.800 | 0.067 | PASS |
| `villager_idle` | npc_villager_child | npc | yes | 4.033 | 0.000 | PASS |
| `villager_wave` | npc_villager_child | npc | no | 1.800 | 0.000 | PASS |
| `guard_salute` | guard | npc | no | 1.600 | 0.000 | PASS |
| `guard_patrol` | guard | npc | yes | 0.833 | 0.000 | PASS |
| `merchant_greet` | npc_merchant | npc | no | 1.800 | 0.000 | PASS |
| `golem_standby` | golem | npc | yes | 4.033 | 0.000 | PASS |

Root offsets are in metres (glTF axes: `+Z` forward, `+Y` up). One-shots hold
their final pose for 12 frames so `end_settled_mps` is measured on a real
settled tail. Loops duplicate frame 0 at the last key with no velocity pop.

## Numeric proof

`manifest.json` → `conventions.gates_warn_fail` uses the same `[warn, fail]`
pairs as the player set (game-animation `references/validation.md`):
seam 0.5/2.0 cm, seam velocity 0.3/0.8 m/s, root velocity 0.1/0.3 m/s,
planted-contact slide 2/5 cm, ground penetration 1/3 cm, float p10 2/5 cm,
end settled 0.05/0.15 m/s.

- **30/30 clips PASS**, 0 WARN, 0 FAIL.
- Loop seam position **0.000 cm** on all four loops; seam/root velocity excess
  **0.000 m/s**.
- Reloaded foot slide ≤ **1.31 cm** worst case (`villager_idle`), ground
  penetration **0.00 cm** on every clip.
- `manifest.json` records per-clip duration, loop flag, root offset in metres,
  `mean_speed_mps`, all gate values, contact pivots, model SHA-256 and model
  height / 1.55 u scale.

Independent verification (`verify_wave2_animations.py`) re-derives the contact
tips from the source model bounds, re-imports each exported GLB in a fresh
Blender process, binds its slotted action and re-runs the gates: **30/30
passed**. It also asserts a single correctly named animation, no scale
channels, all rig nodes present, and sampler times spanning exactly
`0 … (frames-1)/30` seconds.

## Reproduce

```bash
blender -b --python scripts/generate_wave2_animations.py    # rebuild + manifest
blender -b --python scripts/verify_wave2_animations.py      # reload + verify
```
