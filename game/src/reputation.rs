//! Per-village reputation.
//!
//! Standing is a small clamped integer keyed by the deterministic village tile
//! from `game::poi::village_sites`. Quests, honest trade and defending a hamlet
//! raise it; it is never derived from world state, so it must be saved. Merchants
//! reward good standing with cheaper buys / better sell prices and guards change
//! their greeting. A fresh or old save has no entries and reads as neutral.

use serde::{Deserialize, Serialize};

/// Clamp bounds: reputation cannot be farmed without limit in either direction.
pub const MIN_STANDING: i32 = -100;
pub const MAX_STANDING: i32 = 100;
/// Radius (tiles) around a village centre where a kill counts as defending it.
pub const VILLAGE_DEFENSE_RADIUS: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReputationTier {
    Hostile,
    Neutral,
    Friendly,
    Allied,
}

impl ReputationTier {
    pub fn label(self) -> &'static str {
        match self {
            ReputationTier::Hostile => "Hostile",
            ReputationTier::Neutral => "Neutral",
            ReputationTier::Friendly => "Friendly",
            ReputationTier::Allied => "Allied",
        }
    }

    /// Line a guard uses when the player walks up. Pure text, deterministic.
    pub fn guard_line(self) -> &'static str {
        match self {
            ReputationTier::Hostile => "Guard: \"Outsider, watch your step.\"",
            ReputationTier::Neutral => "Guard: \"Keep to the paths and the watch won't mind.\"",
            ReputationTier::Friendly => "Guard: \"Good to see you again, friend.\"",
            ReputationTier::Allied => "Guard: \"A champion of our village! Welcome home.\"",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Reputation {
    /// Kept as a flat vector (not a HashMap) so JSON saves stay stable and
    /// deterministic in ordering. Villages are few (3 per world).
    #[serde(default)]
    standings: Vec<((i32, i32), i32)>,
}

impl Reputation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn standing(&self, village: (i32, i32)) -> i32 {
        self.standings
            .iter()
            .find(|(v, _)| *v == village)
            .map_or(0, |(_, s)| *s)
    }

    /// Change standing and return the new clamped value. Unknown villages are
    /// inserted; clamping keeps prices bounded forever.
    pub fn adjust(&mut self, village: (i32, i32), delta: i32) -> i32 {
        let next = if let Some(entry) = self.standings.iter_mut().find(|(v, _)| *v == village) {
            entry.1 = (entry.1.saturating_add(delta)).clamp(MIN_STANDING, MAX_STANDING);
            entry.1
        } else {
            let value = delta.clamp(MIN_STANDING, MAX_STANDING);
            self.standings.push((village, value));
            // Keep ordering deterministic regardless of the order villages were
            // visited in (saves diff cleanly and tests are reproducible).
            self.standings.sort_by_key(|(v, _)| *v);
            value
        };
        next
    }

    pub fn tier(&self, village: (i32, i32)) -> ReputationTier {
        match self.standing(village) {
            s if s <= -30 => ReputationTier::Hostile,
            s if s < 15 => ReputationTier::Neutral,
            s if s < 60 => ReputationTier::Friendly,
            _ => ReputationTier::Allied,
        }
    }

    /// Guard greeting for a village, ready to drop into a toast.
    pub fn guard_line_of(&self, village: (i32, i32)) -> &'static str {
        self.tier(village).guard_line()
    }

    /// Buy multiplier: 1.25 at hostile, 1.00 at neutral, 0.80 at allied.
    pub fn buy_factor(&self, village: (i32, i32)) -> f32 {
        price_factor(self.standing(village))
    }

    /// Sell multiplier: the mirror of `buy_factor` (0.80 .. 1.25).
    pub fn sell_factor(&self, village: (i32, i32)) -> f32 {
        price_factor(-self.standing(village))
    }

    pub fn entries(&self) -> &[((i32, i32), i32)] {
        &self.standings
    }

    /// Which known village (if any) is nearest to a world position. Uses the
    /// same tile coordinates the NPC home anchors use.
    pub fn nearest_village(
        &self,
        x: f32,
        y: f32,
        villages: &[(i32, i32, String)],
    ) -> Option<(i32, i32)> {
        villages
            .iter()
            .map(|(vx, vy, _)| {
                let d = (x - (*vx as f32 + 0.5)).hypot(y - (*vy as f32 + 0.5));
                (d, (*vx, *vy))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, v)| v)
    }

    /// Standing bonus for slaying a hostile in a village's defence radius.
    /// Returns the village credited, if any. `delta` is +1 for ordinary kills,
    /// +2 for elites/bosses (the caller decides which).
    pub fn credit_defense(
        &mut self,
        x: f32,
        y: f32,
        villages: &[(i32, i32, String)],
        delta: i32,
    ) -> Option<(i32, i32)> {
        let v = self.nearest_village(x, y, villages)?;
        let (vx, vy) = v;
        let d = (x - (vx as f32 + 0.5)).hypot(y - (vy as f32 + 0.5));
        if d <= VILLAGE_DEFENSE_RADIUS {
            self.adjust(v, delta);
            Some(v)
        } else {
            None
        }
    }
}

/// Linear, deterministic price curve. Neutral (0) is exactly 1.0 so existing
/// prices and tests are unchanged until the player actually earns standing.
fn price_factor(standing: i32) -> f32 {
    let s = standing.clamp(MIN_STANDING, MAX_STANDING) as f32;
    1.0 - 0.0025 * s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_old_saves_are_neutral() {
        let r: Reputation = serde_json::from_str("{}").expect("missing field defaults");
        assert_eq!(r.standing((3, 4)), 0);
        assert_eq!(r.tier((3, 4)), ReputationTier::Neutral);
        assert!((r.buy_factor((3, 4)) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn standing_is_per_village_and_clamped() {
        let mut r = Reputation::new();
        assert_eq!(r.adjust((3, 4), 20), 20);
        assert_eq!(r.standing((3, 4)), 20);
        assert_eq!(r.standing((9, 9)), 0, "other villages unaffected");
        r.adjust((3, 4), 1000);
        assert_eq!(r.standing((3, 4)), MAX_STANDING);
        r.adjust((3, 4), -1000);
        assert_eq!(r.standing((3, 4)), MIN_STANDING);
        assert_eq!(r.entries().len(), 1, "one entry per village");
    }

    #[test]
    fn tiers_and_guard_dialogue_change_with_standing() {
        let mut r = Reputation::new();
        assert_eq!(r.tier((0, 0)), ReputationTier::Neutral);
        r.adjust((0, 0), -40);
        assert_eq!(r.tier((0, 0)), ReputationTier::Hostile);
        assert!(r.guard_line_of((0, 0)).contains("Outsider"));
        r.adjust((0, 0), 120);
        assert_eq!(r.tier((0, 0)), ReputationTier::Allied);
        assert!(r.guard_line_of((0, 0)).contains("champion"));
    }

    #[test]
    fn prices_move_in_opposite_directions() {
        let mut r = Reputation::new();
        let neutral = r.buy_factor((0, 0));
        r.adjust((0, 0), 80);
        assert!(r.buy_factor((0, 0)) < neutral, "friends buy cheaper");
        assert!(r.sell_factor((0, 0)) > neutral, "friends sell for more");
        r.adjust((0, 0), -160);
        assert!(r.buy_factor((0, 0)) > neutral, "hostiles pay more");
        assert!(r.sell_factor((0, 0)) < neutral);
    }

    #[test]
    fn defense_kills_credit_only_nearby_villages() {
        let mut r = Reputation::new();
        let villages = vec![(10, 10, "Bryn".to_string())];
        assert_eq!(r.credit_defense(12.0, 10.5, &villages, 2), Some((10, 10)));
        assert_eq!(r.standing((10, 10)), 2);
        assert_eq!(r.credit_defense(90.0, 90.0, &villages, 2), None);
        assert_eq!(r.standing((10, 10)), 2, "far kills pay nothing");
    }

    #[test]
    fn round_trips_through_json() {
        let mut r = Reputation::new();
        r.adjust((7, -2), 35);
        r.adjust((1, 1), -12);
        let json = serde_json::to_string(&r).unwrap();
        let back: Reputation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
        assert_eq!(back.standing((7, -2)), 35);
    }
}
