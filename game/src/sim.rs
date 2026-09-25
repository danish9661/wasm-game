use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::building::{Structure, StructureKind, lit_within, try_build, LIGHT_SAFE_RADIUS};
use crate::combat::Arrow;
use crate::daynight::{daylight, temperature, DAY_LENGTH};
use crate::weapons::WeaponKind;
use crate::enemy::{AiState, Enemy, EnemyKind, EnemyRegistry, spawner_on};
use crate::items::{Inventory, ItemKind};
use crate::player::{self, Player};
use crate::poi::{ruins_at, village_sites};
use crate::quest::QuestLog;
use crate::resources::{resource_on, NodeRegistry, ResourceKind};
use crate::world::{edited_tile, tile_at, ChunkCache, TileEdits, WorldGen, TileKind};

const MOVE_SPEED: f32 = player::PLAYER_SPEED;
const CONTACT_RANGE: f32 = 1.3;
const SPIKE_DPS: f32 = 14.0;
const RESPAWN_SECS: f32 = 15.0;
const HARVEST_RANGE: f32 = 1.7;
const BUILD_RANGE: f32 = 4.0;
const SPAWN_RADIUS: i32 = 14;
const HEAL_RADIUS: f32 = 4.0;
const HEAL_RATE: f32 = 8.0;
const TURRET_RANGE: f32 = 9.0;
const TURRET_CD: f32 = 1.1;

/// Wire protocol version. Bumped 1 -> 2 for binary frames + deltas, 2 -> 3
/// for the `ng_cycle` campaign field, 3 -> 4 for the 9-weapon `u16` unlock
/// bitmask, 4 -> 5 for shared interiors (`room` input, enter/attack
/// messages, `rooms` snapshots). JSON text frames remain accepted so old
/// clients keep working; new clients send/receive bincode `Binary` frames.
/// v4 `Input` still decodes (stale tabs keep moving); v4 clients simply
/// never see rooms.
pub const PROTOCOL_VERSION: u32 = 5;
/// Ticks between authoritative full snapshots. Ticks in between carry
/// `SimDelta` (dynamic entities + optionally changed statics), so the ~200
/// static village/town structures are not re-sent 30x/sec.
pub const FULL_SNAPSHOT_INTERVAL: u32 = 30;
/// Interest-management view radius (tiles) for per-client culling. Players are
/// always all sent (tiny); enemies/arrows/structures/resources outside this
/// radius from the viewer are omitted.
pub const VIEW_RADIUS: f32 = 36.0;

fn default_protocol() -> u32 {
    1
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMsg {
    Join {
        name: String,
        token: Option<String>,
        /// Co-op room code. Clients that join the same code share one
        /// authoritative world. The server creates the room on first join.
        room: String,
    },
    Input(PlayerInput),
    Leave,
    /// Step into a building interior (the server materializes the shared
    /// room, spawns its foes and pays the pantry once). v5+.
    EnterInterior { tx: i32, ty: i32 },
    /// Leave the current interior. v5+.
    ExitInterior,
    /// A melee swing inside a room: building tile + room-space swing origin
    /// and facing. Weapon/enchant come from the synced `PlayerInput`; `mult`
    /// carries the heavy-finisher multiplier. v5+.
    RoomAttack {
        spot: RoomSpot,
        fx: f32,
        fy: f32,
        mult: f32,
    },
    /// Sound the Challenge Horn: the next unrecovered fragment guardian
    /// hunts the caller down (or a brutal elite when none remain). v5+.
    SoundHorn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMsg {
    Welcome {
        player_id: u32,
        tick_rate: u32,
        seed: u32,
        /// Wire protocol version (see `PROTOCOL_VERSION`). Defaults to 1 when
        /// talking to an old server that omits it, so legacy JSON still parses.
        #[serde(default = "default_protocol")]
        protocol: u32,
    },
    Snapshot(SimSnapshot),
    /// Incremental update against tick `base_tick`. The client merges it onto
    /// its last full snapshot (see `SimSnapshot::apply_delta`). Static
    /// `structures`/`resources` are `None` when unchanged since the base.
    Delta(SimDelta),
    Disconnect { reason: String },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct PlayerInput {
    pub move_x: f32,
    pub move_y: f32,
    pub dodge: bool,
    pub attack: bool,
    pub harvest: bool,
    pub eat: bool,
    pub shoot: bool,
    pub build: Option<(StructureKind, i32, i32)>,
    /// Equipped weapon index (WeaponKind::as_u8) so the authoritative sim uses
    /// the player's real damage/reach instead of flat constants.
    pub weapon: u8,
    /// Bitmask of owned weapons, so the sim can validate/permit weapon use.
    /// `u16`: the roster holds 9 weapons (v4 protocol; v3 peers send 8 bits
    /// and validate the low byte — Mace stays locked for them).
    pub weapon_unlocked: u16,
    /// Enchantment level of the equipped weapon (raises its damage). Sent by the
    /// client; the server trusts it (co-op is collaborative, not competitive).
    pub enchant: u8,
    /// Craft request: an `ItemKind::as_u8` value to craft at a nearby Anvil this
    /// tick, or `None`. Consumed (taken) by the sim so it fires once per press.
    pub craft: Option<u8>,
    /// Room-space position while inside a building interior (v5+; `None`
    /// outdoors and for v4 peers). Lets the server simulate shared room
    /// foes around the occupants.
    pub room: Option<RoomSpot>,
}

/// Where a co-op player stands inside a building, in room coordinates
/// (relative to the room center, like the client's `Interior::px/py`).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct RoomSpot {
    /// Building tile (matches a world `Structure`).
    pub bx: i32,
    pub by: i32,
    /// Room floor (dungeons stack two).
    pub floor: u8,
    /// Player position within the room.
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub hunger: f32,
    pub stamina: f32,
    pub facing: (f32, f32),
    pub alive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemySnapshot {
    pub x: f32,
    pub y: f32,
    pub kind: EnemyKind,
    pub hp: f32,
    pub facing: (f32, f32),
    pub state: AiState,
    pub windup: f32,
    pub flash: f32,
}

/// Something worth a toast/XP happened inside a shared room. Sequenced per
/// room (`seq`); clients apply each `seq` once and ignore replays from full
/// snapshots. XP and drops resolve client-side from the kind (see
/// `EnemyKind::xp` and the vault/pantry tables), so events stay tiny.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RoomEvent {
    /// A foe fell: the killer earns its XP + drops.
    Kill {
        seq: u32,
        killer: u32,
        kind: EnemyKind,
        x: f32,
        y: f32,
    },
    /// Pantry/vault claimed: the claimer grants the loot locally.
    Claim {
        seq: u32,
        claimer: u32,
        item: ItemKind,
        n: u32,
        xp: u32,
    },
}

/// Authoritative state of one entered building for one tick.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RoomSnapshot {
    pub bx: i32,
    pub by: i32,
    pub floor: u8,
    pub loot_taken: bool,
    pub foes: Vec<EnemySnapshot>,
    /// Recent kill/claim events (last few, sequenced).
    pub events: Vec<RoomEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructureSnapshot {
    pub tx: i32,
    pub ty: i32,
    pub kind: StructureKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub tx: i32,
    pub ty: i32,
    pub kind: ResourceKind,
    pub depleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArrowSnapshot {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub from_player: bool,
    /// Crossbow bolts render thicker client-side. Defaults false for v2 peers.
    #[serde(default)]
    pub piercing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimSnapshot {
    pub tick: u32,
    pub time_of_day: f32,
    pub weather: u8,
    pub players: Vec<PlayerSnapshot>,
    pub enemies: Vec<EnemySnapshot>,
    pub structures: Vec<StructureSnapshot>,
    pub resources: Vec<ResourceSnapshot>,
    pub arrows: Vec<ArrowSnapshot>,
    /// Authoritative campaign progress, broadcast so every co-op client can
    /// render the same quest objective / crafting milestone in its HUD.
    pub quest_stage: u8,
    pub iron_crafted: bool,
    /// Room NG+ cycle (see `Simulation::ng_cycle`). Defaults to 0 when
    /// talking to a v2 peer that omits it.
    #[serde(default)]
    pub ng_cycle: u32,
    /// Shared building interiors the viewer stands in (usually 0-1 rooms):
    /// authoritative foe roster, loot flags and recent kill/claim events.
    /// v5+; older snapshots simply omit it.
    #[serde(default)]
    pub rooms: Vec<RoomSnapshot>,
}

/// Incremental world update. Carries everything dynamic every tick
/// (players, enemies, arrows, clock, quest) but only carries the static
/// `structures`/`resources` when they changed since `base_tick` — villages
/// don't move, so most ticks omit ~200 static entries entirely.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimDelta {
    pub base_tick: u32,
    pub tick: u32,
    pub time_of_day: f32,
    pub weather: u8,
    pub players: Vec<PlayerSnapshot>,
    pub enemies: Vec<EnemySnapshot>,
    pub arrows: Vec<ArrowSnapshot>,
    pub structures: Option<Vec<StructureSnapshot>>,
    pub resources: Option<Vec<ResourceSnapshot>>,
    pub quest_stage: u8,
    pub iron_crafted: bool,
    pub ng_cycle: u32,
    /// Shared interiors (full send every tick; tiny — only entered rooms
    /// exist server-side).
    pub rooms: Vec<RoomSnapshot>,
}

impl SimSnapshot {
    /// Cheap change fingerprint for the static layers (structure count +
    /// coordinates hash). The server compares it against what it last sent a
    /// client to decide whether a delta can omit statics.
    pub fn statics_hash(&self) -> u64 {
        let mut h: u64 = self.structures.len() as u64 * 0x9E3779B1;
        for s in &self.structures {
            h = h.wrapping_add((s.tx as u64).wrapping_mul(73856093) ^ (s.ty as u64).wrapping_mul(19349663) ^ (s.kind as u64));
        }
        h = h.wrapping_add(self.resources.len() as u64 * 0x85EBCA6B);
        for r in &self.resources {
            h = h.wrapping_add((r.tx as u64).wrapping_mul(83492791) ^ (r.ty as u64).wrapping_mul(2971215073));
        }
        h
    }

    /// Build the delta against a previously-sent snapshot. `base` is `None`
    /// for a brand-new client (statics always included).
    pub fn delta_from(&self, base: Option<&SimSnapshot>) -> SimDelta {
        let include_statics = match base {
            None => true,
            Some(b) => b.statics_hash() != self.statics_hash(),
        };
        SimDelta {
            base_tick: base.map_or(self.tick, |b| b.tick),
            tick: self.tick,
            time_of_day: self.time_of_day,
            weather: self.weather,
            players: self.players.clone(),
            enemies: self.enemies.clone(),
            arrows: self.arrows.clone(),
            structures: if include_statics { Some(self.structures.clone()) } else { None },
            resources: if include_statics { Some(self.resources.clone()) } else { None },
            quest_stage: self.quest_stage,
            iron_crafted: self.iron_crafted,
            ng_cycle: self.ng_cycle,
            rooms: self.rooms.clone(),
        }
    }

    /// Merge a delta onto this snapshot (the client keeps one base snapshot
    /// and rolls it forward). Stale deltas (older than the base) are ignored.
    pub fn apply_delta(&mut self, d: SimDelta) {
        if d.tick < self.tick {
            return;
        }
        self.tick = d.tick;
        self.time_of_day = d.time_of_day;
        self.weather = d.weather;
        self.players = d.players;
        self.enemies = d.enemies;
        self.arrows = d.arrows;
        if let Some(s) = d.structures {
            self.structures = s;
        }
        if let Some(r) = d.resources {
            self.resources = r;
        }
        self.quest_stage = d.quest_stage;
        self.iron_crafted = d.iron_crafted;
        self.ng_cycle = d.ng_cycle;
        self.rooms = d.rooms;
    }
}

/// Bincode wire codec (M2). Binary `Binary` WebSocket frames; JSON text frames
/// remain supported for backward compatibility.
pub fn encode_client(msg: &ClientMsg) -> Vec<u8> {
    bincode::serialize(msg).unwrap_or_default()
}

/// v4 `PlayerInput` (no trailing `room` field): stale tabs keep moving
/// after the v5 upgrade instead of being dropped.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
struct PlayerInputV4 {
    move_x: f32,
    move_y: f32,
    dodge: bool,
    attack: bool,
    harvest: bool,
    eat: bool,
    shoot: bool,
    build: Option<(StructureKind, i32, i32)>,
    weapon: u8,
    weapon_unlocked: u16,
    enchant: u8,
    craft: Option<u8>,
}

/// v4 wire messages (no interior variants): byte-compatible prefix of v5,
/// so a v4 `Input` still decodes after the upgrade.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum ClientMsgV4 {
    Join {
        name: String,
        token: Option<String>,
        room: String,
    },
    Input(PlayerInputV4),
    Leave,
}

impl ClientMsgV4 {
    fn upgrade(self) -> ClientMsg {
        match self {
            ClientMsgV4::Join { name, token, room } => ClientMsg::Join { name, token, room },
            ClientMsgV4::Input(i) => ClientMsg::Input(PlayerInput {
                move_x: i.move_x,
                move_y: i.move_y,
                dodge: i.dodge,
                attack: i.attack,
                harvest: i.harvest,
                eat: i.eat,
                shoot: i.shoot,
                build: i.build,
                weapon: i.weapon,
                weapon_unlocked: i.weapon_unlocked,
                enchant: i.enchant,
                craft: i.craft,
                room: None,
            }),
            ClientMsgV4::Leave => ClientMsg::Leave,
        }
    }
}

pub fn decode_client_bin(bytes: &[u8]) -> Option<ClientMsg> {
    if let Ok(m) = bincode::deserialize::<ClientMsg>(bytes) {
        return Some(m);
    }
    // Stale v4 tab: same enum prefix, shorter `Input` payload.
    bincode::deserialize::<ClientMsgV4>(bytes).ok().map(|m| m.upgrade())
}

pub fn encode_server(msg: &ServerMsg) -> Vec<u8> {
    bincode::serialize(msg).unwrap_or_default()
}

pub fn decode_server_bin(bytes: &[u8]) -> Option<ServerMsg> {
    bincode::deserialize(bytes).ok()
}

/// Persisted player state for cross-device saves (keyed by an account
/// `token` on the server). Everything here is serde so it can round-trip to
/// JSON on disk.
#[derive(Serialize, Deserialize, Default)]
pub struct SaveData {
    pub inv: Inventory,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub hunger: f32,
    pub stamina: f32,
    /// Persisted town layout (tile + kind). Captured when first generated so the
    /// city is rebuilt identically instead of re-rolled on every load.
    pub town: Option<Vec<(i32, i32, StructureKind)>>,
    /// Whether the player has already visited the town (so its creation animation
    /// only plays once).
    pub town_visited: bool,
}

struct NetPlayer {
    id: u32,
    name: String,
    token: Option<String>,
    player: Player,
    inv: Inventory,
    input: PlayerInput,
    attack_cd: f32,
    eat_cd: f32,
    harvest_cd: f32,
    dodge_cd: f32,
    shoot_cd: f32,
    respawn_timer: f32,
    /// Previous tick's attack held-state (rising-edge swings).
    prev_attack: bool,
    /// Seconds J/K has been held (heavy finisher past 0.35s on release).
    charge_t: f32,
}

/// One shared building interior on the server: the authoritative foe
/// roster and loot flags for a (building tile, floor) room. Positions stay
/// in room coordinates; occupants are tracked in `Simulation::inside`.
struct ServerRoom {
    kind: StructureKind,
    floor: u8,
    foes: Vec<Enemy>,
    loot_taken: bool,
    /// Monotonic event counter (see `RoomEvent::seq`).
    seq: u32,
    /// Recent kill/claim events (capped; clients dedupe by `seq`).
    events: std::collections::VecDeque<RoomEvent>,
}

impl ServerRoom {
    fn push_event(&mut self, ev: RoomEvent) {
        self.events.push_back(ev);
        while self.events.len() > 8 {
            self.events.pop_front();
        }
    }
}

pub struct Simulation {
    world: WorldGen,
    cache: ChunkCache,
    nodes: NodeRegistry,
    enemies: EnemyRegistry,
    structures: Vec<Structure>,
    /// Player-worked tiles (dig/place). Spawning, blocking and harvesting
    /// read the edited tile; temperature/biome keep the base tile.
    edits: TileEdits,
    /// Shared building interiors, keyed by (building tile, floor). Created
    /// on first entry; loot flags persist while the room lives.
    rooms: HashMap<(i32, i32, u8), ServerRoom>,
    /// Which building each player currently stands in (room coordinates,
    /// refreshed every tick from `PlayerInput::room`).
    inside: HashMap<u32, RoomSpot>,
    arrows: Vec<Arrow>,
    players: HashMap<u32, NetPlayer>,
    next_id: u32,
    tick: u32,
    time_of_day: f32,
    weather: u8,
    weather_timer: f32,
    spawn_point: (f32, f32),
    quest: QuestLog,
    turret_cd: HashMap<(i32, i32), f32>,
    seed: u32,
    /// Campaign-progress counters fed into `QuestLog::update` each step.
    slimes_killed: u32,
    iron_crafted: bool,
    chests_opened: u32,
    fragments_recovered: u8,
    altar_used: bool,
    colossus_defeated: bool,
    /// Room NG+ cycle (0 = first run). Advances by one each time the crew
    /// recovers all five Crown Fragments (then the counter resets so the next
    /// cycle can be earned). Scales enemy damage (+25%/cycle) and day length
    /// (-17%/cycle) — the same formulas as single-player `ng_plus`.
    ng_cycle: u32,
    /// Days survived (midnight crossings) for the Shattered Night schedule.
    day_count: u32,
    /// Whether tonight's bonus hunting pack already spawned (reset at dawn).
    shatter_pack: bool,
    /// Seconds until the next roaming elite / night-raider band. The
    /// server runs these because co-op has no client spawning them (and
    /// snapshots would wipe client-side spawns anyway).
    elite_timer: f32,
    raider_timer: f32,
}

impl Simulation {
    /// Enemy damage multiplier for the current NG+ cycle (parity with the
    /// single-player `ng_damage_mult`).
    pub fn ng_damage_mult(&self) -> f32 {
        1.0 + 0.25 * self.ng_cycle as f32
    }

    /// Day length in seconds for the current NG+ cycle (faster nights).
    pub fn ng_day_length(&self) -> f32 {
        DAY_LENGTH / (1.0 + 0.20 * self.ng_cycle as f32)
    }

    pub fn new(seed: u32) -> Self {
        let world = WorldGen::new(seed);
        let mut cache = ChunkCache::new(256);
        // Spawn inside the first village plaza, exactly like the client's
        // `reset_world` (falls back to origin spiral if no village exists).
        let spawn = village_sites(seed, 3, |tx, ty| {
            tile_at(&world, &mut cache, tx, ty).walkable()
        })
        .first()
        .map(|(vx, vy)| (*vx as f32 + 0.5, *vy as f32 + 0.5))
        .unwrap_or_else(|| find_spawn(&world, &mut cache));
        // Shared world: the same POI layout the single-player client builds
        // for this seed (ruins, villages, town, dungeons), so co-op players
        // collide with, shelter in, and loot the same settlements.
        let structures = crate::poi::poi_structures(seed, &world, &mut cache);
        Self {
            world,
            cache,
            nodes: NodeRegistry::new(),
            enemies: EnemyRegistry::new(),
            structures,
            edits: TileEdits::default(),
            rooms: HashMap::new(),
            inside: HashMap::new(),
            arrows: Vec::new(),
            players: HashMap::new(),
            next_id: 1,
            tick: 0,
            time_of_day: crate::daynight::START_TIME,
            weather: 0,
            weather_timer: 30.0,
            spawn_point: spawn,
            quest: QuestLog::new(),
            turret_cd: HashMap::new(),
            seed,
            slimes_killed: 0,
            iron_crafted: false,
            chests_opened: 0,
            fragments_recovered: 0,
            altar_used: false,
            colossus_defeated: false,
            ng_cycle: 0,
            day_count: 0,
            shatter_pack: false,
            elite_timer: 60.0,
            raider_timer: 75.0,
        }
    }

    pub fn add_player(&mut self, name: String, token: Option<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let (x, y) = self.spawn_point;
        self.players.insert(
            id,
            NetPlayer {
                id,
                name: name.clone(),
                token: token.clone(),
                player: Player::new(x, y),
                inv: Inventory::new(),
                input: PlayerInput::default(),
                attack_cd: 0.0,
                eat_cd: 0.0,
                harvest_cd: 0.0,
                dodge_cd: 0.0,
                shoot_cd: 0.0,
                respawn_timer: 0.0,
                prev_attack: false,
                charge_t: 0.0,
            },
        );
        id
    }

    /// Build `SaveData` for a connected player (used when they leave so the
    /// server can persist progress to disk).
    pub fn save_player(&self, id: u32) -> Option<SaveData> {
        let p = self.players.get(&id)?;
        Some(SaveData {
            inv: p.inv.clone(),
            x: p.player.x,
            y: p.player.y,
            hp: p.player.hp,
            hunger: p.player.hunger,
            stamina: p.player.stamina,
            town: None,
            town_visited: false,
        })
    }

    /// Restore a player's persisted state after (re)joining with a token.
    pub fn restore_player(&mut self, id: u32, save: &SaveData) {
        if let Some(p) = self.players.get_mut(&id) {
            p.inv = save.inv.clone();
            p.player.x = save.x;
            p.player.y = save.y;
            p.player.hp = save.hp;
            p.player.hunger = save.hunger;
            p.player.stamina = save.stamina;
        }
    }

    /// Account token for a player (used by the server to pickle save files).
    pub fn token_of(&self, id: u32) -> Option<String> {
        self.players.get(&id).and_then(|p| p.token.clone())
    }

    pub fn remove_player(&mut self, id: u32) {
        self.players.remove(&id);
        self.inside.remove(&id);
    }

    pub fn set_input(&mut self, id: u32, input: PlayerInput) {
        if let Some(p) = self.players.get_mut(&id) {
            p.input = input;
        }
    }

    /// Step into a building interior: materialize the shared room (spawning
    /// its dungeon patrol on first entry) and pay house pantries once per
    /// building. Returns false for non-enterable tiles.
    pub fn enter_interior(&mut self, id: u32, tx: i32, ty: i32) -> bool {
        use crate::building::pantry_loot;
        let kind = match self.structures.iter().find(|s| s.tx == tx && s.ty == ty) {
            Some(s)
                if matches!(
                    s.kind,
                    StructureKind::House
                        | StructureKind::Cabin
                        | StructureKind::Hut
                        | StructureKind::Inn
                        | StructureKind::Barn
                        | StructureKind::Watchtower
                        | StructureKind::Dungeon
                ) =>
            {
                s.kind
            }
            _ => return false,
        };
        self.inside.insert(
            id,
            RoomSpot { bx: tx, by: ty, floor: 1, x: 0.0, y: 0.0 },
        );
        self.materialize_room(tx, ty, 1);
        let is_dungeon = kind == StructureKind::Dungeon;
        let room = match self.rooms.get_mut(&(tx, ty, 1)) {
            Some(r) => r,
            None => return false,
        };
        // Pantries pay on entry, once per building. Dungeons pay at the
        // vault instead (see `step_rooms`).
        if !is_dungeon && !room.loot_taken {
            if let Some((item, n)) = pantry_loot(kind) {
                room.loot_taken = true;
                room.seq += 1;
                let seq = room.seq;
                room.push_event(RoomEvent::Claim { seq, claimer: id, item, n, xp: 0 });
            }
        }
        true
    }

    /// Leave the current interior (also implied when `PlayerInput::room`
    /// goes `None`).
    pub fn exit_interior(&mut self, id: u32) {
        self.inside.remove(&id);
    }

    /// Get-or-spawn the room for a (building, floor): stairs arrivals land
    /// here with no loot events (fresh patrol on first visit, persistence
    /// after — the stair-hop farm stays closed).
    fn materialize_room(&mut self, bx: i32, by: i32, floor: u8) {
        if self.rooms.contains_key(&(bx, by, floor)) {
            return;
        }
        let kind = match self.structures.iter().find(|s| s.tx == bx && s.ty == by) {
            Some(s)
                if matches!(
                    s.kind,
                    StructureKind::House
                        | StructureKind::Cabin
                        | StructureKind::Hut
                        | StructureKind::Inn
                        | StructureKind::Barn
                        | StructureKind::Watchtower
                        | StructureKind::Dungeon
                ) =>
            {
                s.kind
            }
            _ => return,
        };
        let foes = if kind == StructureKind::Dungeon {
            crate::dungeon::dungeon_foes(bx, by, floor)
                .into_iter()
                .map(|(k, x, y)| Enemy::new(x, y, k))
                .collect()
        } else {
            Vec::new()
        };
        self.rooms.insert(
            (bx, by, floor),
            ServerRoom { kind, floor, foes, loot_taken: false, seq: 0, events: VecDeque::new() },
        );
    }

    /// Resolve a melee swing inside a room: authenticated weapon, shared
    /// foe HP, kill events for XP/drops, scythe lifesteal. Stairs floors
    /// materialize on demand (fresh patrol, like a first visit).
    pub fn room_attack(&mut self, id: u32, spot: RoomSpot, fx: f32, fy: f32, mult: f32) {
        // The swing must come from inside the room being hit.
        match self.inside.get(&id) {
            Some(cur) if cur.bx == spot.bx && cur.by == spot.by => {}
            _ => return,
        }
        if let Some(cur) = self.inside.get_mut(&id) {
            cur.x = spot.x;
            cur.y = spot.y;
            cur.floor = spot.floor;
        }
        let (weapon, enchant) = match self.players.get(&id) {
            Some(np) => {
                let w = WeaponKind::from_u8(np.input.weapon);
                if (np.input.weapon_unlocked & (1u16 << (w as u16))) == 0 {
                    return;
                }
                (w, np.input.enchant)
            }
            None => return,
        };
        let key = (spot.bx, spot.by, spot.floor);
        // Stairs floors materialize on demand (see `materialize_room`).
        self.materialize_room(spot.bx, spot.by, spot.floor);
        let (dealt, scythe) = {
            let room = match self.rooms.get_mut(&key) {
                Some(r) => r,
                None => return,
            };
            // Scratch attacker so the shared swing test applies in room space.
            // Heavy finishers reach further and hit harder (client parity).
            let mut dummy = Player::new(spot.x, spot.y);
            dummy.facing = (fx, fy);
            let reach = if mult > 1.0 { weapon.reach() * 1.2 } else { weapon.reach() };
            let hits = crate::combat::swing_hits(&dummy, room.foes.iter_mut(), reach);
            let mut dealt = 0.0f32;
            let mut kills = Vec::new();
            for foe in hits {
                if !foe.alive() {
                    continue;
                }
                let bs = crate::combat::backstab_mult(
                    (spot.x, spot.y),
                    (foe.x, foe.y),
                    foe.facing,
                    weapon,
                );
                let dmg = weapon.damage()
                    * (1.0 + 0.15 * enchant as f32)
                    * foe.kind.weakness_to(weapon)
                    * bs
                    * mult;
                foe.tagged = true;
                foe.take_damage(dmg);
                foe.flash = 1.0;
                dealt += dmg;
                if !foe.alive() {
                    kills.push((foe.kind, foe.x, foe.y));
                }
            }
            room.foes.retain(|f| f.alive());
            for (kind, x, y) in kills {
                room.seq += 1;
                let seq = room.seq;
                room.push_event(RoomEvent::Kill { seq, killer: id, kind, x, y });
            }
            (dealt, weapon == WeaponKind::Scythe)
        };
        // Reaper's due, server-side (client HP is adopted from snapshots).
        if scythe && dealt > 0.0 {
            if let Some(np) = self.players.get_mut(&id) {
                if np.player.alive {
                    let max = np.player.max_hp();
                    np.player.hp = (np.player.hp + dealt * 0.25).min(max);
                }
            }
        }
    }

    fn nearest_player(&self, x: f32, y: f32) -> Option<u32> {
        let mut best: Option<(u32, f32)> = None;
        for p in self.players.values() {
            if !p.player.alive {
                continue;
            }
            let d = (p.player.x - x).hypot(p.player.y - y);
            if best.map_or(true, |(_, bd)| d < bd) {
                best = Some((p.id, d));
            }
        }
        best.map(|(id, _)| id)
    }

    fn give_loot(&mut self, x: f32, y: f32, items: Vec<ItemKind>) {
        if let Some(id) = self.nearest_player(x, y) {
            if let Some(p) = self.players.get_mut(&id) {
                for it in items {
                    p.inv.add(it, 1);
                }
            }
        }
    }

    /// Simulate shared interiors: foe AI around the occupants, spike
    /// hazards, vault claims, death ejection. Mirrors the client's room
    /// rules (`App::update_interior`) so solo and co-op converge.
    fn step_rooms(&mut self, dt: f32) {
        use crate::building::interior_dims;
        use crate::dungeon::{dungeon_hazards, vault_loot};
        // Living occupants by room.
        let mut occ: HashMap<(i32, i32, u8), Vec<(u32, f32, f32)>> = HashMap::new();
        for (id, spot) in &self.inside {
            if self.players.get(id).map_or(false, |p| p.player.alive) {
                occ.entry((spot.bx, spot.by, spot.floor))
                    .or_default()
                    .push((*id, spot.x, spot.y));
            }
        }
        let mut dead: Vec<u32> = Vec::new();
        for (key, members) in &occ {
            let (bx, by, _floor) = *key;
            let room = match self.rooms.get_mut(key) {
                Some(r) => r,
                None => continue,
            };
            let (rw, rh, _) = interior_dims(room.kind);
            // Pace follows the strongest occupant (parity with the client).
            let lvl = members
                .iter()
                .filter_map(|(id, _, _)| self.players.get(id))
                .map(|p| p.player.level)
                .max()
                .unwrap_or(1);
            for foe in room.foes.iter_mut() {
                if !foe.alive() {
                    continue;
                }
                foe.speed_mult = Enemy::speed_scale_for_level(lvl);
                let (oid, tx, ty) = members
                    .iter()
                    .map(|(id, x, y)| (*id, *x, *y))
                    .min_by(|a, b| {
                        let da = (a.1 - foe.x).hypot(a.2 - foe.y);
                        let db = (b.1 - foe.x).hypot(b.2 - foe.y);
                        da.total_cmp(&db)
                    })
                    .unwrap();
                if let Some(dmg) = foe.update((tx, ty), dt, |x, y| {
                    x.abs() > rw as i32 + 1 || y.abs() > rh as i32 + 1
                }) {
                    // Crafted armor is client-side state the sim never sees;
                    // rooms use raw contact damage (documented).
                    if let Some(np) = self.players.get_mut(&oid) {
                        np.player.take_damage(dmg);
                        if !np.player.alive {
                            dead.push(oid);
                        }
                    }
                }
                foe.x = foe.x.clamp(-rw + 0.3, rw - 0.3);
                foe.y = foe.y.clamp(-rh + 0.3, rh - 0.3);
            }
            // Spike hazards sting occupants standing on them (flat drain, no
            // i-frame gate — parity with the client's room rules).
            if room.kind == StructureKind::Dungeon {
                for (hx, hy) in dungeon_hazards(room.kind, room.floor) {
                    for (oid, px, py) in members.iter().map(|(a, b, c)| (*a, *b, *c)) {
                        if px.round() as i32 == hx && py.round() as i32 == hy {
                            if let Some(np) = self.players.get_mut(&oid) {
                                np.player.hp = (np.player.hp - 12.0 * dt).max(0.0);
                                np.player.hurt_timer = 0.3;
                                if np.player.hp <= 0.0 {
                                    np.player.alive = false;
                                    dead.push(oid);
                                }
                            }
                        }
                    }
                }
            }
            // Vault: the north-wall zone of floor 2 pays once per dungeon.
            if room.kind == StructureKind::Dungeon && room.floor == 2 && !room.loot_taken {
                let my2 = rh - 0.5;
                let claimer = members
                    .iter()
                    .find(|(_, px, py)| (py + my2).abs() < 0.5 && px.abs() < 1.5)
                    .map(|(id, _, _)| *id);
                if let Some(oid) = claimer {
                    let (item, n, xp) = vault_loot(bx, by);
                    room.loot_taken = true;
                    room.seq += 1;
                    let seq = room.seq;
                    room.push_event(RoomEvent::Claim { seq, claimer: oid, item, n, xp });
                }
            }
        }
        for id in dead {
            self.inside.remove(&id);
        }
    }

    /// Next unrecovered Crown Fragment guardian (campaign order), if any.
    fn next_guardian(&self) -> Option<EnemyKind> {
        const ORDER: [(u8, EnemyKind); 5] = [
            (0, EnemyKind::Boss),
            (1, EnemyKind::ScorpionQueen),
            (2, EnemyKind::FrostGolem),
            (3, EnemyKind::ToadKing),
            (4, EnemyKind::OceanLeviathan),
        ];
        ORDER
            .iter()
            .find(|(b, _)| self.fragments_recovered & (1 << b) == 0)
            .map(|(_, k)| *k)
    }

    /// Walkable tile 14-20 tiles from (px, py), deterministic per tick.
    fn wilds_spot(&mut self, px: f32, py: f32, salt: f32) -> Option<(i32, i32)> {
        let ang = (self.tick as f32 * 0.37 + salt + px * 0.7).fract()
            * std::f32::consts::TAU;
        let dist = 14.0 + (self.tick as f32 * 0.53 + salt).fract() * 6.0;
        let tx = (px + ang.cos() * dist).floor() as i32;
        let ty = (py + ang.sin() * dist).floor() as i32;
        let tile = edited_tile(&self.edits, &self.world, &mut self.cache, tx, ty);
        tile.walkable().then_some((tx, ty))
    }

    /// Sound the Challenge Horn for a player: guardian (or brutal elite).
    pub fn sound_horn(&mut self, id: u32) {
        let (px, py) = match self.players.get(&id) {
            Some(p) if p.player.alive => (p.player.x, p.player.y),
            _ => return,
        };
        let (kind, elite) = match self.next_guardian() {
            Some(k) => (k, 1.0),
            None => (EnemyKind::Brute, 3.0),
        };
        if let Some((tx, ty)) = self.wilds_spot(px, py, 1.7) {
            self.enemies
                .spawn_elite(kind, tx as f32 + 0.5, ty as f32 + 0.5, elite);
        }
    }

    /// Roaming elites + night raiders (parity with the client's pressure
    /// systems — snapshots would wipe client-side spawns, so the server
    /// runs these itself; darkness doubles the pace on both sides).
    fn step_events(&mut self, dt: f32) {
        use crate::daynight::{daylight, NIGHTFALL};
        let night = daylight(self.time_of_day) < NIGHTFALL;
        let pace = if night { 2.0 } else { 1.0 };
        self.elite_timer -= dt * pace;
        if self.elite_timer <= 0.0 {
            self.elite_timer = 70.0 + (self.tick as f32 * 0.37).fract() * 70.0;
            let ids: Vec<u32> = self
                .players
                .iter()
                .filter(|(_, p)| p.player.alive)
                .map(|(id, _)| *id)
                .collect();
            if !ids.is_empty() {
                let id = ids[(self.tick as usize) % ids.len()];
                self.sound_horn(id);
            }
        }
        self.raider_timer -= dt * pace;
        if self.raider_timer <= 0.0 {
            self.raider_timer = 55.0 + (self.tick as f32 * 0.53).fract() * 45.0;
            if night && !self.structures.is_empty() {
                let idx = (self.tick as usize) % self.structures.len();
                let (sx, sy) = {
                    let s = &self.structures[idx];
                    (s.tx, s.ty)
                };
                for i in 0..2 {
                    let ang = (self.tick as f32 * 0.71 + i as f32 * 3.1).fract()
                        * std::f32::consts::TAU;
                    let tx = (sx as f32 + ang.cos() * 4.0) as i32;
                    let ty = (sy as f32 + ang.sin() * 4.0) as i32;
                    let tile =
                        edited_tile(&self.edits, &self.world, &mut self.cache, tx, ty);
                    if tile.walkable() {
                        self.enemies.spawn_elite(
                            EnemyKind::Raider,
                            tx as f32 + 0.5,
                            ty as f32 + 0.5,
                            1.0,
                        );
                    }
                }
            }
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.tick += 1;
        let nt = (self.time_of_day + dt / self.ng_day_length()).rem_euclid(1.0);
        if nt < self.time_of_day {
            // Midnight crossing: new day, tonight's pack slate resets.
            self.day_count += 1;
            self.shatter_pack = false;
        }
        self.time_of_day = nt;
        // Shattered Nights: every 4th night a bonus hunting pack stalks a
        // random player till dawn (mirrors the client's schedule).
        if !self.shatter_pack
            && crate::daynight::shattered_night(self.day_count, self.seed)
            && crate::daynight::daylight(self.time_of_day) < crate::daynight::NIGHTFALL
        {
            if let Some(id) = self.players.keys().next().copied() {
                let (px, py) = {
                    let p = &self.players[&id];
                    (p.player.x, p.player.y)
                };
                let mut spawned = false;
                for (i, kind) in [EnemyKind::Raider, EnemyKind::Wolf].into_iter().enumerate() {
                    let ang = (self.tick as f32 * 0.37 + i as f32 * 2.4).fract()
                        * std::f32::consts::TAU;
                    let tx = (px + ang.cos() * 16.0).floor() as i32;
                    let ty = (py + ang.sin() * 16.0).floor() as i32;
                    let tile = edited_tile(&self.edits, &self.world, &mut self.cache, tx, ty);
                    if tile.walkable() {
                        // Pace is re-scaled per frame in `step_enemies`, like
                        // every other foe.
                        self.enemies.spawn_elite(kind, tx as f32 + 0.5, ty as f32 + 0.5, 1.5);
                        spawned = true;
                    }
                }
                self.shatter_pack = spawned;
            }
        }
        self.weather_timer -= dt;
        if self.weather_timer <= 0.0 {
            let r = (self.time_of_day * 311.0 + self.tick as f32 * 0.013).fract();
            if self.weather != 0 {
                self.weather = 0;
                self.weather_timer = 25.0 + r * 20.0;
            } else if r < 0.30 {
                self.weather = if r < 0.15 { 1 } else { 2 };
                self.weather_timer = 20.0 + r * 20.0;
            } else {
                self.weather_timer = 25.0 + r * 20.0;
            }
        }

        let temp = temperature(self.time_of_day);
        let wet = self.weather == 1;

        // Room presence follows the clients' per-tick room spots. Occupants
        // keep ticking survival but don't walk, swing, harvest or build on
        // the surface while inside (their room body is simulated below).
        let spots: Vec<(u32, Option<RoomSpot>)> = self
            .players
            .iter()
            .map(|(id, np)| (*id, np.input.room))
            .collect();
        for (id, spot) in spots {
            match spot {
                Some(s) => {
                    self.inside.insert(id, s);
                    // Stairs arrivals land here: the floor room must exist
                    // before `step_rooms` runs (vault zones pay on presence).
                    self.materialize_room(s.bx, s.by, s.floor);
                    if let Some(np) = self.players.get_mut(&id) {
                        np.input.move_x = 0.0;
                        np.input.move_y = 0.0;
                        np.input.attack = false;
                        np.input.harvest = false;
                        np.input.dodge = false;
                        np.input.build = None;
                    }
                }
                None => {
                    self.inside.remove(&id);
                }
            }
        }

        self.spawn_near_players(dt);

        let spawn = self.spawn_point;
        for p in self.players.values_mut() {
            let crafted = Self::step_player(
                p,
                &self.world,
                &mut self.cache,
                &self.edits,
                &mut self.enemies,
                &mut self.structures,
                &mut self.nodes,
                &mut self.arrows,
                spawn,
                dt,
                temp,
                wet,
            );
            if crafted == Some(ItemKind::IronPlate) {
                self.iron_crafted = true;
            }
        }

        self.step_enemies(dt);
        self.step_rooms(dt);
        self.step_events(dt);
        self.step_arrows(dt);
        self.step_structures(dt);
    }

    fn spawn_near_players(&mut self, dt: f32) {
        let mut seen = HashSet::new();
        for p in self.players.values() {
            let cx = p.player.x;
            let cy = p.player.y;
            for dx in -SPAWN_RADIUS..=SPAWN_RADIUS {
                for dy in -SPAWN_RADIUS..=SPAWN_RADIUS {
                    let tx = (cx + dx as f32) as i32;
                    let ty = (cy + dy as f32) as i32;
                    if !seen.insert((tx, ty)) {
                        continue;
                    }
                    let tile = edited_tile(&self.edits, &self.world, &mut self.cache, tx, ty);
                    if let Some(kind) = spawner_on(tx, ty, tile) {
                        // Nocturnal enemies only emerge after dark.
                        if kind.nocturnal() && daylight(self.time_of_day) > 0.5 {
                            continue;
                        }
                        // Lit ground is safe ground: light around a home
                        // suppresses spawns (the Minecraft torch rule).
                        if lit_within(&self.structures, tx, ty, LIGHT_SAFE_RADIUS) {
                            continue;
                        }
                        self.enemies.get(tx, ty, kind, dt);
                    }
                }
            }
        }
    }

fn tile_blocked(
    world: &WorldGen,
    cache: &mut ChunkCache,
    edits: &TileEdits,
    structures: &[Structure],
    nodes: &NodeRegistry,
    tx: i32,
    ty: i32,
) -> bool {
    let tile = edited_tile(edits, world, cache, tx, ty);
    if !tile.walkable() {
        return true;
    }
    if structures
        .iter()
        .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
    {
        return true;
    }
    if let Some(k) = resource_on(tx, ty, tile) {
        if !nodes.is_depleted(tx, ty) && k.blocks_movement() {
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn step_player(
    np: &mut NetPlayer,
    world: &WorldGen,
    cache: &mut ChunkCache,
    edits: &TileEdits,
    enemies: &mut EnemyRegistry,
    structures: &mut Vec<Structure>,
    nodes: &mut NodeRegistry,
    arrows: &mut Vec<Arrow>,
    spawn_point: (f32, f32),
    dt: f32,
    temp: f32,
    wet: bool,
) -> Option<ItemKind> {
    np.attack_cd = (np.attack_cd - dt).max(0.0);
    np.eat_cd = (np.eat_cd - dt).max(0.0);
    np.harvest_cd = (np.harvest_cd - dt).max(0.0);
    np.dodge_cd = (np.dodge_cd - dt).max(0.0);
    np.shoot_cd = (np.shoot_cd - dt).max(0.0);

    // Sync the equipped weapon from the client so the sim uses the player's real
    // damage/reach rather than flat constants. Only trust a weapon the client has
    // actually unlocked (bitmask) — stops a tampered client from wielding gear
    // it shouldn't have in co-op.
    let w = WeaponKind::from_u8(np.input.weapon);
    if (np.input.weapon_unlocked & (1u16 << (w as u16))) != 0 {
        np.player.weapon = w;
        np.player.unlocked = np.input.weapon_unlocked;
    }
    np.player.enchant = np.input.enchant;

    if !np.player.alive {
        np.respawn_timer -= dt;
        if np.respawn_timer <= 0.0 {
            np.player.respawn();
            np.player.x = spawn_point.0;
            np.player.y = spawn_point.1;
        }
        return None;
    }

    let (mx, my) = (np.input.move_x, np.input.move_y);
    let len = (mx * mx + my * my).sqrt();
    if len > 1e-4 {
        let nx = mx / len;
        let ny = my / len;
        let step = MOVE_SPEED * dt;
        let tx = np.player.x + nx * step;
        let ty = np.player.y + ny * step;
        if !Self::tile_blocked(world, cache, edits, structures, nodes, tx as i32, ty as i32) {
            np.player.x = tx;
            np.player.y = ty;
        } else if !Self::tile_blocked(world, cache, edits, structures, nodes, tx as i32, np.player.y as i32) {
            np.player.x = tx;
        } else if !Self::tile_blocked(world, cache, edits, structures, nodes, np.player.x as i32, ty as i32) {
            np.player.y = ty;
        }
        np.player.facing = (nx, ny);
    }

    if np.input.dodge && np.dodge_cd <= 0.0 {
        if np.player.try_dodge(np.player.facing) {
            np.dodge_cd = 0.8;
        }
    }

    // Rising edge -> normal swing (tap). A sustained hold charges instead of
    // machine-gunning: on release past 0.35s a heavy finisher lands.
    // (Client mirrors this exactly; the input shape is unchanged.)
    let atk_now = np.input.attack;
    if atk_now && !np.prev_attack && np.attack_cd <= 0.0 {
        np.attack_cd = 0.4;
        let w = np.player.weapon;
        // Dodge-strike: lunging out of a roll extends the blow.
        let dashing = np.player.dodge_timer > 0.0;
        let reach = w.reach() + if dashing { 2.0 } else { 0.0 };
        let dash_mult = if dashing { 1.2 } else { 1.0 };
        let mut hits = crate::combat::swing_hits(&np.player, enemies.enemies_mut(), reach);
        for e in hits.iter_mut() {
            let bs = crate::combat::backstab_mult(
                (np.player.x, np.player.y),
                (e.x, e.y),
                e.facing,
                w,
            );
            e.tagged = true;
            e.take_damage(np.player.weapon_damage() * e.kind.weakness_to(w) * bs * dash_mult);
            if w == WeaponKind::Hammer {
                e.flash = 1.0;
                e.windup = e.windup.max(0.55);
            }
        }
    }
    if atk_now {
        np.charge_t += dt;
    } else {
        if np.charge_t >= 0.35 && !np.player.weapon.ranged() {
            // Heavy finisher: 1.5x-2.5x by charge, stretched reach, stagger.
            let w = np.player.weapon;
            let mult = 1.5 + (np.charge_t - 0.35).min(1.0);
            let mut hits =
                crate::combat::swing_hits(&np.player, enemies.enemies_mut(), w.reach() * 1.2);
            for e in hits.iter_mut() {
                let bs = crate::combat::backstab_mult(
                    (np.player.x, np.player.y),
                    (e.x, e.y),
                    e.facing,
                    w,
                );
                e.tagged = true;
                e.take_damage(np.player.weapon_damage() * mult * bs);
                e.flash = 1.0;
                e.windup = e.windup.max(0.45);
            }
        }
        np.charge_t = 0.0;
    }
    np.prev_attack = atk_now;

    if np.input.shoot && np.shoot_cd <= 0.0 {
        let w = np.player.weapon;
        let (fx, fy) = np.player.facing;
        if w.ranged() && (fx * fx + fy * fy) > 1e-4 {
            if w.piercing() {
                arrows.push(Arrow::bolt(np.player.x, np.player.y, fx, fy, np.player.weapon_damage()));
            } else {
                let mut a = Arrow::new(np.player.x, np.player.y, fx, fy);
                a.damage = np.player.weapon_damage();
                arrows.push(a);
            }
            np.shoot_cd = 0.5;
        }
    }

    if np.input.harvest && np.harvest_cd <= 0.0 {
        if let Some((tx, ty, kind)) = Self::node_in_range(world, cache, edits, nodes, np.player.x, np.player.y) {
            // Work tools bite deeper here too (parity with the client).
            let power = np.player.chop_power(kind);
            let tool = np.player.work_tool(kind);
            let mut hit = false;
            for _ in 0..power {
                if let Some(item) = nodes.chop(tx, ty, kind) {
                    np.inv.add(item, 1);
                    hit = true;
                } else {
                    break;
                }
            }
            if hit {
                if let Some(t) = tool {
                    np.player.wear_tool(t);
                }
                np.harvest_cd = 0.35;
            }
        }
    }

    if np.input.eat && np.eat_cd <= 0.0 {
        if np.player.eat(&mut np.inv) {
            np.eat_cd = 0.5;
        }
    }

    if let Some((kind, tx, ty)) = np.input.build.take() {
        let d = (np.player.x - (tx as f32 + 0.5)).hypot(np.player.y - (ty as f32 + 0.5));
        if d <= BUILD_RANGE {
            if let Ok(s) = try_build(kind, tx, ty, &mut np.inv) {
                structures.push(s);
            }
        }
    }

    // Crafting: forge an item at a nearby Anvil.
    let mut crafted: Option<ItemKind> = None;
    if let Some(kind_u8) = np.input.craft.take() {
        if let Some(kind) = ItemKind::from_u8(kind_u8) {
            if let Some(recipe) = crate::craft::recipe_for(kind) {
                let near_anvil = structures.iter().any(|s| {
                    s.kind == StructureKind::Anvil
                        && (s.tx as f32 + 0.5 - np.player.x)
                            .hypot(s.ty as f32 + 0.5 - np.player.y)
                            < BUILD_RANGE
                });
                if near_anvil && crate::craft::craft(&mut np.inv, recipe) {
                    crafted = Some(kind);
                }
            }
        }
    }

    let ptx = np.player.x.floor() as i32;
    let pty = np.player.y.floor() as i32;
    let biome = tile_at(world, cache, ptx, pty);
    let warm = structures.iter().any(|s| {
        s.kind.emits_light()
            && (s.tx as f32 + 0.5 - np.player.x).hypot(s.ty as f32 + 0.5 - np.player.y) < 2.5
    });
    np.player.tick(dt, temp, warm, wet, biome, 0);
    if warm && np.player.hp < np.player.max_hp() {
        np.player.hp = (np.player.hp + dt * 3.0).min(np.player.max_hp());
    }
    crafted
}

    fn node_in_range(
        world: &WorldGen,
        cache: &mut ChunkCache,
        edits: &TileEdits,
        nodes: &NodeRegistry,
        x: f32,
        y: f32,
    ) -> Option<(i32, i32, ResourceKind)> {
        let r = HARVEST_RANGE as i32;
        let cx = x.floor() as i32;
        let cy = y.floor() as i32;
        for dx in -r..=r {
            for dy in -r..=r {
                let tx = cx + dx;
                let ty = cy + dy;
                let tile = edited_tile(edits, world, cache, tx, ty);
                if let Some(kind) = resource_on(tx, ty, tile) {
                    if !nodes.is_depleted(tx, ty) {
                        return Some((tx, ty, kind));
                    }
                }
            }
        }
        None
    }

    fn step_enemies(&mut self, dt: f32) {
        let targets: Vec<(f32, f32)> =
            self.players.values().map(|p| (p.player.x, p.player.y)).collect();
        let world = &self.world;
        let cache = &mut self.cache;
        let edits = &self.edits;
        let structs = &self.structures;
        let nodes = &self.nodes;
        let mut blocked = |tx: i32, ty: i32| -> bool {
            let tile = edited_tile(edits, world, cache, tx, ty);
            if !tile.walkable() {
                return true;
            }
            if structs
                .iter()
                .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
            {
                return true;
            }
            if let Some(k) = resource_on(tx, ty, tile) {
                if !nodes.is_depleted(tx, ty) && k.blocks_movement() {
                    return true;
                }
            }
            false
        };

        let mut contacts: Vec<(f32, f32, f32)> = Vec::new();
        // Enemies keep pace with the strongest player's progression so a high-level
        // group isn't trivially outrun.
        let lvl = self.players.values().map(|p| p.player.level).max().unwrap_or(0);
        let enemy_speed = Enemy::speed_scale_for_level(lvl);
        for e in self.enemies.enemies_mut() {
            e.speed_mult = enemy_speed;
            let target = targets
                .iter()
                .min_by(|a, b| {
                    (a.0 - e.x).hypot(a.1 - e.y).total_cmp(&(b.0 - e.x).hypot(b.1 - e.y))
                })
                .copied()
                .unwrap_or((0.0, 0.0));
            if let Some(dmg) = e.update(target, dt, &mut blocked) {
                contacts.push((e.x, e.y, dmg));
            }
            // Nocturnal undead burn in daylight.
            e.daylight_burn(dt, daylight(self.time_of_day));
            if let Some((dx, dy)) = e.pending_shot.take() {
                self.arrows.push(Arrow::enemy(e.x, e.y, dx, dy));
            }
            let etx = e.x.floor() as i32;
            let ety = e.y.floor() as i32;
            if self
                .structures
                .iter()
                .any(|s| s.tx == etx && s.ty == ety && (s.kind == StructureKind::Spike || s.kind == StructureKind::Trap))
            {
                e.take_damage(SPIKE_DPS * dt);
            }
        }

        // Hoisted: `ng_damage_mult` borrows all of self, which would clash
        // with the per-player mutable borrows below.
        let dmg_mult = self.ng_damage_mult();
        for (ex, ey, dmg) in contacts {
            if let Some(id) = self.nearest_player(ex, ey) {
                if let Some(p) = self.players.get_mut(&id) {
                    if (p.player.x - ex).hypot(p.player.y - ey) < CONTACT_RANGE {
                        p.player.take_damage(dmg * dmg_mult);
                        // Knock the struck player back along the enemy→player vector.
                        let dx = p.player.x - ex;
                        let dy = p.player.y - ey;
                        p.player.knockback(dx, dy, 0.45);
                    }
                }
            }
        }

        let mut dead = Vec::new();
        let mut loot: Vec<(f32, f32, Vec<ItemKind>)> = Vec::new();
        let mut xp_grants: Vec<(u32, u32)> = Vec::new();
        // Hoisted: `on_enemy_slain` borrows all of self, which clashes with
        // the registry iteration — collect kinds, process after the loop.
        let mut slain: Vec<EnemyKind> = Vec::new();
        for (k, e) in self.enemies.iter_mut_with_key() {
            if !e.alive() {
                let mut best: Option<(u32, f32)> = None;
                for pl in self.players.values() {
                    if !pl.player.alive {
                        continue;
                    }
                    let d = (pl.player.x - e.x).hypot(pl.player.y - e.y);
                    if best.map_or(true, |(_, bd)| d < bd) {
                        best = Some((pl.id, d));
                    }
                }
                if let Some((pid, _)) = best {
                    eprintln!("DBG sweep: kind={:?} tagged={} pid={pid}", e.kind, e.tagged);
                    // Loot is shared spoils (any death feeds the crew), but XP
                    // and campaign credit require the player's tag — idling
                    // behind guards/turrets can't farm progression.
                    loot.push((e.x, e.y, e.drops()));
                    if e.tagged {
                        xp_grants.push((pid, e.kind.xp()));
                        slain.push(e.kind);
                    }
                }
                dead.push(k);
            }
        }
        for (tx, ty) in dead {
            self.enemies.kill(tx, ty, RESPAWN_SECS);
        }
        for kind in slain {
            self.on_enemy_slain(kind);
        }
        for (x, y, items) in loot {
            self.give_loot(x, y, items);
        }
        for (pid, xp) in xp_grants {
            if let Some(p) = self.players.get_mut(&pid) {
                p.player.add_xp(xp);
            }
        }
        self.maybe_advance_ng();
        self.step_quests();
    }

    /// Shared kill bookkeeping for melee (`step_enemies`) and arrow
    /// (`step_arrows`) deaths: slime counter, Crown Fragment recovery (drives
    /// quest stages 6-8), and the Colossus true-ending flag.
    fn on_enemy_slain(&mut self, kind: EnemyKind) {
        if kind == EnemyKind::Slime {
            self.slimes_killed += 1;
        }
        if let Some(bit) = kind.fragment_bit() {
            self.fragments_recovered |= 1 << bit;
        }
        if kind == EnemyKind::Colossus {
            self.colossus_defeated = true;
        }
    }

    /// Advance the room's NG+ cycle once the crew recovers all five Crown
    /// Fragments, then reset the counter so the next cycle can be earned
    /// (guardians respawn on their tiles). The altar rite itself is
    /// single-player; in co-op the full recovery IS the victory that hardens
    /// the world (+25% enemy damage, faster days per cycle).
    fn maybe_advance_ng(&mut self) {
        if self.fragments_recovered == 0b11111 {
            self.ng_cycle += 1;
            self.fragments_recovered = 0;
        }
    }

    /// Feed real world facts into the story `QuestLog` so the campaign advances
    /// as the player actually gathers, builds, crafts, fights and explores.
    fn step_quests(&mut self) {
        // Facts are taken from the first living player (co-op uses the host's
        // progression for the shared story line).
        let facts = self
            .players
            .values()
            .find(|p| p.player.alive)
            .map(|p| {
                let inv = &p.inv;
                let has_wall = self
                    .structures
                    .iter()
                    .any(|s| s.kind == StructureKind::Wall);
                let has_campfire = self
                    .structures
                    .iter()
                    .any(|s| s.kind == StructureKind::Campfire);
                let has_anvil = self
                    .structures
                    .iter()
                    .any(|s| s.kind == StructureKind::Anvil);
                let ruins = ruins_at(self.seed, |tx, ty| {
                    crate::world::tile_at(&self.world, &mut self.cache, tx, ty).walkable()
                });
                let near_ruins =
                    (ruins.0 as f32 - p.player.x).hypot(ruins.1 as f32 - p.player.y) < 6.0;
                (
                    inv.count(ItemKind::Wood),
                    inv.count(ItemKind::Stone),
                    has_wall,
                    has_campfire,
                    has_anvil,
                    self.iron_crafted,
                    self.slimes_killed,
                    near_ruins,
                    self.chests_opened > 0,
                    self.fragments_recovered,
                    self.altar_used,
                    self.colossus_defeated,
                )
            });
        if let Some((wood, stone, hw, hc, ha, ci, sk, nr, co, fr, au, cd)) = facts {
            self.quest.update(
                wood, stone, hw, hc, ha, ci, sk, nr, co, fr, au, cd,
            );
        }
    }

    fn step_arrows(&mut self, dt: f32) {
        let arrows = std::mem::replace(&mut self.arrows, Vec::new());
        let dmg_mult = self.ng_damage_mult();
        let mut alive = Vec::new();
        for mut a in arrows {
            if !a.step(dt) {
                continue;
            }
            if a.from_player {
                // Piercing bolts wound every foe in their path (once each —
                // struck tiles are remembered); normal arrows stop at the
                // first victim. Corpses are removed at once (no double-loot).
                let mut loot: Vec<(i32, i32, f32, f32, EnemyKind, bool, Vec<ItemKind>)> = Vec::new();
                let mut consumed = false;
                for e in self.enemies.enemies_mut() {
                    if (e.x - a.x).hypot(e.y - a.y) >= 0.8 {
                        continue;
                    }
                    if a.piercing {
                        let et = (e.x.floor() as i32, e.y.floor() as i32);
                        if a.struck.contains(&et) {
                            continue;
                        }
                        a.struck.push(et);
                    }
                    let was_alive = e.alive();
                    e.take_damage(a.damage);
                    if was_alive && !e.alive() {
                        loot.push((e.x.floor() as i32, e.y.floor() as i32, e.x, e.y, e.kind, a.tagged, e.drops()));
                    }
                    if !a.piercing {
                        consumed = true;
                        break;
                    }
                }
                // Arrow kills progress campaign + XP exactly like melee kills —
                // but only player-tagged arrows (turret shots share spoils).
                for (tx, ty, x, y, kind, tagged, items) in loot {
                    self.give_loot(x, y, items);
                    if tagged {
                        self.on_enemy_slain(kind);
                        if let Some(id) = self.nearest_player(x, y) {
                            if let Some(p) = self.players.get_mut(&id) {
                                p.player.add_xp(kind.xp());
                            }
                        }
                    }
                    self.enemies.kill(tx, ty, RESPAWN_SECS);
                }
                if a.piercing || !consumed {
                    alive.push(a);
                }
            } else {
                let mut hit = false;
                for p in self.players.values_mut() {
                    if (p.player.x - a.x).hypot(p.player.y - a.y) < 0.8 {
                        p.player.take_damage(a.damage * dmg_mult);
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    alive.push(a);
                }
            }
        }
        self.arrows = alive;
    }

    fn step_structures(&mut self, dt: f32) {
        for s in self.structures.iter() {
            if s.kind == StructureKind::HealingTotem {
                for p in self.players.values_mut() {
                    if (p.player.x - (s.tx as f32 + 0.5)).hypot(p.player.y - (s.ty as f32 + 0.5))
                        < HEAL_RADIUS
                    {
                        p.player.hp = (p.player.hp + HEAL_RATE * dt).min(player::MAX_HP);
                    }
                }
            }
        }
        for (k, cd) in self.turret_cd.iter_mut() {
            *cd = (*cd - dt).max(0.0);
            let _ = k;
        }
        for s in self.structures.iter() {
            if s.kind == StructureKind::Turret {
                let key = (s.tx, s.ty);
                let cd = self.turret_cd.get(&key).copied().unwrap_or(0.0);
                if cd > 0.0 {
                    continue;
                }
                let (tx, ty) = (s.tx as f32 + 0.5, s.ty as f32 + 0.5);
                if let Some((ex, ey)) = self.nearest_enemy_in(tx, ty, TURRET_RANGE) {
                    let dx = ex - tx;
                    let dy = ey - ty;
                    // Turret shots are untagged: shared spoils, no XP/quest
                    // credit (same rule as guard kills).
                    let mut ta = Arrow::new(tx, ty, dx, dy);
                    ta.tagged = false;
                    self.arrows.push(ta);
                    self.turret_cd.insert(key, TURRET_CD);
                }
            }
        }
    }

    fn nearest_enemy_in(&self, x: f32, y: f32, range: f32) -> Option<(f32, f32)> {
        let mut best: Option<(f32, f32)> = None;
        let mut bd = range;
        for e in self.enemies.enemies() {
            let d = (e.x - x).hypot(e.y - y);
            if d <= bd {
                bd = d;
                best = Some((e.x, e.y));
            }
        }
        best
    }

    pub fn snapshot(&self) -> SimSnapshot {
        let players = self
            .players
            .values()
            .map(|p| PlayerSnapshot {
                id: p.id,
                name: p.name.clone(),
                x: p.player.x,
                y: p.player.y,
                hp: p.player.hp,
                hunger: p.player.hunger,
                stamina: p.player.stamina,
                facing: p.player.facing,
                alive: p.player.alive,
            })
            .collect();

        let enemies = self
            .enemies
            .enemies()
            .map(|e| EnemySnapshot {
                x: e.x,
                y: e.y,
                kind: e.kind,
                hp: e.hp,
                facing: e.facing,
                state: e.state,
                windup: e.windup,
                flash: e.flash,
            })
            .collect();

        let structures = self
            .structures
            .iter()
            .map(|s| StructureSnapshot {
                tx: s.tx,
                ty: s.ty,
                kind: s.kind,
            })
            .collect();

        let resources = self
            .nodes
            .all()
            .into_iter()
            .map(|(tx, ty, kind, depleted)| ResourceSnapshot {
                tx,
                ty,
                kind,
                depleted,
            })
            .collect();

        let arrows = self
            .arrows
            .iter()
            .map(|a| ArrowSnapshot {
                x: a.x,
                y: a.y,
                dx: a.dx,
                dy: a.dy,
                from_player: a.from_player,
                piercing: a.piercing,
            })
            .collect();

        SimSnapshot {
            tick: self.tick,
            time_of_day: self.time_of_day,
            weather: self.weather,
            players,
            enemies,
            structures,
            resources,
            arrows,
            quest_stage: self.quest.stage,
            iron_crafted: self.iron_crafted,
            ng_cycle: self.ng_cycle,
            // `snapshot_for` fills the viewer's rooms after culling.
            rooms: Vec::new(),
        }
    }

    /// World position of a connected player (for interest management).
    pub fn player_pos(&self, id: u32) -> Option<(f32, f32)> {
        self.players.get(&id).map(|p| (p.player.x, p.player.y))
    }

    /// Per-client culled snapshot (M4): the full world state filtered to
    /// `VIEW_RADIUS` tiles around `viewer`. Players, clock, weather and quest
    /// are always included (tiny); enemies, arrows, structures and resources
    /// outside the radius are omitted. Unknown viewers get the full snapshot.
    pub fn snapshot_for(&self, viewer: u32) -> SimSnapshot {
        let mut snap = self.snapshot();
        let (vx, vy) = match self.player_pos(viewer) {
            Some(p) => p,
            None => return snap,
        };
        let r2 = VIEW_RADIUS * VIEW_RADIUS;
        snap.enemies.retain(|e| (e.x - vx).powi(2) + (e.y - vy).powi(2) <= r2);
        snap.arrows.retain(|a| (a.x - vx).powi(2) + (a.y - vy).powi(2) <= r2);
        snap.structures.retain(|s| {
            let (sx, sy) = (s.tx as f32 + 0.5, s.ty as f32 + 0.5);
            (sx - vx).powi(2) + (sy - vy).powi(2) <= r2
        });
        snap.resources.retain(|res| {
            let (sx, sy) = (res.tx as f32 + 0.5, res.ty as f32 + 0.5);
            (sx - vx).powi(2) + (sy - vy).powi(2) <= r2
        });
        // Occupants get their room: authoritative foes, loot flags, events.
        if let Some(spot) = self.inside.get(&viewer) {
            if let Some(room) = self.rooms.get(&(spot.bx, spot.by, spot.floor)) {
                snap.rooms.push(room_snapshot(room, spot.bx, spot.by, spot.floor));
            }
        }
        snap
    }
}

/// Flatten one server room for the wire (see `RoomSnapshot`).
fn room_snapshot(room: &ServerRoom, bx: i32, by: i32, floor: u8) -> RoomSnapshot {
    RoomSnapshot {
        bx,
        by,
        floor,
        loot_taken: room.loot_taken,
        foes: room
            .foes
            .iter()
            .map(|e| EnemySnapshot {
                x: e.x,
                y: e.y,
                kind: e.kind,
                hp: e.hp,
                facing: e.facing,
                state: e.state,
                windup: e.windup,
                flash: e.flash,
            })
            .collect(),
        events: room.events.iter().cloned().collect(),
    }
}

fn find_spawn(world: &WorldGen, cache: &mut ChunkCache) -> (f32, f32) {
    for r in 0..32i32 {
        for dx in -r..=r {
            for dy in -r..=r {
                let x = dx as f32;
                let y = dy as f32;
                let tile = tile_at(world, cache, x.floor() as i32, y.floor() as i32);
                if tile.walkable() && !matches!(tile, TileKind::ShallowWater) {
                    return (x, y);
                }
            }
        }
    }
    (0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::StructureKind;
    use crate::enemy::EnemyKind;
    use crate::items::ItemKind;
    use crate::weapons::WeaponKind;

    fn input() -> PlayerInput {
        PlayerInput {
            move_x: 0.0,
            move_y: 0.0,
            dodge: false,
            attack: false,
            harvest: false,
            eat: false,
            shoot: false,
            build: None,
            weapon: WeaponKind::Fists.as_u8(),
            weapon_unlocked: 1, // Fists
            enchant: 0,
            craft: None,
            room: None,
        }
    }

    #[test]
    fn killing_an_enemy_grants_xp_and_may_level() {
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("hero".into(), None);
        let (px, py) = {
            let p = &sim.players[&id].player;
            (p.x, p.y)
        };
        // Drop a slime right next to the hero, facing them (no backstab —
        // freshly spawned foes look east, which would read as "behind").
        let gx = px.floor() as i32 + 1;
        let gy = py.floor() as i32;
        sim.enemies.get(gx, gy, EnemyKind::Slime, 0.0);
        for e in sim.enemies.enemies_mut() {
            if e.kind == EnemyKind::Slime {
                e.facing = (-1.0, 0.0);
            }
        }

        // Tap attack (rising edges, spaced past the 0.4s cooldown): three
        // fists swings (4 dmg) fell the 12 HP slime.
        let start_xp = sim.players[&id].player.xp;
        for _ in 0..4 {
            let mut i = input();
            i.attack = true;
            sim.set_input(id, i);
            sim.step(1.0 / 30.0);
            for _ in 0..14 {
                sim.set_input(id, input());
                sim.step(1.0 / 30.0);
            }
        }
        let p = &sim.players[&id].player;
        assert!(p.xp > start_xp, "slaying the slime should grant XP");
    }

    #[test]
    fn holding_attack_swings_once_then_charges() {
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("hero".into(), None);
        // Sword for legible numbers: tap 10, heavy 10 x 1.65 after 0.5s hold.
        sim.players.get_mut(&id).unwrap().player.weapon = WeaponKind::Sword;
        sim.players.get_mut(&id).unwrap().player.unlocked = 0b11;
        let (px, py) = {
            let p = &sim.players[&id].player;
            (p.x, p.y)
        };
        sim.enemies
            .get(px.floor() as i32 + 1, py.floor() as i32, EnemyKind::Slime, 0.0);
        for e in sim.enemies.enemies_mut() {
            if e.kind == EnemyKind::Slime {
                e.facing = (-1.0, 0.0); // face the hero: neutral math, no backstab
            }
        }
        // Hold for 15 ticks: exactly one tap-swing (10 dmg), then charging.
        for _ in 0..15 {
            let mut i = input();
            i.attack = true;
            i.weapon = WeaponKind::Sword.as_u8();
            i.weapon_unlocked = 0b11;
            sim.set_input(id, i);
            sim.step(1.0 / 30.0);
        }
        let slime_hp = sim.enemies.enemies().map(|e| e.hp).fold(f32::MAX, f32::min);
        assert!((slime_hp - 2.0).abs() < 1e-3, "hold must swing exactly once, hp={slime_hp}");
        // Release: the charged heavy (10 x 1.65) finishes the slime.
        sim.set_input(id, input());
        sim.step(1.0 / 30.0);
        assert_eq!(sim.slimes_killed, 1, "release must loose the heavy finisher");
    }

    #[test]
    fn sim_backstab_matches_combat_math() {
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("rogue".into(), None);
        // Sword rogue behind an east-facing slime: 10 x 1.5 = 15 >= 12 HP.
        sim.players.get_mut(&id).unwrap().player.weapon = WeaponKind::Sword;
        sim.players.get_mut(&id).unwrap().player.unlocked = 0b11;
        let (px, py) = sim.player_pos(id).unwrap();
        let gx = px.floor() as i32 + 1;
        let gy = py.floor() as i32;
        sim.enemies.get(gx, gy, EnemyKind::Slime, 0.0);
        for e in sim.enemies.enemies_mut() {
            if e.kind == EnemyKind::Slime {
                e.x = gx as f32 + 0.5;
                e.y = gy as f32 + 0.5;
                e.facing = (1.0, 0.0); // looking away: player is behind
                e.tagged = true;
            }
        }
        let mut i = input();
        i.attack = true;
        i.weapon = WeaponKind::Sword.as_u8();
        i.weapon_unlocked = 0b11;
        sim.set_input(id, i);
        sim.step(1.0 / 30.0);
        assert_eq!(sim.slimes_killed, 1, "backstab must one-shot the slime");
    }

    #[test]
    fn piercing_bolt_wounds_a_rank() {
        use crate::combat::Arrow;
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("xbow".into(), None);
        let (px, py) = sim.player_pos(id).unwrap();
        let gy = py.floor() as i32;
        // Two slimes in a row on the bolt's path (distinct tiles).
        for k in 0..2 {
            sim.enemies.get(px.floor() as i32 + 1 + k, gy, EnemyKind::Slime, 0.0);
        }
        let a = Arrow::bolt(px.floor() + 0.5, py, 1.0, 0.0, 500.0);
        sim.arrows.push(a);
        sim.set_input(id, input());
        for _ in 0..10 {
            sim.step(1.0 / 30.0);
        }
        // Both corpses cleared (no double-loot), both counted.
        assert!(sim.enemies.enemies().all(|e| e.alive()));
        assert_eq!(sim.slimes_killed, 2, "piercing must kill the whole rank");
    }


    


    
    #[test]
    fn quest_advances_when_resources_gathered() {
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("hero".into(), None);
        assert_eq!(sim.quest.stage, 0);
        sim.players.get_mut(&id).unwrap().inv.add(ItemKind::Wood, 5);
        sim.players.get_mut(&id).unwrap().inv.add(ItemKind::Stone, 1);
        sim.step(1.0 / 30.0);
        assert_eq!(sim.quest.stage, 1, "5 wood + 1 stone unlocks the shelter beat");
    }

    #[test]
    fn crafting_iron_plate_at_anvil_sets_milestone() {
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("smith".into(), None);
        let (px, py) = {
            let p = &sim.players[&id].player;
            (p.x, p.y)
        };
        // Place an anvil right next to the player.
        sim.structures.push(Structure {
            tx: px as i32 + 1,
            ty: py as i32,
            kind: StructureKind::Anvil,
        });
        sim.players.get_mut(&id).unwrap().inv.add(ItemKind::Iron, 2);
        sim.players.get_mut(&id).unwrap().inv.add(ItemKind::Stone, 4);

        let mut i = input();
        i.craft = Some(ItemKind::IronPlate.as_u8());
        sim.set_input(id, i);
        sim.step(1.0 / 30.0);

        assert!(sim.iron_crafted, "crafting iron plate should set the milestone");
        assert_eq!(sim.players[&id].inv.count(ItemKind::IronPlate), 1);
        assert_eq!(sim.players[&id].inv.count(ItemKind::Iron), 0);
        assert_eq!(sim.players[&id].inv.count(ItemKind::Stone), 0);
    }

    #[test]
    fn coop_two_players_share_one_snapshot() {
        let mut sim = Simulation::new(1337);
        let a = sim.add_player("alice".into(), None);
        let b = sim.add_player("bob".into(), None);
        assert_ne!(a, b);
        sim.set_input(a, input());
        sim.set_input(b, input());
        sim.step(1.0 / 30.0);
        let snap = sim.snapshot();
        assert_eq!(snap.players.len(), 2, "both co-op players appear in the snapshot");
        let ids: Vec<u32> = snap.players.iter().map(|p| p.id).collect();
        assert!(ids.contains(&a) && ids.contains(&b));
    }

    #[test]
    fn snapshot_carries_authoritative_quest_and_craft() {
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("hero".into(), None);
        // Advance the campaign to stage 1 via gathered resources...
        sim.players.get_mut(&id).unwrap().inv.add(ItemKind::Wood, 5);
        sim.players.get_mut(&id).unwrap().inv.add(ItemKind::Stone, 1);
        sim.step(1.0 / 30.0);
        // ...and forge Iron Plate at an anvil so the craft milestone is set.
        let (px, py) = {
            let p = &sim.players[&id].player;
            (p.x, p.y)
        };
        sim.structures.push(Structure {
            tx: px as i32 + 1,
            ty: py as i32,
            kind: StructureKind::Anvil,
        });
        sim.players.get_mut(&id).unwrap().inv.add(ItemKind::Iron, 2);
        sim.players.get_mut(&id).unwrap().inv.add(ItemKind::Stone, 4);
        let mut ci = input();
        ci.craft = Some(ItemKind::IronPlate.as_u8());
        sim.set_input(id, ci);
        sim.step(1.0 / 30.0);

        let snap = sim.snapshot();
        assert_eq!(snap.quest_stage, sim.quest.stage, "snapshot must mirror the authoritative quest stage");
        assert!(snap.quest_stage >= 1, "quest should have advanced past stage 0");
        assert_eq!(snap.iron_crafted, sim.iron_crafted, "snapshot must mirror the craft milestone");
        assert!(snap.iron_crafted, "iron plate should be marked crafted");
    }

    #[test]
    fn bincode_roundtrips_client_and_server_messages() {
        let join = ClientMsg::Join { name: "hero".into(), token: Some("tok".into()), room: "R1".into() };
        let bytes = encode_client(&join);
        // Binary encoding is far smaller than the equivalent JSON text frame.
        let json = serde_json::to_string(&join).unwrap();
        assert!(bytes.len() < json.len(), "bincode should beat JSON on size");
        let back = decode_client_bin(&bytes).expect("bincode client decode");
        assert!(matches!(back, ClientMsg::Join { .. }));

        let mut sim = Simulation::new(1337);
        let id = sim.add_player("hero".into(), None);
        sim.set_input(id, input());
        sim.step(1.0 / 30.0);
        let snap = sim.snapshot();
        let msg = ServerMsg::Snapshot(snap);
        let bytes = encode_server(&msg);
        let back = decode_server_bin(&bytes).expect("bincode server decode");
        assert!(matches!(back, ServerMsg::Snapshot(_)));
    }

    #[test]
    fn legacy_json_welcome_still_parses_with_default_protocol() {
        // Old servers omit `protocol`; serde default must keep old captures working.
        let msg: ServerMsg = serde_json::from_str(
            r#"{"Welcome":{"player_id":7,"tick_rate":30,"seed":1337}}"#,
        )
        .expect("legacy welcome parses");
        match msg {
            ServerMsg::Welcome { player_id, protocol, .. } => {
                assert_eq!(player_id, 7);
                assert_eq!(protocol, 1);
            }
            _ => panic!("expected Welcome"),
        }
    }

    #[test]
    fn delta_omits_unchanged_statics_and_applies_cleanly() {
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("hero".into(), None);
        sim.set_input(id, input());
        sim.step(1.0 / 30.0);
        let a = sim.snapshot();
        sim.step(1.0 / 30.0);
        let b = sim.snapshot();

        // Nothing built between ticks: statics must be omitted from the delta.
        let d = b.delta_from(Some(&a));
        assert_eq!(d.base_tick, a.tick);
        assert_eq!(d.tick, b.tick);
        assert!(d.structures.is_none(), "unchanged structures must be omitted");
        assert!(d.resources.is_none(), "unchanged resources must be omitted");

        let mut merged = a.clone();
        merged.apply_delta(d);
        assert_eq!(merged.tick, b.tick);
        assert_eq!(merged.players.len(), b.players.len());
        assert_eq!(merged.structures.len(), b.structures.len());

        // Building a wall changes the statics hash: next delta includes them.
        sim.structures.push(Structure { tx: 0, ty: 0, kind: StructureKind::Wall });
        let c = sim.snapshot();
        let d2 = c.delta_from(Some(&b));
        assert!(d2.structures.is_some(), "changed structures must be included");
    }

    #[test]
    fn snapshot_for_culls_distant_entities_but_keeps_players() {        let mut sim = Simulation::new(1337);
        let a = sim.add_player("alice".into(), None);
        let b = sim.add_player("bob".into(), None);
        // Teleport Bob far away; spawn an enemy next to him (far from Alice).
        {
            let pb = sim.players.get_mut(&b).unwrap();
            pb.player.x += VIEW_RADIUS * 3.0;
            pb.player.y += VIEW_RADIUS * 3.0;
        }
        let (bx, by) = sim.player_pos(b).unwrap();
        sim.enemies.get(bx as i32, by as i32, EnemyKind::Slime, 0.0);
        sim.step(1.0 / 30.0);

        let full = sim.snapshot();
        let culled = sim.snapshot_for(a);
        // Both players always visible (co-op tags + interpolation need them).
        assert_eq!(culled.players.len(), 2);
        // The far-away enemy is outside Alice's interest radius.
        assert!(
            culled.enemies.len() <= full.enemies.len(),
            "culling must not add entities"
        );
        for e in &culled.enemies {
            let (ax, ay) = sim.player_pos(a).unwrap();
            assert!(
                (e.x - ax).hypot(e.y - ay) <= VIEW_RADIUS + 2.0,
                "culled enemies must be near the viewer"
            );
        }
    }

    #[test]
    fn server_world_matches_client_poi_layout() {
        use crate::building::StructureKind;
        // Co-op parity: the authoritative sim must build the same settlements
        // the single-player client builds for a seed (same generator now).
        let mut sim = Simulation::new(1337);
        let snap = sim.snapshot();
        let kinds: Vec<StructureKind> = snap.structures.iter().map(|s| s.kind).collect();
        for need in [StructureKind::Chest, StructureKind::House, StructureKind::Anvil, StructureKind::Portal, StructureKind::Train, StructureKind::Dungeon] {
            assert!(kinds.contains(&need), "server world must contain {need:?}");
        }
        // ...and players spawn in the first village plaza, like the client.
        let id = sim.add_player("hero".into(), None);
        let (x, y) = sim.player_pos(id).unwrap();
        let world = WorldGen::new(1337);
        let mut cache = ChunkCache::new(64);
        let sites = crate::poi::village_sites(1337, 3, |tx, ty| {
            crate::world::tile_at(&world, &mut cache, tx, ty).walkable()
        });
        let (vx, vy) = sites[0];
        assert!(
            (x - (vx as f32 + 0.5)).abs() < 1e-3 && (y - (vy as f32 + 0.5)).abs() < 1e-3,
            "server spawn must be the first village plaza"
        );
    }


    #[test]
    fn guardian_kills_recover_fragments_and_advance_ng() {
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("hero".into(), None);
        let (px, py) = sim.player_pos(id).unwrap();
        // Slay each Crown Fragment guardian (lethal damage, then one step so
        // the death sweep runs).
        for (bit, kind) in [
            (0u8, EnemyKind::Boss),
            (1, EnemyKind::ScorpionQueen),
            (2, EnemyKind::FrostGolem),
            (3, EnemyKind::ToadKing),
            (4, EnemyKind::OceanLeviathan),
        ] {
            // NOTE: floor(), not `as i32` (truncates toward zero and misses
            // for negative coordinates); distinct tiles (one foe per tile).
            let gx = px.floor() as i32 + bit as i32;
            let gy = py.floor() as i32;
            sim.enemies.get(gx, gy, kind, 0.0);
            for e in sim.enemies.enemies_mut() {
                if e.kind == kind {
                    // Tag + lethal damage = a player-earned kill.
                    e.tagged = true;
                    e.take_damage(99999.0);
                }
            }
            sim.set_input(id, input());
            sim.step(1.0 / 30.0);
            // Full recovery advances NG immediately and resets the counter...
            if bit < 4 {
                assert!(
                    sim.fragments_recovered & (1 << bit) != 0,
                    "{kind:?} kill must set fragment bit {bit}"
                );
            }
        }
        assert_eq!(sim.ng_cycle, 1, "full recovery must advance the room to NG+1");
        assert_eq!(sim.fragments_recovered, 0, "counter resets for the next cycle");
        assert_eq!(sim.snapshot().ng_cycle, 1, "snapshot must carry the cycle");
    }

    #[test]
    fn ng_scaling_matches_single_player_formulas() {
        let mut sim = Simulation::new(1337);
        assert_eq!(sim.ng_damage_mult(), 1.0);
        assert_eq!(sim.ng_cycle, 0);
        sim.ng_cycle = 2;
        assert_eq!(sim.ng_damage_mult(), 1.5);
        assert!(sim.ng_day_length() < DAY_LENGTH, "NG days must run faster");
        // ...and contact damage actually scales: slime 3.0 x 1.5 = 4.5.
        let id = sim.add_player("hero".into(), None);
        let (px, py) = sim.player_pos(id).unwrap();
        // Same tile as the player (floor, not truncation, for negatives).
        sim.enemies.get(px.floor() as i32, py.floor() as i32, EnemyKind::Slime, 0.0);
        sim.set_input(id, input());
        for _ in 0..40 {
            sim.step(1.0 / 30.0);
            if sim.players[&id].player.hp < 100.0 {
                break;
            }
        }
        let hp = sim.players[&id].player.hp;
        assert!((hp - 95.5).abs() < 0.6, "NG2 slime hit must deal ~4.5, hp={hp}");
    }

    #[test]
    fn arrow_kills_loot_once_and_progress_campaign() {
        use crate::combat::Arrow;
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("archer".into(), None);
        let (px, py) = sim.player_pos(id).unwrap();
        // Park a lethal arrow right on top of a slime (same tile as the foe).
        let ex = px.floor() as i32 + 1;
        let ey = py.floor() as i32;
        sim.enemies.get(ex, ey, EnemyKind::Slime, 0.0);
        let mut a = Arrow::new(ex as f32 + 0.5, ey as f32 + 0.5, 0.0, 0.0);
        a.damage = 500.0;
        sim.arrows.push(a);
        let xp0 = sim.players[&id].player.xp;
        sim.set_input(id, input());
        for _ in 0..10 {
            sim.step(1.0 / 30.0);
        }
        // Corpse removed immediately (no double-loot on the next sweep)...
        assert!(
            sim.enemies.enemies().all(|e| e.alive()),
            "no arrow-killed corpse may linger for the sweep"
        );
        // ...loot granted exactly once, XP paid, slime counter ticked.
        let inv = &sim.players[&id].inv;
        let loot_total = inv.count(ItemKind::Food) + inv.count(ItemKind::Herb) + inv.count(ItemKind::Gold);
        assert_eq!(
            loot_total,
            EnemyKind::Slime.drops().len() as u32,
            "exactly one full drop table for one arrow kill"
        );
        assert!(sim.players[&id].player.xp > xp0, "arrow kills must grant XP");
        assert_eq!(sim.slimes_killed, 1, "arrow kills must count for quests");
    }

    #[test]
    fn untagged_kills_drop_loot_but_grant_no_credit() {
        // Guard/spike/burn kills (no player tag): the crew still gets the
        // spoils, but no XP and no quest progress — idling can't farm.
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("idler".into(), None);
        let (px, py) = sim.player_pos(id).unwrap();
        sim.enemies.get(px.floor() as i32, py.floor() as i32, EnemyKind::Slime, 0.0);
        for e in sim.enemies.enemies_mut() {
            if e.kind == EnemyKind::Slime {
                e.take_damage(99999.0); // no tag: environmental death
            }
        }
        let xp0 = sim.players[&id].player.xp;
        sim.set_input(id, input());
        sim.step(1.0 / 30.0);
        let inv = &sim.players[&id].inv;
        let loot_total = inv.count(ItemKind::Food) + inv.count(ItemKind::Herb) + inv.count(ItemKind::Gold);
        assert_eq!(loot_total, EnemyKind::Slime.drops().len() as u32, "spoils still drop");
        assert_eq!(sim.players[&id].player.xp, xp0, "no XP without the tag");
        assert_eq!(sim.slimes_killed, 0, "no quest credit without the tag");
    }

    #[test]
    fn delta_carries_ng_cycle() {
        let mut sim = Simulation::new(1337);
        sim.add_player("hero".into(), None);
        sim.ng_cycle = 3;
        sim.step(1.0 / 30.0);
        let full = sim.snapshot();
        assert_eq!(full.ng_cycle, 3);
        sim.step(1.0 / 30.0);
        let next = sim.snapshot();
        let d = next.delta_from(Some(&full));
        assert_eq!(d.ng_cycle, 3);
        let mut merged = full;
        merged.apply_delta(d);
        assert_eq!(merged.ng_cycle, 3, "delta merge must preserve the cycle");
    }

    #[test]
    fn shattered_night_sends_one_hunting_pack() {
        use crate::enemy::EnemyKind;
        // Seed 1337 shatters on day 2: dusk brings exactly one bonus pack.
        let mut sim = Simulation::new(1337);
        sim.add_player("hero".into(), None);
        sim.day_count = 2;
        sim.time_of_day = 0.0; // deep night
        fn pack(sim: &Simulation) -> usize {
            sim.enemies
                .enemies()
                .filter(|e| {
                    matches!(e.kind, EnemyKind::Raider | EnemyKind::Wolf) && e.elite > 1.0
                })
                .count()
        }
        for _ in 0..400 {
            sim.step(1.0 / 30.0);
            if pack(&sim) == 2 {
                break;
            }
        }
        assert_eq!(pack(&sim), 2, "one Raider + one Wolf elite, once per night");
        for _ in 0..100 {
            sim.step(1.0 / 30.0);
        }
        assert_eq!(pack(&sim), 2, "no second pack the same night");
    }

    #[test]
    fn server_sends_elites_raiders_and_horn_answers() {
        // Roaming elite fires when its timer lapses: the first unrecovered
        // guardian hunts (elite 1.0 bosses are the summons, not brutes).
        let mut sim = Simulation::new(1337);
        sim.add_player("hero".into(), None);
        sim.elite_timer = 0.01;
        for _ in 0..5 {
            sim.step(1.0 / 30.0);
        }
        assert!(
            sim.enemies.enemies().any(|e| e.kind == EnemyKind::Boss),
            "the Forest Warden roams first"
        );
        // Night raiders muster around structures after dark.
        let mut sim = Simulation::new(1337);
        sim.add_player("hero".into(), None);
        sim.time_of_day = 0.0;
        for _ in 0..40 {
            sim.raider_timer = 0.01;
            sim.step(1.0 / 30.0);
            if sim.enemies.enemies().any(|e| e.kind == EnemyKind::Raider) {
                break;
            }
        }
        assert!(
            sim.enemies
                .enemies()
                .any(|e| e.kind == EnemyKind::Raider),
            "raiders muster at night"
        );
        // The horn answers with the first unrecovered guardian.
        let mut sim = Simulation::new(1337);
        let id = sim.add_player("hero".into(), None);
        sim.sound_horn(id);
        assert!(
            sim.enemies
                .enemies()
                .any(|e| e.kind == EnemyKind::Boss),
            "the Forest Warden answers first"
        );
        // Unknown ids summon nothing (and panic nothing).
        sim.sound_horn(9999);
        assert!(sim.enemies.enemies().count() >= 1);
    }

    #[test]
    fn co_op_rooms_share_foes_and_pay_once() {
        use crate::building::Structure;
        let mut sim = Simulation::new(1337);
        let a = sim.add_player("a".into(), None);
        let b = sim.add_player("b".into(), None);
        sim.structures.push(Structure { tx: 100, ty: 100, kind: StructureKind::Dungeon });
        sim.structures.push(Structure { tx: 104, ty: 100, kind: StructureKind::House });
        // Bare ground rejects entry.
        assert!(!sim.enter_interior(a, 0, 0));
        // Dungeon entry spawns the floor-1 patrol once, shared by late joiners.
        assert!(sim.enter_interior(a, 100, 100));
        assert_eq!(sim.rooms.get(&(100, 100, 1)).map(|r| r.foes.len()), Some(2));
        assert!(sim.enter_interior(b, 100, 100));
        assert_eq!(sim.rooms.get(&(100, 100, 1)).map(|r| r.foes.len()), Some(2));
        // Fists (4 dmg x2 swings) fell the 8 HP bats from room center.
        // (The stock `input()` carries the Fists unlock bit the stock
        // default lacks — same as a real client's first sync.)
        sim.set_input(a, input());
        let spot = RoomSpot { bx: 100, by: 100, floor: 1, x: 0.0, y: 0.0 };
        sim.room_attack(a, spot, 1.0, 0.0, 1.0);
        sim.room_attack(a, spot, 1.0, 0.0, 1.0); // bats carry 8 HP, fists hit 4
        assert_eq!(
            sim.rooms.get(&(100, 100, 1)).map(|r| r.foes.len()),
            Some(0),
            "shared roster: one swing clears for everyone"
        );
        // B sees the shared empty roster plus both kill events naming A.
        let snap_b = sim.snapshot_for(b);
        assert_eq!(snap_b.rooms.len(), 1);
        assert!(snap_b.rooms[0].foes.is_empty());
        assert_eq!(snap_b.rooms[0].events.len(), 2);
        assert!(matches!(
            snap_b.rooms[0].events[0],
            RoomEvent::Kill { killer, .. } if killer == a
        ));
        // House pantry pays once across players.
        assert!(sim.enter_interior(a, 104, 100));
        assert_eq!(sim.rooms.get(&(104, 100, 1)).map(|r| r.events.len()), Some(1));
        assert!(sim.enter_interior(b, 104, 100));
        assert_eq!(
            sim.rooms.get(&(104, 100, 1)).map(|r| r.events.len()),
            Some(1),
            "second entry must not re-pay the pantry"
        );
    }

    #[test]
    fn co_op_vault_claims_once_at_the_back_wall() {
        use crate::building::Structure;
        let mut sim = Simulation::new(1337);
        let a = sim.add_player("a".into(), None);
        let b = sim.add_player("b".into(), None);
        sim.structures.push(Structure { tx: 100, ty: 100, kind: StructureKind::Dungeon });
        assert!(sim.enter_interior(a, 100, 100));
        assert!(sim.enter_interior(b, 100, 100));
        // A climbs to the vault back wall (dungeon rh 2.5 -> my2 2.0).
        let mut i = input();
        i.room = Some(RoomSpot { bx: 100, by: 100, floor: 2, x: 0.0, y: -1.6 });
        sim.set_input(a, i);
        sim.step(1.0 / 30.0);
        let room = sim.rooms.get(&(100, 100, 2)).expect("stairs floor materializes");
        assert!(room.loot_taken, "vault claims on zone entry");
        assert_eq!(room.events.len(), 1);
        assert!(matches!(
            room.events[0],
            RoomEvent::Claim { claimer, xp: 60, .. } if claimer == a
        ));
        // B walks the same wall: nothing re-pays.
        let mut j = input();
        j.room = Some(RoomSpot { bx: 100, by: 100, floor: 2, x: 0.5, y: -1.6 });
        sim.set_input(b, j);
        for _ in 0..5 {
            sim.step(1.0 / 30.0);
        }
        assert_eq!(
            sim.rooms.get(&(100, 100, 2)).map(|r| r.events.len()),
            Some(1),
            "vault pays once per dungeon"
        );
    }

    #[test]
    fn torchlight_suppresses_nearby_spawns() {
        use crate::building::{lit_within, Structure, LIGHT_SAFE_RADIUS};
        // Locate a spawner tile within the light bubble of the spawn plaza.
        let mut probe = Simulation::new(1337);
        let (px, py) = probe.spawn_point;
        let mut target = None;
        for dx in -14..=14 {
            for dy in -14..=14 {
                let tx = px.floor() as i32 + dx;
                let ty = py.floor() as i32 + dy;
                let tile = tile_at(&probe.world, &mut probe.cache, tx, ty);
                if let Some(kind) = spawner_on(tx, ty, tile) {
                    // Skip ground the village lamps already keep safe: the
                    // test needs a tile that hydrates on bare ground.
                    if lit_within(&probe.structures, tx, ty, LIGHT_SAFE_RADIUS) {
                        continue;
                    }
                    target = Some((tx, ty, kind));
                    break;
                }
            }
            if target.is_some() {
                break;
            }
        }
        let (tx, ty, _) = target.expect("seed 1337 needs a spawner near spawn");
        let near = |sim: &Simulation| {
            sim.enemies.enemies().any(|e| {
                (e.x - (tx as f32 + 0.5)).hypot(e.y - (ty as f32 + 0.5)) < 1.5
            })
        };
        // Bare ground at night: the tile hydrates a foe.
        let mut bare = Simulation::new(1337);
        bare.add_player("a".into(), None);
        bare.time_of_day = 0.0;
        for _ in 0..10 {
            bare.step(1.0 / 30.0);
        }
        for e in bare.enemies.enemies() {
            println!("DBG foe {:?} at ({:.1},{:.1})", e.kind, e.x, e.y);
        }
        assert!(near(&bare), "spawner tile should hydrate without light");
        // Same seed with a torch on the tile: nothing spawns near it.
        let mut lit = Simulation::new(1337);
        lit.add_player("b".into(), None);
        lit.time_of_day = 0.0;
        lit.structures.push(Structure { tx, ty, kind: StructureKind::Torch });
        assert!(lit_within(&lit.structures, tx, ty, LIGHT_SAFE_RADIUS));
        for _ in 0..10 {
            lit.step(1.0 / 30.0);
        }
        assert!(!near(&lit), "torchlight must suppress the spawner tile");
    }

    #[test]
    fn v5_wire_round_trips_through_bincode() {
        // Every new v5 message shape must survive the real binary wire
        // (browsers fall back to JSON, so a bincode-only skew would hide).
        let spot = RoomSpot { bx: 6, by: 5, floor: 2, x: 1.5, y: -1.5 };
        for msg in [
            ClientMsg::EnterInterior { tx: 6, ty: 5 },
            ClientMsg::ExitInterior,
            ClientMsg::RoomAttack { spot, fx: 1.0, fy: 0.0, mult: 1.5 },
            ClientMsg::Input(PlayerInput { room: Some(spot), ..input() }),
            ClientMsg::Leave,
        ] {
            let bytes = encode_client(&msg);
            let back = decode_client_bin(&bytes).expect("v5 msg must decode");
            let re = encode_client(&back);
            assert_eq!(bytes, re, "message must round-trip byte-identically");
        }
        // A stale v4 tab (shorter `Input`) still decodes, roomless.
        let old = ClientMsgV4::Input(PlayerInputV4 {
            move_x: 1.0,
            move_y: 0.0,
            dodge: false,
            attack: true,
            harvest: false,
            eat: false,
            shoot: false,
            build: None,
            weapon: 0,
            weapon_unlocked: 1,
            enchant: 0,
            craft: None,
        });
        let bytes = bincode::serialize(&old).expect("v4 encodes");
        match decode_client_bin(&bytes).expect("v4 input must decode") {
            ClientMsg::Input(i) => {
                assert_eq!(i.move_x, 1.0);
                assert!(i.attack);
                assert_eq!(i.room, None);
            }
            other => panic!("v4 input upgraded wrong: {other:?}"),
        }
        // Snapshots carrying rooms survive both directions.
        let mut sim = Simulation::new(1337);
        let a = sim.add_player("a".into(), None);
        sim.structures.push(crate::building::Structure {
            tx: 100,
            ty: 100,
            kind: crate::building::StructureKind::Dungeon,
        });
        assert!(sim.enter_interior(a, 100, 100));
        let mut i = input();
        i.room = Some(RoomSpot { bx: 100, by: 100, floor: 1, x: 0.0, y: 0.0 });
        sim.set_input(a, i);
        sim.step(1.0 / 30.0);
        let snap = sim.snapshot_for(a);
        assert_eq!(snap.rooms.len(), 1, "occupant sees their room");
        assert_eq!(snap.rooms[0].foes.len(), 2, "patrol rides the snapshot");
        let bytes = encode_server(&ServerMsg::Snapshot(snap));
        let back = decode_server_bin(&bytes).expect("snapshot must decode");
        let re = encode_server(&back);
        assert_eq!(bytes, re, "snapshot must round-trip byte-identically");
    }
}
