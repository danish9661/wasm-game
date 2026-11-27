# Fourth animation wave — directional locomotion, boss phase 2, work and interactions

105 clips on the same bound-derived pivot rigs as `../wave2/` and the amplified
poses of `../wave3/`. Three new rigs (`npc_blacksmith`, `npc_farmer`,
`npc_innkeeper`) follow the same rule: every pivot is derived at generation time
from the source GLB's real mesh-node bounds, with no hardcoded joint positions;
travel, leans and contact targets scale with `model_height / 1.55 u`. 30 fps,
LINEAR samplers, no scale keys, one named Action per GLB.

| Clip | Model | Group | Loop | Frames | Duration (s) | Root offset (m) | Slide (cm) | Gate |
|---|---|---|---|---:|---:|---:|---:|---|
| `wolf_walk_f` | npc_dire_wolf | enemy | yes | 25 | 0.833 | 0.408 | 0.00 | PASS |
| `wolf_walk_fl` | npc_dire_wolf | enemy | yes | 25 | 0.833 | 0.408 | 0.00 | PASS |
| `wolf_walk_l` | npc_dire_wolf | enemy | yes | 25 | 0.833 | 0.408 | 0.00 | PASS |
| `wolf_walk_bl` | npc_dire_wolf | enemy | yes | 25 | 0.833 | 0.408 | 0.00 | PASS |
| `wolf_walk_b` | npc_dire_wolf | enemy | yes | 25 | 0.833 | 0.408 | 0.00 | PASS |
| `wolf_walk_br` | npc_dire_wolf | enemy | yes | 25 | 0.833 | 0.408 | 0.00 | PASS |
| `wolf_walk_r` | npc_dire_wolf | enemy | yes | 25 | 0.833 | 0.408 | 0.00 | PASS |
| `wolf_walk_fr` | npc_dire_wolf | enemy | yes | 25 | 0.833 | 0.408 | 0.00 | PASS |
| `wolf_run_f` | npc_dire_wolf | enemy | yes | 19 | 0.633 | 0.469 | 0.00 | PASS |
| `wolf_run_fl` | npc_dire_wolf | enemy | yes | 19 | 0.633 | 0.469 | 0.00 | PASS |
| `wolf_run_l` | npc_dire_wolf | enemy | yes | 19 | 0.633 | 0.469 | 0.00 | PASS |
| `wolf_run_bl` | npc_dire_wolf | enemy | yes | 19 | 0.633 | 0.469 | 0.00 | PASS |
| `wolf_run_b` | npc_dire_wolf | enemy | yes | 19 | 0.633 | 0.469 | 0.00 | PASS |
| `wolf_run_br` | npc_dire_wolf | enemy | yes | 19 | 0.633 | 0.469 | 0.00 | PASS |
| `wolf_run_r` | npc_dire_wolf | enemy | yes | 19 | 0.633 | 0.469 | 0.00 | PASS |
| `wolf_run_fr` | npc_dire_wolf | enemy | yes | 19 | 0.633 | 0.469 | 0.00 | PASS |
| `goblin_walk_f` | goblin | enemy | yes | 25 | 0.833 | 0.320 | 0.00 | PASS |
| `goblin_walk_fl` | goblin | enemy | yes | 25 | 0.833 | 0.320 | 0.00 | PASS |
| `goblin_walk_l` | goblin | enemy | yes | 25 | 0.833 | 0.320 | 0.00 | PASS |
| `goblin_walk_bl` | goblin | enemy | yes | 25 | 0.833 | 0.320 | 0.00 | PASS |
| `goblin_walk_b` | goblin | enemy | yes | 25 | 0.833 | 0.320 | 0.00 | PASS |
| `goblin_walk_br` | goblin | enemy | yes | 25 | 0.833 | 0.320 | 0.00 | PASS |
| `goblin_walk_r` | goblin | enemy | yes | 25 | 0.833 | 0.320 | 0.00 | PASS |
| `goblin_walk_fr` | goblin | enemy | yes | 25 | 0.833 | 0.320 | 0.00 | PASS |
| `goblin_run_f` | goblin | enemy | yes | 19 | 0.633 | 0.368 | 0.00 | PASS |
| `goblin_run_fl` | goblin | enemy | yes | 19 | 0.633 | 0.368 | 0.00 | PASS |
| `goblin_run_l` | goblin | enemy | yes | 19 | 0.633 | 0.368 | 0.00 | PASS |
| `goblin_run_bl` | goblin | enemy | yes | 19 | 0.633 | 0.368 | 0.00 | PASS |
| `goblin_run_b` | goblin | enemy | yes | 19 | 0.633 | 0.368 | 0.00 | PASS |
| `goblin_run_br` | goblin | enemy | yes | 19 | 0.633 | 0.368 | 0.00 | PASS |
| `goblin_run_r` | goblin | enemy | yes | 19 | 0.633 | 0.368 | 0.00 | PASS |
| `goblin_run_fr` | goblin | enemy | yes | 19 | 0.633 | 0.368 | 0.00 | PASS |
| `skeleton_walk_f` | skeleton | enemy | yes | 25 | 0.833 | 0.490 | 0.00 | PASS |
| `skeleton_walk_fl` | skeleton | enemy | yes | 25 | 0.833 | 0.490 | 0.00 | PASS |
| `skeleton_walk_l` | skeleton | enemy | yes | 25 | 0.833 | 0.490 | 0.00 | PASS |
| `skeleton_walk_bl` | skeleton | enemy | yes | 25 | 0.833 | 0.490 | 0.00 | PASS |
| `skeleton_walk_b` | skeleton | enemy | yes | 25 | 0.833 | 0.490 | 0.00 | PASS |
| `skeleton_walk_br` | skeleton | enemy | yes | 25 | 0.833 | 0.490 | 0.00 | PASS |
| `skeleton_walk_r` | skeleton | enemy | yes | 25 | 0.833 | 0.490 | 0.00 | PASS |
| `skeleton_walk_fr` | skeleton | enemy | yes | 25 | 0.833 | 0.490 | 0.00 | PASS |
| `skeleton_run_f` | skeleton | enemy | yes | 19 | 0.633 | 0.564 | 0.00 | PASS |
| `skeleton_run_fl` | skeleton | enemy | yes | 19 | 0.633 | 0.564 | 0.00 | PASS |
| `skeleton_run_l` | skeleton | enemy | yes | 19 | 0.633 | 0.564 | 0.00 | PASS |
| `skeleton_run_bl` | skeleton | enemy | yes | 19 | 0.633 | 0.564 | 0.00 | PASS |
| `skeleton_run_b` | skeleton | enemy | yes | 19 | 0.633 | 0.564 | 0.00 | PASS |
| `skeleton_run_br` | skeleton | enemy | yes | 19 | 0.633 | 0.564 | 0.00 | PASS |
| `skeleton_run_r` | skeleton | enemy | yes | 19 | 0.633 | 0.564 | 0.00 | PASS |
| `skeleton_run_fr` | skeleton | enemy | yes | 19 | 0.633 | 0.564 | 0.00 | PASS |
| `archer_walk_f` | archer | enemy | yes | 25 | 0.833 | 0.380 | 0.00 | PASS |
| `archer_walk_fl` | archer | enemy | yes | 25 | 0.833 | 0.380 | 0.00 | PASS |
| `archer_walk_l` | archer | enemy | yes | 25 | 0.833 | 0.380 | 0.00 | PASS |
| `archer_walk_bl` | archer | enemy | yes | 25 | 0.833 | 0.380 | 0.00 | PASS |
| `archer_walk_b` | archer | enemy | yes | 25 | 0.833 | 0.380 | 0.00 | PASS |
| `archer_walk_br` | archer | enemy | yes | 25 | 0.833 | 0.380 | 0.00 | PASS |
| `archer_walk_r` | archer | enemy | yes | 25 | 0.833 | 0.380 | 0.00 | PASS |
| `archer_walk_fr` | archer | enemy | yes | 25 | 0.833 | 0.380 | 0.00 | PASS |
| `archer_run_f` | archer | enemy | yes | 19 | 0.633 | 0.437 | 0.00 | PASS |
| `archer_run_fl` | archer | enemy | yes | 19 | 0.633 | 0.437 | 0.00 | PASS |
| `archer_run_l` | archer | enemy | yes | 19 | 0.633 | 0.437 | 0.00 | PASS |
| `archer_run_bl` | archer | enemy | yes | 19 | 0.633 | 0.437 | 0.00 | PASS |
| `archer_run_b` | archer | enemy | yes | 19 | 0.633 | 0.437 | 0.00 | PASS |
| `archer_run_br` | archer | enemy | yes | 19 | 0.633 | 0.437 | 0.00 | PASS |
| `archer_run_r` | archer | enemy | yes | 19 | 0.633 | 0.437 | 0.00 | PASS |
| `archer_run_fr` | archer | enemy | yes | 19 | 0.633 | 0.437 | 0.00 | PASS |
| `scorpion_queen_phase2_telegraph` | scorpion_queen | boss | no | 60 | 2.000 | 0.057 | 0.00 | PASS |
| `scorpion_queen_phase2_slam` | scorpion_queen | boss | no | 54 | 1.800 | 0.036 | 0.00 | PASS |
| `toad_king_phase2_telegraph` | toad_king | boss | no | 60 | 2.000 | 0.058 | 0.03 | PASS |
| `toad_king_phase2_slam` | toad_king | boss | no | 54 | 1.800 | 0.037 | 0.01 | PASS |
| `stormcaller_phase2_telegraph` | stormcaller | boss | no | 60 | 2.000 | 0.092 | 0.00 | PASS |
| `stormcaller_phase2_slam` | stormcaller | boss | no | 54 | 1.800 | 0.108 | 0.00 | WARN |
| `ocean_leviathan_phase2_telegraph` | ocean_leviathan | boss | no | 60 | 2.000 | 0.076 | 0.00 | PASS |
| `ocean_leviathan_phase2_slam` | ocean_leviathan | boss | no | 54 | 1.800 | 0.105 | 0.00 | PASS |
| `colossus_phase2_telegraph` | colossus | boss | no | 60 | 2.000 | 0.079 | 0.06 | PASS |
| `colossus_phase2_slam` | colossus | boss | no | 54 | 1.800 | 0.050 | 0.01 | PASS |
| `blacksmith_hammer_loop` | npc_blacksmith | npc | yes | 37 | 1.233 | 0.000 | 0.02 | PASS |
| `merchant_greet_loop` | npc_merchant | npc | yes | 43 | 1.433 | 0.000 | 0.01 | PASS |
| `guard_salute_loop` | guard | npc | yes | 37 | 1.233 | 0.000 | 0.01 | PASS |
| `golem_standby_loop` | golem | npc | yes | 121 | 4.033 | 0.000 | 0.00 | PASS |
| `farmer_harvest` | npc_farmer | interaction | no | 48 | 1.600 | 0.000 | 0.03 | PASS |
| `farmer_build` | npc_farmer | interaction | yes | 43 | 1.433 | 0.000 | 0.02 | PASS |
| `farmer_mine` | npc_farmer | interaction | no | 48 | 1.600 | 0.000 | 0.02 | PASS |
| `innkeeper_cook` | npc_innkeeper | interaction | yes | 49 | 1.633 | 0.000 | 0.01 | PASS |
| `innkeeper_sleep` | npc_innkeeper | interaction | yes | 121 | 4.033 | 0.000 | 0.01 | PASS |
| `merchant_open_chest` | npc_merchant | interaction | no | 54 | 1.800 | 0.000 | 0.05 | PASS |
| `goblin_hit_light` | goblin | enemy | no | 42 | 1.400 | 0.058 | 0.01 | PASS |
| `goblin_hit_heavy` | goblin | enemy | no | 48 | 1.600 | 0.127 | 0.02 | PASS |
| `goblin_death_alt` | goblin | enemy | no | 60 | 2.000 | 0.148 | 0.01 | PASS |
| `skeleton_hit_light` | skeleton | enemy | no | 42 | 1.400 | 0.087 | 0.02 | PASS |
| `skeleton_hit_heavy` | skeleton | enemy | no | 48 | 1.600 | 0.189 | 0.03 | PASS |
| `skeleton_death_alt` | skeleton | enemy | no | 60 | 2.000 | 0.198 | 0.03 | PASS |
| `wolf_hit_light` | npc_dire_wolf | enemy | no | 42 | 1.400 | 0.078 | 0.02 | PASS |
| `wolf_hit_heavy` | npc_dire_wolf | enemy | no | 48 | 1.600 | 0.170 | 1.36 | PASS |
| `wolf_death_alt` | npc_dire_wolf | enemy | no | 60 | 2.000 | 0.861 | 0.05 | PASS |
| `archer_hit_light` | archer | enemy | no | 42 | 1.400 | 0.084 | 0.00 | PASS |
| `archer_hit_heavy` | archer | enemy | no | 48 | 1.600 | 0.183 | 0.00 | PASS |
| `archer_death_alt` | archer | enemy | no | 60 | 2.000 | 0.192 | 0.02 | PASS |
| `brute_hit_light` | brute | enemy | no | 42 | 1.400 | 0.097 | 0.00 | PASS |
| `brute_hit_heavy` | brute | enemy | no | 48 | 1.600 | 0.210 | 0.00 | PASS |
| `brute_death_alt` | brute | enemy | no | 60 | 2.000 | 0.221 | 0.02 | PASS |
| `spider_hit_light` | spider | enemy | no | 42 | 1.400 | 0.000 | 0.00 | PASS |
| `spider_hit_heavy` | spider | enemy | no | 48 | 1.600 | 0.000 | 0.00 | PASS |
| `spider_death_alt` | spider | enemy | no | 60 | 2.000 | 0.101 | 0.00 | PASS |
| `slime_hit_light` | slime | enemy | no | 42 | 1.400 | 0.000 | 0.00 | PASS |
| `slime_hit_heavy` | slime | enemy | no | 48 | 1.600 | 0.000 | 0.00 | PASS |
| `slime_death_alt` | slime | enemy | no | 60 | 2.000 | 0.442 | 0.00 | PASS |

## What is new

- **8-direction walk/run** (`wolf`, `goblin`, `skeleton`, `archer` × `walk|run` ×
  `f fl l bl b br r fr`, 64 clips). Root carries the named yaw **and** a real
  one-stride metre offset; contact tips and their lowest sole vertices are
  pinned to a root-local treadmill target so a planted sole is still in world
  space. One foot is in mid-stance at the seam, so seam velocity is continuous.
  The archer and brute use their 38/65 cm upper-leg pivots (the wave-2
  12/15 cm lower-leg pivots cannot carry a stride).
- **Boss phase-2 pairs** (`*_phase2_telegraph`, `*_phase2_slam`, 10 clips):
  a held, readable wind-up and its heavy payoff, distinct from wave-3
  `*_enrage` / `*_ultimate`. Hovering casters use a shallower lean so their
  hem/coils stay near their rest height.
- **NPC work loops** (4 clips): blacksmith hammer, plus loop-published greet,
  salute and standby variants with closed seams.
- **NPC interactions** (6 clips): farmer harvest/build/mine, innkeeper
  cook/sleep, merchant open chest.
- **Hit/death variants** (21 clips): `*_hit_light`, `*_hit_heavy` and
  `*_death_alt` for all seven enemy families (goblin, skeleton, wolf, archer,
  brute, spider, slime). Alternative deaths fold forward rather than backslide.

Root offsets are in metres (glTF `+Z` forward, `+Y` up). One-shots hold their
final pose for 12 frames so `end_settled_mps` is measured on a real settled
tail; loops duplicate frame 0 at the last key in root-relative pose.

## Numeric proof

`manifest.json` → `conventions.gates_warn_fail` uses the project's
`[warn, fail]` pairs (game-animation `references/validation.md`): seam 0.5/2.0 cm,
seam velocity 0.3/0.8 m/s, root velocity 0.1/0.3 m/s, planted-contact slide
2/5 cm, penetration 1/3 cm, float p10 2/5 cm, end settled 0.05/0.15 m/s, plus a
travel-direction gate of 5/12 degrees. A second, informational slide field
(`foot_slide_fast_cm`) re-measures planted soles with a 1.0 m/s velocity window
so faster gaits cannot hide skate; it is reported, not gated, because a planted
sole's *root-relative* motion is the stride and is not slide.

- **105/105 clips: 104 PASS, 1 WARN, 0 FAIL.** The single WARN is
  `stormcaller_phase2_slam` (float p10 2.16 cm vs 2.0 warn): the stormcaller
  rig has no ground contacts and the robe hem lifts slightly on the staff
  slam; the same hover convention is used by wave 2's stormcaller clips. The
  reason is recorded in that clip's manifest note.
- All 64 directional gaits: seam position 0.000 cm, direction error 0.0 deg,
  0 ground penetration, reported slide within gate.
- `scripts/verify_wave4_animations.py` re-derives contact tips from the source
  GLB bounds, re-imports every exported GLB in a fresh Blender process, rebinds
  its slotted action, and re-runs the gates plus the extra fields:
  **105/105 passed**.

## Reproduce / verify

```bash
blender -b --python scripts/generate_wave4_animations.py
blender -b --python scripts/verify_wave4_animations.py
```
