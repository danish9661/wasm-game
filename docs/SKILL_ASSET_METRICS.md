# Starfall - Skill-built asset metrics

One hero prop authored with the **game-asset-modeling** skill pipeline (brief -> placeholder -> bmesh form -> finish -> validate -> Babylon review -> manifest). All numbers below are read from the exported files, not from the brief.

## `prop_star_crown_shrine`

A Starfall shrine plinth displaying the Star Crown: stepped stone base, four gold-capped posts, a six-sided gold halo and a hovering arcane crown with an emissive gem. Built from the brief at `assets/src/props/prop_star_crown_shrine/brief.json`.

| Measurement | Value |
|---|---:|
| Brief dimensions (W x D x H, Blender axes) | 1.30 x 0.72 x 2.20 u |
| Measured W x D x H (X x Z x Y, glTF axes) | 1.240 x 0.720 x 2.190 u |
| Player-relative height (x1.55) | 1.41x |
| Base Y | 0.000 u |
| Footprint centre (X, Z) | (+0.000, +0.000) u |
| Render triangles (LOD0) | 398 |
| Project verifier triangles (render + collider) | 410 |
| Materials | 4 (stone, gold, star gem, dim arcane) |
| Triangle budget | 500 (prop) |

## Scale contract

- **1 unit = 1 tile = 1 metre.** Blender is Z-up; exported glTF is Y-up. W x D x H above are in glTF axes (X footprint x Z footprint x Y height).
- Player reference **1.55 u**; the shrine is 1.41x the player, below the 2.5 u storey.
- Bottom-centre pivot at the world origin; base Y = 0 and footprint centre (0, 0) in the exported GLB.
- Flat-shaded, palette-only Principled materials, no textures, no smooth normals.

## Skill checks

- `gam_cli.py validate`: **PASS** - 0 fail, 0 warn. Includes naming, hierarchy, applied transforms, budget, dimensions, pivot, collider, normals, UV0, lightmap UV, materials, palette and Khronos.
- `gam_cli.py review`: **0 engine errors**. Views: front, three-quarter, eye, iso, iso_context, night and two crown close-ups; sheet at `assets/review/prop_star_crown_shrine.png`.
- Emissive accent verified in the night view: the purple gem and two shards read against a dark scene.
- `python3 scripts/verify_models.py`: **223 GLBs, 223 passed, 0 failed**; `python3 scripts/check_metrics.py`: OK.

## Reproduce

```bash
BLENDER=~/.local/blender/blender gam_cli.py build assets/src/props/prop_star_crown_shrine/brief.json
gam_cli.py validate assets/src/props/prop_star_crown_shrine/brief.json
gam_cli.py review assets/src/props/prop_star_crown_shrine/brief.json
gam_cli.py manifest assets/src/props/prop_star_crown_shrine/brief.json
python3 scripts/verify_models.py
python3 scripts/check_metrics.py
python3 scripts/build_skill_asset_metrics.py
```
