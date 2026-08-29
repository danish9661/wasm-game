use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::building::{Structure, StructureKind, try_build};
use crate::combat::Arrow;
use crate::daynight::{daylight, temperature, DAY_LENGTH};
use crate::weapons::WeaponKind;
use crate::enemy::{AiState, Enemy, EnemyKind, EnemyRegistry, spawner_on};
use crate::items::{Inventory, ItemKind};
use crate::player::{self, Player};
use crate::poi::ruins_at;
use crate::quest::QuestLog;
use crate::resources::{resource_on, NodeRegistry, ResourceKind};
use crate::world::{tile_at, ChunkCache, WorldGen, TileKind};

const MOVE_SPEED: f32 = 4.0;
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMsg {
    Welcome { player_id: u32, tick_rate: u32, seed: u32 },
    Snapshot(SimSnapshot),
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
    pub weapon_unlocked: u8,
    /// Enchantment level of the equipped weapon (raises its damage). Sent by the
    /// client; the server trusts it (co-op is collaborative, not competitive).
    pub enchant: u8,
    /// Craft request: an `ItemKind::as_u8` value to craft at a nearby Anvil this
    /// tick, or `None`. Consumed (taken) by the sim so it fires once per press.
    pub craft: Option<u8>,
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
    salves: u32,
    input: PlayerInput,
    attack_cd: f32,
    eat_cd: f32,
    harvest_cd: f32,
    dodge_cd: f32,
    shoot_cd: f32,
    respawn_timer: f32,
}

pub struct Simulation {
    world: WorldGen,
    cache: ChunkCache,
    nodes: NodeRegistry,
    enemies: EnemyRegistry,
    structures: Vec<Structure>,
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
}

impl Simulation {
    pub fn new(seed: u32) -> Self {
        let world = WorldGen::new(seed);
        let mut cache = ChunkCache::new(256);
        let spawn = find_spawn(&world, &mut cache);
        Self {
            world,
            cache,
            nodes: NodeRegistry::new(),
            enemies: EnemyRegistry::new(),
            structures: Vec::new(),
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
                salves: 0,
                input: PlayerInput::default(),
                attack_cd: 0.0,
                eat_cd: 0.0,
                harvest_cd: 0.0,
                dodge_cd: 0.0,
                shoot_cd: 0.0,
                respawn_timer: 0.0,
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
    }

    pub fn set_input(&mut self, id: u32, input: PlayerInput) {
        if let Some(p) = self.players.get_mut(&id) {
            p.input = input;
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

    pub fn step(&mut self, dt: f32) {
        self.tick += 1;
        self.time_of_day = (self.time_of_day + dt / DAY_LENGTH).rem_euclid(1.0);
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

        self.spawn_near_players(dt);

        let spawn = self.spawn_point;
        for p in self.players.values_mut() {
            let crafted = Self::step_player(
                p,
                &self.world,
                &mut self.cache,
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
                    let tile = tile_at(&self.world, &mut self.cache, tx, ty);
                    if let Some(kind) = spawner_on(tx, ty, tile) {
                        // Nocturnal enemies only emerge after dark.
                        if kind.nocturnal() && daylight(self.time_of_day) > 0.5 {
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
    structures: &[Structure],
    nodes: &NodeRegistry,
    tx: i32,
    ty: i32,
) -> bool {
    let tile = tile_at(world, cache, tx, ty);
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
    if (np.input.weapon_unlocked & (1u8 << (w as u8))) != 0 {
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
        if !Self::tile_blocked(world, cache, structures, nodes, tx as i32, ty as i32) {
            np.player.x = tx;
            np.player.y = ty;
        } else if !Self::tile_blocked(world, cache, structures, nodes, tx as i32, np.player.y as i32) {
            np.player.x = tx;
        } else if !Self::tile_blocked(world, cache, structures, nodes, np.player.x as i32, ty as i32) {
            np.player.y = ty;
        }
        np.player.facing = (nx, ny);
    }

    if np.input.dodge && np.dodge_cd <= 0.0 {
        if np.player.try_dodge(np.player.facing) {
            np.dodge_cd = 0.8;
        }
    }

    if np.input.attack && np.attack_cd <= 0.0 {
        np.attack_cd = 0.4;
        let w = np.player.weapon;
        let mut hits = crate::combat::swing_hits(&np.player, enemies.enemies_mut(), w.reach());
        for e in hits.iter_mut() {
            e.take_damage(np.player.weapon_damage());
        }
    }

    if np.input.shoot && np.shoot_cd <= 0.0 {
        let w = np.player.weapon;
        let (fx, fy) = np.player.facing;
        if w.ranged() && (fx * fx + fy * fy) > 1e-4 {
            let mut a = Arrow::new(np.player.x, np.player.y, fx, fy);
            a.damage = np.player.weapon_damage();
            arrows.push(a);
            np.shoot_cd = 0.5;
        }
    }

    if np.input.harvest && np.harvest_cd <= 0.0 {
        if let Some((tx, ty, kind)) = Self::node_in_range(world, cache, nodes, np.player.x, np.player.y) {
            if let Some(item) = nodes.chop(tx, ty, kind) {
                np.inv.add(item, 1);
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
    if warm && np.player.hp < player::MAX_HP {
        np.player.hp = (np.player.hp + dt * 3.0).min(player::MAX_HP);
    }
    crafted
}

    fn node_in_range(
        world: &WorldGen,
        cache: &mut ChunkCache,
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
                let tile = tile_at(world, cache, tx, ty);
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
        let structs = &self.structures;
        let nodes = &self.nodes;
        let mut blocked = |tx: i32, ty: i32| -> bool {
            let tile = tile_at(world, cache, tx, ty);
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

        for (ex, ey, dmg) in contacts {
            if let Some(id) = self.nearest_player(ex, ey) {
                if let Some(p) = self.players.get_mut(&id) {
                    if (p.player.x - ex).hypot(p.player.y - ey) < CONTACT_RANGE {
                        p.player.take_damage(dmg);
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
                    loot.push((e.x, e.y, e.drops()));
                    xp_grants.push((pid, e.kind.xp()));
                    if e.kind == EnemyKind::Slime {
                        self.slimes_killed += 1;
                    }
                }
                dead.push(k);
            }
        }
        for (tx, ty) in dead {
            self.enemies.kill(tx, ty, RESPAWN_SECS);
        }
        for (x, y, items) in loot {
            self.give_loot(x, y, items);
        }
        for (pid, xp) in xp_grants {
            if let Some(p) = self.players.get_mut(&pid) {
                p.player.add_xp(xp);
            }
        }
        self.step_quests();
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
        let mut alive = Vec::new();
        for mut a in arrows {
            if !a.step(dt) {
                continue;
            }
            if a.from_player {
                let mut hit = false;
                let mut loot: Vec<(f32, f32, Vec<ItemKind>)> = Vec::new();
                for e in self.enemies.enemies_mut() {
                    if (e.x - a.x).hypot(e.y - a.y) < 0.8 {
                        if e.take_damage(a.damage) {
                            loot.push((e.x, e.y, e.drops()));
                        }
                        hit = true;
                        break;
                    }
                }
                for (x, y, items) in loot {
                    self.give_loot(x, y, items);
                }
                if !hit {
                    alive.push(a);
                }
            } else {
                let mut hit = false;
                for p in self.players.values_mut() {
                    if (p.player.x - a.x).hypot(p.player.y - a.y) < 0.8 {
                        p.player.take_damage(a.damage);
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
                    self.arrows.push(Arrow::new(tx, ty, dx, dy));
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
        }
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
        // Drop a slime right next to the hero.
        sim.enemies
            .get(px as i32 + 1, py as i32, EnemyKind::Slime, 0.0);

        let start_xp = sim.players[&id].player.xp;
        for _ in 0..60 {
            let mut i = input();
            i.attack = true;
            sim.set_input(id, i);
            sim.step(1.0 / 30.0);
        }
        let p = &sim.players[&id].player;
        assert!(p.xp > start_xp, "slaying the slime should grant XP");
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
}
