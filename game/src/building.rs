use crate::items::{Inventory, ItemKind};
use crate::render::Sprite;
use serde::{Deserialize, Serialize};

/// Reach required (chebyshev distance, tile units) to open a chest. Slightly
/// larger than the harvest range: ruins sit between flanking walls, and the
/// diagonal-only movement + wall sliding makes a tight 1.5-tile approach
/// unreachable from some directions.
pub const CHEST_RANGE: f32 = 2.0;

/// Placeable structures. Walls block movement; campfires emit light;
/// chests (only ever placed by the world's ruins POI) hold loot; the
/// Reforging Altar is where the Crown is reforged to end the campaign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureKind {
    Campfire,
    Wall,
    Chest,
    Altar,
}

impl StructureKind {
    /// Build cost as (item, amount) pairs. Chests and altars are not buildable.
    pub fn cost(self) -> &'static [(ItemKind, u32)] {
        match self {
            StructureKind::Campfire => &[(ItemKind::Wood, 3), (ItemKind::Stone, 1)],
            StructureKind::Wall => &[(ItemKind::Wood, 2)],
            StructureKind::Chest => &[],
            StructureKind::Altar => &[],
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            StructureKind::Campfire => [1.0, 0.22, 0.05],
            StructureKind::Wall => [0.66, 0.60, 0.50],
            StructureKind::Chest => [0.85, 0.65, 0.25],
            StructureKind::Altar => [0.98, 0.80, 0.30],
        }
    }

    pub fn blocks_movement(self) -> bool {
        matches!(self, StructureKind::Wall)
    }

    pub fn emits_light(self) -> bool {
        matches!(self, StructureKind::Campfire | StructureKind::Altar)
    }

    pub fn is_chest(self) -> bool {
        matches!(self, StructureKind::Chest)
    }

    pub fn is_altar(self) -> bool {
        matches!(self, StructureKind::Altar)
    }

    pub fn sprite(self, tx: i32, ty: i32) -> Sprite {
        let (hw, hh, lift) = match self {
            StructureKind::Campfire => (10.0, 8.0, 1.0),
            StructureKind::Wall => (20.0, 12.0, 2.0),
            StructureKind::Chest => (16.0, 12.0, 6.0),
            StructureKind::Altar => (18.0, 22.0, 4.0),
        };
        Sprite::new(tx, ty, self.color(), hw, hh, lift)
    }
}

/// A placed structure at a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Structure {
    pub tx: i32,
    pub ty: i32,
    pub kind: StructureKind,
}

/// Pays `kind`'s cost from `inv` and returns the placed structure.
/// On failure nothing is deducted.
pub fn try_build(
    kind: StructureKind,
    tx: i32,
    ty: i32,
    inv: &mut Inventory,
) -> Result<Structure, ItemKind> {
    for (item, n) in kind.cost() {
        if inv.count(*item) < *n {
            return Err(*item);
        }
    }
    for (item, n) in kind.cost() {
        inv.remove(*item, *n);
    }
    Ok(Structure { tx, ty, kind })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_deducts_cost() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Wood, 5);
        let s = try_build(StructureKind::Wall, 3, 4, &mut inv).unwrap();
        assert_eq!(s, Structure { tx: 3, ty: 4, kind: StructureKind::Wall });
        assert_eq!(inv.count(ItemKind::Wood), 3, "wall costs 2 wood");
    }

    #[test]
    fn build_campfire_requires_stone() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Wood, 5);
        assert_eq!(try_build(StructureKind::Campfire, 0, 0, &mut inv), Err(ItemKind::Stone));
        assert_eq!(inv.count(ItemKind::Wood), 5, "failed build must not deduct");
    }

    #[test]
    fn build_fails_short_on_wood() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Wood, 1);
        assert_eq!(try_build(StructureKind::Wall, 0, 0, &mut inv), Err(ItemKind::Wood));
        assert_eq!(inv.count(ItemKind::Wood), 1);
    }

    #[test]
    fn campfire_build_consumes_both() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Wood, 3);
        inv.add(ItemKind::Stone, 2);
        assert!(try_build(StructureKind::Campfire, 1, 1, &mut inv).is_ok());
        assert_eq!(inv.count(ItemKind::Wood), 0);
        assert_eq!(inv.count(ItemKind::Stone), 1);
    }

    #[test]
    fn wall_blocks_movement_campfire_does_not() {
        assert!(StructureKind::Wall.blocks_movement());
        assert!(!StructureKind::Campfire.blocks_movement());
        assert!(StructureKind::Campfire.emits_light());
        assert!(!StructureKind::Wall.emits_light());
        assert!(StructureKind::Chest.is_chest());
        assert!(!StructureKind::Chest.blocks_movement());
        assert!(StructureKind::Chest.cost().is_empty(), "chests are not buildable");
    }

    #[test]
    fn altar_is_a_non_blocking_light_source() {
        assert!(StructureKind::Altar.is_altar());
        assert!(!StructureKind::Altar.blocks_movement());
        assert!(StructureKind::Altar.emits_light());
        assert!(StructureKind::Altar.cost().is_empty(), "altars are not buildable");
    }
}