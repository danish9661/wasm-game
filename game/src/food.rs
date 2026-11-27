//! Food preparation and spoilage.
//!
//! Raw food is the common drop (bushes, farms, merchants) and slowly goes bad
//! while it is carried. Cooking at a campfire turns two raw portions into one
//! cooked meal that is safe from spoilage and restores far more hunger (plus a
//! little health and thirst). All of it is plain data + deterministic ticks so
//! single-player, the co-op server and the headless tests agree exactly.

use serde::{Deserialize, Serialize};

use crate::items::{Inventory, ItemKind};
use crate::player::Player;

/// Seconds one raw portion survives in the pack before it spoils.
pub const SPOIL_SECONDS: f32 = 120.0;
/// Raw portions consumed by one cooked meal.
pub const COOK_INPUT: u32 = 2;
/// Raw food restores hunger only (kept as the baseline `Player::eat` value).
pub const RAW_HUNGER: f32 = 34.0;
/// A cooked meal restores substantially more hunger.
pub const COOKED_HUNGER: f32 = 58.0;
/// Cooked meals are warming: they mend a little health as well.
pub const COOKED_HP: f32 = 12.0;
/// ...and quench a little thirst.
pub const COOKED_THIRST: f32 = 10.0;

/// Carried-food state layered on top of the existing `ItemKind::Food` stack
/// (which stays the raw count). Only the cooked count and the spoil clock need
/// extra persistence; both are `#[serde(default)]` so old saves still load.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FoodSupply {
    #[serde(default)]
    cooked: u32,
    #[serde(default)]
    spoil_clock: f32,
}

impl Default for FoodSupply {
    fn default() -> Self {
        Self::new()
    }
}

impl FoodSupply {
    pub fn new() -> Self {
        Self { cooked: 0, spoil_clock: 0.0 }
    }

    /// Raw portions in the ordinary inventory stack.
    pub fn raw(&self, inv: &Inventory) -> u32 {
        inv.count(ItemKind::Food)
    }

    pub fn cooked(&self) -> u32 {
        self.cooked
    }

    /// Seconds until the next raw portion spoils (full interval if none).
    pub fn seconds_to_spoil(&self) -> f32 {
        (SPOIL_SECONDS - self.spoil_clock).max(0.0)
    }

    /// Cook two raw portions into one meal. Fails atomically when short, so a
    /// rejected cook never eats a single portion. The caller checks that a
    /// campfire is nearby — the cooking primitive itself is station-agnostic.
    pub fn cook(&mut self, inv: &mut Inventory) -> bool {
        if !inv.remove(ItemKind::Food, COOK_INPUT) {
            return false;
        }
        self.cooked += 1;
        true
    }

    /// Eat one portion, preferring the cooked meal. Returns false (changing
    /// nothing) when the pack is empty.
    pub fn eat(&mut self, player: &mut Player, inv: &mut Inventory) -> bool {
        if self.cooked > 0 {
            self.cooked -= 1;
            player.apply_nutrition(COOKED_HUNGER, COOKED_HP, COOKED_THIRST);
            return true;
        }
        player.eat(inv)
    }

    /// Eat specifically raw or cooked food, so tests and UI can be explicit.
    /// A cooked meal is always preferred when `cooked` is true and available.
    pub fn eat_choice(&mut self, player: &mut Player, inv: &mut Inventory, cooked: bool) -> bool {
        if cooked && self.cooked > 0 {
            self.cooked -= 1;
            player.apply_nutrition(COOKED_HUNGER, COOKED_HP, COOKED_THIRST);
            return true;
        }
        player.eat(inv)
    }

    /// Advance spoilage. Every `SPOIL_SECONDS` one raw portion rots away;
    /// cooked meals never spoil. Returns how many raw portions were lost so
    /// the caller can surface a single toast per tick.
    pub fn tick(&mut self, inv: &mut Inventory, dt: f32) -> u32 {
        if dt <= 0.0 {
            return 0;
        }
        if self.raw(inv) == 0 {
            self.spoil_clock = 0.0;
            return 0;
        }
        self.spoil_clock += dt;
        let mut spoiled = 0;
        while self.spoil_clock >= SPOIL_SECONDS && self.raw(inv) > 0 {
            self.spoil_clock -= SPOIL_SECONDS;
            inv.remove(ItemKind::Food, 1);
            spoiled += 1;
        }
        if self.raw(inv) == 0 {
            self.spoil_clock = 0.0;
        }
        spoiled
    }

    /// HUD hint: both counts plus the spoil warning when raw food is close to
    /// rotting. Cooked food is deliberately shown first as the safe option.
    pub fn hint(&self, inv: &Inventory) -> String {
        let raw = self.raw(inv);
        if raw == 0 && self.cooked == 0 {
            return "No food — forage berries, fish the shore or farm".to_string();
        }
        let mut s = format!("Raw {} · Cooked {}", raw, self.cooked);
        if raw > 0 {
            s.push_str(&format!(" (spoils in {:.0}s)", self.seconds_to_spoil()));
        }
        if self.cooked > 0 {
            s.push_str(" — cooked restores more");
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::MAX_HUNGER;

    #[test]
    fn cook_consumes_two_raw_and_yields_one_meal() {
        let mut food = FoodSupply::new();
        let mut inv = Inventory::new();
        inv.add(ItemKind::Food, 3);
        assert!(food.cook(&mut inv));
        assert_eq!(inv.count(ItemKind::Food), 1);
        assert_eq!(food.cooked(), 1);
        assert!(!food.cook(&mut inv), "one raw left cannot pay for a meal");
        assert_eq!(inv.count(ItemKind::Food), 1, "failed cook changes nothing");
        assert_eq!(food.cooked(), 1);
    }

    #[test]
    fn cooked_restores_more_than_raw() {
        let mut raw_p = Player::new(0.0, 0.0);
        raw_p.hunger = 10.0;
        let mut inv = Inventory::new();
        inv.add(ItemKind::Food, 2);
        assert!(raw_p.eat(&mut inv));
        assert!((raw_p.hunger - (10.0 + RAW_HUNGER)).abs() < 1e-4);

        let mut food = FoodSupply::new();
        inv.add(ItemKind::Food, 2);
        assert!(food.cook(&mut inv));
        let mut cooked_p = Player::new(0.0, 0.0);
        cooked_p.hunger = 10.0;
        cooked_p.hp = 50.0;
        cooked_p.thirst = 40.0;
        assert!(food.eat_choice(&mut cooked_p, &mut inv, true));
        assert!(
            cooked_p.hunger > raw_p.hunger,
            "cooked {} must beat raw {}",
            cooked_p.hunger,
            raw_p.hunger
        );
        assert!((cooked_p.hp - (50.0 + COOKED_HP)).abs() < 1e-4);
        assert!((cooked_p.thirst - (40.0 + COOKED_THIRST)).abs() < 1e-4);
        assert!(cooked_p.hunger <= MAX_HUNGER);
    }

    #[test]
    fn eating_prefers_cooked_and_is_clamped() {
        let mut food = FoodSupply::new();
        let mut inv = Inventory::new();
        inv.add(ItemKind::Food, 2);
        food.cook(&mut inv);
        let mut p = Player::new(0.0, 0.0);
        assert!(food.eat(&mut p, &mut inv));
        assert_eq!(food.cooked(), 0);
        assert_eq!(inv.count(ItemKind::Food), 0, "raw stack untouched");
        assert_eq!(p.hunger, MAX_HUNGER);
        assert!(!food.eat(&mut p, &mut inv));
    }

    #[test]
    fn raw_spoils_on_a_deterministic_clock() {
        let mut food = FoodSupply::new();
        let mut inv = Inventory::new();
        inv.add(ItemKind::Food, 3);
        assert_eq!(food.tick(&mut inv, SPOIL_SECONDS - 1.0), 0);
        assert_eq!(inv.count(ItemKind::Food), 3);
        assert_eq!(food.tick(&mut inv, 1.0), 1);
        assert_eq!(inv.count(ItemKind::Food), 2);
        assert_eq!(food.tick(&mut inv, SPOIL_SECONDS * 2.0), 2);
        assert_eq!(inv.count(ItemKind::Food), 0);
        assert_eq!(food.tick(&mut inv, SPOIL_SECONDS), 0, "nothing left to rot");
        assert_eq!(food.seconds_to_spoil(), SPOIL_SECONDS);
    }

    #[test]
    fn cooked_never_spoils_and_hint_shows_both_counts() {
        let mut food = FoodSupply::new();
        let mut inv = Inventory::new();
        inv.add(ItemKind::Food, 2);
        assert!(food.cook(&mut inv));
        assert!(!food.cook(&mut inv));
        let mut inv2 = Inventory::new();
        assert!(!food.cook(&mut inv2), "no raw food to cook");
        assert_eq!(food.tick(&mut inv2, SPOIL_SECONDS * 10.0), 0);
        assert_eq!(food.cooked(), 1);
        let hint = food.hint(&inv2);
        assert!(hint.contains("Cooked 1"), "{hint}");
    }

    #[test]
    fn serde_defaults_keep_old_saves_loadable() {
        let json = r#"{}"#;
        let mut food: FoodSupply = serde_json::from_str(json).expect("missing fields default");
        assert_eq!(food, FoodSupply::new());
        let mut inv = Inventory::new();
        inv.add(ItemKind::Food, 1);
        assert_eq!(food.tick(&mut inv, SPOIL_SECONDS), 1);
    }
}
