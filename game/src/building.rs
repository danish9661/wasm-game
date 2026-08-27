use crate::items::{Inventory, ItemKind};
use crate::render::{Sprite, SpriteStyle};
use crate::world::TileKind;
use serde::{Deserialize, Serialize};

/// Reach required (chebyshev distance, tile units) to open a chest. Slightly
/// larger than the harvest range: ruins sit between flanking walls, and the
/// diagonal-only movement + wall sliding makes a tight 1.5-tile approach
/// unreachable from some directions.
pub const CHEST_RANGE: f32 = 2.0;

/// Placeable structures. Walls block movement; campfires emit light;
/// chests (only ever placed by the world's ruins POI) hold loot; the
/// Reforging Altar is where the Crown is reforged to end the campaign.
///
/// In addition to the player-buildable kinds there are purely decorative
/// props (Sign, Barrel, Totem, RockPile, Statue) that are spawned by the
/// world generator and can never be built — they sit in the world as flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureKind {
    Campfire,
    Wall,
    Chest,
    Altar,
    // Buildable
    Fence,
    Torch,
    Anvil,
    Bed,
    Well,
    // New buildables: trap + farm
    Spike,
    FarmPlot,
    // New buildables: defensive + support
    Turret,
    HealingTotem,
    // New buildable: a spiked trap that damages enemies (and the player) that
    // step on it — a cheap, proactive defense.
    Trap,
    // Decorative (world-gen only)
    Sign,
    Barrel,
    Totem,
    RockPile,
    Statue,
    Lantern,
    Brazier,
    Crate,
    Pillar,
    BonePile,
    Cactus,
    Vines,
    Lilypad,
    Reed,
    Rubble,
    RuinTower,
    // Default world buildings (non-interactive decor, scattered by worldgen).
    House,
    Cabin,
    Hut,
    // Dungeon entrance: a stone archway you can step into (Enter) to explore a
    // trapped vault. World-gen only; not buildable.
    Dungeon,
    // Arcane portal: placed in villages, steps you through to the walled town.
    // World-gen only; not buildable. Emits a soft glow.
    Portal,
    // Old-world town decor: abandoned vehicles and a railway crossing.
    Car,
    Train,
    Rail,
    // War Banner: buildable; empowers nearby guards, reinforcing base defense.
    Banner,
}

impl StructureKind {
    /// Build cost as (item, amount) pairs. Chests, altars and decorative props
    /// are not buildable.
    pub fn cost(self) -> &'static [(ItemKind, u32)] {
        match self {
            StructureKind::Campfire => &[(ItemKind::Wood, 3), (ItemKind::Stone, 1)],
            StructureKind::Wall => &[(ItemKind::Wood, 2)],
            StructureKind::Fence => &[(ItemKind::Wood, 2)],
            StructureKind::Torch => &[(ItemKind::Wood, 2), (ItemKind::Stone, 1)],
            StructureKind::Anvil => &[(ItemKind::Stone, 4)],
            StructureKind::Bed => &[(ItemKind::Wood, 4)],
            StructureKind::Well => &[(ItemKind::Stone, 6)],
            StructureKind::Spike => &[(ItemKind::Wood, 2), (ItemKind::Stone, 1)],
            StructureKind::FarmPlot => &[(ItemKind::Wood, 3), (ItemKind::Stone, 2)],
            StructureKind::Lantern => &[(ItemKind::Wood, 1), (ItemKind::Stone, 1)],
            StructureKind::Turret => &[(ItemKind::Wood, 4), (ItemKind::Stone, 4), (ItemKind::Gem, 1)],
            StructureKind::HealingTotem => &[(ItemKind::Wood, 3), (ItemKind::Herb, 2)],
            StructureKind::Trap => &[(ItemKind::Wood, 3), (ItemKind::Stone, 2)],
            StructureKind::Banner => &[(ItemKind::Wood, 2), (ItemKind::Gem, 1)],
            _ => &[],
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            StructureKind::Campfire => [1.0, 0.22, 0.05],
            StructureKind::Wall => [0.66, 0.60, 0.50],
            StructureKind::Chest => [0.85, 0.65, 0.25],
            StructureKind::Altar => [0.98, 0.80, 0.30],
            StructureKind::Fence => [0.45, 0.32, 0.18],
            StructureKind::Torch => [0.90, 0.45, 0.12],
            StructureKind::Anvil => [0.30, 0.30, 0.34],
            StructureKind::Bed => [0.50, 0.34, 0.18],
            StructureKind::Well => [0.55, 0.53, 0.52],
            StructureKind::Spike => [0.62, 0.64, 0.68],
            StructureKind::FarmPlot => [0.45, 0.55, 0.30],
            StructureKind::Turret => [0.50, 0.50, 0.55],
            StructureKind::HealingTotem => [0.50, 0.35, 0.20],
            StructureKind::Trap => [0.55, 0.40, 0.42],
            StructureKind::Banner => [0.85, 0.20, 0.20],
            StructureKind::Sign => [0.60, 0.42, 0.24],
            StructureKind::Barrel => [0.50, 0.34, 0.18],
            StructureKind::Totem => [0.50, 0.35, 0.20],
            StructureKind::RockPile => [0.55, 0.55, 0.60],
            StructureKind::Statue => [0.70, 0.70, 0.72],
            StructureKind::Lantern => [0.95, 0.80, 0.35],
            StructureKind::Brazier => [0.85, 0.45, 0.15],
            StructureKind::Crate => [0.55, 0.40, 0.22],
            StructureKind::Pillar => [0.72, 0.72, 0.74],
            StructureKind::BonePile => [0.86, 0.82, 0.70],
            StructureKind::Cactus => [0.30, 0.55, 0.30],
            StructureKind::Vines => [0.25, 0.45, 0.22],
            StructureKind::Lilypad => [0.30, 0.55, 0.28],
            StructureKind::Reed => [0.45, 0.55, 0.30],
            StructureKind::Rubble => [0.50, 0.50, 0.55],
            StructureKind::RuinTower => [0.62, 0.60, 0.58],
            StructureKind::House => [0.74, 0.72, 0.66],
            StructureKind::Cabin => [0.52, 0.34, 0.20],
            StructureKind::Hut => [0.66, 0.52, 0.30],
            StructureKind::Car => [0.62, 0.18, 0.18],
            StructureKind::Train => [0.32, 0.36, 0.44],
            StructureKind::Rail => [0.34, 0.32, 0.28],
            StructureKind::Dungeon => [0.55, 0.50, 0.46],
            StructureKind::Portal => [0.55, 0.85, 1.0],
        }
    }

    pub fn blocks_movement(self) -> bool {
        matches!(
            self,
            StructureKind::Wall
                | StructureKind::Fence
                | StructureKind::Well
                | StructureKind::Turret
                | StructureKind::HealingTotem
                | StructureKind::Car
                | StructureKind::Train
        )
    }

    pub fn emits_light(self) -> bool {
        matches!(
            self,
            StructureKind::Campfire
                | StructureKind::Altar
                | StructureKind::Torch
                | StructureKind::Lantern
                | StructureKind::Brazier
                | StructureKind::HealingTotem
                | StructureKind::Portal
        )
    }

    /// True for the decorative props that the world generator scatters but the
    /// player can never craft.
    pub fn is_decor(self) -> bool {
        matches!(
            self,
            StructureKind::Sign
                | StructureKind::Barrel
                | StructureKind::Totem
                | StructureKind::RockPile
                | StructureKind::Statue
                | StructureKind::Lantern
                | StructureKind::Brazier
                | StructureKind::Crate
                | StructureKind::Pillar
                | StructureKind::BonePile
                | StructureKind::Cactus
                | StructureKind::Vines
                | StructureKind::Lilypad
                | StructureKind::Reed
                | StructureKind::Rubble
                | StructureKind::RuinTower
                | StructureKind::House
                | StructureKind::Cabin
                | StructureKind::Hut
                | StructureKind::Car
                | StructureKind::Train
                | StructureKind::Rail
                | StructureKind::Portal
        )
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
            StructureKind::Fence => (15.0, 16.0, 2.0),
            StructureKind::Torch => (5.0, 18.0, 1.0),
            StructureKind::Anvil => (14.0, 12.0, 2.0),
            StructureKind::Bed => (18.0, 8.0, 1.0),
            StructureKind::Well => (16.0, 14.0, 2.0),
            StructureKind::Spike => (16.0, 6.0, 1.0),
            StructureKind::FarmPlot => (16.0, 8.0, 1.0),
            StructureKind::Turret => (11.0, 14.0, 1.0),
            StructureKind::HealingTotem => (8.0, 22.0, 1.0),
            StructureKind::Trap => (16.0, 6.0, 1.0),
            StructureKind::Banner => (10.0, 26.0, 1.0),
            StructureKind::Sign => (12.0, 14.0, 1.0),
            StructureKind::Barrel => (8.0, 16.0, 1.0),
            StructureKind::Totem => (8.0, 26.0, 1.0),
            StructureKind::RockPile => (10.0, 8.0, 1.0),
            StructureKind::Statue => (10.0, 26.0, 1.0),
            StructureKind::Lantern => (6.0, 18.0, 1.0),
            StructureKind::Brazier => (12.0, 18.0, 1.0),
            StructureKind::Crate => (10.0, 12.0, 1.0),
            StructureKind::Pillar => (10.0, 30.0, 1.0),
            StructureKind::BonePile => (12.0, 10.0, 1.0),
            StructureKind::Cactus => (9.0, 22.0, 1.0),
            StructureKind::Vines => (8.0, 24.0, 1.0),
            StructureKind::Lilypad => (14.0, 6.0, 1.0),
            StructureKind::Reed => (10.0, 20.0, 1.0),
            StructureKind::Rubble => (12.0, 8.0, 1.0),
            StructureKind::RuinTower => (13.0, 34.0, 1.0),
            StructureKind::House => (13.0, 22.0, 1.0),
            StructureKind::Cabin => (11.0, 18.0, 1.0),
            StructureKind::Hut => (9.0, 14.0, 1.0),
            StructureKind::Car => (16.0, 12.0, 2.0),
            StructureKind::Train => (20.0, 16.0, 4.0),
            StructureKind::Rail => (16.0, 6.0, 0.0),
            StructureKind::Dungeon => (14.0, 20.0, 2.0),
            StructureKind::Portal => (13.0, 26.0, 2.0),
        };
        let style = match self {
            StructureKind::Campfire => SpriteStyle::Campfire,
            StructureKind::Wall => SpriteStyle::Wall,
            StructureKind::Chest => SpriteStyle::Chest,
            StructureKind::Altar => SpriteStyle::Altar,
            StructureKind::Fence => SpriteStyle::Fence,
            StructureKind::Torch => SpriteStyle::Torch,
            StructureKind::Anvil => SpriteStyle::Anvil,
            StructureKind::Bed => SpriteStyle::Bed,
            StructureKind::Well => SpriteStyle::Well,
            StructureKind::Spike => SpriteStyle::Spike,
            StructureKind::FarmPlot => SpriteStyle::FarmPlot,
            StructureKind::Turret => SpriteStyle::Turret,
            StructureKind::HealingTotem => SpriteStyle::HealingTotem,
            StructureKind::Trap => SpriteStyle::Spike,
            StructureKind::Banner => SpriteStyle::Totem,
            StructureKind::Sign => SpriteStyle::Sign,
            StructureKind::Barrel => SpriteStyle::Barrel,
            StructureKind::Totem => SpriteStyle::Totem,
            StructureKind::RockPile => SpriteStyle::RockPile,
            StructureKind::Statue => SpriteStyle::Statue,
            StructureKind::Lantern => SpriteStyle::Lantern,
            StructureKind::Brazier => SpriteStyle::Brazier,
            StructureKind::Crate => SpriteStyle::Crate,
            StructureKind::Pillar => SpriteStyle::Pillar,
            StructureKind::BonePile => SpriteStyle::BonePile,
            StructureKind::Cactus => SpriteStyle::Cactus,
            StructureKind::Vines => SpriteStyle::Vines,
            StructureKind::Lilypad => SpriteStyle::Lilypad,
            StructureKind::Reed => SpriteStyle::Reed,
            StructureKind::Rubble => SpriteStyle::Rubble,
            StructureKind::RuinTower => SpriteStyle::RuinTower,
            StructureKind::House => SpriteStyle::House,
            StructureKind::Cabin => SpriteStyle::Cabin,
            StructureKind::Hut => SpriteStyle::Hut,
            StructureKind::Car => SpriteStyle::Car,
            StructureKind::Train => SpriteStyle::Train,
            StructureKind::Rail => SpriteStyle::Rail,
            StructureKind::Dungeon => SpriteStyle::RuinTower,
            StructureKind::Portal => SpriteStyle::Portal,
        };
        Sprite::new(tx, ty, self.color(), hw, hh, lift).with_style(style)
    }
}

/// Buildable structures shown in the build menu, in display order, with their
/// hotkey and human label. Single source of truth for the UI.
pub const BUILDABLE: &[(StructureKind, &str, &str)] = &[
    (StructureKind::Campfire, "F", "Campfire"),
    (StructureKind::Wall, "V", "Wall"),
    (StructureKind::Fence, "G", "Fence"),
    (StructureKind::Torch, "T", "Torch"),
    (StructureKind::Anvil, "N", "Anvil"),
    (StructureKind::Bed, "B", "Bed"),
    (StructureKind::Well, "H", "Well"),
    (StructureKind::Spike, "X", "Spike Trap"),
    (StructureKind::FarmPlot, "U", "Farm Plot"),
    (StructureKind::Turret, "Y", "Turret"),
    (StructureKind::HealingTotem, "M", "Healing Totem"),
    (StructureKind::Lantern, "L", "Lantern"),
    (StructureKind::Trap, "0", "Trap"),
    (StructureKind::Banner, "1", "War Banner"),
];

/// Stateless decorative-prop placement: a few flavor props sprinkled on biomes
/// so the world isn't just trees/rocks. Decorative props never block movement,
/// emit light, or appear in the build menu. Same seed → same layout forever.
pub fn decor_on(tx: i32, ty: i32, tile: TileKind) -> Option<StructureKind> {
    let h = tx.wrapping_mul(73856093) ^ ty.wrapping_mul(19349663) ^ 0x0bad_c0de;
    match tile {
        TileKind::Grass if h.rem_euclid(53) == 0 => Some(StructureKind::Sign),
        TileKind::Grass if h.rem_euclid(97) == 0 => Some(StructureKind::Statue),
        TileKind::Grass if h.rem_euclid(151) == 0 => Some(StructureKind::Lantern),
        TileKind::Grass if h.rem_euclid(211) == 0 => Some(StructureKind::Crate),
        TileKind::Grass if h.rem_euclid(167) == 0 => Some(StructureKind::Pillar),
        TileKind::Grass if h.rem_euclid(223) == 0 => Some(StructureKind::Rubble),
        // Default settlements: scattered homes so the grasslands feel inhabited.
        TileKind::Grass if h.rem_euclid(83) == 0 => Some(StructureKind::House),
        TileKind::Grass if h.rem_euclid(127) == 0 => Some(StructureKind::Cabin),
        TileKind::Forest if h.rem_euclid(139) == 0 => Some(StructureKind::Hut),
        TileKind::Tundra if h.rem_euclid(97) == 0 => Some(StructureKind::Cabin),
        TileKind::Forest if h.rem_euclid(67) == 0 => Some(StructureKind::Totem),
        TileKind::Forest if h.rem_euclid(71) == 0 => Some(StructureKind::BonePile),
        TileKind::Forest if h.rem_euclid(173) == 0 => Some(StructureKind::Vines),
        TileKind::Forest if h.rem_euclid(191) == 0 => Some(StructureKind::Rubble),
        TileKind::Stone if h.rem_euclid(37) == 0 => Some(StructureKind::RockPile),
        TileKind::Stone if h.rem_euclid(41) == 0 => Some(StructureKind::Brazier),
        TileKind::Stone if h.rem_euclid(59) == 0 => Some(StructureKind::Pillar),
        TileKind::Stone if h.rem_euclid(199) == 0 => Some(StructureKind::Rubble),
        TileKind::Stone if h.rem_euclid(101) == 0 => Some(StructureKind::RuinTower),
        TileKind::Sand if h.rem_euclid(61) == 0 => Some(StructureKind::Barrel),
        TileKind::Sand if h.rem_euclid(43) == 0 => Some(StructureKind::Cactus),
        TileKind::Sand if h.rem_euclid(79) == 0 => Some(StructureKind::Reed),
        TileKind::Sand if h.rem_euclid(89) == 0 => Some(StructureKind::Crate),
        TileKind::Water if h.rem_euclid(29) == 0 => Some(StructureKind::Lilypad),
        TileKind::Water if h.rem_euclid(47) == 0 => Some(StructureKind::Reed),
        TileKind::Tundra if h.rem_euclid(59) == 0 => Some(StructureKind::RockPile),
        TileKind::Tundra if h.rem_euclid(83) == 0 => Some(StructureKind::Pillar),
        TileKind::Desert if h.rem_euclid(61) == 0 => Some(StructureKind::Barrel),
        TileKind::Desert if h.rem_euclid(73) == 0 => Some(StructureKind::Cactus),
        TileKind::Jungle if h.rem_euclid(67) == 0 => Some(StructureKind::Hut),
        TileKind::Jungle if h.rem_euclid(131) == 0 => Some(StructureKind::House),
        TileKind::Jungle if h.rem_euclid(113) == 0 => Some(StructureKind::Vines),
        TileKind::Jungle if h.rem_euclid(149) == 0 => Some(StructureKind::Statue),
        TileKind::Jungle if h.rem_euclid(89) == 0 => Some(StructureKind::RockPile),
        TileKind::Volcanic if h.rem_euclid(103) == 0 => Some(StructureKind::Pillar),
        TileKind::Volcanic if h.rem_euclid(127) == 0 => Some(StructureKind::Rubble),
        TileKind::Volcanic if h.rem_euclid(151) == 0 => Some(StructureKind::RockPile),
        _ => None,
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
    if kind.is_decor() {
        // decorative props are world-gen only and can never be crafted
        return Err(ItemKind::Wood);
    }
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

    #[test]
    fn turret_and_totem_are_buildable_and_block() {
        assert!(StructureKind::Turret.blocks_movement());
        assert!(!StructureKind::Turret.emits_light());
        assert!(!StructureKind::Turret.cost().is_empty(), "turret should cost materials");

        assert!(StructureKind::HealingTotem.blocks_movement());
        assert!(StructureKind::HealingTotem.emits_light(), "totem glows as a light");
        assert!(!StructureKind::HealingTotem.cost().is_empty());

        // Lantern becomes a cheap, buildable light source.
        assert!(StructureKind::Lantern.emits_light());
        assert!(!StructureKind::Lantern.cost().is_empty(), "lantern should now be buildable");

        // New buildables appear in the build menu.
        assert!(BUILDABLE
            .iter()
            .any(|(k, _, _)| *k == StructureKind::Turret || *k == StructureKind::HealingTotem
                || *k == StructureKind::Lantern));
    }
}