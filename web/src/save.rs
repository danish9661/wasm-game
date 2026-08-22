use game::building::Structure;
use game::enemy::EnemyKind;
use game::items::ItemKind;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerSave {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub hunger: f32,
    pub stamina: f32,
    pub facing: (f32, f32),
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
    pub enemies: Vec<(EnemyKind, f32, f32, f32)>,
    pub quest_stage: u8,
    pub slimes_killed: u32,
    pub boss_killed: u32,
    pub boss_spawned: bool,
    pub altar_placed: bool,
    pub altar_tile: Option<(i32, i32)>,
    pub ending_pending: bool,
    pub ending: Option<u8>,
    pub ng_plus: u32,
    pub time_of_day: f32,
    pub spawn_point: (f32, f32),
}
