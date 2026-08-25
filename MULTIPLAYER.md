# Starfall — Multiplayer Plan & Server Scaffold

Status: **implemented (co-op, server-authoritative)**. The full `game`
simulation now runs authoritatively on the server and M0/M1/M3/M5 are done:
rooms, shareable join codes, snapshot interpolation, and per-token save
persistence are live. This document captures the architecture and protocol that
shipped.

---

## 1. Current architecture (what we have)

- `game/` — pure Rust **systems library** (no wasm deps, has `serde`). Exposes
  data types and per-system helpers: `Player`, `Enemy`, `WorldGen`, `ChunkCache`,
  `tile_at`, `Inventory`, `QuestLog`, `Structure`, combat, day/night, etc.
- `web/` — the **WASM + WebGPU client**. It owns *all game state* and runs the
  simulation loop (`renderer.rs` orchestrates movement, AI, combat, quests…).
  There is **no central `World::step`** to reuse as-is.
- `pkg/` — built client (gitignored).

**Implication:** multiplayer cannot simply "reuse `game::World::step`" because
that function does not exist yet. We must extract the authoritative simulation
into `game` first.

---

## 2. Target architecture

```
            ┌──────────────────────────────────────────┐
            │  game crate (becomes the shared sim)      │
            │  pub struct Simulation { ... }            │
            │  fn step(&mut self, inputs, dt) -> Events │
            │  fn snapshot(&self, viewer) -> Snapshot   │
            └───────────────▲───────────────▲──────────┘
                            │                │  (same Rust)
            ┌───────────────┴──┐     ┌───────┴───────────┐
            │ server/ (native) │     │ web/ (wasm client)│
            │ authoritative    │     │ thin: send input, │
            │ sim + netcode    │     │ render snapshots, │
            │ WebSocket/WebRTC │     │ predict local      │
            └──────────────────┘     └────────────────────┘
```

### Phases
- **M0 — Extract `game::Simulation`** (the big one). Move the orchestration that
  today lives in `renderer.rs` into `game` as `Simulation` owning `players`,
  `enemies`, `world`, `resources`, `day_night`, `inventory`, `quests`,
  `structures`. Expose `step(&mut self, &Inputs, dt)` and `snapshot(...)`. The
  client's `renderer.rs` then *holds a `Simulation` and renders it* instead of
  owning state — unifying single-player and multiplayer code paths.
- **M1 — Server crate (this scaffold)**. Native binary, fixed-tick loop,
  connection registry, broadcast. Currently uses a `StubSimulation`
  (movement only) until M0 lands, then swaps to `game::Simulation`.
- **M2 — Wire protocol**. `ClientMsg` (Join, Input, Leave) / `ServerMsg`
  (Welcome, Snapshot, Disconnect). JSON now (debuggable); **bincode/flatbuffers**
  later for bandwidth.
- **M3 — Client netcode**. Capture input → send `ClientMsg::Input`. On
  `Snapshot`: **interpolate** remote players, **predict + reconcile** the local
  player (apply input locally, correct against server on each snapshot).
- **M4 — Interest management & deltas**. Send each client only entities near
  them; send deltas, not full snapshots. (Critical for bandwidth — see §6.)
- **M5 — Lobby / rooms / persistence**. Join codes, multiple worlds, save sync.

### Scope decision (recommended)
Start **co-op only**: the server is authoritative for the shared world
(enemies, loot, damage, spawning, day/night). PvP (player damage, friendly
fire flags) is an additive layer later.

---

## 3. Wire protocol (scaffold version)

All messages are `serde` structs. Encode as JSON text frames for now.

```rust
struct Input { move_x: f32, move_y: f32, dodge: bool, attack: bool }

enum ClientMsg {
    Join { name: String, token: Option<String>, room: String },
    Input(Input),
    Leave,
}

struct PlayerState { id, name, x, y, hp, facing:(f32,f32), alive }
struct Snapshot   { tick: u32, players: Vec<PlayerState> }

enum ServerMsg {
    Welcome { player_id: u32, tick_rate: u32, seed: u32 },
    Snapshot(Snapshot),
    Disconnect { reason: String },
}
```

Tick rate: **30 Hz** server step. Client renders at display rate and
interpolates between snapshots.

---

## 4. Server design (scaffold)

`server/` (native Rust binary):

- `Simulation` **trait** (`sim.rs`) decouples the network layer from the sim:
  `add_player`, `remove_player`, `set_input`, `step(dt)`, `snapshot()`.
- `GameSim` implements it today with a *minimal authoritative sim*: spawns
  players on walkable tiles (via `game::world::tile_at`), moves them by input
  (clamped to `TileKind::walkable`), and broadcasts `PlayerState`. Enemies /
  combat / quests are **TODO** and will come from `game::Simulation` after M0.
- `main.rs`: `tokio` runtime; `TcpListener` on `0.0.0.0:8081`; `accept_async`
  (WebSocket). Per connection: parse `ClientMsg`, register player, push
  `ServerMsg` to a broadcast registry. A background task ticks the sim at 30 Hz
  and fans out `Snapshot` to all clients.
- Shared state: `Arc<Mutex<Simulation>>` + `Arc<Mutex<HashMap<u32, Sender>>>`.

Build/run (native, separate from the wasm build):
```
cd server && cargo run --release
```
Client (later): `WebSocket` to `ws://<host>:8081`.

---

## 5. Client integration (M3, not in scaffold yet)

- Keep the existing `web` render pipeline. Add a `Network` module:
  - On `Join`, receive `Welcome` → store `player_id`.
  - Each frame: gather local input, send `ClientMsg::Input`.
  - Local player: **predict** (apply input immediately for responsiveness).
  - On `Snapshot`: store latest remote states; **interpolate** remote players
    over the snapshot interval; **reconcile** local player (server pos wins,
    replay unacked inputs).
- When not connected, `renderer.rs` runs the sim locally as today
  (single-player unchanged).

---

## 6. Performance analysis

### Server (authoritative, no GPU)
- CPU: Rust sim is cheap. Cost ≈ `players × (nearby entities)` per tick at 30 Hz.
  A few hundred entities is trivial; thousands need spatial partitioning
  (already tile/chunk based — reuse `WorldGen` chunks for broadphase).
- Bandwidth (the real limiter): a full snapshot of N players is tiny
  (~30–60 bytes/player). Enemies/loot are the bulk → **M4 (deltas + interest
  management)** keeps this bounded: each client receives only entities within
  its view radius. With interest management, a 4-player server uses well under
  100 KB/s total.
- Mitigations: fixed 30 Hz sim (decoupled from render), snapshot deltas,
  per-client view culling, optional snapshot compression.

### Client (WASM + WebGPU)
- **Render cost is unchanged** — same draw calls, just driven by networked
  positions instead of local sim.
- Netcode adds: deserialize snapshots (~µs), interpolate a few floats
  (negligible), and (local player) replay inputs. **FPS impact: negligible**
  if snapshots are deltas and we don't allocate per frame (reuse buffers).
- Latency: with client-side prediction, local movement feels instant; only
  remote players lag by one RTT (smoothly interpolated). Worst case (bad
  prediction) is a small position correction — acceptable for co-op.
- Risk to avoid: **sending the full world every frame** (bandwidth + GC
  churn). The scaffold already only sends player states; M4 extends this
  correctly.

### Net: enabling multiplayer should **not** reduce single-player FPS. The
client render path is identical; networking is a small additive layer.

---

## 7. Risks / open questions
- **M0 extraction** is the largest effort and touches live code; do it behind
  the `Simulation` trait so the client keeps working throughout.
- **Authoritative anti-cheat**: server must validate movement speed, damage,
  loot — never trust the client. (Enforced in `game::Simulation::step`.)
- **Transport**: WebSocket (chosen for scaffold, simple, ~works everywhere).
  WebRTC data channels are lower-latency but need a signaling server — defer.
- **Determinism**: not required (server is source of truth); clients need only
  interpolate, not re-simulate, except local-player prediction.
- **Game elements still in flux** (enemies, quests, structures, balance): the
  `Simulation` trait means the server stays decoupled until those systems are
  final, then we swap the stub for `game::Simulation` in one place.

---

## 8. Milestones summary
| M | Work | Depends on |
|---|------|-----------|
| M0 | Extract `game::Simulation` (orchestration from `renderer.rs`) | ✅ done |
| M1 | Server crate + netcode (real sim swapped in) | ✅ done |
| M2 | Protocol hardening (bincode, deltas) | M1 |
| M3 | Client prediction + interpolation | ✅ done (interpolation + local predict) |
| M4 | Interest management / view culling | M2 |
| M5 | Lobby, rooms, persistence | ✅ done (room codes + token saves) |
