# Second-wave visual effects

Twelve deterministic FX GLBs that make the second-wave animation clips
readable. All are 30 fps, LINEAR samplers, one named Action per GLB, exported
Y-up. Geometry is flat-shaded low-poly primitives only (cubes, 4–8 sided
cones/cylinders, ico spheres subdiv 0–1); there are no textures and no
external dependencies. Motion is authored from `sin`/`cos` and seeded key
patches — no randomness, so repeated generation is byte-identical.

Scale keys **are** allowed and intentional here: they provide pop-in,
expansion and fade-out for particles that have no alpha material. Character
rigs in this project still forbid scale keys.

| Effect | Frames | Loop | Prims | Palette | Trigger (source clip → frame) |
|---|---:|---|---:|---|---|
| `fx_boss_telegraph_ring` | 48 | no | 18 | arcane, enemy, gold | all five `*_telegraph` → 0 |
| `fx_boss_impact_burst` | 30 | no | 19 | enemy, fire, gold, steel | all nine `*_attack` → 15 |
| `fx_slash_arc` | 18 | no | 11 | steel, snow | sword/claw attacks → 15 |
| `fx_hit_flash` | 15 | no | 9 | gold, fire, snow | `*_hurt` → 4 |
| `fx_death_puff` | 45 | no | 18 | grass, snow, steel | `*_death` → 24 |
| `fx_hop_dust` | 24 | yes | 12 | grass, snow | `slime_hop`, `spider_scuttle` → 12 |
| `fx_staff_lightning` | 24 | no | 13 | arcane, steel, snow | `stormcaller_attack` → 12 |
| `fx_tongue_splash` | 30 | no | 13 | water, snow | `toad_king_attack` → 10 |
| `fx_stinger_drip` | 36 | no | 13 | grass, arcane | `scorpion_queen_attack` → 12 |
| `fx_tentacle_wake` | 36 | no | 17 | water, snow | `ocean_leviathan_attack` → 12 |
| `fx_mine_sparks` | 30 | no | 14 | gold, steel, fire | player `mine` → 10 and 21 |
| `fx_build_dust` | 24 | no | 11 | grass, snow, gold | player `build` → 8 |

The frame in the table is the frame of the **source clip** at which the FX is
started. Every FX's own event happens at its local frame 0 and decays over its
lifetime. `fx_hop_dust` samples one full 24-frame period with no duplicate
terminal key, so it loops seamlessly.

`fx_manifest.json` is the machine-readable contract: per-effect `name`, `path`,
`duration_s`, `fps`, `loop`, `frames`, `primitive_count`, palette colours used,
and a `sync` object (`clip`/`clips` plus `frame`/`frames`) naming only clip
names that exist in `assets/animations/wave2/manifest.json` or
`assets/animations/player/manifest.json`.

## Verify

```bash
blender -b --python scripts/verify_wave2_fx.py
```

Each GLB is re-imported into a fresh scene: the verifier asserts one correctly
named animation, translation/rotation/scale channels on every mesh node,
sampler times spanning `0 … (frames-1)/30 s`, and that the manifest frame
count agrees with both the reloaded keys and the samplers. Last run:
**12/12 passed**.

## Reproduce

```bash
blender -b --python scripts/generate_wave2_fx.py    # rebuild GLBs + manifest
blender -b --python scripts/verify_wave2_fx.py      # reload + verify
```
