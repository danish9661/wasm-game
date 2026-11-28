# Runtime animation integration

The 3D pass animates **characters from their authored GLB clips**, driven by the
same AI state that drives the 2.5D silhouettes. The earlier build only sampled
player clips and gave enemies/NPCs a generic sine hop; this document describes
the compact runtime table that connects every enemy/NPC family to its clips.

## Where the pieces live

| File | Role |
|---|---|
| `game/src/anim.rs` | Pure `AnimState` / `ClipSet` / style→clip table, state selection, and baked-frame resolution (`BakedClipRow`, `resolve_baked_frame`). No GPU dependency, so `cargo test -p game` covers it. |
| `game/src/render.rs` | `Sprite` carries `anim: AnimState` alongside the existing `walk`/`attack`/`flash` signals. |
| `scripts/gen_animation_data.py` | Blender export of sampled root/hip (`torso` for enemies) motion to Rust. |
| `web/src/generated_animations.rs` | Generated per-frame samples: `[root_x, root_y, root_z, hip_x, hip_y, hip_z]` in metres, first frame subtracted. 30 fps authored rate. |
| `assets/models/poses.json` | Optional player baked-pose manifest (`pose_<clip>_<NN>.glb`). |
| `scripts/gen_pose_list.py` | Generates `web/src/generated_poses.rs` from the player manifest (empty if absent). |
| `assets/models/npc_poses.json`, `assets/models/prop_poses.json` | Optional NPC/prop baked-pose manifests (`npcpose_<family>_<clip>_<NN>.glb`, `animprop_<name>_<NN>.glb`). |
| `scripts/gen_extra_poses.py` | Generates `web/src/generated_extra_poses.rs` (NPC + prop clip tables, empty if manifests are absent). |
| `web/src/renderer.rs` | `anim_motion`, `character_anim`, the unchanged player path, the baked-frame helpers (`npc_pose_model`, `prop_pose_model`), and the 3D instance loop that applies offsets/lean and swaps baked GLBs. |
| `web/src/model.rs`, `web/src/pipeline3d.rs` | `rotation_x`/`rotation_z` helpers and `instance_leaning`, which composes yaw + lean + roll + uniform scale. Props keep zero lean/roll; only their model index can change when a baked frame is available. |

## States and priority

`AnimState` = `Idle | Walk | Run | Attack | Hurt | Death`.
`game::anim::state_from_signals(walk, attack, hurt, dead)` applies the priority
**death > hurt > attack > run > walk > idle**. The sprite builder in
`web/src/renderer.rs` uses the actual AI state instead of the bob phase:

* enemies: `AiState::Idle/Chase/Flee/Attack` → `Idle/Walk/Run/Attack`; a live
  `windup` → `Attack`, a `flash > 0.25` → `Hurt`, `hp <= 0` → `Death`;
* townsfolk: `NpcState::Wander/Patrol/Alert` → `Walk`, `Flee` → `Run`, the rest
  → `Idle`;
* remote co-op players → `Walk` (they have no richer state on the wire).

The 2.5D raster path continues to drive its procedural poses from `walk`,
`attack` and `flash`. Adding `Sprite::anim` neither changes those values nor
any vector passed to the flat renderer, so the fallback is unaffected.

## Clip table

`clips_for_style(style) -> Option<ClipSet>` maps each character `SpriteStyle`
to one family. Props, HP bars and projectiles return `None` and are never
animated. Each `ClipSet` names six authored clips:

| Family | Styles | idle / walk / run / attack / hurt / death |
|---|---|---|
| goblin | Goblin | `goblin_idle`, `goblin_walk_f`, `goblin_run_f`, `goblin_attack`, `goblin_hit_light`, `goblin_death` |
| skeleton | Skeleton, Wraith | `skeleton_idle`, `skeleton_walk_f`, `skeleton_run_f`, `skeleton_attack`, `skeleton_hit_light`, `skeleton_death` |
| wolf | Wolf | `wolf_idle`, `wolf_walk_f`, `wolf_run_f`, `wolf_attack`, `wolf_hit_light`, `wolf_death` |
| archer | Archer, Stoneslinger | `archer_idle`, `archer_walk_f`, `archer_run_f`, `archer_shoot`, `archer_hit_light`, `archer_death` |
| spider | Spider | `spider_idle`, `spider_scuttle` (walk/run), `spider_attack`, `spider_hit_light`, `spider_death_alt` |
| slime | Slime | `slime_idle`, `slime_hop` (walk/run/attack), `slime_hit_light`, `slime_death_alt` |
| brute | Brute, Ogre | `brute_idle`, `brute_walk`, `brute_run`, `brute_smash`, `brute_hit_light`, `brute_death` |
| civilian | Humanoid, Guard | `villager_idle`, `guard_patrol` (walk/run), `guard_salute`, `hit_light`, `death` |
| golem | Golem | `golem_standby`, `golem_work` (walk/run/attack), `hit_light`, `death` |
| imp / raider | Imp, Bat / Raider | `imp_hover`/`raider_sneak`, work loop, `imp_mischief`/`raider_slash` attack, goblin hurt/death |
| bosses | Colossus / ScorpionQueen / ToadKing / Stormcaller / OceanLeviathan | own telegraph/ultimate for attack; closest creature's locomotion, hurt and death where no boss clip was authored |

Fallbacks are explicit in the table, not implicit in a lookup failure: slime
has no dedicated attack (the hop doubles as the strike), and the boss waves
author telegraphs/ultimates but not walks/hurts/deaths. All 55 enemy/NPC clip
names above already exist as GLBs — the missing piece was their sampled runtime
data, so `gen_animation_data.py` was extended to generate it; no new asset GLBs
were needed, and nothing is silently substituted at runtime.

## Sampling and per-instance motion

`anim_motion(name, t)` finds the clip, indexes `(t * 30) % frames.len()`,
linearly interpolates neighbouring samples, and converts glTF axes (+Z up,
−Y forward) to engine axes. From one sample it derives:

* `stride` — root travel along the facing (`root_y`), centred on half the
  authored cycle's total travel (the sprite already moves with AI, so adding
  the whole ramp would double-count/snaps) and amplified 1.0× walk / 1.3× run;
* `lateral` — hip sway across the facing (`root_x`);
* `bob` — hip/torso height (`hip_z`) × 4–6, plus a small stride hop while moving;
* `lean` — hip-vs-root offset plus the rate of authored root travel, clamped to
  ±0.35 rad; attack adds `0.22 * sprite.attack`, hurt tips back, death slumps;
* `roll` — lateral motion, clamped to ±0.25 rad; idles get a tiny breathing sway.

Each instance is placed with
`pipeline3d::instance_leaning(x + dx, h + bob, y + dz, yaw, lean, roll, scale)`
where `dx/dz` rotate `stride`/`lateral` by the sprite facing. A position-derived
seed (`x*2 + y*3`) offsets the clip phase per instance so a pack does not move
in lockstep.

The player keeps its original path (`player_anim_clip` → `anim_offset` →
`instance_at`) and its five player states (`idle/walk/attack_1/dodge_roll/
hit_light`). `anim_offset` now delegates to `anim_motion`, but returns the same
`(right, up, forward)` tuple, so player placement is unchanged.

## Baked NPC / prop pose frames

Two optional asset pipelines can supply per-frame GLBs instead of relying on
the procedural root/hip offsets above:

| Manifest | GLB name pattern | Generated table |
|---|---|---|
| `assets/models/npc_poses.json` | `npcpose_<family>_<clip>_<NN>.glb` | `NPC_POSE_CLIPS` |
| `assets/models/prop_poses.json` | `animprop_<name>_<NN>.glb` | `PROP_POSE_CLIPS` |

`scripts/gen_extra_poses.py` (called by `build.sh`) reads them into
`web/src/generated_extra_poses.rs`: one row per `(group, clip)` with GLB stems
and an `fps`. Nested `family → clip → {frames, fps}` maps, flat
`"<family>_<clip>"` keys and plain `name → frames` maps are all accepted;
`fps` falls back to one clip-second (frame count) when unspecified. **Neither
manifest is required**: when one is missing, its table is emitted empty and
the runtime behaves exactly as before.

Resolution lives in the pure `game::anim` helpers (`baked_frame_at`,
`resolve_baked_frame`) so it is covered by `cargo test -p game`:

* names are compared case-insensitively with `_`/`-`/spaces ignored, so
  `ScorpionQueen` and `scorpion_queen` resolve to the same row;
* an NPC tries semantic states (`idle`, `walk`, `run`, `attack`, `hurt`,
  `death`) first, then authored clip names (`goblin_idle`, ...), across family
  aliases (`npc_family_names`); a prop is resolved by its generated model name
  (plus short aliases such as `crate` for `crate_box`);
* `(t * fps)` indexes the frames with `rem_euclid`, so clips loop in both
  directions and large/negative times never panic; a position-derived seed
  offsets each instance's phase (the player keeps its unseeded path);
* `None` means no manifest, no matching family/clip, empty frames, or an
  invalid rate — the caller keeps the original static GLB.

In `update_3d_scene` the baked stem is used only if it also appears in the
generated `GLB_MODELS` list, which keeps stale manifests harmless. Only the
model index changes inside the GLB branch: ground quads, billboards/HP bars,
camera yaw, tile height, tint and particles stay on their original paths, and
the procedural bob/stride/lean is skipped because the GLB already has authored
motion. Nothing in the player path or the 2.5D raster path is altered.

## Regenerate / verify

```bash
blender -b --python scripts/gen_animation_data.py   # rebuild generated_animations.rs
python3 scripts/gen_extra_poses.py                  # rebuild NPC/prop tables (empty if absent)
cargo test -p game                                  # clip-table + baked-frame regression tests
cargo check --target wasm32-unknown-unknown -p wasm_game
./build.sh                                          # regenerates tables/samples when Blender is present
node tests/smoke_3d.mjs                             # 3D pass + 2.5D fallback (port 8879)
python3 scripts/verify_models.py && python3 scripts/check_metrics.py
```

`cargo test -p game anim::tests` covers the regression requirement: one goblin
resolves five distinct clip names for its five states, distinct kinds choose
distinct clips, distinct gameplay signals map to distinct states, props resolve
to no clip at all, and baked-frame tests cover distinct states → distinct
frames, wrapping (including negative time), prop lookup and the missing-manifest
fallback.
