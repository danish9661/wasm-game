//! Story progression plus the deterministic reward curve for NPC fetch quests.
//!
//! Reward scaling is stateless integer math: no new save field is introduced,
//! and the values are clamped before they reach the inventory or XP counter.

use serde::{Deserialize, Serialize};

use crate::reputation::{ReputationTier, tier_for_standing};

/// Ceilings for scaled rewards. XP and stack counts stay small enough that
/// `add_xp` / `Inventory::add` cannot overflow even after a long session.
pub const MAX_QUEST_XP: u32 = 10_000;
pub const MAX_QUEST_COUNT: u32 = 9;
/// Max bonus percent from level (+10%/level above 1, capped) and standing.
pub const MAX_LEVEL_BONUS_PCT: u64 = 150;

/// Reward after level/reputation scaling. Plain integers: safe to serialise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct QuestReward {
    #[serde(default)]
    pub xp: u32,
    #[serde(default)]
    pub count: u32,
}

/// Integer percentage bonus from player level (0 at level 1).
pub fn level_bonus_pct(level: u32) -> u64 {
    (level.saturating_sub(1) as u64 * 10).min(MAX_LEVEL_BONUS_PCT)
}

/// Integer percentage bonus from village standing: allies pay more, hostile
/// villages pay less. Uses the same tiers as prices and guard dialogue.
pub fn standing_bonus_pct(standing: i32) -> i64 {
    match tier_for_standing(standing) {
        ReputationTier::Hostile => -20,
        ReputationTier::Neutral => 0,
        ReputationTier::Friendly => 10,
        ReputationTier::Allied => 25,
    }
}

/// Scale a shop quest's base XP and item count. Both inputs and outputs are
/// integers, clamped, and monotonic in level/standing. No RNG and no floats,
/// so old saves and co-op clients compute exactly the same reward.
pub fn scaled_reward(base_xp: u32, base_count: u32, level: u32, standing: i32) -> QuestReward {
    let pct = (100i64 + level_bonus_pct(level) as i64 + standing_bonus_pct(standing))
        .clamp(50, 275) as u64;
    let xp = (base_xp as u64).saturating_mul(pct) / 100;
    // +1 item for every 8 levels and another for allied standing, capped.
    let count = base_count
        .saturating_add(level / 8)
        .saturating_add((standing >= 60) as u32)
        .min(MAX_QUEST_COUNT);
    QuestReward {
        xp: xp.min(MAX_QUEST_XP as u64) as u32,
        count: if base_count == 0 { 0 } else { count.max(1) },
    }
}

/// Chapter 1-4 story beats. A small FSM advanced by world facts; the web
/// layer feeds it facts each frame and the HUD shows `quest_text()`.
///
/// Stages: 0 gather, 1 shelter, 2 craft iron plate, 3 first kill, 4 find
/// ruins, 5 open chest, 6 defeat a Crown Fragment guardian, 7 recover all five
/// fragments, 8 reforge the Crown at the altar (campaign complete). Defeating
/// the bonus Colossus as well unlocks the *true* ending text at stage 9.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestLog {
    pub stage: u8,
    pub colossus_defeated: bool,
}

impl QuestLog {
    pub fn new() -> Self {
        Self {
            stage: 0,
            colossus_defeated: false,
        }
    }

    /// Advance the story from the current game facts. Facts are cheap to
    /// gather, so this is safe to call every frame.
    pub fn update(
        &mut self,
        wood: u32,
        stone: u32,
        has_wall: bool,
        has_campfire: bool,
        has_anvil: bool,
        crafted_iron: bool,
        slimes_killed: u32,
        near_ruins: bool,
        chest_opened: bool,
        fragments: u8,
        altar_used: bool,
        colossus_defeated: bool,
    ) {
        let s = self.stage;
        if s == 0 && wood >= 5 && stone >= 1 {
            self.stage = 1;
        } else if s == 1 && has_wall && has_campfire {
            self.stage = 2;
        } else if s == 2 && has_anvil && crafted_iron {
            self.stage = 3;
        } else if s == 3 && slimes_killed >= 1 {
            self.stage = 4;
        } else if s == 4 && near_ruins {
            self.stage = 5;
        } else if s == 5 && chest_opened {
            self.stage = 6;
        } else if s == 6 && fragments >= 1 {
            self.stage = 7;
        } else if s == 7 && fragments == 0b11111 {
            self.stage = 8;
        } else if s == 8 && altar_used {
            self.stage = 9;
        }
        self.colossus_defeated = colossus_defeated;
    }

    /// True when the player has seen the secret (Colossus) ending.
    pub fn true_ending(&self) -> bool {
        self.stage >= 9 && self.colossus_defeated
    }

    pub fn quest_text(&self, fragments: u8) -> String {
        match self.stage {
            0 => "Gather 5 wood and 1 stone".into(),
            1 => "Build a wall and a campfire".into(),
            2 => "Craft Iron Plate at an Anvil".into(),
            3 => "A slime blocks the ruins - defeat it".into(),
            4 => "Find the ancient ruins".into(),
            5 => "Open the ruins chest".into(),
            6 => format!(
                "Hunt the Crown Fragment guardians ({}/5 recovered)",
                fragments.count_ones()
            ),
            7 => "Carry all 5 fragments to the altar where you woke".into(),
            8 => {
                if self.colossus_defeated {
                    "Answer the altar (Y = Reign, N = Shatter)".into()
                } else {
                    "Answer the altar (Y/N) — slay the Stone Colossus first for the true ending".into()
                }
            }
            _ => {
                if self.colossus_defeated {
                    "The Twin Star Crowns blaze - the world is whole (true ending)".into()
                } else {
                    "The Star Crown blazes - the world is healed".into()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_rewards_scale_with_level_and_standing() {
        let neutral = scaled_reward(20, 2, 1, 0);
        assert_eq!(neutral, QuestReward { xp: 20, count: 2 }, "level 1 neutral is base");
        let high = scaled_reward(20, 2, 20, 80);
        assert!(high.xp > neutral.xp, "high level allied pays more xp");
        assert!(high.count > neutral.count, "and a bigger stack");
        let hostile = scaled_reward(20, 2, 1, -50);
        assert!(hostile.xp < neutral.xp, "hostile villages pay less");
        assert_eq!(hostile.count, 2);
        // Neither dimension alone can skip the other.
        assert!(scaled_reward(20, 2, 20, 0).xp > neutral.xp);
        assert!(scaled_reward(20, 2, 1, 80).xp > neutral.xp);
    }

    #[test]
    fn rewards_are_bounded_and_serialisable() {
        let huge = scaled_reward(u32::MAX, u32::MAX, u32::MAX, i32::MAX);
        assert!(huge.xp <= MAX_QUEST_XP);
        assert!(huge.count <= MAX_QUEST_COUNT);
        let tiny = scaled_reward(1, 0, 0, i32::MIN);
        assert_eq!(tiny.count, 0, "no item reward stays none");
        assert!(tiny.xp <= 1, "xp stays a tiny non-negative integer");
        let json = serde_json::to_string(&huge).unwrap();
        let back: QuestReward = serde_json::from_str(&json).unwrap();
        assert_eq!(back, huge);
        let legacy: QuestReward = serde_json::from_str("{}").unwrap();
        assert_eq!(legacy, QuestReward::default());
        assert_eq!(level_bonus_pct(u32::MAX), MAX_LEVEL_BONUS_PCT);
    }

    #[test]
    fn advances_through_chapter_one() {
        let mut q = QuestLog::new();
        assert_eq!(q.stage, 0);

        q.update(5, 1, false, false, false, false, 0, false, false, 0, false, false);
        assert_eq!(q.stage, 1, "harvest milestone");

        q.update(5, 1, true, true, false, false, 0, false, false, 0, false, false);
        assert_eq!(q.stage, 2, "shelter milestone");

        // crafting iron plate requires an anvil AND the craft having happened
        q.update(5, 1, true, true, true, false, 0, false, false, 0, false, false);
        assert_eq!(q.stage, 2, "anvil alone is not enough");
        q.update(5, 1, true, true, true, true, 0, false, false, 0, false, false);
        assert_eq!(q.stage, 3, "crafting milestone");

        q.update(5, 1, true, true, true, true, 1, false, false, 0, false, false);
        assert_eq!(q.stage, 4, "first kill milestone");

        q.update(5, 1, true, true, true, true, 1, true, false, 0, false, false);
        assert_eq!(q.stage, 5, "found the ruins");

        q.update(5, 1, true, true, true, true, 1, true, true, 0, false, false);
        assert_eq!(q.stage, 6, "chest opened");
    }

    #[test]
    fn advances_through_boss_and_finale() {
        let mut q = QuestLog::new();
        q.stage = 6;
        q.update(5, 1, true, true, true, true, 1, true, true, 1, false, false);
        assert_eq!(q.stage, 7, "one fragment recovered");

        q.update(5, 1, true, true, true, true, 1, true, true, 0b11111, false, false);
        assert_eq!(q.stage, 8, "all five fragments -> altar beat");

        q.update(5, 1, true, true, true, true, 1, true, true, 0b11111, true, false);
        assert_eq!(q.stage, 9, "altar used -> campaign complete");
    }

    #[test]
    fn colossus_gates_the_true_ending() {
        // first (Warden-only) ending
        let mut q = QuestLog::new();
        q.stage = 9;
        q.update(5, 1, true, true, true, true, 1, true, true, 0b11111, true, false);
        assert!(!q.true_ending());
        assert_eq!(q.quest_text(0b11111), "The Star Crown blazes - the world is healed");

        // defeating the Colossus flips the true ending on
        q.update(5, 1, true, true, true, true, 1, true, true, 0b11111, true, true);
        assert!(q.true_ending());
        assert!(q.quest_text(0b11111).contains("true ending"));
    }

    #[test]
    fn does_not_skip_milestones() {
        let mut q = QuestLog::new();
        q.update(5, 1, true, true, true, true, 3, true, true, 0b11111, true, true);
        assert_eq!(q.stage, 1, "shelter facts must not skip the harvest beat");
    }

    #[test]
    fn text_exists_for_every_stage() {
        let q = QuestLog::new();
        for s in 0..=9 {
            let mut q = q.clone();
            q.stage = s;
            assert!(!q.quest_text(0b10101).is_empty());
        }
    }
}
