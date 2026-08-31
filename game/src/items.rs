use serde::{Deserialize, Serialize};

/// Item kinds the player can carry. Each maps to a fixed inventory slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Wood,
    Stone,
    Food,
    /// Crown Fragment — the story macguffin dropped by biome bosses. Counts
    /// toward the 5 fragments needed to reforge the Star Crown.
    Fragment,
    /// Gathered from flowers, ferns and grass tufts — a forageable.
    Herb,
    /// Gathered from crystal nodes — a shiny crafting material.
    Gem,
    /// A treasure map: while carried, buried caches are revealed on the minimap.
    Map,
    /// Smelted iron ingot, gathered from ore nodes — a crafting material.
    Iron,
    /// Iron Plate: forged at an Anvil from iron + stone. A campaign milestone
    /// (the "craft iron plate" story beat) and a defensive material.
    IronPlate,
    /// Gold coin: earned from kills and sold to merchants for supplies.
    Gold,
}

impl ItemKind {
    pub fn name(self) -> &'static str {
        match self {
            ItemKind::Wood => "wood",
            ItemKind::Stone => "stone",
            ItemKind::Food => "food",
            ItemKind::Fragment => "crown fragment",
            ItemKind::Herb => "herb",
            ItemKind::Gem => "gem",
            ItemKind::Map => "treasure map",
            ItemKind::Iron => "iron",
            ItemKind::IronPlate => "iron plate",
            ItemKind::Gold => "gold",
        }
    }

    /// Stable index used to send a crafted item over the network input.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Inverse of `as_u8`; returns None for out-of-range values.
    pub fn from_u8(v: u8) -> Option<ItemKind> {
        match v {
            0 => Some(ItemKind::Wood),
            1 => Some(ItemKind::Stone),
            2 => Some(ItemKind::Food),
            3 => Some(ItemKind::Fragment),
            4 => Some(ItemKind::Herb),
            5 => Some(ItemKind::Gem),
            6 => Some(ItemKind::Map),
            7 => Some(ItemKind::Iron),
            8 => Some(ItemKind::IronPlate),
            9 => Some(ItemKind::Gold),
            _ => None,
        }
    }

    /// Display color for ground loot / UI.
    pub fn color(self) -> [f32; 3] {
        match self {
            ItemKind::Wood => [0.55, 0.38, 0.20],
            ItemKind::Stone => [0.62, 0.62, 0.66],
            ItemKind::Food => [0.85, 0.35, 0.25],
            ItemKind::Fragment => [1.00, 0.84, 0.30],
            ItemKind::Herb => [0.45, 0.80, 0.40],
            ItemKind::Gem => [0.45, 0.85, 0.95],
            ItemKind::Map => [0.86, 0.74, 0.42],
            ItemKind::Iron => [0.70, 0.72, 0.78],
            ItemKind::IronPlate => [0.78, 0.80, 0.86],
            ItemKind::Gold => [1.00, 0.84, 0.00],
        }
    }
}

const SLOTS: usize = 10;

/// Simple stack inventory: one count per item kind.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Inventory {
    counts: [u32; SLOTS],
}

impl Inventory {
    pub fn new() -> Self {
        Self { counts: [0; SLOTS] }
    }

    fn slot(kind: ItemKind) -> usize {
        match kind {
            ItemKind::Wood => 0,
            ItemKind::Stone => 1,
            ItemKind::Food => 2,
            ItemKind::Fragment => 3,
            ItemKind::Herb => 4,
            ItemKind::Gem => 5,
            ItemKind::Map => 6,
            ItemKind::Iron => 7,
            ItemKind::IronPlate => 8,
            ItemKind::Gold => 9,
        }
    }

    pub fn count(&self, kind: ItemKind) -> u32 {
        self.counts[Self::slot(kind)]
    }

    pub fn total(&self) -> u32 {
        self.counts.iter().sum()
    }

    pub fn add(&mut self, kind: ItemKind, n: u32) {
        self.counts[Self::slot(kind)] += n;
    }

    /// Removes `n` items; returns false (and changes nothing) if short.
    pub fn remove(&mut self, kind: ItemKind, n: u32) -> bool {
        let slot = Self::slot(kind);
        if self.counts[slot] < n {
            return false;
        }
        self.counts[slot] -= n;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut inv = Inventory::new();
        assert_eq!(inv.count(ItemKind::Wood), 0);
        inv.add(ItemKind::Wood, 3);
        inv.add(ItemKind::Stone, 2);
        assert_eq!(inv.count(ItemKind::Wood), 3);
        assert_eq!(inv.count(ItemKind::Stone), 2);
        assert_eq!(inv.total(), 5);
    }

    #[test]
    fn remove_deducts() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Wood, 3);
        assert!(inv.remove(ItemKind::Wood, 2));
        assert_eq!(inv.count(ItemKind::Wood), 1);
    }

    #[test]
    fn remove_insufficient_fails_atomically() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Wood, 1);
        assert!(!inv.remove(ItemKind::Wood, 2));
        assert_eq!(inv.count(ItemKind::Wood), 1, "must not deduct on failure");
        assert!(!inv.remove(ItemKind::Stone, 1));
    }

    #[test]
    fn kinds_are_independent() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Wood, 5);
        inv.remove(ItemKind::Wood, 5);
        assert_eq!(inv.count(ItemKind::Stone), 0);
    }

    #[test]
    fn fragment_is_a_distinct_slot() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Fragment, 3);
        assert_eq!(inv.count(ItemKind::Fragment), 3);
        assert_eq!(inv.count(ItemKind::Wood), 0);
        inv.add(ItemKind::Wood, 2);
        assert_eq!(inv.count(ItemKind::Fragment), 3, "wood must not touch the fragment slot");
    }
}