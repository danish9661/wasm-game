use crate::items::ItemKind;
use crate::render::{Sprite, SpriteStyle};
use crate::world::TileKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reach required (chebyshev distance, tile units) to harvest a node.
pub const HARVEST_RANGE: f32 = 1.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Buried treasure cache: a walk-through dig spot that yields a random
    /// bundle of loot (or, rarely, a treasure map) when harvested.
    Treasure,
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
            ResourceKind::Treasure => 1,
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
            ResourceKind::Treasure => ItemKind::Gem,
        }
    }

    /// Whether a live node of this kind blocks player/enemy movement (acts as a
    /// solid obstacle). Trees, rocks, ore veins and crystals block; small
    /// forageables (bush, mushroom, flower, grass, fern) are walk-through.
    pub fn blocks_movement(self) -> bool {
        matches!(
            self,
            ResourceKind::Tree | ResourceKind::Rock | ResourceKind::Ore | ResourceKind::Crystal
        )
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
            ResourceKind::Treasure => [0.55, 0.42, 0.25],
        }
    }

    /// Sprite geometry for this node kind (diamond centered on the tile).
    pub fn sprite(self, tx: i32, ty: i32) -> Sprite {
        // Trees get a small per-tile green variation so a forest doesn't read as
        // one repeated sprite.
        let color = if matches!(self, ResourceKind::Tree) {
            let h = tx.wrapping_mul(73856093) ^ ty.wrapping_mul(19349663);
            let v = (((h % 7) as f32) - 3.0) * 0.03;
            [
                0.06,
                (0.30 + v).clamp(0.0, 1.0),
                (0.12 + v).clamp(0.0, 1.0),
            ]
        } else {
            self.color()
        };
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
            ResourceKind::Treasure => (13.0, 7.0, 0.0),
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
            ResourceKind::Treasure => SpriteStyle::Treasure,
        };
        Sprite::new(tx, ty, color, hw, hh, lift).with_style(style)
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
        TileKind::Forest if h.rem_euclid(11) == 0 => Some(ResourceKind::Tree),
        TileKind::Grass if h.rem_euclid(23) == 0 => Some(ResourceKind::Bush),
        TileKind::Stone if h.rem_euclid(13) == 0 => Some(ResourceKind::Rock),
        TileKind::Stone if h.rem_euclid(53) == 0 => Some(ResourceKind::Ore),
        TileKind::Forest if h.rem_euclid(37) == 0 => Some(ResourceKind::Fern),
        TileKind::Forest if h.rem_euclid(53) == 0 => Some(ResourceKind::Mushroom),
        TileKind::Stone if h.rem_euclid(23) == 0 => Some(ResourceKind::Crystal),
        TileKind::Grass if h.rem_euclid(41) == 0 => Some(ResourceKind::GrassTuft),
        TileKind::Grass if h.rem_euclid(61) == 0 => Some(ResourceKind::Flower),
        TileKind::Tundra if h.rem_euclid(41) == 0 => Some(ResourceKind::Rock),
        TileKind::Tundra if h.rem_euclid(61) == 0 => Some(ResourceKind::Flower),
        TileKind::Desert if h.rem_euclid(37) == 0 => Some(ResourceKind::Crystal),
        TileKind::Desert if h.rem_euclid(53) == 0 => Some(ResourceKind::Rock),
        TileKind::Jungle if h.rem_euclid(11) == 0 => Some(ResourceKind::Tree),
        TileKind::Jungle if h.rem_euclid(29) == 0 => Some(ResourceKind::Bush),
        TileKind::Jungle if h.rem_euclid(47) == 0 => Some(ResourceKind::Fern),
        TileKind::Jungle if h.rem_euclid(61) == 0 => Some(ResourceKind::Flower),
        TileKind::Volcanic if h.rem_euclid(7) == 0 => Some(ResourceKind::Rock),
        TileKind::Volcanic if h.rem_euclid(41) == 0 => Some(ResourceKind::Ore),
        TileKind::Volcanic if h.rem_euclid(29) == 0 => Some(ResourceKind::Crystal),
        // Buried treasure caches: very rare, scattered on open ground and shores.
        TileKind::Grass if h.rem_euclid(257) == 0 => Some(ResourceKind::Treasure),
        TileKind::Sand if h.rem_euclid(211) == 0 => Some(ResourceKind::Treasure),
        TileKind::Sand if h.rem_euclid(193) == 0 => Some(ResourceKind::Treasure),
        _ => None,
    }
}

/// Harvest wobble offset (world tiles, added to the sprite's x): a fast
/// decaying side-to-side shake, phased per tile so neighbouring nodes
/// don't move in unison. `shake` is 1.0 on the struck frame, decaying to
/// 0 (~0.45s in the renderer). Max amplitude ≈ 0.09 tiles (~6px) — felt,
/// never seasick.
pub fn shake_offset(shake: f32, tx: i32, ty: i32) -> f32 {
    if shake <= 0.0 {
        return 0.0;
    }
    let phase = (tx * 31 + ty * 17) as f32 * 0.37;
    (shake * 22.0 + phase).sin() * 0.09 * shake
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
        if self.kind == ResourceKind::Treasure {
            // Deterministic-by-tile loot: usually supplies, sometimes a map.
            let r = (self.tx.wrapping_mul(73856093) ^ self.ty.wrapping_mul(19349663)) % 100;
            if r < 22 {
                return Some(ItemKind::Map);
            }
            return Some(match r % 4 {
                0 => ItemKind::Food,
                1 => ItemKind::Gem,
                2 => ItemKind::Herb,
                _ => ItemKind::Wood,
            });
        }
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

    /// All depleted (harvested) nodes, for persistence across save/load.
    pub fn depleted_list(&self) -> Vec<(i32, i32, ResourceKind)> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.depleted())
            .map(|(&(tx, ty), n)| (tx, ty, n.kind))
            .collect()
    }

    /// Restore a harvested node after loading a save so it does not respawn.
    pub fn restore_depleted(&mut self, tx: i32, ty: i32, kind: ResourceKind) {
        self.nodes.insert((tx, ty), ResourceNode { tx, ty, kind, hp: 0 });
    }

    /// Every node (live or depleted) with its depleted flag — used by the
    /// multiplayer server to describe harvestable resources to clients.
    pub fn all(&self) -> Vec<(i32, i32, ResourceKind, bool)> {
        self.nodes
            .iter()
            .map(|(&(tx, ty), n)| (tx, ty, n.kind, n.depleted()))
            .collect()
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

    #[test]
    fn only_solid_props_block_movement() {        assert!(ResourceKind::Tree.blocks_movement());
        assert!(ResourceKind::Rock.blocks_movement());
        assert!(ResourceKind::Ore.blocks_movement());
        assert!(ResourceKind::Crystal.blocks_movement());
        assert!(!ResourceKind::Bush.blocks_movement());
        assert!(!ResourceKind::Mushroom.blocks_movement());
        assert!(!ResourceKind::Flower.blocks_movement());
        assert!(!ResourceKind::GrassTuft.blocks_movement());
        assert!(!ResourceKind::Fern.blocks_movement());
    }

    #[test]
    fn harvest_shake_is_small_fast_and_phased() {
        // Rest: exactly zero (no drift on idle nodes).
        assert_eq!(super::shake_offset(0.0, 3, 4), 0.0);
        assert_eq!(super::shake_offset(-0.5, 3, 4), 0.0);
        // Struck frame: bounded, never more than ~0.09 tiles.
        for &(tx, ty) in &[(0, 0), (3, 4), (-22, 30), (100, -7)] {
            let o = super::shake_offset(1.0, tx, ty);
            assert!(o.abs() <= 0.09 + 1e-5, "shake capped at {o} for ({tx},{ty})");
        }
        // Decays with the timer: the amplitude envelope (0.09 * shake)
        // shrinks even though the instantaneous sine phase varies.
        for &sh in &[1.0f32, 0.5, 0.25] {
            let o = super::shake_offset(sh, 3, 4);
            assert!(o.abs() <= 0.09 * sh + 1e-5, "envelope must shrink with {sh}");
        }
        // Phased: neighbouring tiles don't move in unison.
        let a = super::shake_offset(1.0, 3, 4);
        let b = super::shake_offset(1.0, 4, 4);
        assert!((a - b).abs() > 1e-4, "adjacent tiles must phase-offset");
    }
}