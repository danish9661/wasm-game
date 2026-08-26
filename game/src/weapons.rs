use serde::{Deserialize, Serialize};

/// Weapons the player can find in chests or as enemy drops. They change the
/// feel of combat: damage, reach, swing cadence, and whether the attack is a
/// ranged volley. Fists are the always-available fallback.
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
            WeaponKind::Sword => 11.0,
            WeaponKind::Axe => 14.0,
            WeaponKind::Spear => 9.0,
            WeaponKind::Hammer => 18.0,
            WeaponKind::Bow => 10.0,
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
            WeaponKind::Axe => 0.55,
            WeaponKind::Spear => 0.42,
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
}
