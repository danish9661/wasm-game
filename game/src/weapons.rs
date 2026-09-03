use serde::{Deserialize, Serialize};

/// Weapons the player can find in chests or as enemy drops. They change the
/// feel of combat: damage, reach, swing cadence, and whether the attack is a
/// ranged volley. Fists are the always-available fallback.
///
/// Balance (base DPS = damage / cooldown, before enchant/weak-points):
/// | weapon | dmg | cd   | dps  | reach  | niche                              |
/// |--------|-----|------|------|--------|------------------------------------|
/// | Fists  |   4 | 0.34 | 11.8 |    2.4 | fallback                           |
/// | Sword  |  10 | 0.30 | 33.3 |    3.2 | best sustained dps (duelist)       |
/// | Axe    |  15 | 0.50 | 30.0 |    3.0 | one-shots swarm (12-14 HP packs)   |
/// | Spear  |   9 | 0.38 | 23.7 |    4.4 | longest reach (safe kiting)        |
/// | Hammer |  24 | 0.75 | 32.0 |    2.8 | burst king (fewest boss swings)    |
/// | Bow    |  12 | 0.60 | 20.0 | ranged | safe at range (ranged tax)         |
///
/// Sword wins sustained dps; hammer wins burst (fewest hits = least exposure)
/// plus stagger; axe clears 12-14 HP packs in one swing; spear outranges
/// every contact attack; bow trades dps for safety. Weak-points (1.5x) and
/// enchant (+15%/level) stack on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponKind {
    Fists,
    Sword,
    Axe,
    Spear,
    Hammer,
    Bow,
}

impl WeaponKind {
    /// Human-readable label shown in the HUD / on pickup.
    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Fists => "Fists",
            WeaponKind::Sword => "Sword",
            WeaponKind::Axe => "Axe",
            WeaponKind::Spear => "Spear",
            WeaponKind::Hammer => "Hammer",
            WeaponKind::Bow => "Bow",
        }
    }

    /// Base damage per hit.
    pub fn damage(self) -> f32 {
        match self {
            WeaponKind::Fists => 4.0,
            WeaponKind::Sword => 10.0,
            WeaponKind::Axe => 15.0,
            WeaponKind::Spear => 9.0,
            WeaponKind::Hammer => 24.0,
            WeaponKind::Bow => 12.0,
        }
    }

    /// Melee reach in tiles (bow ignores this and fires a projectile).
    pub fn reach(self) -> f32 {
        match self {
            WeaponKind::Fists => 2.4,
            WeaponKind::Sword => 3.2,
            WeaponKind::Axe => 3.0,
            WeaponKind::Spear => 4.4,
            WeaponKind::Hammer => 2.8,
            WeaponKind::Bow => 0.0,
        }
    }

    /// Seconds between swings (lower = faster).
    pub fn cooldown(self) -> f32 {
        match self {
            WeaponKind::Fists => 0.34,
            WeaponKind::Sword => 0.30,
            WeaponKind::Axe => 0.50,
            WeaponKind::Spear => 0.38,
            WeaponKind::Hammer => 0.75,
            WeaponKind::Bow => 0.6,
        }
    }

    /// True for ranged weapons (fire an arrow instead of a melee swing).
    pub fn ranged(self) -> bool {
        matches!(self, WeaponKind::Bow)
    }

    /// Arrow speed multiplier for ranged weapons (1.0 = base arrow speed).
    pub fn projectile_speed(self) -> f32 {
        match self {
            WeaponKind::Bow => 1.0,
            _ => 1.0,
        }
    }

    /// Display tint for the weapon / its ground pickup.
    pub fn color(self) -> [f32; 3] {
        match self {
            WeaponKind::Fists => [0.8, 0.7, 0.55],
            WeaponKind::Sword => [0.85, 0.88, 0.95],
            WeaponKind::Axe => [0.75, 0.55, 0.30],
            WeaponKind::Spear => [0.80, 0.72, 0.40],
            WeaponKind::Hammer => [0.70, 0.70, 0.78],
            WeaponKind::Bow => [0.55, 0.40, 0.22],
        }
    }

    /// All findable weapons (everything except the default Fists).
    pub fn findable() -> &'static [WeaponKind] {
        &[
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Spear,
            WeaponKind::Hammer,
            WeaponKind::Bow,
        ]
    }

    /// Roll a weapon drop. Common ones are more likely than the heavy/rare
    /// Hammer. Returns None ~70% of the time so drops stay special.
    pub fn roll_drop() -> Option<WeaponKind> {
        // Deterministic-ish pseudo-random from the caller's entropy isn't
        // available here; use a simple time-free hash of a counter via the
        // caller. We instead return a weighted pick driven by a 0..100 roll
        // passed in. See `roll_drop_with`.
        None
    }

    /// Roll a drop given a `roll` in 0..100. ~30% chance to drop; the rarer the
    /// weapon the higher the roll needed.
    pub fn roll_drop_with(roll: u32) -> Option<WeaponKind> {
        if roll >= 70 {
            return None;
        }
        match roll % 10 {
            0 | 1 => Some(WeaponKind::Sword),
            2 | 3 => Some(WeaponKind::Axe),
            4 | 5 => Some(WeaponKind::Spear),
            6 => Some(WeaponKind::Hammer),
            _ => Some(WeaponKind::Bow),
        }
    }

    /// Stable index for serialization (save files / network messages).
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Inverse of `as_u8`; returns Fists for any out-of-range value.
    pub fn from_u8(v: u8) -> WeaponKind {
        match v {
            0 => WeaponKind::Fists,
            1 => WeaponKind::Sword,
            2 => WeaponKind::Axe,
            3 => WeaponKind::Spear,
            4 => WeaponKind::Hammer,
            5 => WeaponKind::Bow,
            _ => WeaponKind::Fists,
        }
    }

    /// Resource cost to craft this weapon at an anvil. Fists can't be crafted.
    pub fn craft_cost(self) -> Option<(u32, u32, u32)> {
        // (wood, stone, herb)
        match self {
            WeaponKind::Fists => None,
            WeaponKind::Sword => Some((5, 3, 0)),
            WeaponKind::Axe => Some((6, 1, 0)),
            WeaponKind::Spear => Some((4, 2, 0)),
            WeaponKind::Hammer => Some((2, 8, 0)),
            WeaponKind::Bow => Some((5, 0, 1)),
        }
    }

    /// Sustained base DPS (damage / cooldown). Used by balance tests; the
    /// game itself applies damage per discrete swing, never this average.
    pub fn dps(self) -> f32 {
        self.damage() / self.cooldown()
    }

    /// Swings to kill a `hp` pool (rounded up).
    pub fn swings_to_kill(self, hp: f32) -> u32 {
        (hp / self.damage()).ceil() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemy::EnemyKind;

    #[test]
    fn dps_order_matches_design() {
        // Sword: best sustained dps. Hammer second (burst). Axe close third.
        // Spear/bow trade dps for reach/safety. Fists last.
        let mut v = [
            WeaponKind::Fists,
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Spear,
            WeaponKind::Hammer,
            WeaponKind::Bow,
        ];
        v.sort_by(|a, b| b.dps().partial_cmp(&a.dps()).unwrap());
        assert_eq!(
            v,
            [
                WeaponKind::Sword,
                WeaponKind::Hammer,
                WeaponKind::Axe,
                WeaponKind::Spear,
                WeaponKind::Bow,
                WeaponKind::Fists,
            ],
            "dps order is the balance contract"
        );
    }

    #[test]
    fn axe_one_shots_swarm_packs() {
        // 12-14 HP packs (slime/wolf/spider) die to one axe swing, two sword.
        for hp in [12.0, 14.0] {
            assert_eq!(WeaponKind::Axe.swings_to_kill(hp), 1);
            assert_eq!(WeaponKind::Sword.swings_to_kill(hp), 2);
        }
    }

    #[test]
    fn hammer_needs_fewest_swings_vs_bosses() {
        // Burst king: fewest discrete hits vs 70-110 HP guardians (least
        // exposure + stagger), even though sword wins sustained dps.
        for boss in [
            EnemyKind::Boss,
            EnemyKind::ScorpionQueen,
            EnemyKind::ToadKing,
            EnemyKind::OceanLeviathan,
            EnemyKind::FrostGolem,
        ] {
            let hp = boss.max_hp();
            let hammer = WeaponKind::Hammer.swings_to_kill(hp);
            for w in [WeaponKind::Sword, WeaponKind::Axe, WeaponKind::Spear, WeaponKind::Bow] {
                assert!(
                    hammer <= w.swings_to_kill(hp),
                    "hammer must need fewest swings vs {boss:?}"
                );
            }
        }
    }

    #[test]
    fn spear_outreaches_every_melee() {
        for w in [WeaponKind::Fists, WeaponKind::Sword, WeaponKind::Axe, WeaponKind::Hammer] {
            assert!(WeaponKind::Spear.reach() > w.reach());
        }
        assert!(WeaponKind::Bow.ranged());
    }

    #[test]
    fn serialization_indices_are_stable() {
        // Save files + network messages persist these discriminants.
        assert_eq!(
            [
                WeaponKind::Fists.as_u8(),
                WeaponKind::Sword.as_u8(),
                WeaponKind::Axe.as_u8(),
                WeaponKind::Spear.as_u8(),
                WeaponKind::Hammer.as_u8(),
                WeaponKind::Bow.as_u8(),
            ],
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(WeaponKind::from_u8(99), WeaponKind::Fists);
    }
}
