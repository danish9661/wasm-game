//! Armour and equipment slots (the defensive half of the progression tree).
//!
//! Weapons live on `Player::weapon`; tools live in `Player::tool_hp`. Armour is
//! a tiny fixed-size array of optional kinds, so it stays `Copy`, serialises
//! flat into saves, and can be passed around the deterministic simulation
//! without allocations.

use serde::{Deserialize, Serialize};

use crate::items::{Inventory, ItemKind};

/// Equipment slots, in HUD order.
pub const SLOT_NAMES: [&str; 4] = ["Helmet", "Armor", "Shield", "Accessory"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmorKind {
    LeatherCap,
    IronHelm,
    IronPlateArmor,
    WarmCloak,
    WoodShield,
    CrownRing,
}

impl ArmorKind {
    /// Stable id for UI buttons / network messages.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(ArmorKind::LeatherCap),
            1 => Some(ArmorKind::IronHelm),
            2 => Some(ArmorKind::IronPlateArmor),
            3 => Some(ArmorKind::WarmCloak),
            4 => Some(ArmorKind::WoodShield),
            5 => Some(ArmorKind::CrownRing),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ArmorKind::LeatherCap => "Leather Cap",
            ArmorKind::IronHelm => "Iron Helm",
            ArmorKind::IronPlateArmor => "Iron Plate Armor",
            ArmorKind::WarmCloak => "Warm Cloak",
            ArmorKind::WoodShield => "Wood Shield",
            ArmorKind::CrownRing => "Crown Ring",
        }
    }

    pub fn slot(self) -> usize {
        match self {
            ArmorKind::LeatherCap | ArmorKind::IronHelm => 0,
            ArmorKind::IronPlateArmor | ArmorKind::WarmCloak => 1,
            ArmorKind::WoodShield => 2,
            ArmorKind::CrownRing => 3,
        }
    }

    /// Fraction of incoming damage removed, before blocking (0..1).
    pub fn defense(self) -> f32 {
        match self {
            ArmorKind::LeatherCap => 0.06,
            ArmorKind::IronHelm => 0.16,
            ArmorKind::IronPlateArmor => 0.28,
            ArmorKind::WarmCloak => 0.08,
            ArmorKind::WoodShield => 0.22,
            ArmorKind::CrownRing => 0.04,
        }
    }

    /// Fraction of cold/heat damage removed (0..1). Cloaks matter in the tundra.
    pub fn insulation(self) -> f32 {
        match self {
            ArmorKind::LeatherCap => 0.05,
            ArmorKind::IronHelm => 0.02,
            ArmorKind::IronPlateArmor => 0.12,
            ArmorKind::WarmCloak => 0.35,
            ArmorKind::WoodShield => 0.0,
            ArmorKind::CrownRing => 0.15,
        }
    }

    /// Anvil recipe. Gems and plates gate the late tiers behind progression.
    pub fn cost(self) -> &'static [(ItemKind, u32)] {
        match self {
            ArmorKind::LeatherCap => &[(ItemKind::Herb, 3), (ItemKind::Wood, 2)],
            ArmorKind::IronHelm => &[(ItemKind::Iron, 3), (ItemKind::IronPlate, 1)],
            ArmorKind::IronPlateArmor => &[(ItemKind::IronPlate, 3), (ItemKind::Gem, 1)],
            ArmorKind::WarmCloak => &[(ItemKind::Herb, 5), (ItemKind::Food, 2)],
            ArmorKind::WoodShield => &[(ItemKind::Wood, 5), (ItemKind::Iron, 1)],
            ArmorKind::CrownRing => &[(ItemKind::Gem, 4), (ItemKind::Gold, 5)],
        }
    }
}

/// Four equipped pieces. Empty by default, so old saves load unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub slots: [Option<ArmorKind>; 4],
}

impl Equipment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn equipped(&self, kind: ArmorKind) -> bool {
        self.slots[kind.slot()] == Some(kind)
    }

    pub fn equip(&mut self, kind: ArmorKind) {
        self.slots[kind.slot()] = Some(kind);
    }

    /// Combined damage multiplier: pieces stack multiplicatively but never
    /// reduce a hit below 35% (bosses must still hurt).
    pub fn damage_multiplier(&self) -> f32 {
        let mut m = 1.0f32;
        for slot in self.slots.iter().flatten() {
            m *= 1.0 - slot.defense();
        }
        m.max(0.35)
    }

    /// Combined insulation against cold/heat zones (capped at 75%).
    pub fn insulation(&self) -> f32 {
        let mut v = 0.0f32;
        for slot in self.slots.iter().flatten() {
            v += slot.insulation();
        }
        v.min(0.75)
    }

    pub fn total_defense(&self) -> f32 {
        1.0 - self.damage_multiplier()
    }

    /// Craft a piece at an anvil: consumes its recipe and equips it. Re-crafting
    /// an identical piece is a no-op (no wasted materials).
    pub fn craft(&mut self, kind: ArmorKind, inv: &mut Inventory) -> bool {
        if self.equipped(kind) {
            return false;
        }
        for (item, n) in kind.cost() {
            if inv.count(*item) < *n {
                return false;
            }
        }
        for (item, n) in kind.cost() {
            inv.remove(*item, *n);
        }
        self.equip(kind);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crafting_consumes_and_equips() {
        let mut inv = Inventory::new();
        let mut eq = Equipment::new();
        assert!(!eq.craft(ArmorKind::IronHelm, &mut inv));
        inv.add(ItemKind::Iron, 3);
        inv.add(ItemKind::IronPlate, 1);
        assert!(eq.craft(ArmorKind::IronHelm, &mut inv));
        assert!(eq.equipped(ArmorKind::IronHelm));
        assert_eq!(inv.count(ItemKind::Iron), 0);
        assert!(!eq.craft(ArmorKind::IronHelm, &mut inv), "no duplicate craft");
    }

    #[test]
    fn armour_stacks_with_a_floor() {
        let mut eq = Equipment::new();
        for k in [
            ArmorKind::IronHelm,
            ArmorKind::IronPlateArmor,
            ArmorKind::WoodShield,
            ArmorKind::CrownRing,
        ] {
            eq.equip(k);
        }
        assert!(eq.damage_multiplier() >= 0.35);
        assert!(eq.damage_multiplier() < 0.6);
        assert!(eq.insulation() <= 0.75);
    }
}
