# Starfall — wave 5 asset briefs

Every wave-5 asset was briefed **before** modelling. The three hero interiors
also have machine-readable briefs at `assets/src/env/<id>/brief.json`.
Measured dimensions and player ratios live in `docs/MODELS_V5_METRICS.md`.

## Shared contract

- **What / where:** props and enemies for the existing biome, town and dungeon
  sets, plus three complete interiors that drop into the 1 u town/dungeon kit.
- **Units:** 1 u = 1 tile = 1 m. Blender is Z-up, glTF is Y-up. Player 1.55 u;
  storey 2.5 u; doors 0.9 x 2.0 u; window sill 1.1 u; grid pitch 1 u.
- **Pivot / collider:** `bottom_center` — base exactly z = 0, footprint centred
  at (x, z) = (0, 0). Props use a simple box collider; interiors use a box
  covering the 4x4 footprint and one storey. No sockets or states in this wave.
- **Props / variants / states:** none. Each asset is a single static palette
  mesh; no LOD children, no external textures.
- **Views.primary:** `iso` for interiors (the diorama cutaway read); `front`
  for enemies and town/dungeon props; `three_quarter` for biome scatter.
- **Style notes:** Low-Poly Diorama — chunky, faceted, flat-shaded, saturated
  palette accents, readable at ~60 px, no textures, no smooth shading. Palette
  shades are lightness variants only. **Never:** smooth normals, invented hues,
  external textures, faces nobody sees, fake scale.

## 5A — Biome variety

| Asset | What / where | Target W x D x H (u) | Pivot | Primary view |
|---|---|---:|---|---|
| `swamp_reeds` | Cattail reed bed with pools; swamp floor scatter | 1.2 x 0.9 x 1.6 | bottom_center | three_quarter |
| `mangrove_roots` | Arched mangrove root tangle in mud and shallow water | 1.7 x 1.4 x 1.4 | bottom_center | three_quarter |
| `ice_spire` | Jagged frozen spike cluster; tundra landmark scatter | 1.1 x 1.0 x 2.0 | bottom_center | three_quarter |
| `frozen_log` | Ice-coated fallen log with frost branches; tundra floor | 1.8 x 1.0 x 0.8 | bottom_center | three_quarter |
| `desert_ruin_wall` | Crumbled sandstone courses with a dune; desert ruin kit | 1.8 x 0.6 x 1.5 | bottom_center | three_quarter |
| `sandstone_pillar` | Fluted sandstone column with capital; desert ruin kit | 0.7 x 0.7 x 2.4 | bottom_center | front |
| `wreck_hull_section` | Broken hull planks and ribs; ocean wreck dressing | 2.0 x 1.0 x 1.2 | bottom_center | three_quarter |
| `wreck_rudder` | Detached rudder with tiller and iron pintles | 1.0 x 0.5 x 1.6 | bottom_center | front |
| `wreck_debris` | Salted planking, sail scrap and barnacles; beach scatter | 1.7 x 1.2 x 0.6 | bottom_center | three_quarter |
| `obsidian_spire` | Black volcanic-glass spire with lava seams | 1.0 x 1.0 x 2.0 | bottom_center | front |
| `volcanic_bomb` | Cracked molten ejecta with embers and ash | 1.0 x 1.0 x 0.9 | bottom_center | three_quarter |
| `basalt_columns` | Hexagonal basalt columns with cooling cracks | 1.3 x 1.2 x 1.7 | bottom_center | three_quarter |

## 5B — Enemy variants

All four are standalone silhouettes at player scale and do not reuse the
existing `goblin`, `wraith`, `golem` or `npc_ice_elemental` forms.

| Asset | What / where | Target W x D x H (u) | Pivot | Primary view |
|---|---|---:|---|---|
| `armored_goblin` | Plate-breastplated goblin with shield and sword | 0.95 x 0.7 x 1.35 | bottom_center | front |
| `wraith_knight` | Crowned hooded wraith with sword and wisps | 1.05 x 0.7 x 1.9 | bottom_center | front |
| `fire_elemental` | Molten core with flame crown and ember fists | 1.2 x 0.9 x 2.0 | bottom_center | front |
| `stone_elemental` | Rock body with arcane crystal veins | 1.4 x 1.0 x 1.85 | bottom_center | front |

## 5C — Town props

| Asset | What / where | Target W x D x H (u) | Pivot | Primary view |
|---|---|---:|---|---|
| `market_crates` | Stacked stall crates with produce on a low stand | 1.6 x 1.0 x 1.4 | bottom_center | front |
| `shop_sign` | Hanging painted mug sign on a post and arm | 1.0 x 0.4 x 1.7 | bottom_center | front |
| `bench_ornate` | Carved two-seat bench with finials and braces | 1.6 x 0.7 x 1.1 | bottom_center | front |
| `vendor_cart` | Canopied market cart with crates and fruit | 1.8 x 1.1 x 2.0 | bottom_center | three_quarter |
| `street_planter` | Stone flowering trough with shrubs and stems | 1.2 x 0.7 x 1.1 | bottom_center | front |
| `hitching_post` | Hitching rail with three iron rings | 1.8 x 0.4 x 1.2 | bottom_center | front |

## 5D — Dungeon props

| Asset | What / where | Target W x D x H (u) | Pivot | Primary view |
|---|---|---:|---|---|
| `blood_altar` | Tiered chalice altar with candles and runes | 1.4 x 0.9 x 1.3 | bottom_center | front |
| `chain_hoist` | Ceiling wheel, chain links and a hanging cage | 0.9 x 0.7 x 2.1 | bottom_center | front |
| `open_sarcophagus` | Open-lidded stone coffin with mask and bones | 2.0 x 1.0 x 0.9 | bottom_center | three_quarter |
| `portcullis_gate` | Spiked iron gate with jambs, lintel and chains | 1.7 x 0.4 x 2.5 | bottom_center | front |
| `crypt_brazier` | Tripod iron fire bowl with coal and flame | 0.8 x 0.8 x 1.3 | bottom_center | front |
| `bone_chandelier` | Six-candle bone ring with skull and hanging chains | 1.2 x 1.2 x 1.4 | bottom_center | three_quarter |

## 5E — Hero interiors

| Asset | What / where | Target W x D x H (u) | Pivot | Primary view |
|---|---|---:|---|---|
| `tavern_interior` | Complete 4x4 tavern: bar, stools, tables, hearth, lanterns | 5.0 x 5.0 x 2.5 | bottom_center | iso |
| `smithy_interior` | Complete 4x4 smithy: forge, anvil, rack, trough, torches | 5.0 x 5.0 x 2.5 | bottom_center | iso |
| `shrine_interior` | Complete 4x4 shrine: dais, altar, crystal, banners, censer | 4.75 x 4.75 x 2.5 | bottom_center | iso |

Each hero uses bmesh-authored forms, a selective 1-segment bevel on its
silhouette parts, bmesh normal recalculation, weighted normals, flat shading
and an explicit UV pack. The 1 u wall grid, 2.5 u storey and 0.9 x 2.0 u south
door are shared by all three. Footprints include the corner posts: the tavern
and smithy use 1 u posts on the wall grid (5.0 u overall), the shrine uses
0.75 u posts plus 0.125 u wall infills (4.75 u overall) so its north windows
stay clear of the posts in the isometric view. The two camera-facing walls
(south and east) are 1.1 u diorama cutaways; north and west keep all three
2.5 u courses. Quality gates: Khronos glTF-Validator (0 errors and 0 warnings)
and Babylon review renders with 0 engine errors. All three were iterated with
independent fresh-eyes critics until no major findings remained.
