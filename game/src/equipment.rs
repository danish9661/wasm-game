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

    /// Durability of a freshly forged piece. Higher tiers last longer, so the
    /// repair bill (iron/gold) is the real progression cost, not constant snaps.
    pub fn max_durability(self) -> u16 {
        match self {
            ArmorKind::LeatherCap => 40,
            ArmorKind::IronHelm => 70,
            ArmorKind::IronPlateArmor => 100,
            ArmorKind::WarmCloak => 50,
            ArmorKind::WoodShield => 60,
            ArmorKind::CrownRing => 80,
        }
    }

    /// Anvil repair bill. Iron and gold only (no inventory of spare pieces):
    /// the amount scales with the tier, so a broken plate is a real expense.
    pub fn repair_cost(self) -> &'static [(ItemKind, u32)] {
        match self {
            ArmorKind::LeatherCap => &[(ItemKind::Iron, 1)],
            ArmorKind::IronHelm => &[(ItemKind::Iron, 2)],
            ArmorKind::IronPlateArmor => &[(ItemKind::Iron, 2), (ItemKind::Gold, 1)],
            ArmorKind::WarmCloak => &[(ItemKind::Iron, 1), (ItemKind::Gold, 1)],
            ArmorKind::WoodShield => &[(ItemKind::Iron, 1)],
            ArmorKind::CrownRing => &[(ItemKind::Gold, 2)],
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

/// Serde fallback for saves written before durability existed: a missing
/// array means "fresh", so already-equipped pieces do not load as broken.
fn full_durability() -> [u16; 4] {
    [100; 4]
}

/// Four equipped pieces. Empty by default, so old saves load unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub slots: [Option<ArmorKind>; 4],
    /// Uses left per slot (parallel to `slots`). Equipping always fills it;
    /// hits wear it down and an anvil repair restores it. The serde fallback
    /// keeps pre-durability saves at full condition.
    #[serde(default = "full_durability")]
    pub durability: [u16; 4],
}

/// A worn piece still protects a little (35% of its rating) — broken armour is
/// a liability, not a naked character, so the difficulty curve stays smooth.
pub const BROKEN_ARMOR_EFFICIENCY: f32 = 0.35;

fn condition_factor(condition: f32) -> f32 {
    BROKEN_ARMOR_EFFICIENCY + (1.0 - BROKEN_ARMOR_EFFICIENCY) * condition.clamp(0.0, 1.0)
}

impl Equipment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn equipped(&self, kind: ArmorKind) -> bool {
        self.slots[kind.slot()] == Some(kind)
    }

    pub fn equip(&mut self, kind: ArmorKind) {
        let slot = kind.slot();
        self.slots[slot] = Some(kind);
        self.durability[slot] = kind.max_durability();
    }

    /// Condition of the piece in a slot in 0..1 (1 = pristine, 0 = broken).
    /// Empty slots read as 0 so HUD code can display '-' without special cases.
    pub fn condition(&self, slot: usize) -> f32 {
        match self.slots.get(slot).copied().flatten() {
            Some(kind) => {
                (self.durability[slot] as f32 / kind.max_durability() as f32).clamp(0.0, 1.0)
            }
            None => 0.0,
        }
    }

    /// Condition of a specific kind, or 0 when it is not equipped.
    pub fn condition_of(&self, kind: ArmorKind) -> f32 {
        if self.equipped(kind) {
            self.condition(kind.slot())
        } else {
            0.0
        }
    }

    /// Wear every equipped piece by one hit's worth. `damage` makes heavy
    /// blows (bosses) bite harder; wear is integer and saturates at zero, so
    /// it is deterministic and cannot underflow. Returns the slots that broke.
    pub fn wear(&mut self, damage: f32) -> [bool; 4] {
        let steps = (1.0 + damage / 12.0).max(1.0) as u16;
        let mut broke = [false; 4];
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.is_some() {
                let hp = &mut self.durability[i];
                let before = *hp;
                *hp = hp.saturating_sub(steps);
                broke[i] = before > 0 && *hp == 0;
            }
        }
        broke
    }

    /// Combined damage multiplier: pieces stack multiplicatively but never
    /// reduce a hit below 35% (bosses must still hurt). Each piece's rating is
    /// scaled by its condition, so worn gear stops protecting gradually.
    pub fn damage_multiplier(&self) -> f32 {
        let mut m = 1.0f32;
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(kind) = slot {
                m *= 1.0 - kind.defense() * condition_factor(self.condition(i));
            }
        }
        m.max(0.35)
    }

    /// Combined insulation against cold/heat zones (capped at 75%), scaled by wear.
    pub fn insulation(&self) -> f32 {
        let mut v = 0.0f32;
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(kind) = slot {
                v += kind.insulation() * condition_factor(self.condition(i));
            }
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

    /// Repair an equipped piece at an anvil using iron/gold. Fails atomically
    /// when the piece is missing, already pristine, or the bill is unaffordable.
    pub fn repair(&mut self, kind: ArmorKind, inv: &mut Inventory) -> bool {
        let slot = kind.slot();
        if !self.equipped(kind) || self.durability[slot] >= kind.max_durability() {
            return false;
        }
        for (item, n) in kind.repair_cost() {
            if inv.count(*item) < *n {
                return false;
            }
        }
        for (item, n) in kind.repair_cost() {
            inv.remove(*item, *n);
        }
        self.durability[slot] = kind.max_durability();
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

    #[test]
    fn old_saves_load_equipped_pieces_at_full_condition() {
        // Pre-durability JSON has only `slots`; the serde fallback must not
        // turn existing armour into broken gear.
        let json = r#"{"slots":[null,"IronPlateArmor",null,null]}"#;
        let eq: Equipment = serde_json::from_str(json).expect("legacy equipment loads");
        assert!(eq.equipped(ArmorKind::IronPlateArmor));
        assert_eq!(eq.condition_of(ArmorKind::IronPlateArmor), 1.0);
        let full = Equipment::default();
        let round: Equipment =
            serde_json::from_str(&serde_json::to_string(&full).unwrap()).unwrap();
        assert_eq!(round, full);
    }

    #[test]
    fn wear_reduces_protection_and_repair_restores_it() {
        let mut inv = Inventory::new();
        let mut eq = Equipment::new();
        inv.add(ItemKind::Iron, 5);
        inv.add(ItemKind::IronPlate, 1);
        inv.add(ItemKind::Gold, 1);
        assert!(eq.craft(ArmorKind::IronHelm, &mut inv));
        assert_eq!(inv.count(ItemKind::Iron), 2, "craft spent 3 of 5 iron");
        let pristine = eq.damage_multiplier();
        let wore = eq.wear(10.0);
        assert!(!wore[0], "a light hit wears but does not snap the helm");
        assert_eq!(eq.durability[0], ArmorKind::IronHelm.max_durability() - 1);
        assert!(eq.condition(0) < 1.0);
        assert!(eq.damage_multiplier() > pristine, "worn armour protects less");

        // Nothing equipped in the other slots can be worn or repaired.
        assert!(!eq.repair(ArmorKind::WoodShield, &mut inv));
        assert_eq!(inv.count(ItemKind::Iron), 2, "failed repair spends nothing");

        while eq.durability[0] > 0 {
            eq.wear(10.0);
        }
        assert_eq!(eq.condition(0), 0.0);
        let broken = eq.damage_multiplier();
        assert!(broken > pristine);
        assert!(eq.repair(ArmorKind::IronHelm, &mut inv));
        assert_eq!(eq.condition(0), 1.0);
        assert_eq!(eq.damage_multiplier(), pristine);
        assert_eq!(inv.count(ItemKind::Iron), 0, "iron helm repair costs 2 iron");
        assert!(!eq.repair(ArmorKind::IronHelm, &mut inv), "no repair at full");
    }

    #[test]
    fn repair_bill_is_atomic_and_gold_is_required_for_late_tiers() {
        let mut inv = Inventory::new();
        let mut eq = Equipment::new();
        eq.equip(ArmorKind::IronPlateArmor);
        eq.durability[1] = 1;
        inv.add(ItemKind::Iron, 2);
        assert!(!eq.repair(ArmorKind::IronPlateArmor, &mut inv), "gold missing");
        assert_eq!(eq.durability[1], 1);
        assert_eq!(inv.count(ItemKind::Iron), 2, "failed repair spends nothing");
        inv.add(ItemKind::Gold, 1);
        assert!(eq.repair(ArmorKind::IronPlateArmor, &mut inv));
        assert_eq!(inv.count(ItemKind::Iron), 0);
        assert_eq!(inv.count(ItemKind::Gold), 0);
    }
}
