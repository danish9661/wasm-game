use crate::enemy::Enemy;
use crate::player::Player;

/// Melee: one swing per press, hits every enemy within reach (movement is
/// diagonal-only, so a swing is an omnidirectional swipe).
pub const SWING_REACH: f32 = 3.0;
pub const SWING_DAMAGE: f32 = 5.0;

/// Arrow: speed in tiles/second, damage on hit.
pub const ARROW_SPEED: f32 = 14.0;
pub const ARROW_DAMAGE: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arrow {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub life: f32,
    /// Damage applied on hit. Player arrows carry the equipped weapon's damage;
    /// enemy arrows use the base `ARROW_DAMAGE`.
    pub damage: f32,
    /// true = fired by the player (hits enemies); false = fired by an enemy
    /// (hits the player).
    pub from_player: bool,
    /// true = loosed by the player (tags the victim for XP/quest credit).
    /// Turret and trap arrows leave this false: shared spoils, no credit.
    pub tagged: bool,
}

impl Arrow {
    pub fn new(x: f32, y: f32, dx: f32, dy: f32) -> Self {
        let len = (dx * dx + dy * dy).sqrt();
        let (dx, dy) = if len == 0.0 { (1.0, 0.0) } else { (dx / len, dy / len) };
        Self {
            x,
            y,
            dx,
            dy,
            life: 3.0,
            damage: ARROW_DAMAGE,
            from_player: true,
            tagged: true,
        }
    }

    /// Enemy-fired arrow (damages the player).
    pub fn enemy(x: f32, y: f32, dx: f32, dy: f32) -> Self {
        let mut a = Self::new(x, y, dx, dy);
        a.from_player = false;
        a.tagged = false;
        a
    }

    /// Advance one tick; returns true while the arrow is still alive.
    pub fn step(&mut self, dt: f32) -> bool {
        self.life -= dt;
        if self.life <= 0.0 {
            return false;
        }
        self.x += self.dx * ARROW_SPEED * dt;
        self.y += self.dy * ARROW_SPEED * dt;
        true
    }
}

/// Returns enemies hit by a melee swing: every enemy within `reach` tiles
/// (one swing hits all adjacent enemies — no facing requirement).
pub fn swing_hits<'a>(
    player: &Player,
    enemies: impl Iterator<Item = &'a mut Enemy>,
    reach: f32,
) -> Vec<&'a mut Enemy> {
    let mut hits = Vec::new();
    for e in enemies {
        let to = (e.x - player.x, e.y - player.y);
        let dist = (to.0 * to.0 + to.1 * to.1).sqrt();
        if dist > reach || dist < 0.001 {
            continue;
        }
        hits.push(e);
    }
    hits
}

/// Returns the first enemy the arrow collides with this tick (hit radius
/// 0.8 tiles), removing it from the world via the returned index.
pub fn arrow_hits<'a>(arrow: &Arrow, mut enemies: impl Iterator<Item = &'a Enemy>) -> Option<&'a Enemy> {
    enemies.find(|e| {
        let dx = e.x - arrow.x;
        let dy = e.y - arrow.y;
        dx * dx + dy * dy <= 0.8 * 0.8
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemy::{Enemy, EnemyKind};
    use crate::player::Player;

    #[test]
    fn swing_hits_everyone_in_reach() {
        let p = Player::new(0.0, 0.0);
        let mut near = Enemy::new(1.0, 0.0, EnemyKind::Slime);
        let mut far = Enemy::new(3.2, 0.0, EnemyKind::Slime);
        let mut behind = Enemy::new(-1.0, 0.0, EnemyKind::Slime);
        let enemies = [&mut near, &mut far, &mut behind];
        let hits = swing_hits(&p, enemies.into_iter(), SWING_REACH);
        assert_eq!(hits.len(), 2, "near + behind in reach: {hits:?}");
    }

    #[test]
    fn swing_misses_out_of_reach() {
        let p = Player::new(0.0, 0.0);
        let mut e = Enemy::new(3.2, 0.0, EnemyKind::Slime);
        let hits = swing_hits(&p, std::iter::once(&mut e), SWING_REACH);
        assert!(hits.is_empty(), "3.2 > 3.0 reach");
    }

    #[test]
    fn swing_hits_behind_player() {
        let p = Player::new(0.0, 0.0);
        let mut e = Enemy::new(0.5, 0.5, EnemyKind::Slime);
        let hits = swing_hits(&p, std::iter::once(&mut e), SWING_REACH);
        assert_eq!(hits.len(), 1, "swipe is omnidirectional");
    }

    #[test]
    fn arrow_travels_and_ages_out() {
        let mut a = Arrow::new(0.0, 0.0, 1.0, 0.0);
        let mut t = 0.0;
        while a.step(0.1) && t < 5.0 {
            t += 0.1;
        }
        assert!(a.life <= 0.0, "arrow must expire");
        assert!(t < 4.0, "arrow should expire by ~3s");
    }

    #[test]
    fn arrow_hits_enemy_en_route() {
        let mut a = Arrow::new(0.0, 0.0, 1.0, 0.0);
        let e = Enemy::new(2.0, 0.0, EnemyKind::Slime);
        let mut hit = false;
        for _ in 0..60 {
            if !a.step(0.05) {
                break;
            }
            if arrow_hits(&a, std::iter::once(&e)).is_some() {
                hit = true;
                break;
            }
        }
        assert!(hit, "arrow should intersect the enemy");
    }

    // Full round-trip of the core combat loop, in pure Rust (no GPU needed):
    // player swing damages an in-reach enemy, and enemy contact damages the
    // player while respecting the hurt-timer i-frames.
    #[test]
    fn player_swing_damages_enemy() {
        let mut p = Player::new(0.0, 0.0);
        let mut e = Enemy::new(0.8, 0.0, EnemyKind::Skeleton);
        let max = e.hp;
        let dmg = p.weapon_damage();
        for hit in swing_hits(&p, std::iter::once(&mut e), SWING_REACH) {
            hit.take_damage(dmg);
        }
        assert!(e.hp < max, "enemy must lose hp from the swing");
    }

    #[test]
    fn enemy_contact_respects_iframes() {
        let mut p = Player::new(0.0, 0.0);
        let start = p.hp;
        assert!(p.take_damage(12.0), "first hit should land");
        assert!(!p.take_damage(12.0), "i-frames must block the immediate second hit");
        assert!((start - p.hp - 12.0).abs() < 1e-3, "only one hit's damage applied");
        // Once the hurt-timer expires, damage resumes.
        p.hurt_timer = 0.0;
        assert!(p.take_damage(12.0), "damage resumes after i-frames");
    }
}