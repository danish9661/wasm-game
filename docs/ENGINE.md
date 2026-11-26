# Starfall engine

This document describes **our** engine as it exists in this repository: a pure-Rust
simulation crate (`game`), a WebAssembly/WebGPU client (`web`), a native
authoritative server (`server`), and the HTML/JS page shell in
`web/static/index.html`.

## 1. Architecture at a glance

```
                       ┌──────────────────────────────┐
  browser (JS)         │ web/static/index.html         │
  requestAnimationFrame│  HUD · menus · WebAudio shell │
                       └──────────────┬───────────────┘
                                      │ step(dt), set_mouse(), … (wasm-bindgen)
                       ┌──────────────▼───────────────┐
                       │ web crate (wasm_game)         │
                       │ lib.rs exports · renderer.rs  │
                       │ App: camera, sprites, wgpu    │
                       └──────────────┬───────────────┘
                                      │ game::* calls
                       ┌──────────────▼───────────────┐
                       │ game crate (no wasm deps)     │
                       │ world · player · combat · AI  │
                       │ quests · building · sim       │
                       └──────────────▲───────────────┘
                                      │ same crate, native target
                       ┌──────────────┴───────────────┐
                       │ server (tokio + WebSocket)   │
                       │ authoritative co-op rooms    │
                       └──────────────────────────────┘
```

### `game` crate — deterministic simulation

`game/src/lib.rs` declares the modules; nothing in this crate depends on wasm,
wgpu or the DOM, so the same code compiles natively for the multiplayer server
and for headless tests (`cargo test -p game`).

| Module | Responsibility |
|---|---|
| `world.rs` | Seed-driven procedural chunks (`CHUNK_SIZE = 32`, cached in `ChunkCache`), tile kinds (grass/forest/desert/snow/swamp/water/…), walkability, wading, tile heights and player edits. |
| `player.rs` | Movement, survival stats (HP/hunger/thirst/stamina), leveling, weapon state, camera follow, humanoid sprite parts. |
| `combat.rs`, `weapons.rs`, `enemy.rs` | Melee/ranged hit tests, damage and knockback, enemy registry and AI states (patrol → aggro → telegraph → attack), boss phases and drops. |
| `quest.rs`, `items.rs`, `craft.rs`, `trade.rs`, `npc.rs` | Quest milestones, inventory, anvil recipes, merchant trading and villagers. |
| `building.rs`, `poi.rs`, `dungeon.rs` | Buildable catalogue and placement validation, villages/towns/ruins, multi-floor building interiors and dungeon rooms. |
| `daynight.rs` | `DAY_LENGTH`, `START_TIME`, `NIGHTFALL`, `daylight(t)`, clock formatting, temperature and the every-fourth-night "Shattered Night" schedule. |
| `resources.rs`, `poi.rs`, `world.rs` | Harvest nodes (trees, ore, crystals, …) and water sources for thirst. Weather is client-side cosmetic (`web/src/renderer.rs`, `weather` 0..4) over the shared day/night clock. |
| `sim.rs` | `Simulation::step(dt)`: the authoritative tick. Owns players, enemies, arrows, structures, events and interiors; also the bincode wire protocol (v5), snapshots/deltas and shared-room events used by the server. |
| `render.rs`, `iso.rs`, `elements/` | *Pure data* rendering helpers: iso projection (64×32 diamonds), `Sprite`/`SpriteStyle`, `visible_tiles`, and `build_tile_mesh` which writes the 6-float-per-vertex quad stream. No GPU calls live here. |

`game::Simulation` is the single source of truth. Single-player runs it inside
the WASM app; co-op runs a second copy in the native server, which sends
snapshots/deltas to clients (`ClientMsg` / `ServerMsg`, bincode).

### `web` crate — wasm-bindgen + wgpu client

`web/src/lib.rs` owns a thread-local `App` and exports the JS-facing surface:
`step(dt)`, `app_ready()`, `resize()`, `get_stats()`, `get_frame_dump()`,
`get_ui_data()`, `set_mouse()`, `set_build_mode()`, `place_selected()`,
`craft()`, `serialize_save()`/`deserialize_save()`, `zoom_step()`,
`debug_enter_interior()`, etc. Input handlers (keyboard, resize) are installed
from Rust; touch/UI controls call the exports from JS.

`web/src/renderer.rs` contains `App`: camera and zoom, world/chunk caches,
player/inventory/enemies/quest state, interiors, particles, build ghost,
network client, and the wgpu device/surface/pipeline/buffers. The 3D vertical
slice lives in `model.rs` (GLB -> GPU geometry), `pipeline3d.rs` (reusable
instanced scene) and `terrain3d.rs` (heightfield from world tiles). Also:

* `network.rs` — WebSocket client for co-op rooms (same crate as `sim` protocol).
* `save.rs`, `zip.rs` — versioned JSON saves and `.zip` export/import.

`web/static/index.html` is the page shell: two canvases (`#game` for the WebGPU
surface, `#blit` for the display fallback), HUD/overlays/menus, and a
**procedural WebAudio engine** (`ensureAudio`, `startMusic`, `playSfx`,
`playBiomeEvent`, `audioLevel()`). Audio is gesture-gated: `actx` is created on
first interaction. Rust triggers named effects by looking up `window.playSfx`
(and `window.toast`) through `js_sys::Reflect`, so sound stays entirely in JS.

### `server` crate

`server/src/main.rs` runs the identical `game::Simulation` under tokio and
tungstenite, authoritative for shared-world state and co-op room events.

## 2. Data flow: one frame

1. **`requestAnimationFrame`** in `index.html` calls the WASM export
   `step(Math.min((t - last) / 1000, 0.1))`; the Rust side clamps `dt` to 0..0.1
   again. The JS loop owns frame scheduling; WASM does one update + draw per call.
2. `App::update(dt)` advances visual clocks and particles; if indoors it runs
   the light room walk (`update_interior`), otherwise it steps gameplay
   (input → player movement, enemy AI, combat, quests, day/night, weather,
   structures), follows the camera, refreshes visible tiles, and calls
   `net_sync()` to send input / apply server snapshots.
3. `App::sprites()` builds the frame's `Vec<Sprite>` (terrain props, structures,
   enemies, NPCs, loot, particles, player) from sim state.
4. `game::render::build_tile_mesh(..., &sprites, ...)` fills `App::vertices`:
   **6 floats per vertex — `x, y, r, g, b, a`** (`VERTEX_STRIDE_BYTES = 24`).
   Sprites are depth-sorted by `game::iso::depth_order` (painter's algorithm);
   zoom scales these screen-space vertices about the viewport centre.
5. `App::render()` writes uniforms (viewport, daylight, time-of-day tint, up to
   8 campfire point lights), uploads the vertex buffer with `queue.write_buffer`,
   and runs one wgpu render pass: `rp.draw(0..quad_count * 6, 0..1)`. The WGSL
   shader is a flat 2D vertex/fragment pair; it applies night blending, tint,
   point-light falloff, vignette and horizon fog. Weather motes/rain/snow and
   HP vignettes are drawn as 2D overlays after the frame.
6. **Presentation:** normally the WebGPU canvas `#game` is shown directly
   (`setBackend('gpu')`). If the surface cannot be composited (e.g. headed
   SwiftShader where `present()` is a no-op), `App` switches to the fallback:
   copy the surface back to CPU (`Rgba8Unorm` readback) and `putImageData` into
   the 2D `#blit` canvas, or use the faster GPU→GPU `drawImage` variant
   (`blit_via_draw`) where available. `setBackend('blit')` toggles which canvas
   is visible. `?gpu` forces the direct path; adaptive resolution
   (`RES_LEVELS`, `set_adaptive_res`) keeps slow backends playable.

Screenshots use the same readback (`capture()` / `trigger_readback()`), and the
headless test surface `get_frame_dump()` emits every sprite in screen space as
JSON so tests can assert layout without a GPU.

## 3. Build and run

Requirements: stable Rust + `wasm32-unknown-unknown`, `wasm-pack`, a WebGPU-capable
browser. Co-op additionally needs the native server.

```bash
./build.sh                                   # wasm-pack build web --target web --release -> pkg/
python3 -m http.server 8000 -d pkg           # then open http://localhost:8000
```

`build.sh` copies `web/static/*` and `documentation.html` into `pkg/` and
stamps JS/WASM URLs with the git SHA for cache busting.

```bash
./scripts/play.sh                            # build + serve + launch Chrome with SwiftShader WebGPU flags
cargo run -p starfall-server                 # optional: authoritative co-op server on :8081
cargo test -p game                           # headless simulation tests (no browser needed)
python3 scripts/verify_models.py             # asset contract check -> metrics.json + ASSET_METRICS.md
python3 scripts/check_metrics.py             # lightweight invariants over the metrics file
```

`play.sh` honours `PLAY_PORT` and `CHROME_BIN`. Static hosting works from any
directory server; `pkg/index.html` is the entry point.

## 4. Asset pipeline

1. **Style contract:** `docs/ART_DIRECTION.md` — "Low-Poly Diorama", palette,
   proportions (character ~1.6 u, tree 2.5–3 u, house ~3 u), 1 tile = 1 unit,
   flat shading, <500 triangles, base at z=0, centred at (0,0), Y-up export.
2. **Generators:** `scripts/blender/gen_*.py` (characters, nature, structures,
   weapons, interiors, complex sets) import `scripts/blender/common.py`, build
   one collection per asset and call `export_asset(name, col)` to write a GLB.
   `common.py` also makes contact-sheet previews through `contact_sheet()`.
3. **Outputs:** GLBs in `assets/models/`, previews in `assets/previews/`.
   Generator scripts are the source of truth for model names; the verifier
   reads their `ASSETS`/`BUILDERS` lists to categorise models automatically.
4. **Verification:** `scripts/verify_models.py` parses each GLB directly
   (JSON + BIN chunks, node transforms applied to POSITION accessor bounds),
   checks base/centring/triangle budget, and writes machine-readable
   `assets/models/metrics.json` plus human-readable `docs/ASSET_METRICS.md`.
   `scripts/check_metrics.py` re-asserts the headline invariants. Both are
   pure Python 3 standard library — no Blender needed.

**Status:** the GLBs are now consumed by the shipping client's 3D vertical
slice (see §5): the game canvas fetches `player`, `tree`, `rock`, `house`,
`bush`, `campfire`, `crystal` and `banner` from `pkg/assets/models/` at boot and
draws them with the instanced pipeline. The original 2.5D procedural geometry
remains the fallback (`?2d` / `set_render3d(false)`).

## 5. Current rendering limitations and the planned 3D glTF path

### Where we are

* The renderer is 2.5D: screen-space quads/diamonds with per-vertex colours,
  sorted back-to-front; **no depth buffer, no instancing, no glTF loader**.
* Lighting is faked in the 2D fragment shader (daylight tint + screen-space
  point lights); there are no normals, materials, shadows or PBR.
* Terrain is a tile grid of diamonds generated each frame into a CPU `Vec<f32>`;
  it is not a heightfield mesh.
* Models from `assets/models/*.glb` are authored and validated but not yet
  drawn by the runtime. Animation is procedural (walk cycle/lunge from sprite
  parts), not skeletal.

### Additive 3D glTF path (in progress — not the production renderer)

A parallel work stream is adding `web/src/model.rs` and `web/src/pipeline3d.rs`.
They are deliberately **additive**: the `step()` loop, `App` and all gameplay
exports stay untouched. The 3D scene uses a separate `#demo3d` canvas with its
own wgpu device, surface, depth buffer and pipeline.

Intended interface (landing in parallel, exact signatures may still move):

* **`model.rs`**: a small, dependency-free GLB reader — `load_glb(bytes) ->
  Model`. It bakes node transforms into positions/normals, copies the material
  base colour into a per-vertex colour, exposes `MODEL_VERTEX_FLOATS = 9`
  (position 3 + normal 3 + colour 3), one index buffer and accessor bounds, and
  keeps Y-up coordinates. It follows the same GLB rules as
  `scripts/verify_models.py`, so metrics and runtime agree on bounds/triangles.
* **`pipeline3d.rs`**: instanced draws (one `draw_indexed` per model) with
  `Instance { transform: Mat4, tint }`, an orthographic view-projection matching
  the art contract (yaw 45°, pitch 30°), a depth buffer and a shared time-of-day
  uniform. The wasm exports are `load_demo_models()` (fetch
  `/assets/models/*.glb`) and `render_3d_demo()` in `web/src/lib.rs`.
* **Production integration seam**: `App` currently obtains drawable entities
  from `App::sprites()` / `game::render::build_tile_mesh()`. Wiring the new
  path into gameplay means mapping each sprite's style/label to a model key and
  emitting instances from the same sim state; terrain would become a heightfield
  with per-biome materials. The 2D pipeline stays for particles, weather
  overlays, HUD-adjacent effects and the `#blit` fallback.

**Vertical slice (shipped in `web/` only):** `App::init_3d_scene()` fetches the
GLBs above, uploads them through `model::upload_geometry()`, and installs a
shared `pipeline3d::Scene3D` on the *game* device. Each `update()` builds:
(1) one instanced player GLB at the sim player's position/facing/terrain height,
(2) a generated `terrain3d` heightfield with per-biome vertex colours and
height-derived side skirts (one identity instance), and (3) GLB props mapped
from the same `SpriteStyle` stream that fed `build_tile_mesh` (tree, rock,
house, bush, campfire, crystal, banner). `App::render()` records the 3D pass
into whichever colour attachment it presents (surface or offscreen blit target)
with a depth buffer; the 2D quad pass, particles/weather overlays and frame
dump are untouched and stay behind the `?2d` / `set_render3d(false)` toggle.
`game/src/*` remains unchanged. Verification:
`cargo check --target wasm32-unknown-unknown -p wasm_game`, `./build.sh`, and
`node tests/smoke_3d.mjs` (headless Playwright; install `playwright-core`
locally and set `PLAYWRIGHT_MODULE`/`CHROME_BIN` if it is not on the module
path — 0 console errors). The standalone
`#demo3d` preview and its `load_demo_models()` / `render_3d_demo()` exports are
also preserved — they now share the same `Scene3D` core.

## 6. Why not switch engines?

The rendering shortcomings are real, but they are isolated. Everything expensive
to rebuild — procedural world/chunks, survival stats, combat math, enemy AI,
quests, interiors, building, the day/night and weather schedule, co-op protocol,
saves and the native server — already lives in the engine-agnostic `game` crate
and is covered by headless tests. Swapping engines would mean porting all of it
before drawing a single pixel. By contrast, replacing the 2D quad pipeline with
a glTF/wgpu pipeline is a local change: keep `App`'s sim state and exports,
replace the mesh builder and add one render module. That is why the 3D work is
a renderer swap inside `web/`, not an engine migration.
