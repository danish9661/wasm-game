use game::building::Structure;
use game::enemy::EnemyKind;
use game::items::ItemKind;
use game::resources::ResourceKind;
use serde::{Deserialize, Deserializer, Serialize};

/// Bump this when `SaveState`'s layout changes. Loads from a different version
/// are rejected so an old save can never silently corrupt a new build.
pub const CURRENT_SAVE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct PlayerSave {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub hunger: f32,
    pub stamina: f32,
    pub facing: (f32, f32),
    /// Experience points and level, so progression survives a reload.
    #[serde(default)]
    pub xp: u32,
    #[serde(default)]
    pub level: u32,
}

/// Accept a `u8` (v1 saves) or `u16` (v2+) for the weapon bitmask so old
/// save files keep loading after the roster grew past 8 slots.
fn u16_from_u8_or_u16<'de, D>(d: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    struct Wide;
    impl<'de> serde::de::Visitor<'de> for Wide {
        type Value = u16;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a u8 or u16 weapon bitmask")
        }
        fn visit_u8<E: serde::de::Error>(self, v: u8) -> Result<u16, E> {
            Ok(v as u16)
        }
        fn visit_u16<E: serde::de::Error>(self, v: u16) -> Result<u16, E> {
            Ok(v)
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<u16, E> {
            Ok(v as u16)
        }
    }
    d.deserialize_any(Wide)
}

/// Everything needed to resume a run exactly where the player left off.
/// The terrain itself is deterministic from `world_seed`, so we only persist
/// the dynamic state layered on top of it.
#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub version: u32,
    pub world_seed: u32,
    pub player: PlayerSave,
    pub inv: Vec<(ItemKind, u32)>,
    pub structures: Vec<Structure>,
    pub opened_chests: Vec<(i32, i32)>,
    /// Harvested resource nodes so they stay depleted after a reload.
    #[serde(default)]
    pub depleted_nodes: Vec<(i32, i32, ResourceKind)>,
    pub enemies: Vec<(EnemyKind, f32, f32, f32)>,
    pub quest_stage: u8,
    pub slimes_killed: u32,
    pub boss_killed: u32,
    pub colossus_killed: u32,
    /// Bitmask (bits 0..5) of the five Crown Fragments recovered from the biome
    /// bosses. 0b11111 means all five are in hand and the Crown can be reforged.
    #[serde(default)]
    pub fragments: u8,
    /// Enemy kinds seen (Bestiary / Codex), persisted so the codex survives reloads.
    #[serde(default)]
    pub discovered: Vec<game::enemy::EnemyKind>,
    pub boss_spawned: bool,
    pub altar_placed: bool,
    pub altar_tile: Option<(i32, i32)>,
    pub ending_pending: bool,
    pub ending: Option<u8>,
    pub ng_plus: u32,
    pub time_of_day: f32,
    pub spawn_point: (f32, f32),
    /// Crafting bonuses unlocked at an Anvil.
    #[serde(default)]
    pub craft_harvest: u32,
    #[serde(default)]
    pub craft_armor: f32,
    #[serde(default)]
    pub salves: u32,
    /// Equipped weapon (WeaponKind index) so it survives a reload.
    #[serde(default)]
    pub weapon: u8,
    /// Bitmask of owned weapons (mirrors `Player::unlocked`). Stored wide
    /// (`u16`) since the Mace took the roster to 9; v1 saves stored a `u8`,
    /// which the custom deserializer below still accepts.
    #[serde(default, deserialize_with = "u16_from_u8_or_u16")]
    pub weapon_unlocked: u16,
    /// Enchantment level of the equipped weapon (mirrors `Player::enchant`).
    #[serde(default)]
    pub enchant: u8,
    /// Persisted town layout (tile + kind). Captured when first generated so the
    /// city is rebuilt identically instead of re-rolled on every load.
    #[serde(default)]
    pub town: Option<Vec<(i32, i32, game::building::StructureKind)>>,
    /// Whether the player has already visited the town (its creation animation
    /// only plays on the very first arrival).
    #[serde(default)]
    pub town_visited: bool,
}
