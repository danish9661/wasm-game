/// Chapter 1 story beats. A small FSM advanced by world events; the web
/// layer feeds it facts each frame and the HUD shows `quest_text()`.
///
/// Stages: 0 gather, 1 shelter, 2 first kill, 3 find ruins, 4 open chest,
/// 5 done (Chapter 2 tease).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestLog {
    pub stage: u8,
}

impl QuestLog {
    pub fn new() -> Self {
        Self { stage: 0 }
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
        }
    }

    pub fn quest_text(&self) -> &'static str {
        match self.stage {
            0 => "Gather 5 wood and 1 stone",
            1 => "Build a wall and a campfire",
            2 => "A slime blocks the ruins - defeat it",
            3 => "Find the ancient ruins",
            4 => "Open the ruins chest",
            _ => "The crown shard glows - Chapter 2 awaits",
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

        q.update(5, 1, false, false, 0, false, false);
        assert_eq!(q.stage, 1, "harvest milestone");

        q.update(5, 1, true, true, 0, false, false);
        assert_eq!(q.stage, 2, "shelter milestone");

        q.update(5, 1, true, true, 1, false, false);
        assert_eq!(q.stage, 3, "first kill milestone");

        q.update(5, 1, true, true, 1, true, false);
        assert_eq!(q.stage, 4, "found the ruins");

        q.update(5, 1, true, true, 1, true, true);
        assert_eq!(q.stage, 5, "chest opened");
    }

    #[test]
    fn does_not_skip_milestones() {
        let mut q = QuestLog::new();
        q.update(5, 1, true, true, 3, true, true);
        assert_eq!(q.stage, 1, "shelter facts must not skip the harvest beat");
    }

    #[test]
    fn text_exists_for_every_stage() {
        let q = QuestLog::new();
        for s in 0..=5 {
            let mut q = q.clone();
            q.stage = s;
            assert!(!q.quest_text().is_empty());
        }
    }
}