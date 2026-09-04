use serde::{Deserialize, Serialize};

/// Weapons the player can find in chests or as enemy drops. They change the
/// feel of combat: damage, reach, swing cadence, and whether the attack is a
/// ranged volley. Fists are the always-available fallback.
///
/// Balance (base DPS = damage / cooldown, before enchant/weak-points):
/// | weapon   | dmg | cd   | dps  | reach  | niche                              |
/// |----------|-----|------|------|--------|------------------------------------|
/// | Fists    |   4 | 0.34 | 11.8 |    2.4 | fallback                           |
/// | Sword    |  10 | 0.30 | 33.3 |    3.2 | best sustained dps (duelist)       |
/// | Axe      |  15 | 0.50 | 30.0 |    3.0 | one-shots swarm (12-14 HP packs)   |
/// | Dagger   |   6 | 0.22 | 27.3 |    2.0 | blinding flurry + 2.5x backstabs   |
/// | Spear    |   9 | 0.38 | 23.7 |    4.4 | longest reach (safe kiting)        |
/// | Hammer   |  24 | 0.75 | 32.0 |    2.8 | burst king (fewest boss swings)    |
/// | Bow      |  12 | 0.60 | 20.0 | ranged | safe at range (ranged tax)         |
/// | Crossbow |  14 | 0.90 | 15.6 | ranged | piercing line shot (hits all)      |
/// | Mace     |  16 | 0.60 | 26.7 |    2.6 | anti-armor (1.5x vs stone/plate)    |
///
/// Sword wins sustained dps; hammer wins burst (fewest hits = least exposure)
/// plus stagger; axe clears 12-14 HP packs in one swing; dagger trades reach
/// for speed and unmatched backstabs; mace cracks armored foes; spear
/// outranges every contact attack; bow/xbow trade dps for safety, xbow
/// piercing ranks. Weak-points (1.5x) and enchant (+15%/level) stack on top.
///
/// NOTE: variants are bitmask-indexed into a `u16` (`unlocked`) — v1 saves
/// stored a `u8` and still load via the save compat shim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponKind {
    Fists,
    Sword,
    Axe,
    Spear,
    Hammer,
    Bow,
    Dagger,
    Crossbow,
    Mace,
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
            WeaponKind::Dagger => "Dagger",
            WeaponKind::Crossbow => "Crossbow",
            WeaponKind::Mace => "Mace",
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
            WeaponKind::Dagger => 6.0,
            WeaponKind::Crossbow => 14.0,
            WeaponKind::Mace => 16.0,
        }
    }

    /// Melee reach in tiles (bows ignore this and fire a projectile).
    pub fn reach(self) -> f32 {
        match self {
            WeaponKind::Fists => 2.4,
            WeaponKind::Sword => 3.2,
            WeaponKind::Axe => 3.0,
            WeaponKind::Spear => 4.4,
            WeaponKind::Hammer => 2.8,
            WeaponKind::Bow => 0.0,
            WeaponKind::Dagger => 2.0,
            WeaponKind::Crossbow => 0.0,
            WeaponKind::Mace => 2.6,
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
            WeaponKind::Dagger => 0.22,
            WeaponKind::Crossbow => 0.9,
            WeaponKind::Mace => 0.6,
        }
    }

    /// True for ranged weapons (fire an arrow instead of a melee swing).
    pub fn ranged(self) -> bool {
        matches!(self, WeaponKind::Bow | WeaponKind::Crossbow)
    }

    /// True for piercing shots: the bolt damages every foe in its path
    /// instead of stopping at the first.
    pub fn piercing(self) -> bool {
        matches!(self, WeaponKind::Crossbow)
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
            WeaponKind::Dagger => [0.45, 0.47, 0.52],
            WeaponKind::Crossbow => [0.50, 0.32, 0.18],
            WeaponKind::Mace => [0.72, 0.55, 0.28],
        }
    }

    /// All findable weapons (everything except the default Fists).
    pub fn findable() -> &'static [WeaponKind] {
        &[
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Dagger,
            WeaponKind::Spear,
            WeaponKind::Hammer,
            WeaponKind::Mace,
            WeaponKind::Bow,
            WeaponKind::Crossbow,
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
    /// weapon the higher the roll needed. Daggers are common, crossbows and
    /// maces rare.
    pub fn roll_drop_with(roll: u32) -> Option<WeaponKind> {
        if roll >= 70 {
            return None;
        }
        match roll % 12 {
            0 | 1 => Some(WeaponKind::Sword),
            2 => Some(WeaponKind::Axe),
            3 | 4 => Some(WeaponKind::Dagger),
            5 => Some(WeaponKind::Spear),
            6 => Some(WeaponKind::Hammer),
            7 | 8 => Some(WeaponKind::Bow),
            9 => Some(WeaponKind::Mace),
            10 => Some(WeaponKind::Crossbow),
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
            6 => WeaponKind::Dagger,
            7 => WeaponKind::Crossbow,
            8 => WeaponKind::Mace,
            _ => WeaponKind::Fists,
        }
    }

    /// Cycle order for the P key / weapon bar: story order, ranged tier last.
    pub fn cycle_order() -> &'static [WeaponKind] {
        &[
            WeaponKind::Fists,
            WeaponKind::Dagger,
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Spear,
            WeaponKind::Hammer,
            WeaponKind::Mace,
            WeaponKind::Bow,
            WeaponKind::Crossbow,
        ]
    }

    /// Resource cost to craft this weapon at an anvil. Fists can't be crafted.
    pub fn craft_cost(self) -> Option<(u32, u32, u32)> {
        // (wood, stone, herb)
        match self {
            WeaponKind::Fists => None,
            WeaponKind::Sword => Some((5, 3, 0)),
            WeaponKind::Axe => Some((6, 1, 0)),
            WeaponKind::Dagger => Some((4, 2, 0)),
            WeaponKind::Spear => Some((4, 2, 0)),
            WeaponKind::Hammer => Some((2, 8, 0)),
            WeaponKind::Bow => Some((5, 0, 1)),
            WeaponKind::Crossbow => Some((7, 3, 1)),
            WeaponKind::Mace => Some((3, 6, 0)),
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
        // Sword: best sustained dps. Hammer second (burst). Axe close third,
        // dagger fourth (reach tax), mace fifth, then spear, bow, crossbow,
        // fists.
        let mut v = [
            WeaponKind::Fists,
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Spear,
            WeaponKind::Hammer,
            WeaponKind::Bow,
            WeaponKind::Dagger,
            WeaponKind::Crossbow,
            WeaponKind::Mace,
        ];
        v.sort_by(|a, b| b.dps().partial_cmp(&a.dps()).unwrap());
        assert_eq!(
            v,
            [
                WeaponKind::Sword,
                WeaponKind::Hammer,
                WeaponKind::Axe,
                WeaponKind::Dagger,
                WeaponKind::Mace,
                WeaponKind::Spear,
                WeaponKind::Bow,
                WeaponKind::Crossbow,
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
                WeaponKind::Dagger.as_u8(),
                WeaponKind::Crossbow.as_u8(),
                WeaponKind::Mace.as_u8(),
            ],
            [0, 1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(WeaponKind::from_u8(99), WeaponKind::Fists);
    }

    #[test]
    fn backstab_pays_positioning() {
        use crate::combat::backstab_mult;
        // Victim facing +x; attacker directly behind (at -x).
        let behind = backstab_mult((-2.0, 0.0), (0.0, 0.0), (1.0, 0.0), WeaponKind::Sword);
        assert_eq!(behind, 1.5);
        let dagger = backstab_mult((-2.0, 0.0), (0.0, 0.0), (1.0, 0.0), WeaponKind::Dagger);
        assert_eq!(dagger, 2.5, "daggers live for the backstab");
        // Face-to-face: no bonus.
        let front = backstab_mult((2.0, 0.0), (0.0, 0.0), (1.0, 0.0), WeaponKind::Dagger);
        assert_eq!(front, 1.0);
        // Degenerate geometry never pays.
        assert_eq!(backstab_mult((0.0, 0.0), (0.0, 0.0), (1.0, 0.0), WeaponKind::Sword), 1.0);
    }

    #[test]
    fn mace_cracks_armor() {
        for foe in [
            EnemyKind::Ogre,
            EnemyKind::FrostGolem,
            EnemyKind::Colossus,
            EnemyKind::Stoneslinger,
            EnemyKind::Raider,
        ] {
            assert_eq!(foe.weakness_to(WeaponKind::Mace), 1.5, "{foe:?} is armored");
        }
        assert_eq!(EnemyKind::Slime.weakness_to(WeaponKind::Mace), 1.0);
        // Mace sits between dagger and spear on raw dps.
        assert!(WeaponKind::Mace.dps() < WeaponKind::Dagger.dps());
        assert!(WeaponKind::Mace.dps() > WeaponKind::Spear.dps());
        // ...but out-damages the sword against plate.
        assert!(16.0 * 1.5 > WeaponKind::Sword.damage());
    }

    #[test]
    fn crossbow_bolts_piece_but_bows_dont() {
        assert!(WeaponKind::Crossbow.piercing());
        assert!(WeaponKind::Crossbow.ranged());
        assert!(!WeaponKind::Bow.piercing());
        let bolt = crate::combat::Arrow::bolt(0.0, 0.0, 1.0, 0.0, 14.0);
        assert!(bolt.piercing && bolt.from_player && bolt.tagged);
        assert!(bolt.life < crate::combat::Arrow::new(0.0, 0.0, 1.0, 0.0).life);
    }
}
