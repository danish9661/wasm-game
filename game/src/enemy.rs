use crate::items::ItemKind;
use crate::render::Sprite;
use crate::world::TileKind;
use std::collections::{BinaryHeap, HashMap};

/// Aggro range: enemies start chasing within this many tiles (chebyshev).
pub const AGGRO_RANGE: f32 = 6.0;
/// Contact damage range for attacks (chebyshev, tile units).
pub const ATTACK_RANGE: f32 = 1.1;
/// How often (seconds) an enemy replans its A* path.
pub const REPLAN_INTERVAL: f32 = 0.5;
/// Enemy speed in tiles/second (slower than the player, so you can escape).
pub const ENEMY_SPEED: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Slime,
}

impl EnemyKind {
    pub fn max_hp(self) -> f32 {
        match self {
            EnemyKind::Slime => 12.0,
        }
    }

    /// Contact damage dealt per hit.
    pub fn damage(self) -> f32 {
        match self {
            EnemyKind::Slime => 4.0,
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            EnemyKind::Slime => [0.30, 0.78, 0.36],
        }
    }

    /// Sprite geometry: slightly larger than the player so it reads clearly.
    /// Alpha fades as hp drops (a visible health telegraph).
    pub fn sprite(self, x: f32, y: f32, hp_frac: f32) -> Sprite {
        let mut s = Sprite::new_center(x, y, self.color(), 14.0, 14.0, 2.0);
        s.alpha = 0.8 + 0.2 * hp_frac;
        s
    }
}

/// Internal AI state of one enemy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiState {
    Idle,
    Chase,
    Attack,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub kind: EnemyKind,
    pub hp: f32,
    pub state: AiState,
    pub facing: (f32, f32),
    pub attack_timer: f32,
    path_timer: f32,
    path: Vec<(i32, i32)>,
    wander: f32,
}

impl Enemy {
    pub fn new(x: f32, y: f32, kind: EnemyKind) -> Self {
        Self {
            x,
            y,
            kind,
            hp: kind.max_hp(),
            state: AiState::Idle,
            facing: (1.0, 0.0),
            attack_timer: 0.0,
            path_timer: 0.0,
            path: Vec::new(),
            wander: 0.0,
        }
    }

    pub fn alive(&self) -> bool {
        self.hp > 0.0
    }

    pub fn take_damage(&mut self, dmg: f32) -> bool {
        if !self.alive() {
            return false;
        }
        self.hp -= dmg;
        if self.hp <= 0.0 {
            self.hp = 0.0;
        }
        true
    }

    /// Drops from a dead slime.
    pub fn drops(&self) -> Vec<ItemKind> {
        match self.kind {
            EnemyKind::Slime => vec![ItemKind::Food],
        }
    }

    /// One AI tick. `player` is the target; `is_blocked` mirrors player
    /// collision (tiles + structures + other enemies). Returns Some if the
    /// enemy is close enough to land a hit this tick.
    pub fn update(
        &mut self,
        player: (f32, f32),
        dt: f32,
        mut is_blocked: impl FnMut(i32, i32) -> bool,
    ) -> Option<f32> {
        self.attack_timer = (self.attack_timer - dt).max(0.0);
        let d = (player.0 - self.x)
            .abs()
            .max((player.1 - self.y).abs());

        if d <= ATTACK_RANGE {
            self.state = AiState::Attack;
            if self.attack_timer <= 0.0 {
                self.attack_timer = 0.8;
                self.facing = normalize(player.0 - self.x, player.1 - self.y);
                return Some(self.kind.damage());
            }
            return None;
        }

        if d <= AGGRO_RANGE {
            self.state = AiState::Chase;
            self.path_timer -= dt;
            if self.path_timer <= 0.0 || self.path.is_empty() {
                self.path = astar(
                    (self.x.floor() as i32, self.y.floor() as i32),
                    (player.0.floor() as i32, player.1.floor() as i32),
                    &mut is_blocked,
                );
                self.path_timer = REPLAN_INTERVAL;
            }
            if let Some(&(tx, ty)) = self.path.first() {
                let to = (tx as f32 + 0.5, ty as f32 + 0.5);
                let (dx, dy) = normalize(to.0 - self.x, to.1 - self.y);
                self.facing = (dx, dy);
                self.x += dx * ENEMY_SPEED * dt;
                self.y += dy * ENEMY_SPEED * dt;
                if (self.x.floor() as i32, self.y.floor() as i32) == (tx, ty) {
                    self.path.remove(0);
                }
            }
            return None;
        }

        // idle: gentle drift back toward the spawn tile
        self.state = AiState::Idle;
        self.wander = (self.wander - dt).max(0.0);
        if self.wander <= 0.0 {
            self.wander = 1.0 + (self.x * 7.13 + self.y * 3.71).fract().abs() * 2.0;
        }
        return None;
    }
}

fn normalize(dx: f32, dy: f32) -> (f32, f32) {
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        (0.0, 0.0)
    } else {
        (dx / len, dy / len)
    }
}

/// A* over the tile grid. Returns a path from `start` toward `goal`
/// (excluding the goal tile itself, which may be the player's tile).
/// Falls back to the empty path if unreachable.
pub fn astar(
    start: (i32, i32),
    goal: (i32, i32),
    is_blocked: &mut dyn FnMut(i32, i32) -> bool,
) -> Vec<(i32, i32)> {
    if start == goal {
        return Vec::new();
    }
    let h = |(x, y): (i32, i32)| (x - goal.0).abs() + (y - goal.1).abs();
    let mut open: BinaryHeap<(i32, i32, i32)> = BinaryHeap::new();
    let mut came: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g: HashMap<(i32, i32), i32> = HashMap::new();
    open.push((0, start.0, start.1));
    g.insert(start, 0);
    let mut best_goal: Option<(i32, i32)> = None;
    let mut best_d = i32::MAX;
    let mut budget = 4000;

    while budget > 0 {
        budget -= 1;
        let Some((_neg_f, x, y)) = open.pop() else {
            break;
        };
        let node = (x, y);
        let cost = g[&node];
        let dist = (x - goal.0).abs() + (y - goal.1).abs();
        if dist < best_d {
            best_d = dist;
            best_goal = Some(node);
        }
        if node == goal {
            break;
        }
        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let next = (x + dx, y + dy);
            if is_blocked(next.0, next.1) {
                continue;
            }
            let ng = cost + 1;
            if ng < *g.get(&next).unwrap_or(&i32::MAX) {
                g.insert(next, ng);
                came.insert(next, node);
                open.push((-(ng + h(next)), next.0, next.1));
            }
        }
    }

    let Some(mut node) = best_goal else {
        return Vec::new();
    };
    let mut path = Vec::new();
    while let Some(&prev) = came.get(&node) {
        path.push(node);
        node = prev;
    }
    path.reverse();
    path
}

/// Stateless enemy placement: roughly 1/23 of Swamp tiles carry a slime
/// spawner (and a few in the open at night later). Enemies are session
/// entities with persistent hp (EnemyRegistry).
pub fn spawner_on(tx: i32, ty: i32, tile: TileKind) -> Option<EnemyKind> {
    let h = tx.wrapping_mul(73856093) ^ ty.wrapping_mul(19349663) ^ 0x51ab_ce0d;
    match tile {
        TileKind::Swamp if h.rem_euclid(23) == 0 => Some(EnemyKind::Slime),
        _ => None,
    }
}

/// Session enemy registry: keeps per-spawner enemy state (alive hp or
/// respawn timer) keyed by spawn tile.
#[derive(Debug, Default)]
pub struct EnemyRegistry {
    enemies: HashMap<(i32, i32), Enemy>,
    respawn: HashMap<(i32, i32), f32>,
}

impl EnemyRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_respawn_pending(&self, tx: i32, ty: i32) -> bool {
        self.respawn.get(&(tx, ty)).is_some_and(|t| *t > 0.0)
    }

/// Gets a live enemy for the spawn tile, creating it on first contact or
/// resurrecting it after its respawn timer elapses (default: 15s).
pub fn get(&mut self, tx: i32, ty: i32, kind: EnemyKind, dt: f32) -> Option<&mut Enemy> {
    if let Some(rt) = self.respawn.get_mut(&(tx, ty)) {
        *rt -= dt;
        if *rt <= 0.0 {
            self.respawn.remove(&(tx, ty));
        } else {
            return None;
        }
    }
    if !self.enemies.contains_key(&(tx, ty)) {
        self.enemies
            .insert((tx, ty), Enemy::new(tx as f32 + 0.5, ty as f32 + 0.5, kind));
    }
    self.enemies.get_mut(&(tx, ty))
}

    pub fn enemies_mut(&mut self) -> impl Iterator<Item = &mut Enemy> {
        self.enemies.values_mut()
    }

    /// Mutable enemies with their spawn-tile keys (needed to resolve kills).
    pub fn iter_mut_with_key(&mut self) -> impl Iterator<Item = ((i32, i32), &mut Enemy)> {
        self.enemies.iter_mut().map(|(&k, e)| (k, e))
    }

    pub fn enemies(&self) -> impl Iterator<Item = &Enemy> {
        self.enemies.values()
    }

    pub fn count(&self) -> usize {
        self.enemies.len()
    }

    /// Marks a spawner dead and starts its respawn timer.
    pub fn kill(&mut self, tx: i32, ty: i32, respawn_s: f32) {
        self.enemies.remove(&(tx, ty));
        self.respawn.insert((tx, ty), respawn_s);
    }
}

/// Collects live enemies in the given tile range.
pub fn enemies_in_range<'a>(
    enemies: impl Iterator<Item = &'a Enemy>,
    cx: f32,
    cy: f32,
    range: f32,
) -> Vec<&'a Enemy> {
    enemies
        .filter(|e| (e.x - cx).abs().max((e.y - cy).abs()) <= range)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{ChunkCache, WorldGen, tile_at};

    fn world() -> (WorldGen, ChunkCache) {
        (WorldGen::new(1337), ChunkCache::new(8))
    }

    fn blocked<'a>(world: &'a WorldGen, cache: &'a mut ChunkCache) -> impl FnMut(i32, i32) -> bool + 'a {
        move |tx, ty| !tile_at(world, cache, tx, ty).walkable()
    }

    #[test]
    fn astar_finds_path_around_obstacle() {
        let mut is_blocked = |tx: i32, ty: i32| tx == 3 && (0..=2).contains(&ty);
        let path = astar((2, 1), (4, 1), &mut is_blocked);
        assert!(path.contains(&(2, 2)), "should detour around the wall: {path:?}");
        assert!(path.contains(&(4, 2)) || path.contains(&(4, 1)), "should reach the goal: {path:?}");
    }

    #[test]
    fn astar_empty_when_unreachable() {
        let mut is_blocked = |tx: i32, _ty: i32| tx != 0 && tx != 5;
        let path = astar((0, 0), (10, 0), &mut is_blocked);
        assert!(path.is_empty(), "should fail cleanly: {path:?}");
    }

    #[test]
    fn astar_shortest_around_wall() {
        // wall occupies x=1, rows 0..=4 — the shortest detour is 8 steps
        let mut is_blocked = |tx: i32, ty: i32| tx == 1 && (0..=4).contains(&ty);
        let path = astar((0, 2), (2, 2), &mut is_blocked);
        assert_eq!(path.len(), 8, "detour around the wall: {path:?}");
        assert!(!path.contains(&(1, 2)), "must not pass through the wall");
    }

    #[test]
    fn slime_attacks_in_range() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Slime);
        let mut blocked = blocked(&w, &mut c);
        let hit = e.update((6.0, 6.0), 0.05, &mut blocked);
        assert!(hit.is_some(), "adjacent slime must hit");
        assert_eq!(e.state, AiState::Attack);
    }

    #[test]
    fn slime_chases_in_aggro_range() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Slime);
        let mut blocked = blocked(&w, &mut c);
        let start = (e.x, e.y);
        for _ in 0..20 {
            e.update((10.5, 5.5), 0.05, &mut blocked);
        }
        assert_eq!(e.state, AiState::Chase);
        assert!(
            (e.x - start.0).abs() > 0.01 || (e.y - start.1).abs() > 0.01,
            "chasing slime must move"
        );
    }

    #[test]
    fn slime_idles_out_of_aggro() {
        let (w, mut c) = world();
        let mut e = Enemy::new(0.5, 0.5, EnemyKind::Slime);
        let mut blocked = blocked(&w, &mut c);
        for _ in 0..10 {
            e.update((40.5, 40.5), 0.05, &mut blocked);
        }
        assert_eq!(e.state, AiState::Idle);
    }

    #[test]
    fn damage_and_death() {
        let mut e = Enemy::new(0.5, 0.5, EnemyKind::Slime);
        let mut taken = 0.0;
        while e.alive() {
            e.take_damage(4.0);
            taken += 1.0;
        }
        assert!(taken >= 3.0, "12 hp / 4 dmg = 3 hits");
        assert_eq!(e.drops(), vec![ItemKind::Food]);
    }

    #[test]
    fn registry_respawns_after_timer() {
        let mut reg = EnemyRegistry::new();
        reg.enemies
            .insert((3, 3), Enemy::new(3.5, 3.5, EnemyKind::Slime));
        reg.kill(3, 3, 5.0);
        assert!(reg.get(3, 3, EnemyKind::Slime, 1.0).is_none(), "dead until timer");
        assert!(reg.get(3, 3, EnemyKind::Slime, 4.5).is_some(), "respawned after timer");
    }

    #[test]
    fn registry_creates_enemy_on_first_get() {
        let mut reg = EnemyRegistry::new();
        let e = reg.get(3, 3, EnemyKind::Slime, 0.0);
        assert!(e.is_some(), "first get must spawn the enemy");
        assert_eq!(reg.count(), 1);
        assert!(!reg.is_respawn_pending(3, 3));
    }

    #[test]
    fn spawner_only_on_swamp() {
        assert!(spawner_on(0, 0, TileKind::Grass).is_none());
        assert!(spawner_on(0, 0, TileKind::Stone).is_none());
        assert!(spawner_on(0, 0, TileKind::Forest).is_none());
    }

    #[test]
    fn swamp_has_some_spawners_in_a_window() {
        let n = (-32..32)
            .flat_map(|tx| (-32..32).map(move |ty| (tx, ty)))
            .filter(|&(tx, ty)| spawner_on(tx, ty, TileKind::Swamp).is_some())
            .count();
        assert!(n > 20, "expected ~1/14 of 4096, got {n}");
    }
}