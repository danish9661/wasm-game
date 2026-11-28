//! Runtime animation integration: style/kind -> authored clip names.
//!
//! The GLB clips live in `assets/animations/{player,wave2,wave3,wave4}` and
//! their sampled root/hip motion is baked into `web/src/generated_animations.rs`
//! by `scripts/gen_animation_data.py`. This module is the small, pure mapping
//! the engine uses to decide *which* authored clip a visible character plays.
//! It deliberately has no GPU or wgpu dependency so the mapping can be tested
//! with `cargo test -p game`.
//!
//! Design notes (see docs/ANIM_INTEGRATION.md):
//! * one `ClipSet` per body family (idle / walk / run / attack / hurt / death);
//! * `SpriteStyle` -> family is a compact match, so adding a creature is a
//!   one-line entry rather than a new animation system;
//! * states have a fixed priority (death > hurt > attack > locomotion > idle),
//!   which matches the telegraphs the gameplay sim already emits;
//! * where a family has no authored clip for a state (slime has no dedicated
//!   attack, most bosses have no walk/hurt/death), the fallback names are
//!   recorded explicitly in the table rather than being silently invented.

use crate::render::SpriteStyle;

/// Semantic animation state of one visible character instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum AnimState {
    /// Standing/breathing/watching.
    #[default]
    Idle,
    /// Steady travel (patrol, civilian wander, chase).
    Walk,
    /// Fast travel (flee, charge).
    Run,
    /// Committed strike or telegraphed wind-up.
    Attack,
    /// Brief hit reaction while the damage flash is active.
    Hurt,
    /// Collapse / corpse pose.
    Death,
}

impl AnimState {
    pub fn as_str(self) -> &'static str {
        match self {
            AnimState::Idle => "idle",
            AnimState::Walk => "walk",
            AnimState::Run => "run",
            AnimState::Attack => "attack",
            AnimState::Hurt => "hurt",
            AnimState::Death => "death",
        }
    }
}

/// The authored clips a body family uses, by semantic state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipSet {
    pub idle: &'static str,
    pub walk: &'static str,
    pub run: &'static str,
    pub attack: &'static str,
    pub hurt: &'static str,
    pub death: &'static str,
}

impl ClipSet {
    pub const fn new(
        idle: &'static str,
        walk: &'static str,
        run: &'static str,
        attack: &'static str,
        hurt: &'static str,
        death: &'static str,
    ) -> Self {
        Self { idle, walk, run, attack, hurt, death }
    }

    /// Resolve a state to a clip name. Families without a distinct run clip
    /// intentionally reuse their walk clip (a hop, scuttle or shamble).
    pub fn get(self, state: AnimState) -> &'static str {
        match state {
            AnimState::Idle => self.idle,
            AnimState::Walk => self.walk,
            AnimState::Run => self.run,
            AnimState::Attack => self.attack,
            AnimState::Hurt => self.hurt,
            AnimState::Death => self.death,
        }
    }
}

// ---- Authored clip sets ----------------------------------------------------
// Every name below exists as a GLB in assets/animations/ and has a sampled
// entry in web/src/generated_animations.rs. Fallbacks are documented inline.

const GOBLIN: ClipSet = ClipSet::new(
    "goblin_idle", "goblin_walk_f", "goblin_run_f",
    "goblin_attack", "goblin_hit_light", "goblin_death");
const SKELETON: ClipSet = ClipSet::new(
    "skeleton_idle", "skeleton_walk_f", "skeleton_run_f",
    "skeleton_attack", "skeleton_hit_light", "skeleton_death");
const WOLF: ClipSet = ClipSet::new(
    "wolf_idle", "wolf_walk_f", "wolf_run_f",
    "wolf_attack", "wolf_hit_light", "wolf_death");
const ARCHER: ClipSet = ClipSet::new(
    "archer_idle", "archer_walk_f", "archer_run_f",
    "archer_shoot", "archer_hit_light", "archer_death");
const SPIDER: ClipSet = ClipSet::new(
    "spider_idle", "spider_scuttle", "spider_scuttle",
    "spider_attack", "spider_hit_light", "spider_death_alt");
// Slime has no dedicated attack in the authored waves: its hop doubles as the
// strike (it is the only aggressive motion the rig has).
const SLIME: ClipSet = ClipSet::new(
    "slime_idle", "slime_hop", "slime_hop",
    "slime_hop", "slime_hit_light", "slime_death_alt");
const BRUTE: ClipSet = ClipSet::new(
    "brute_idle", "brute_walk", "brute_run",
    "brute_smash", "brute_hit_light", "brute_death");
// Civilian / guard humanoids use the first-wave clips for hurt and death:
// those are authored on the standing human rig, unlike the enemy pivot rigs.
const CIVILIAN: ClipSet = ClipSet::new(
    "villager_idle", "guard_patrol", "guard_patrol",
    "guard_salute", "hit_light", "death");
const GOLEM: ClipSet = ClipSet::new(
    "golem_standby", "golem_work", "golem_work",
    "golem_work", "hit_light", "death");
const IMP: ClipSet = ClipSet::new(
    "imp_hover", "imp_mischief", "imp_mischief",
    "imp_mischief", "goblin_hit_light", "goblin_death");
const RAIDER: ClipSet = ClipSet::new(
    "raider_sneak", "raider_sneak", "raider_sneak",
    "raider_slash", "goblin_hit_light", "goblin_death");
// Boss families keep their own telegraph/ultimate for the attack state and
// borrow the closest creature's locomotion/hurt/death where none was authored.
const COLOSSUS: ClipSet = ClipSet::new(
    "golem_standby", "golem_work", "golem_work",
    "colossus_ultimate", "brute_hit_light", "brute_death");
const SCORPION_QUEEN: ClipSet = ClipSet::new(
    "spider_idle", "spider_scuttle", "spider_scuttle",
    "scorpion_queen_attack", "spider_hit_light", "spider_death_alt");
const TOAD_KING: ClipSet = ClipSet::new(
    "slime_idle", "slime_hop", "slime_hop",
    "toad_king_attack", "slime_hit_light", "slime_death_alt");
const STORMCALLER: ClipSet = ClipSet::new(
    "archer_idle", "archer_walk_f", "archer_run_f",
    "stormcaller_attack", "archer_hit_light", "archer_death");
const OCEAN_LEVIATHAN: ClipSet = ClipSet::new(
    "slime_idle", "slime_hop", "slime_hop",
    "ocean_leviathan_attack", "slime_hit_light", "slime_death_alt");

/// Body family, kept private so the public API stays just style -> clips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    Goblin,
    Skeleton,
    Wolf,
    Archer,
    Spider,
    Slime,
    Brute,
    Civilian,
    Golem,
    Imp,
    Raider,
    Colossus,
    ScorpionQueen,
    ToadKing,
    Stormcaller,
    OceanLeviathan,
}

impl Family {
    fn clips(self) -> ClipSet {
        match self {
            Family::Goblin => GOBLIN,
            Family::Skeleton => SKELETON,
            Family::Wolf => WOLF,
            Family::Archer => ARCHER,
            Family::Spider => SPIDER,
            Family::Slime => SLIME,
            Family::Brute => BRUTE,
            Family::Civilian => CIVILIAN,
            Family::Golem => GOLEM,
            Family::Imp => IMP,
            Family::Raider => RAIDER,
            Family::Colossus => COLOSSUS,
            Family::ScorpionQueen => SCORPION_QUEEN,
            Family::ToadKing => TOAD_KING,
            Family::Stormcaller => STORMCALLER,
            Family::OceanLeviathan => OCEAN_LEVIATHAN,
        }
    }
}

/// Map a gameplay sprite style to its body family. Decorative styles and the
/// billboard overlays deliberately fall through to `None` so only characters
/// are animated in the 3D pass.
fn family_for_style(style: SpriteStyle) -> Option<Family> {
    Some(match style {
        SpriteStyle::Goblin => Family::Goblin,
        SpriteStyle::Skeleton | SpriteStyle::Wraith => Family::Skeleton,
        SpriteStyle::Wolf => Family::Wolf,
        SpriteStyle::Archer | SpriteStyle::Stoneslinger => Family::Archer,
        SpriteStyle::Spider => Family::Spider,
        SpriteStyle::Slime => Family::Slime,
        SpriteStyle::Brute | SpriteStyle::Ogre => Family::Brute,
        SpriteStyle::Humanoid | SpriteStyle::Guard => Family::Civilian,
        SpriteStyle::Golem => Family::Golem,
        SpriteStyle::Imp | SpriteStyle::Bat => Family::Imp,
        SpriteStyle::Raider => Family::Raider,
        SpriteStyle::Colossus => Family::Colossus,
        SpriteStyle::ScorpionQueen => Family::ScorpionQueen,
        SpriteStyle::ToadKing => Family::ToadKing,
        SpriteStyle::Stormcaller => Family::Stormcaller,
        SpriteStyle::OceanLeviathan => Family::OceanLeviathan,
        _ => return None,
    })
}

/// The authored clip set for a style, or `None` for props/overlays.
pub fn clips_for_style(style: SpriteStyle) -> Option<ClipSet> {
    family_for_style(style).map(Family::clips)
}

/// Resolve one gameplay state to a clip name for a style.
/// `None` means the style is not an animated character.
pub fn clip_for(style: SpriteStyle, state: AnimState) -> Option<&'static str> {
    clips_for_style(style).map(|set| set.get(state))
}

/// Derive an animation state from the signals the sim already exposes
/// (`walk`/`attack` on Sprite plus a hit flash and a death flag). Priorities
/// match the gameplay telegraphs: a hit or a corpse wins over locomotion.
pub fn state_from_signals(walk: f32, attack: f32, hurt: f32, dead: bool) -> AnimState {
    if dead {
        AnimState::Death
    } else if hurt > 0.25 {
        AnimState::Hurt
    } else if attack > 0.05 {
        AnimState::Attack
    } else if walk >= 0.75 {
        AnimState::Run
    } else if walk > 0.08 {
        AnimState::Walk
    } else {
        AnimState::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_states_select_distinct_clips() {
        // The core regression: a single enemy kind must not play the same
        // authored clip while idle, walking, striking, reacting and dying.
        let set = clips_for_style(SpriteStyle::Goblin).expect("goblin has clips");
        let idle = set.get(AnimState::Idle);
        let walk = set.get(AnimState::Walk);
        let attack = set.get(AnimState::Attack);
        let hurt = set.get(AnimState::Hurt);
        let death = set.get(AnimState::Death);
        let picked = [idle, walk, attack, hurt, death];
        let distinct: std::collections::BTreeSet<_> = picked.iter().copied().collect();
        assert!(
            distinct.len() >= 3,
            "expected >=3 distinct clips for distinct states, got {:?}",
            picked
        );
        assert_eq!(distinct.len(), 5, "all five goblin states should be authored: {:?}", picked);
        assert_eq!(idle, "goblin_idle");
        assert_eq!(walk, "goblin_walk_f");
        assert_eq!(attack, "goblin_attack");
        assert_eq!(hurt, "goblin_hit_light");
        assert_eq!(death, "goblin_death");
    }

    #[test]
    fn distinct_kinds_select_distinct_clips() {
        let goblin = clips_for_style(SpriteStyle::Goblin).unwrap();
        let skeleton = clips_for_style(SpriteStyle::Skeleton).unwrap();
        let wolf = clips_for_style(SpriteStyle::Wolf).unwrap();
        assert_ne!(goblin.idle, skeleton.idle);
        assert_ne!(skeleton.idle, wolf.idle);
        assert_ne!(goblin.walk, wolf.walk);
        // Boss families keep their own telegraph while borrowing locomotion.
        assert_eq!(
            clip_for(SpriteStyle::ScorpionQueen, AnimState::Attack),
            Some("scorpion_queen_attack")
        );
        assert_eq!(
            clip_for(SpriteStyle::Colossus, AnimState::Attack),
            Some("colossus_ultimate")
        );
    }

    #[test]
    fn signals_map_to_distinct_states() {
        let idle = state_from_signals(0.0, 0.0, 0.0, false);
        let walk = state_from_signals(0.5, 0.0, 0.0, false);
        let run = state_from_signals(1.0, 0.0, 0.0, false);
        let attack = state_from_signals(0.0, 1.0, 0.0, false);
        let hurt = state_from_signals(0.0, 0.0, 1.0, false);
        let death = state_from_signals(0.0, 0.0, 0.0, true);
        let states = [idle, walk, run, attack, hurt, death];
        let distinct: std::collections::BTreeSet<_> = states.iter().copied().collect();
        assert!(distinct.len() >= 3, "states collapsed: {:?}", states);
        assert_eq!(states, [
            AnimState::Idle,
            AnimState::Walk,
            AnimState::Run,
            AnimState::Attack,
            AnimState::Hurt,
            AnimState::Death,
        ]);
        // Hurt outranks an active attack, death outranks everything.
        assert_eq!(state_from_signals(1.0, 1.0, 1.0, false), AnimState::Hurt);
        assert_eq!(state_from_signals(1.0, 1.0, 1.0, true), AnimState::Death);
    }

    #[test]
    fn non_characters_have_no_clips() {
        for style in [
            SpriteStyle::Tree,
            SpriteStyle::Rock,
            SpriteStyle::HpBack,
            SpriteStyle::HpFill,
            SpriteStyle::Arrow,
        ] {
            assert_eq!(clips_for_style(style), None, "{:?} must not animate", style);
        }
    }
}
