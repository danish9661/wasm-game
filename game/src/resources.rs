use crate::items::ItemKind;
use crate::render::{Sprite, SpriteStyle};
use crate::world::TileKind;
use std::collections::HashMap;

/// Reach required (chebyshev distance, tile units) to harvest a node.
pub const HARVEST_RANGE: f32 = 1.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Tree,
    Bush,
    Rock,
    Mushroom,
    Crystal,
    Flower,
    GrassTuft,
    Fern,
    Ore,
}

impl ResourceKind {
    pub fn max_hp(self) -> u32 {
        match self {
            ResourceKind::Tree => 3,
            ResourceKind::Bush => 1,
            ResourceKind::Rock => 4,
            ResourceKind::Mushroom => 1,
            ResourceKind::Crystal => 2,
            ResourceKind::Flower => 1,
            ResourceKind::GrassTuft => 1,
            ResourceKind::Fern => 1,
            ResourceKind::Ore => 5,
        }
    }

    pub fn drops(self) -> ItemKind {
        match self {
            ResourceKind::Tree => ItemKind::Wood,
            ResourceKind::Bush => ItemKind::Wood,
            ResourceKind::Rock => ItemKind::Stone,
            ResourceKind::Mushroom => ItemKind::Food,
            ResourceKind::Crystal => ItemKind::Gem,
            ResourceKind::Flower => ItemKind::Herb,
            ResourceKind::GrassTuft => ItemKind::Herb,
            ResourceKind::Fern => ItemKind::Herb,
            ResourceKind::Ore => ItemKind::Gem,
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            ResourceKind::Tree => [0.06, 0.30, 0.12],
            ResourceKind::Bush => [0.30, 0.42, 0.18],
            ResourceKind::Rock => [0.58, 0.58, 0.63],
            ResourceKind::Mushroom => [0.80, 0.18, 0.16],
            ResourceKind::Crystal => [0.45, 0.85, 0.95],
            ResourceKind::Flower => [0.95, 0.55, 0.75],
            ResourceKind::GrassTuft => [0.45, 0.62, 0.30],
            ResourceKind::Fern => [0.20, 0.45, 0.22],
            ResourceKind::Ore => [0.45, 0.40, 0.46],
        }
    }

    /// Sprite geometry for this node kind (diamond centered on the tile).
    pub fn sprite(self, tx: i32, ty: i32) -> Sprite {
        let (hw, hh, lift) = match self {
            ResourceKind::Tree => (14.0, 20.0, 8.0),
            ResourceKind::Bush => (12.0, 12.0, 2.0),
            ResourceKind::Rock => (12.0, 10.0, 2.0),
            ResourceKind::Mushroom => (12.0, 10.0, 2.0),
            ResourceKind::Crystal => (12.0, 14.0, 2.0),
            ResourceKind::Flower => (10.0, 13.0, 1.0),
            ResourceKind::GrassTuft => (12.0, 11.0, 1.0),
            ResourceKind::Fern => (14.0, 12.0, 1.0),
            ResourceKind::Ore => (13.0, 11.0, 2.0),
        };
        let style = match self {
            ResourceKind::Tree => SpriteStyle::Tree,
            ResourceKind::Bush => SpriteStyle::Bush,
            ResourceKind::Rock => SpriteStyle::Rock,
            ResourceKind::Mushroom => SpriteStyle::Mushroom,
            ResourceKind::Crystal => SpriteStyle::Crystal,
            ResourceKind::Flower => SpriteStyle::Flower,
            ResourceKind::GrassTuft => SpriteStyle::GrassTuft,
            ResourceKind::Fern => SpriteStyle::Fern,
            ResourceKind::Ore => SpriteStyle::Ore,
        };
        Sprite::new(tx, ty, self.color(), hw, hh, lift).with_style(style)
    }
}

/// Stateless placement: ~1/7 of Forest tiles carry a Tree, ~1/11 of Grass
/// tiles a Bush, and ~1/8 of Stone tiles a Rock, decided by a coordinate
/// hash (same seed world → same nodes forever). A few more forageables
/// (mushroom, crystal, flower, grass, fern) are sprinkled on top. Node
/// *health* is session state (NodeRegistry).
pub fn resource_on(tx: i32, ty: i32, tile: TileKind) -> Option<ResourceKind> {
    let h = tx.wrapping_mul(73856093) ^ ty.wrapping_mul(19349663) ^ 0x51ab_ce0d;
    match tile {
        TileKind::Forest if h.rem_euclid(7) == 0 => Some(ResourceKind::Tree),
        TileKind::Grass if h.rem_euclid(11) == 0 => Some(ResourceKind::Bush),
        TileKind::Stone if h.rem_euclid(8) == 0 => Some(ResourceKind::Rock),
        TileKind::Stone if h.rem_euclid(53) == 0 => Some(ResourceKind::Ore),
        TileKind::Forest if h.rem_euclid(19) == 0 => Some(ResourceKind::Fern),
        TileKind::Forest if h.rem_euclid(29) == 0 => Some(ResourceKind::Mushroom),
        TileKind::Stone if h.rem_euclid(23) == 0 => Some(ResourceKind::Crystal),
        TileKind::Grass if h.rem_euclid(17) == 0 => Some(ResourceKind::GrassTuft),
        TileKind::Grass if h.rem_euclid(31) == 0 => Some(ResourceKind::Flower),
        _ => None,
    }
}

/// A harvestable node with current health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceNode {
    pub tx: i32,
    pub ty: i32,
    pub kind: ResourceKind,
    pub hp: u32,
}

impl ResourceNode {
    pub fn new(tx: i32, ty: i32, kind: ResourceKind) -> Self {
        Self {
            tx,
            ty,
            kind,
            hp: kind.max_hp(),
        }
    }

    pub fn depleted(&self) -> bool {
        self.hp == 0
    }

    /// One chop: -1 hp. Returns the dropped item (all hits drop one item).
    pub fn chop(&mut self) -> Option<ItemKind> {
        if self.depleted() {
            return None;
        }
        self.hp -= 1;
        Some(self.kind.drops())
    }
}

/// Session-scoped node health. Nodes are generated statelessly; this keeps
/// depletion persistent while the player is alive (no respawn in-session).
#[derive(Debug, Default)]
pub struct NodeRegistry {
    nodes: HashMap<(i32, i32), ResourceNode>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chop(&mut self, tx: i32, ty: i32, kind: ResourceKind) -> Option<ItemKind> {
        let node = self
            .nodes
            .entry((tx, ty))
            .or_insert_with(|| ResourceNode::new(tx, ty, kind));
        node.chop()
    }

    pub fn is_depleted(&self, tx: i32, ty: i32) -> bool {
        self.nodes.get(&(tx, ty)).is_some_and(|n| n.depleted())
    }

    pub fn has_live(&self, tx: i32, ty: i32) -> bool {
        self.nodes.get(&(tx, ty)).is_some_and(|n| !n.depleted())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_is_deterministic() {
        for (tx, ty) in [(0, 0), (3, -2), (-7, 11), (100, -200)] {
            let a = resource_on(tx, ty, TileKind::Forest);
            let b = resource_on(tx, ty, TileKind::Forest);
            assert_eq!(a, b, "same coord must give the same node");
        }
    }

    #[test]
    fn only_forest_grass_and_stone_carry_nodes() {
        assert!(resource_on(0, 0, TileKind::Water).is_none());
        assert!(resource_on(0, 0, TileKind::Sand).is_none());
        assert!(resource_on(0, 0, TileKind::Snow).is_none());
        assert!(resource_on(0, 0, TileKind::Swamp).is_none());
        assert!(resource_on(0, 0, TileKind::DeepWater).is_none());
    }

    #[test]
    fn forest_has_some_trees_in_a_window() {
        let trees = (-16..16)
            .flat_map(|tx| (-16..16).map(move |ty| (tx, ty)))
            .filter(|&(tx, ty)| resource_on(tx, ty, TileKind::Forest) == Some(ResourceKind::Tree))
            .count();
        assert!(trees > 20, "expected ~1/7 of 1024 tiles, got {trees}");
    }

    #[test]
    fn grass_has_some_bushes_in_a_window() {
        let bushes = (-16..16)
            .flat_map(|tx| (-16..16).map(move |ty| (tx, ty)))
            .filter(|&(tx, ty)| resource_on(tx, ty, TileKind::Grass) == Some(ResourceKind::Bush))
            .count();
        assert!(bushes > 30, "expected ~1/11 of 1024 tiles, got {bushes}");
    }

    #[test]
    fn bush_drops_wood_in_one_chop() {
        let mut node = ResourceNode::new(1, 2, ResourceKind::Bush);
        assert_eq!(node.chop(), Some(ItemKind::Wood));
        assert!(node.depleted());
        assert_eq!(node.chop(), None);
    }

    #[test]
    fn chop_drops_and_depletes() {
        let mut node = ResourceNode::new(1, 2, ResourceKind::Tree);
        for _ in 0..ResourceKind::Tree.max_hp() {
            assert_eq!(node.chop(), Some(ItemKind::Wood));
        }
        assert!(node.depleted());
        assert_eq!(node.chop(), None, "depleted nodes yield nothing");
    }

    #[test]
    fn rock_drops_stone() {
        let mut node = ResourceNode::new(1, 2, ResourceKind::Rock);
        assert_eq!(node.chop(), Some(ItemKind::Stone));
    }

    #[test]
    fn registry_tracks_depletion() {
        let mut reg = NodeRegistry::new();
        assert!(!reg.is_depleted(0, 0));
        for _ in 0..ResourceKind::Tree.max_hp() {
            reg.chop(0, 0, ResourceKind::Tree);
        }
        assert!(reg.is_depleted(0, 0));
        assert!(!reg.has_live(0, 0));
        assert_eq!(reg.chop(0, 0, ResourceKind::Tree), None);
    }
}