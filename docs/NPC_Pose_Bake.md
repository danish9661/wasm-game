# NPC pose bake — baked per-frame poses for the enemy / NPC families

`scripts/bake_npc_pose_frames.py` applies the player pipeline in
`docs/Pose_Bake.md` to every main character family. The wave2/3/4 and world
animation GLBs already carry a real object-empty pivot rig with each mesh part
bound to a pivot; the bake keeps that rig, discards the clip's rigid meshes,
skins the family's authored low-poly mesh onto the pivots
(`M_pose @ inv(M_rest)`), and exports one flat, normalised static GLB per frame.

## Run

```bash
/home/danish1075/.local/blender/blender -b --python scripts/bake_npc_pose_frames.py
python3 scripts/verify_npc_poses.py
python3 scripts/verify_models.py
python3 scripts/check_metrics.py
```

The script also renders baked frames with Workbench and assembles the contact
sheet with `scripts/assemble_npc_pose_sheet.py` (system Python + Pillow).

## Output contract

* `assets/models/npcpose_<clip>_<NN>.glb`, e.g. `npcpose_goblin_walk_00.glb`.
* `assets/models/npc_poses.json` uses the **same schema as `poses.json`**:
  `"goblin_walk": {"fps": 15, "frames": [...], "loop": true}`.
* One merged mesh per frame: flat shaded, palette colours only, no textures.
* Every frame is shifted so glTF min Y = 0 and X/Z centre = (0, 0) (Blender:
  min Z = 0, X/Y centre = 0). Frames can be swapped without offsetting.
* No per-frame scaling: each part is rigid under the rig's translation and
  rotation, so edge lengths are *identical* across frames. A family's body
  dimensions therefore never drift; bounding boxes can still grow when an
  authored accessory swings (e.g. the stormcaller raises a long staff, the
  scorpion queen rears its tail).
* Every family is under the 600-triangle budget (158–416 triangles/frame).

## Frame policy

| Source clip | Baked frames | Playback fps | Loop |
|---|---:|---:|---|
| 12-frame loop (walk/idle cycle) | 12 | `12 * 30 / (authored_frames - 1)` | yes |
| 16-frame one-shot (attack/telegraph) | 16 | `16 * 30 / authored_frames` | no |

Loop indices are `round(i * cycle / 12)`, `i = 0..11` (one authored cycle; the
last key duplicates frame 0). One-shot indices are `round(i * (N-1)/15)` and
include the final settle. Frame counts/fps come from the wave/world manifests
(wave manifests win where a clip name appears in both).

54 clips, 748 GLBs. Contact sheet: `assets/previews/npc_pose_bake.png` (6 rows
of 12 frames: goblin walk/attack, skeleton walk, wolf walk, spider walk,
scorpion queen attack). Limb poses visibly alternate across frames.

## Family / source clip table

| Family (base GLB) | `npc_poses.json` clips | Authored source clip(s) |
|---|---|---|
| goblin (`goblin.glb`) | idle, walk, attack | `goblin_idle` (w3), `goblin_walk_f` (w4), `goblin_attack` (w2) |
| skeleton (`skeleton.glb`) | idle, walk, attack | `skeleton_idle` (w3), `skeleton_walk_f` (w4), `skeleton_attack` (w2) |
| wolf (`npc_dire_wolf.glb`) | idle, walk, attack | `wolf_idle` (w3), `wolf_walk_f` (w4), `wolf_attack` (w2) |
| archer (`archer.glb`) | idle, walk, attack | `archer_idle` (w3), `archer_walk_f` (w4), `archer_shoot` (w2) |
| raider (`raider.glb`) | idle, walk, attack | `raider_sneak` (world) for **both** idle and walk (no authored idle), `raider_slash` (world) |
| guard (`guard.glb`) | idle, walk, attack | `guard_watch` (w3), `guard_patrol` (w2), `guard_shield_bash` (world) |
| golem (`golem.glb`) | idle, walk, attack | `golem_standby` (w2), `golem_work` (w3) for **both** walk and attack (no authored strike) |
| brute (`brute.glb`) | idle, walk, attack | `brute_idle`, `brute_walk` (w3), `brute_smash` (w2) |
| slime (`slime.glb`) | idle, walk, attack | `slime_idle` (w3), `slime_hop` (w2) for **both** walk and attack (no authored strike) |
| spider (`spider.glb`) | idle, walk, attack | `spider_idle` (w3), `spider_scuttle`, `spider_attack` (w2) |
| imp (`imp.glb`) | idle, walk, attack | `imp_hover` (world), `imp_mischief` (world) for **both** walk and attack |
| wraith (`wraith.glb`) | idle, walk, attack | **No wraith clip or rig exists**: `wraith.glb` is one merged mesh. Skinned a wraith-palette proxy onto the real skeleton rig (`skeleton_idle`, `skeleton_walk_f`, `skeleton_attack`), matching `game/src/anim.rs` which already maps wraith → skeleton clips. |
| scorpion_queen | idle, walk, attack | `scorpion_queen_telegraph` (w2) for **both** idle and walk (no authored locomotion), `scorpion_queen_attack` (w2) |
| toad_king | idle, walk, attack | `toad_king_telegraph` (w2) for **both** idle and walk, `toad_king_attack` (w2) |
| stormcaller | idle, walk, attack | `stormcaller_telegraph` (w2) for **both** idle and walk, `stormcaller_attack` (w2) |
| colossus | idle, walk, attack | `colossus_telegraph` (w2) for **both** idle and walk, `colossus_attack` (w2) |
| villager (`npc_villager_child.glb`) | idle, walk, wave | `villager_idle`, `villager_wave` (w2), `villager_gather` (w3, used as walk) |
| merchant (`npc_merchant.glb`) | idle, walk, greet | `merchant_count` (w3), `merchant_greet_loop` (w4), `merchant_greet` (w2) |

Duplicated entries produce identical geometry under different stable names so
every family exposes the same three states; this is deliberate and documented
rather than silently inventing motion that was never authored.

## Verification

`scripts/verify_npc_poses.py` parses every baked GLB and checks base/centre,
triangle budget, absence of textures, identical edge-length signature (no
per-frame rescaling) and actual vertex motion between frames. It passes for all
748 frames. `scripts/verify_models.py` and `scripts/check_metrics.py` also pass
with the new GLBs included in `assets/models/metrics.json`.

## Runtime contract (client work, not changed here)

Resolve `frames[i]` relative to `assets/models`, advance a clip-local clock at
that clip's `fps`, wrap at `loop: true` and hold the last frame otherwise. Each
GLB is a complete static mesh at the origin: replace the previous mesh, do not
offset or scale it. `web/src` is intentionally untouched.
