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
}

impl ItemKind {
    pub fn name(self) -> &'static str {
        match self {
            ItemKind::Wood => "wood",
            ItemKind::Stone => "stone",
            ItemKind::Food => "food",
            ItemKind::Fragment => "crown fragment",
        }
    }
}

const SLOTS: usize = 4;

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