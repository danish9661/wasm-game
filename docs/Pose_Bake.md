# Pose bake — real rig animation as per-frame GLBs

`assets/models/player.glb` is one merged mesh, so a renderer can only bob the
whole body. The authored clips in `assets/animations/player/*.glb` do contain a
real object-empty rig (`PARTS` / `CHAIN` from `scripts/generate_player_final.py`)
but ship with that rigid mesh too. This pipeline keeps the rig, discards the
rigid mesh, skins low-poly parts onto the empties and bakes each pose into a
standalone GLB the renderer can swap by filename — no skinning in the client.

## Run

```bash
blender -b --python scripts/bake_pose_frames.py   # Blender 4.2+ (tested 5.1.2)
python3 scripts/verify_models.py
python3 scripts/check_metrics.py
```

`bake_pose_frames.py` also renders the baked walk frames and assembles the
contact sheet with `scripts/assemble_pose_sheet.py` (system Python + Pillow).

## Pipeline

1. **Rig + rest joints.** For each clip, the GLB JSON node TRS is read back
   into Blender Z-up world joints. Those node translations are the rest joint
   positions authored by `generate_player_final.py`; reading them from the clip
   keeps the skin aligned with the rig actually stored in that file. The rig's
   `MESH_TO_JOINT` bindings decide which joint each visual part follows, and
   `CHAIN` supplies segment endpoints (shoulder→elbow→wrist, hip→knee→ankle).
2. **Articulated body.** The rigid `char_player_lod0` mesh and the unused
   `char_player` empty are deleted. A 316-triangle body is built from flat-shaded
   palette primitives: hexagonal limb prisms, boxes for hips/boots, ico spheres
   for head/hair/hands. Colours are only `#FF8C26` (tunic), `#EAB88C` (skin),
   `#6B4423` (leather) and `#592F1A` (dark leather), roughness 0.85, no textures.
   Overall silhouette is ~1.55 u tall.
3. **Bake.** At each authored frame, the empties are evaluated
   (`scene.frame_set`); each part is transformed by
   `M_world(joint) · inverse(rest_world(joint))` and all parts are merged into
   one mesh with per-material primitives. Frames are **translated only** —
   never rescaled — so the silhouette keeps its authored size.
4. **Normalise.** Every frame is shifted so minimum Y = 0 and the X/Z centre is
   (0, 0) in glTF space (Blender: minimum Z = 0, X/Y centre = 0). Swapping
   frames therefore never jumps or drifts.
5. **Export.** One static GLB per frame: `assets/models/pose_<clip>_<NN>.glb`,
   no animation channels, one mesh, four materials.

## Frame policy (`assets/models/poses.json`)

```json
{
  "walk": {"fps": 15, "frames": ["pose_walk_00.glb", ...], "loop": true}
}
```

- **Loops** (`walk`, `idle`): 12 frames evenly spread over one authored cycle
  (24 frames for walk, 120 for idle). `fps` is the playback rate for that
  12-frame list: `12 × 30 / cycle` — 15 fps for walk, 3 fps for idle — so a loop
  still lasts the authored 0.8 s / 4.0 s. `loop: true`.
- **One-shots** (`attack_1`, `dodge_roll`, `hit_light`): every authored frame,
  including the generator's 12-frame settle tail, at the original 30 fps.
  `loop: false`.

| Clip | Baked frames | Playback fps | Loop |
|---|---:|---:|---|
| walk | 12 | 15 | yes |
| idle | 12 | 3 | yes |
| attack_1 | 48 | 30 | no |
| dodge_roll | 54 | 30 | no |
| hit_light | 42 | 30 | no |

Total: **168 pose GLBs**, 316 triangles each.

## Verification

`verify_models.py` parses every GLB (including all poses) and checks base Y ≈ 0,
X/Z centring and the 500-triangle budget; `check_metrics.py` re-checks the
invariants. Both pass. The walk contact sheet at `assets/previews/pose_bake.png`
shows twelve frames; leg and arm poses visibly alternate across the cycle
(also confirmed numerically: all four material primitives change bounding-box
centre between frame 0 and the mid-cycle frame).

## Runtime contract (client work, not changed here)

Load `assets/models/poses.json`, resolve `frames[i]` relative to `assets/models`,
and index with a clip-local clock advancing at that clip's `fps`; wrap to frame
0 when `loop` is true, otherwise clamp/hold the last frame. Each pose GLB is a
complete static mesh at the origin — replace the previous frame's mesh, do not
offset or scale it.
