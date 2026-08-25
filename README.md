# Starfall

A 2.5D isometric survival RPG that runs entirely in the browser via **WebGPU +
WebAssembly**, with optional **server-authoritative co-op multiplayer**. Built in
pure Rust (`wasm-pack` + `wgpu`), no JS framework.

- Single-player runs fully client-side (the `game` crate owns the simulation).
- Co-op runs the *same* simulation on a native server that is authoritative for
  the shared world; the client predicts locally and interpolates remote players.

## Features

- Isometric world with procedural chunks, biomes, day/night cycle.
- Survival loop: gather, harvest, cook, eat, craft, build structures.
- Combat: melee, ranged (bows), enemies with AI and a boss that **enrages** in
  its second phase.
- Quests, inventory, reforging, stat upgrades.
- Mouse + keyboard, gamepad, and on-screen touch joystick (mobile-friendly).
- **Co-op multiplayer** with shareable room codes and cross-device saves.
- Save to **IndexedDB** and export/import a `.zip` save file.

## Tech stack

| Crate    | Role                                                        |
|----------|-------------------------------------------------------------|
| `game`   | Pure Rust simulation library (players, enemies, world, quests, structures, combat). No wasm deps. |
| `web`    | WASM + WebGPU client (`wgpu`/`wasm-bindgen`). Renders the sim and speaks to the server. |
| `server` | Native authoritative server (`tokio` + WebSocket). Runs the real `game::Simulation`. |

## Build

Requirements: Rust (stable + `wasm32-unknown-unknown` target), `wasm-pack`,
`cmake` (for native deps), a WebGPU-capable browser (Chrome/Edge/Firefox).

```bash
# 1. Install the wasm target (one time)
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# 2. Build the client into ./pkg
./build.sh
# -> serves locally with:  python3 -m http.server 8000 -d pkg
#    open http://localhost:8000

# 3. Build & run the multiplayer server (separate terminal)
cargo run -p starfall-server
# listens on 0.0.0.0:8081  (override: BIND=0.0.0.0:9000 SEED=123 cargo run -p starfall-server)
```

`build.sh` runs `wasm-pack build web --target web --out-dir ../pkg --release`
then copies `web/static/*` into `pkg/`.

## Play (single-player)

Open the built site (`pkg/`) and use the main menu:

- **New Game** — start a fresh world (optionally enter a seed).
- **Continue** — resume the autosaved IndexedDB slot.
- **Options** — video quality, render cap, adaptive resolution, force-GPU.
- **Help** — controls and goals.

Controls: `WASD`/arrows move, mouse aims, left-click attacks, `Space` dodges,
`F` harvests, `E` eats, `1-9` quick actions, `Q` cycles build mode, click to
place, `Esc` menu, `M` map. Gamepad and touch joystick are auto-detected.

## Play (co-op)

1. One player opens **Multiplayer → Create**. The game reloads into a private
   room and shows a **room code** in the HUD (click it to copy / share).
2. The other player opens **Multiplayer → Join**, pastes the code, and connects
   to the same server.
3. Both share one authoritative world: enemies, loot, damage, day/night, and
   structures are synced. Each player keeps their own inventory/quests locally.

The URL form `?mp=ws://host:8081&room=CODE&name=Alias` auto-connects (used by
the menu). The server is authoritative and persists each player's progress by a
stable `token`, so reconnecting to a room restores your character.

> Deploy note: for play over the internet, put the WebSocket server behind a
> `wss://` reverse proxy. See `DEPLOY.md`.

## Save files

- Autosaves go to **IndexedDB** (no `localStorage` quota headaches).
- Use **Download** to export a `starfall-save.zip` (contains `save.json`), and
  **Upload** to import it on any device.

## Project layout

```
game/        simulation library (engine-agnostic, serde-tagged messages)
web/         wasm + WebGPU client, network layer, zip save I/O
web/static/  HTML shell, menus, HUD, controls
server/      native authoritative co-op server
build.sh     wasm build + static copy
DEPLOY.md    self-hosting / reverse-proxy guide
MULTIPLAYER.md  architecture & protocol design notes
```

## License

See repository for license terms.
