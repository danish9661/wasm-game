# Project Specification: 2.5D Open-World Survival & Action RPG (Rust + WASM + WebGPU)

## 1. Project Overview & Vision
A high-performance, browser-first **2.5D Isometric Open-World Survival & Action RPG** game built in **Rust**, compiled to **WebAssembly (WASM)**, and rendered via **WebGPU** for maximum speed (WebGL2 automatic fallback).

**Single-player first.** Multiplayer is explicitly deferred to a future phase and must NOT influence current architecture decisions beyond keeping the action-queue simulation decoupled (which is good engineering anyway).

The game features an infinite procedurally generated world with diverse biomes, resource gathering, base building, real-time action combat, day/night atmospheric cycles, a structured story campaign, and a deterministic action-driven architecture.

---

## 2. Technology Stack & Technical Rationale

| Component | Technology | Rationale |
| :--- | :--- | :--- |
| **Core Logic & Simulation** | **Rust (Edition 2021)** | Memory safety, zero-cost abstractions, deterministic simulation, instant compilation to WASM. |
| **Engine / Architecture** | **Bevy ECS** | Entity Component System (ECS) enables decoupled, high-performance rendering of thousands of entities without GC pauses. |
| **Compilation & Target** | **`wasm-pack` / `wasm32-unknown-unknown`** | Standard Rust toolchain for producing compact, fast web binaries. |
| **Rendering** | **WebGPU via WGPU (primary)** | Hardware-accelerated 2.5D isometric tilemaps, sprite batching, custom shaders, dynamic lighting. WebGL2 is only a fallback for unsupported browsers. |
| **Web Wrapper & UI** | **HTML5 + Vanilla JS / CSS** | Lightweight web shell for canvas embedding, fullscreen management, and responsive touch/desktop input handling. |
| **Storage / Persistence** | **IndexedDB / LocalStorage (via `web-sys`)** | Save and load player inventory, world chunk modifications, story progress, and settings locally in the browser. |

> **Note**: Multiplayer networking stack (serde + bincode/postcard, WebSockets, headless server) is REMOVED from current scope. Serialization is only needed for save files now.

---

## 3. The World

### 3.1 Biome Map
Infinite procedural map divided into biomes. The player always spawns in a fixed-story **"Newborn Valley"** (temperate forest) so the tutorial intro is guaranteed.

| Biome | Terrain | Resources | Danger | Day/Night |
| :--- | :--- | :--- | :--- | :--- |
| **Temperate Forest** (start) | Grass, oak/pine, rolling hills | Wood, berries, copper, stone | Wolves | Standard cycle |
| **Arid Desert** | Sand dunes, canyons | Gold, clay, cactus water | Scorpions, sun heat | Extreme heat, cool nights |
| **Snowy Tundra** | Ice, frozen lakes, pines | Iron, frost herbs | Frost elementals | Long nights, cold damage |
| **Swamp & Marsh** | Murky water, willows | Medicinal herbs, ancient wood | Toxic slimes, poison | Fog, dim light |
| **Deep Ocean** | Coastlines, islands | Rare sunken ruins | Sea creatures | — |

### 3.2 Chunk System
- World divided into `32 × 32` tile chunks, loaded/unloaded in a radius around the player.
- Multi-layered Perlin/Simplex noise for elevation, moisture, temperature.
- Story points of interest (ruins, dungeons, NPC village, boss lairs) are seeded **deterministically** — the same seed always produces the same story world.

### 3.3 Atmosphere
- Day/night cycle ≈ **10 real minutes** per full day. Nights are dangerous — torches, campfires, and shelters matter.
- Damage zones: cold in tundra at night, heat in desert midday → forces gear progression.
- Dynamic tile layering: ground, cliffs, props (rocks, flowers), harvestable resources.

---

## 4. What the Player Can Do

1. **Gather** — Axe (trees), pickaxe (ore), shovel (clay/sand), hands (berries, fiber). Tools have durability + tier (stone → copper → iron → gold).
2. **Craft** — Hand crafting → Workbench → Forge → Alchemy Station. Recipes unlock by building stations.
3. **Build** — Walls, doors, floors, storage chests, campfires, fences, defense turrets. Grid + freeform placement. The base is your safe zone at night.
4. **Fight** — 8-directional movement (WASD/joystick), mouse-aim melee arcs, bow + arrows, mana-based magic, stamina-cost dodge rolls.
5. **Survive** — Health, hunger, stamina, temperature. Eat, drink, sleep, stay warm/cool.
6. **Explore** — Ruins and dungeons hold rare loot, new recipes, and lore notes that push the story forward.
7. **Progress** — Grid inventory, equipment slots (Helmet, Armor, Weapon, Shield, Accessory), quick-bar keys 1–8, tier-based crafting tree.

---

## 5. Storyline

> **Chapter 1 — The Wake**: You wake on a forest altar with a single engraved ring reading *"Return to the Crown"* — and no memory. Night is coming. Tutorial: gather wood, build a shelter, survive your first wolf attack.
>
> **Chapter 2 — The Hollow Land**: Scattered ruins reveal the world was once an empire, wiped out when the **Star Crown** shattered. All magic in the land flows from its fragments — that is why the forest bleeds copper, the tundra breathes ice-elementals, and the swamps breed toxic slimes.
>
> **Chapter 3 — The Fragments**: Following compass hints and dungeon lore, you recover **5 Crown Fragments**, one per biome, each guarded by a biome boss:
> 1. Forest Warden (Temperate Forest)
> 2. Scorpion Queen (Desert)
> 3. Frost Golem (Tundra)
> 4. Toad King (Swamp)
> 5. Ocean Leviathan (Deep Ocean)
>
> Each fragment grants a permanent passive power (e.g., warmth, stamina regen, night vision).
>
> **Chapter 4 — The Reforging**: Restore the Crown at the altar where you woke. Endgame choice:
> - **Reign** — the world heals and you become its guardian (victory ending).
> - **Shatter** — free the souls trapped in the crown → New Game+ (harder world, faster nights, "Shattered" difficulty).

**Target length**: 8–15 hours main path, infinite sandbox after.

---

## 6. Core Systems & Architecture

### A. Isometric 2.5D Depth Sorting & Visuals
- **Projection**: True `2:1` isometric view (atan(0.5) ≈ 26.565°).
- **Y-Sorting**: Dynamic z-ordering by entity ground foot position `screen_y = (x + y) * tile_height / 2` — characters render correctly in front/behind trees and structures.
- **Lighting**: Global directional light (Dawn, Noon, Sunset, Midnight) + local point lights with soft falloff (torches, campfires, glowing mushrooms, spells).

### B. Enemy AI State Machine
- `Idle / Patrol` — wanders local spawn region.
- `Alert / Agro` — detects player via sight radius or noise.
- `Attack / Ability` — charges or casts within range.
- `Flee / Retreat` — low-health retreat or calls nearby allies.

### C. Action-Driven Simulation (Decoupled)
```
┌──────────────────────────────────────────────────────────┐
│                       Input System                        │
│            (Keyboard, Mouse, Touch, Gamepad)              │
└──────────────────────────┬───────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────┐
│                   Action Event Queue                     │
│  enum Action {                                           │
│      Move { entity_id, dir },                            │
│      Attack { entity_id, angle, weapon_id },             │
│      PlaceStructure { tile_x, tile_y, structure_type },  │
│      CraftItem { recipe_id },                            │
│      Interact { entity_id, target_id },                  │
│  }                                                       │
└──────────────────────────┬───────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────┐
│              Deterministic World Simulation              │
│   (Updates Chunk State, Resolves Damage, Spawns Drops)   │
└──────────────────────────┬───────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────┐
│                    Renderer / Audio                      │
│    (WebGPU Drawing, Sprite Animation, SFX, Particles)    │
└──────────────────────────────────────────────────────────┘
```
- Single-player: actions are processed immediately by the local simulation loop.
- This architecture keeps the door open for future multiplayer without a rewrite.

---

## 7. Development Roadmap & Milestones

### Phase 1: Engine Foundation & Isometric World View
- [ ] Initialize Cargo workspace with `wasm32-unknown-unknown` and `wasm-pack` config.
- [ ] Canvas setup, **WebGPU (wgpu) context initialization**, WebGL2 fallback path, 60 FPS game loop.
- [ ] Isometric coordinate conversion math (`world_to_iso` / `iso_to_world`).
- [ ] Basic tile rendering with depth/Y-sorting.

### Phase 2: Procedural Chunks & Player Exploration
- [ ] Simplex/Perlin noise chunk generator (`32 × 32` chunks).
- [ ] Dynamic chunk loading/unloading based on player position.
- [ ] Player movement (WASD/Keyboard/Touch), collisions, camera follow.
- [ ] Multi-biome color palettes (Forest, Desert, Snow, Swamp).

### Phase 3: Gathering, Inventory, & Building
- [ ] Resource entity nodes (Trees, Rocks, Bushes) with health + drop tables.
- [ ] Player inventory and item stack data structures.
- [ ] Grid-based building placement (Walls, Floors, Torches, Chests, Campfires).
- [ ] Day/Night lighting shader with torch point lights.

### Phase 4: Combat & Enemy AI
- [ ] Weapon attack arcs and projectile mechanics (bow/magic).
- [ ] Enemy spawners and AI state machines with A* pathfinding.
- [ ] Player health, damage numbers, death/respawn flow.
- [ ] Hunger/stamina/temperature survival stats.

### Phase 5: Story Campaign & Bosses
- [ ] Tutorial intro + story beats (Chapters 1–2).
- [ ] Ruins/dungeon POIs with loot and lore notes.
- [ ] 5 biome bosses with unique mechanics (Chapters 3).
- [ ] Crown Reforging finale + New Game+ (Chapter 4).

### Phase 6: Polish, Audio, & Save System
- [ ] World/quest save-load via IndexedDB.
- [ ] Web Audio API: footsteps, ambient nature, swings, hits.
- [ ] Touch joystick controls for mobile browser support.

### Phase 7 (Future, NOT now): Multiplayer
- [ ] Standalone headless server in Rust (`tokio` + `axum` WebSockets).
- [ ] Client-side prediction and server reconciliation.
- [ ] Synchronized world events, chat, collaborative building.
