/// Chapter 1-4 story beats. A small FSM advanced by world events; the web
/// layer feeds it facts each frame and the HUD shows `quest_text()`.
///
/// Stages: 0 gather, 1 shelter, 2 first kill, 3 find ruins, 4 open chest,
/// 5 defeat the Warden boss, 6 recover the Crown Fragment, 7 reforge the
/// Crown at the altar (campaign complete). Defeating the Colossus as well
/// unlocks the *true* ending text at stage 8.
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
        slimes_killed: u32,
        near_ruins: bool,
        chest_opened: bool,
        boss_defeated: bool,
        has_fragment: bool,
        altar_used: bool,
        colossus_defeated: bool,
    ) {
        let s = self.stage;
        if s == 0 && wood >= 5 && stone >= 1 {
            self.stage = 1;
        } else if s == 1 && has_wall && has_campfire {
            self.stage = 2;
        } else if s == 2 && slimes_killed >= 1 {
            self.stage = 3;
        } else if s == 3 && near_ruins {
            self.stage = 4;
        } else if s == 4 && chest_opened {
            self.stage = 5;
        } else if s == 5 && boss_defeated {
            self.stage = 6;
        } else if s == 6 && has_fragment {
            self.stage = 7;
        } else if s == 7 && altar_used {
            self.stage = 8;
        }
        self.colossus_defeated = colossus_defeated;
    }

    /// True when the player has seen the secret (Colossus) ending.
    pub fn true_ending(&self) -> bool {
        self.stage >= 8 && self.colossus_defeated
    }

    pub fn quest_text(&self) -> &'static str {
        match self.stage {
            0 => "Gather 5 wood and 1 stone",
            1 => "Build a wall and a campfire",
            2 => "A slime blocks the ruins - defeat it",
            3 => "Find the ancient ruins",
            4 => "Open the ruins chest",
            5 => "The Forest Warden guards a Crown Fragment - also hunt the Colossus",
            6 => "Carry the fragment to the altar where you woke",
            7 => "Press E at the altar to reforge the Crown",
            _ => {
                if self.colossus_defeated {
                    "The Twin Star Crowns blaze - the world is whole (true ending)"
                } else {
                    "The Star Crown blazes - the world is healed"
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

        q.update(5, 1, false, false, 0, false, false, false, false, false, false);
        assert_eq!(q.stage, 1, "harvest milestone");

        q.update(5, 1, true, true, 0, false, false, false, false, false, false);
        assert_eq!(q.stage, 2, "shelter milestone");

        q.update(5, 1, true, true, 1, false, false, false, false, false, false);
        assert_eq!(q.stage, 3, "first kill milestone");

        q.update(5, 1, true, true, 1, true, false, false, false, false, false);
        assert_eq!(q.stage, 4, "found the ruins");

        q.update(5, 1, true, true, 1, true, true, false, false, false, false);
        assert_eq!(q.stage, 5, "chest opened");
    }

    #[test]
    fn advances_through_boss_and_finale() {
        let mut q = QuestLog::new();
        q.stage = 5;
        q.update(5, 1, true, true, 1, true, true, true, false, false, false);
        assert_eq!(q.stage, 6, "boss defeated -> fragment beat");

        q.update(5, 1, true, true, 1, true, true, true, true, false, false);
        assert_eq!(q.stage, 7, "fragment recovered -> altar beat");

        q.update(5, 1, true, true, 1, true, true, true, true, true, false);
        assert_eq!(q.stage, 8, "altar used -> campaign complete");
    }

    #[test]
    fn colossus_gates_the_true_ending() {
        // first (Warden-only) ending
        let mut q = QuestLog::new();
        q.stage = 8;
        q.update(5, 1, true, true, 1, true, true, true, true, true, false);
        assert!(!q.true_ending());
        assert_eq!(
            q.quest_text(),
            "The Star Crown blazes - the world is healed"
        );

        // defeating the Colossus flips the true ending on
        q.update(5, 1, true, true, 1, true, true, true, true, true, true);
        assert!(q.true_ending());
        assert!(q.quest_text().contains("true ending"));
    }

    #[test]
    fn does_not_skip_milestones() {
        let mut q = QuestLog::new();
        q.update(5, 1, true, true, 3, true, true, true, true, true, true);
        assert_eq!(q.stage, 1, "shelter facts must not skip the harvest beat");
    }

    #[test]
    fn text_exists_for_every_stage() {
        let q = QuestLog::new();
        for s in 0..=8 {
            let mut q = q.clone();
            q.stage = s;
            assert!(!q.quest_text().is_empty());
        }
    }
}
