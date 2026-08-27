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
            8 => "Press E at the altar to reforge the Star Crown".into(),
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
