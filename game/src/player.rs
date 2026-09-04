use crate::items::{Inventory, ItemKind};
use crate::weapons::WeaponKind;
use crate::world::{ChunkCache, TileKind, WorldGen, tile_at};

/// Tiles per second when walking (screen-up is (-1,-1) in world coords).
/// Deliberate survival pace: all movement (player, enemies, NPCs) is scaled
/// to ~85% of twitch speed so fights read and kiting takes commitment.
pub const PLAYER_SPEED: f32 = 4.4;
/// Camera follow responsiveness (1/toward-player per second).
pub const CAMERA_FOLLOW: f32 = 10.0;
/// Max hit points; damage taken is subtracted from this.
pub const MAX_HP: f32 = 100.0;
/// Hunger in [0,1]: 1 = full. Drains over time; 0 = starving (lose hp).
pub const MAX_HUNGER: f32 = 100.0;
/// Stamina in [0,1]: attacks cost stamina, it regenerates while idle.
pub const MAX_STAMINA: f32 = 100.0;
/// Thirst in [0,1]: 1 = hydrated. Drains over time; 0 = dehydrated (lose hp).
/// Drains faster in the heat (Desert) and when exposed to rain.
pub const MAX_THIRST: f32 = 100.0;
/// Respawn position for the player (the spawner finds the same tile).
pub const SPAWN: (f32, f32) = (0.5, 0.5);
/// Dodge roll: duration of the burst, cooldown, stamina cost, and the speed
/// multiplier applied during the burst.
pub const DODGE_TIME: f32 = 0.25;
pub const DODGE_CD: f32 = 0.6;
pub const DODGE_STAMINA: f32 = 20.0;
pub const DODGE_BOOST: f32 = 2.4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub hunger: f32,
    pub stamina: f32,
    /// Hydration in [0,1]; 0 = dehydrated and losing health.
    pub thirst: f32,
    /// Last movement direction (also the attack facing).
    pub facing: (f32, f32),
    /// Seconds before the player can take damage again (hit immunity).
    pub hurt_timer: f32,
    /// Remaining dodge-roll burst time and its cooldown.
    pub dodge_timer: f32,
    pub dodge_cd: f32,
    /// Unit direction of the active dodge burst.
    pub dodge_dir: (f32, f32),
    pub alive: bool,
    /// True while the player is holding a block (Shift). Reduces incoming damage
    /// at a stamina cost. Set by the renderer each frame from the key state.
    pub blocking: bool,
    /// Currently equipped weapon. Drives melee reach/damage/cadence and whether
    /// the attack fires a projectile (Bow).
    pub weapon: WeaponKind,
    /// Bitmask of weapons the player owns (bit `k as usize`; Fists is always set).
    pub unlocked: u8,
    /// Enchantment level of the equipped weapon (0 = none). Each level adds +15%
    /// damage and a faint glow. Gems spent at an Enchanting Table raise it.
    pub enchant: u8,
    /// Experience points toward the next level. Killing enemies grants XP.
    pub xp: u32,
    /// Current level; each level raises max HP and partially heals on level-up.
    pub level: u32,
    /// Attack animation progress in [0,1] driven by the renderer from the swing
    /// cooldown. 0 = idle, ramps to 1 as a melee swing lands; the world renderer
    /// uses it to lunge the player's torso/arms forward on a strike.
    pub swing_t: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            hp: MAX_HP,
            hunger: MAX_HUNGER,
            stamina: MAX_STAMINA,
            thirst: MAX_THIRST,
            facing: (1.0, 0.0),
            hurt_timer: 0.0,
            dodge_timer: 0.0,
            dodge_cd: 0.0,
            dodge_dir: (1.0, 0.0),
            alive: true,
            blocking: false,
            weapon: WeaponKind::Fists,
            unlocked: 1, // Fists (bit 0) always available
            enchant: 0,
            xp: 0,
            level: 1,
            swing_t: 0.0,
        }
    }

    pub fn dead(&self) -> bool {
        !self.alive
    }

    /// Effective max HP, scaling +10 per level above 1.
    pub fn max_hp(&self) -> f32 {
        MAX_HP + (self.level - 1) as f32 * 10.0
    }

    /// Grant experience and resolve any level-ups (multiple in one call). Each
    /// level raises max HP and heals a chunk. Returns the number of levels gained.
    pub fn add_xp(&mut self, amount: u32) -> u32 {
        if !self.alive {
            return 0;
        }
        self.xp += amount;
        let mut gained = 0;
        let need = |lv: u32| 50u32 + lv * 50;
        while self.xp >= need(self.level) {
            self.xp -= need(self.level);
            self.level += 1;
            let mh = self.max_hp();
            self.hp = (self.hp + 25.0).min(mh);
            gained += 1;
        }
        gained
    }

    /// Mark a weapon as owned (Fists is always owned via the initial bitmask).
    pub fn unlock_weapon(&mut self, k: WeaponKind) {
        self.unlocked |= 1u8 << (k as usize);
    }

    /// Whether the player owns a given weapon.
    pub fn has_weapon(&self, k: WeaponKind) -> bool {
        (self.unlocked & (1u8 << (k as usize))) != 0
    }

    /// Cycle to the next owned weapon (wraps around). No-op if only Fists owned.
    pub fn cycle_weapon(&mut self) {
        // NOTE: cycle position, not the discriminant — order and as_u8 differ.
        let order = WeaponKind::cycle_order();
        let pos = order.iter().position(|&k| k == self.weapon).unwrap_or(0);
        for i in 1..order.len() {
            let idx = (pos + i) % order.len();
            if self.has_weapon(order[idx]) {
                self.weapon = order[idx];
                return;
            }
        }
    }

    /// Pick up a weapon: own it and equip it immediately.
    pub fn equip_weapon(&mut self, k: WeaponKind) {
        self.unlock_weapon(k);
        self.weapon = k;
    }

    /// Damage of the currently equipped weapon including the enchant bonus
    /// (+15% per enchant level). Both the client and server use this so co-op
    /// damage stays consistent.
    pub fn weapon_damage(&self) -> f32 {
        self.weapon.damage() * (1.0 + 0.15 * self.enchant as f32)
    }

    /// Applies damage; respects the hurt-timer (brief invulnerability after
    /// each hit so enemies can't melt you in one frame). A raised block (Shift)
    /// cuts the damage and costs stamina instead of health. Returns the actual
    /// damage dealt (0 if it was ignored by i-frames).
    pub fn take_damage(&mut self, dmg: f32) -> bool {
        if !self.alive || self.hurt_timer > 0.0 {
            return false;
        }
        if self.blocking && self.stamina > 1.0 {
            self.stamina = (self.stamina - 10.0).max(0.0);
            let reduced = dmg * 0.35;
            self.hurt_timer = 0.25;
            self.hp = (self.hp - reduced).max(0.0);
            if self.hp <= 0.0 {
                self.alive = false;
            }
            return true;
        }
        self.hurt_timer = 0.6;
        self.hp = (self.hp - dmg).max(0.0);
        if self.hp <= 0.0 {
            self.alive = false;
        }
        true
    }

    /// Push the player away from a hit source by `force` tiles (knockback).
    pub fn knockback(&mut self, dx: f32, dy: f32, force: f32) {
        let len = (dx * dx + dy * dy).sqrt().max(1e-4);
        self.x += dx / len * force;
        self.y += dy / len * force;
    }

    /// Death: respawn at the spawn point with full hp but half hunger.
    pub fn respawn(&mut self) {
        self.x = SPAWN.0;
        self.y = SPAWN.1;
        self.hp = MAX_HP;
        self.hunger = MAX_HUNGER / 2.0;
        self.stamina = MAX_STAMINA;
        self.thirst = MAX_THIRST / 2.0;
        self.hurt_timer = 0.0;
        self.dodge_timer = 0.0;
        self.dodge_cd = 0.0;
        self.alive = true;
        self.hp = self.max_hp();
    }

    /// Try to spend stamina; false if exhausted (attack whiffs).
    pub fn spend_stamina(&mut self, cost: f32) -> bool {
        if self.stamina < cost {
            return false;
        }
        self.stamina -= cost;
        true
    }

    /// Eat one food item: restores 30 hunger.
    pub fn eat(&mut self, inv: &mut Inventory) -> bool {
        if inv.remove(ItemKind::Food, 1) {
            self.hunger = (self.hunger + 30.0).min(MAX_HUNGER);
            true
        } else {
            false
        }
    }

    /// Drink from a water source (well or shoreline): restores 45 thirst.
    pub fn drink_water(&mut self) -> bool {
        if self.thirst >= MAX_THIRST {
            return false;
        }
        self.thirst = (self.thirst + 45.0).min(MAX_THIRST);
        true
    }

    /// Begin a dodge roll in `dir` (falls back to facing if stationary).
    /// Returns false (and changes nothing) if on cooldown or low on stamina.
    /// Grants brief i-frames via `hurt_timer`.
    pub fn try_dodge(&mut self, dir: (f32, f32)) -> bool {
        if self.dodge_cd > 0.0 || self.stamina < DODGE_STAMINA || !self.alive {
            return false;
        }
        let d = if dir.0 == 0.0 && dir.1 == 0.0 {
            self.facing
        } else {
            dir
        };
        self.dodge_dir = d;
        self.dodge_timer = DODGE_TIME;
        self.dodge_cd = DODGE_CD;
        self.stamina -= DODGE_STAMINA;
        self.hurt_timer = self.hurt_timer.max(DODGE_TIME);
        true
    }

    /// Per-tick survival: hunger drains slowly (3× faster in the cold at
    /// night), stamina regenerates, starving costs hp, hurt timer ticks down.
    /// `warm` (sheltered by a campfire/lantern/brazier) slows the drain and
    /// stops starvation damage — the light/warmth loop the world hints at.
    /// `wet` (raining) halves stamina regen and makes the cold bite a little
    /// harder, so you want shelter when the storm rolls in.
    /// Base drain ≈ 9 hunger/minute, so a full bar lasts ~11 minutes.
    /// `biome` is the tile under the player; the harsh Tundra/Desert biomes
    /// drain hunger faster and regenerate stamina slower (exposure).
    pub fn tick(
        &mut self,
        dt: f32,
        temperature: f32,
        warm: bool,
        wet: bool,
        biome: TileKind,
        weather: u8,
    ) {
        self.hurt_timer = (self.hurt_timer - dt).max(0.0);
        self.dodge_timer = (self.dodge_timer - dt).max(0.0);
        self.dodge_cd = (self.dodge_cd - dt).max(0.0);
        let cold = ((temperature).min(0.0) / -10.0).max(0.0);
        let harsh = matches!(
            biome,
            TileKind::Tundra | TileKind::Desert | TileKind::Jungle | TileKind::Volcanic
        );
        let storm = weather == 3;
        let heat = weather == 4;
        let mut drain = if warm { 0.05 } else { 0.09 + 0.18 * cold };
        if wet {
            drain += 0.02;
        }
        if storm {
            drain += 0.03;
        }
        if heat {
            drain += 0.03;
        }
        if harsh {
            drain += 0.07;
        }
        self.hunger = (self.hunger - dt * drain).max(0.0);
        if self.hunger <= 0.0 && !warm {
            self.hp = (self.hp - dt * 1.0).max(0.0);
            if self.hp <= 0.0 {
                self.alive = false;
            }
        }
        // Thirst drains a little faster than hunger, and is punished hardest by
        // heat (Desert/Jungle) and rain. Dehydration is deadlier than starvation.
        let mut tdrain = if warm { 0.06 } else { 0.11 + 0.20 * cold };
        if wet {
            tdrain += 0.04;
        }
        if storm {
            tdrain += 0.03;
        }
        if heat {
            tdrain += 0.12;
        }
        if matches!(biome, TileKind::Desert | TileKind::Jungle | TileKind::Volcanic) {
            tdrain += 0.10;
        } else if matches!(biome, TileKind::Tundra) {
            tdrain += 0.02;
        }
        self.thirst = (self.thirst - dt * tdrain).max(0.0);
        if self.thirst <= 0.0 {
            self.hp = (self.hp - dt * 2.0).max(0.0);
            if self.hp <= 0.0 {
                self.alive = false;
            }
        }
        // Harsh biomes (Tundra/Desert) sap health from exposure when you're
        // not sheltered by a light source — warmth is the only real defense.
        // A storm also bites if you're caught out in the open.
        if (harsh || storm) && !warm {
            self.hp = (self.hp - dt * if storm { 0.4 } else { 0.6 }).max(0.0);
            if self.hp <= 0.0 {
                self.alive = false;
            }
        }
        let mut regen = if wet { 6.0 } else { 12.0 };
        if harsh {
            regen *= 0.6;
        }
        if storm {
            regen *= 0.7;
        }
        self.stamina = (self.stamina + dt * regen).min(MAX_STAMINA);
    }
}

/// 8-directional input vector. Screen-up is (-1,-1) in world coords.
pub fn input_dir(up: bool, down: bool, left: bool, right: bool) -> (f32, f32) {
    let mut dx = 0.0;
    let mut dy = 0.0;
    if up {
        dx -= 1.0;
        dy -= 1.0;
    }
    if down {
        dx += 1.0;
        dy += 1.0;
    }
    if left {
        dx -= 1.0;
        dy += 1.0;
    }
    if right {
        dx += 1.0;
        dy -= 1.0;
    }
    (dx, dy)
}

/// Moves the player with axis-separated collision: each axis is rolled back
/// independently if the destination tile is blocked, so the player slides
/// along walls instead of stopping dead. Diagonal movement is normalized.
/// Only a *crossing* into a new tile is blocked — a player standing on a
/// blocked tile (e.g. a wall built underfoot) can always walk out of it.
pub fn move_player(
    player: &mut Player,
    dir: (f32, f32),
    dt: f32,
    speed_mul: f32,
    mut is_blocked: impl FnMut(i32, i32) -> bool,
) {
    let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
    if len == 0.0 || dt <= 0.0 {
        return;
    }
    let (dx, dy) = (dir.0 / len, dir.1 / len);
    // `speed_mul` lets the caller slow the player on rough terrain (snow,
    // swamp) without the per-tile logic living in the movement primitive.
    let step = PLAYER_SPEED * dt * speed_mul;
    player.facing = (dx, dy);

    let (px, py) = (player.x.floor() as i32, player.y.floor() as i32);

    let nx = player.x + dx * step;
    let nxt = nx.floor() as i32;
    if nxt == px || !is_blocked(nxt, py) {
        player.x = nx;
    }

    let ny = player.y + dy * step;
    let nyt = ny.floor() as i32;
    if nyt == py || !is_blocked(player.x.floor() as i32, nyt) {
        player.y = ny;
    }
}

/// Camera eases toward `target`; stops exactly on it.
pub fn follow_camera(cam: &mut crate::render::Camera, target: (f32, f32), dt: f32) {
    let k = (dt * CAMERA_FOLLOW).min(1.0);
    cam.x += (target.0 - cam.x) * k;
    cam.y += (target.1 - cam.y) * k;
}

/// First walkable tile scanning outward from the origin (spiral).
pub fn find_spawn(world: &WorldGen, cache: &mut ChunkCache) -> (f32, f32) {
    let mut r: i32 = 0;
    loop {
        for tx in -r..=r {
            for ty in -r..=r {
                if tx.abs().max(ty.abs()) != r {
                    continue;
                }
                if tile_at(world, cache, tx, ty).walkable() {
                    return (tx as f32 + 0.5, ty as f32 + 0.5);
                }
            }
        }
        r += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::TileKind;

    #[test]
    fn input_directions() {
        assert_eq!(input_dir(true, false, false, false), (-1.0, -1.0));
        assert_eq!(input_dir(false, true, false, false), (1.0, 1.0));
        assert_eq!(input_dir(false, false, true, false), (-1.0, 1.0));
        assert_eq!(input_dir(false, false, false, true), (1.0, -1.0));
        assert_eq!(input_dir(false, false, false, false), (0.0, 0.0));
    }

    #[test]
    fn moves_along_input_dir() {
        let mut p = Player::new(0.0, 0.0);
        move_player(&mut p, (0.0, 1.0), 0.5, 1.0, |_, _| false);
        assert!((p.x - 0.0).abs() < 0.001);
        assert!((p.y - PLAYER_SPEED * 0.5).abs() < 0.001);
    }

    #[test]
    fn diagonal_is_not_faster() {
        let mut p = Player::new(0.0, 0.0);
        move_player(&mut p, (1.0, 1.0), 0.5, 1.0, |_, _| false);
        let per_axis = PLAYER_SPEED * 0.5 / 2.0_f32.sqrt();
        assert!((p.x - per_axis).abs() < 0.001);
        assert!((p.y - per_axis).abs() < 0.001);
    }

    #[test]
    fn blocked_axis_rolls_back_and_other_slides() {
        let mut p = Player::new(0.0, 0.0);
        // block the column x=1 only: moving down-right slides along the wall
        move_player(&mut p, (1.0, 1.0), 0.5, 1.0, |tx, _| tx >= 1);
        assert!((p.x - 0.0).abs() < 0.001, "x must be blocked");
        assert!(
            (p.y - PLAYER_SPEED * 0.5 / 2.0_f32.sqrt()).abs() < 0.001,
            "y must still slide"
        );
    }

    #[test]
    fn blocked_axis_rolls_back() {
        let mut p = Player::new(0.0, 0.0);
        move_player(&mut p, (1.0, 0.0), 0.5, 1.0, |tx, _| tx >= 1);
        assert!((p.x - 0.0).abs() < 0.001);
        assert!((p.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn can_walk_out_of_a_blocked_tile() {
        // player standing inside tile (1,0) whose own tile is blocked
        let mut p = Player::new(1.5, 0.5);
        move_player(&mut p, (1.0, 0.0), 0.5, 1.0, |tx, _| tx == 1);
        assert!((p.x - 1.5 - PLAYER_SPEED * 0.5).abs() < 0.001, "must leave the blocked tile");
        assert!((p.y - 0.5).abs() < 0.001);
    }

    #[test]
    fn cannot_cross_into_a_blocked_tile() {
        let mut p = Player::new(0.6, 0.5);
        // Step far enough to actually reach the adjacent (blocked) tile.
        move_player(&mut p, (1.0, 0.0), 0.2, 1.0, |tx, _| tx == 1);
        assert!(p.x < 1.0, "must not enter the blocked tile (got {})", p.x);
    }

    #[test]
    fn camera_follows_and_settles() {
        use crate::render::Camera;
        let mut cam = Camera::new(0.0, 0.0);
        let target = (10.0, -5.0);
        follow_camera(&mut cam, target, 0.05);
        assert!(cam.x > 0.0 && cam.x < 10.0);
        assert!(cam.y < 0.0 && cam.y > -5.0);
        // repeated stepping with large dt converges exactly
        for _ in 0..100 {
            follow_camera(&mut cam, target, 2.0);
        }
        assert!((cam.x - 10.0).abs() < 0.001);
        assert!((cam.y + 5.0).abs() < 0.001);
    }

    #[test]
    fn tile_kinds_walkability() {
        assert!(!TileKind::Water.walkable());
        assert!(!TileKind::DeepWater.walkable());
        assert!(TileKind::Grass.walkable());
        assert!(TileKind::Stone.walkable());
        assert!(TileKind::Snow.walkable());
        assert!(TileKind::Sand.walkable());
        assert!(TileKind::Forest.walkable());
        assert!(TileKind::Swamp.walkable());
    }

    #[test]
    fn spawn_is_walkable() {
        let world = WorldGen::new(1337);
        let mut cache = ChunkCache::new(64);
        let (x, y) = find_spawn(&world, &mut cache);
        let kind = tile_at(&world, &mut cache, x.floor() as i32, y.floor() as i32);
        assert!(kind.walkable(), "spawn tile must be walkable, got {kind:?}");
    }
}