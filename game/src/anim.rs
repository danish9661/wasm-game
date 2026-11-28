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

// ---- Baked pose frame lookup ----------------------------------------------
// The renderer's NPC/prop pose tables are generated from optional manifests in
// `assets/models`. The resolution rules live here (pure Rust, no wasm-bindgen
// and no GPU types) so `cargo test -p game` can regression-test them on host.

/// One row of a generated baked-pose table. Implemented by the generated
/// `ExtraPoseClip` in the web crate and by `BakedClip` below (tests/fixtures).
pub trait BakedClipRow {
    /// NPC family or prop name the row belongs to.
    fn group(&self) -> &str;
    /// Semantic clip for NPCs (`idle`, `walk`, ...); empty for props.
    fn clip(&self) -> &str;
    /// GLB stems in playback order.
    fn frames(&self) -> &[&str];
    /// Authored playback rate, frames per second.
    fn fps(&self) -> f32;
}

/// A plain baked-pose row, used as a test fixture and as a runtime-friendly
/// shape for callers that build tables by hand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BakedClip<'a> {
    pub group: &'a str,
    pub clip: &'a str,
    pub frames: &'a [&'a str],
    pub fps: f32,
}

impl<'a> BakedClipRow for BakedClip<'a> {
    fn group(&self) -> &str { self.group }
    fn clip(&self) -> &str { self.clip }
    fn frames(&self) -> &[&str] { self.frames }
    fn fps(&self) -> f32 { self.fps }
}

/// Compare generated names ignoring case and `_`/`-`/space separators, so
/// `ScorpionQueen`, `scorpion_queen` and `SCORPION-QUEEN` all resolve.
fn normalized_name(s: &str) -> impl Iterator<Item = char> + '_ {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
}

fn name_eq(a: &str, b: &str) -> bool {
    normalized_name(a).eq(normalized_name(b))
}

/// Frame stem at time `t` seconds, wrapping at the end of the clip. Returns
/// `None` for an empty clip, a non-positive fps, or a non-finite time; callers
/// then keep their original static GLB (clean fallback when data is absent).
pub fn baked_frame_at<'a>(frames: &[&'a str], fps: f32, t: f32) -> Option<&'a str> {
    if frames.is_empty() || !fps.is_finite() || fps <= 0.0 || !t.is_finite() {
        return None;
    }
    let step = (t * fps) as i64;
    let idx = step.rem_euclid(frames.len() as i64) as usize;
    Some(frames[idx])
}

/// Resolve `(group, clip, t)` against a generated table to a baked GLB stem.
/// An empty `clip` matches the neutral/default row of `group` (props have no
/// gameplay states), falling back to its first row.
pub fn resolve_baked_frame<'a, C: BakedClipRow>(
    table: &'a [C],
    group: &str,
    clip: &str,
    t: f32,
) -> Option<&'a str> {
    let row = if clip.is_empty() {
        // Props have no state: prefer the neutral/default clip when a manifest
        // has several, then fall back to the first row for that group.
        table
            .iter()
            .find(|row| {
                name_eq(row.group(), group)
                    && ["", "loop", "idle", "default"].iter().any(|c| name_eq(row.clip(), c))
            })
            .or_else(|| table.iter().find(|row| name_eq(row.group(), group)))?
    } else {
        table
            .iter()
            .find(|row| name_eq(row.group(), group) && name_eq(row.clip(), clip))?
    };
    baked_frame_at(row.frames(), row.fps(), t)
}

/// Candidate baked family names for a sprite style, most specific first.
/// Tolerates manifests named after gameplay kinds, families or sprite styles.
/// Only styles with a baked NPC table are listed; props use their model name.
pub fn npc_family_names(style: SpriteStyle) -> &'static [&'static str] {
    match style {
        SpriteStyle::Goblin => &["goblin", "armored_goblin"],
        SpriteStyle::Skeleton | SpriteStyle::Wraith => &["skeleton", "wraith"],
        SpriteStyle::Wolf => &["wolf"],
        SpriteStyle::Archer => &["archer"],
        SpriteStyle::Spider => &["spider", "scorpion_queen"],
        SpriteStyle::Slime => &["slime", "toad_king"],
        SpriteStyle::Brute | SpriteStyle::Ogre => &["brute", "ogre"],
        SpriteStyle::Humanoid => &["villager", "civilian", "humanoid", "guard"],
        SpriteStyle::Guard => &["guard", "civilian", "villager"],
        SpriteStyle::Golem => &["golem"],
        SpriteStyle::Bat | SpriteStyle::Imp => &["imp", "bat"],
        SpriteStyle::Stoneslinger => &["stoneslinger", "archer"],
        SpriteStyle::Raider => &["raider", "goblin"],
        SpriteStyle::Colossus => &["colossus", "golem"],
        SpriteStyle::ScorpionQueen => &["scorpion_queen", "spider"],
        SpriteStyle::ToadKing => &["toad_king", "slime"],
        SpriteStyle::Stormcaller => &["stormcaller", "archer"],
        SpriteStyle::OceanLeviathan => &["ocean_leviathan", "slime"],
        _ => &[],
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

    // --- Baked pose frame resolution (NPC + prop manifests) ------------------

    const F_IDLE: &[&str] = &["npcpose_goblin_idle_00", "npcpose_goblin_idle_01"];
    const F_WALK: &[&str] = &["npcpose_goblin_walk_00", "npcpose_goblin_walk_01"];
    const F_RUN: &[&str] = &["npcpose_goblin_run_00", "npcpose_goblin_run_01"];
    const F_ATTACK: &[&str] = &["npcpose_goblin_attack_00", "npcpose_goblin_attack_01"];
    const F_HURT: &[&str] = &["npcpose_goblin_hurt_00", "npcpose_goblin_hurt_01"];
    const F_DEATH: &[&str] = &["npcpose_goblin_death_00", "npcpose_goblin_death_01"];
    const F_SKELETON: &[&str] = &["npcpose_skeleton_idle_00", "npcpose_skeleton_idle_01"];

    fn npc_table() -> [BakedClip<'static>; 7] {
        let row = |clip: &'static str, frames: &'static [&'static str]| BakedClip {
            group: "goblin",
            clip,
            frames,
            fps: 12.0,
        };
        [
            row("idle", F_IDLE),
            row("walk", F_WALK),
            row("run", F_RUN),
            row("attack", F_ATTACK),
            row("hurt", F_HURT),
            row("death", F_DEATH),
            BakedClip { group: "skeleton", clip: "idle", frames: F_SKELETON, fps: 12.0 },
        ]
    }

    #[test]
    fn baked_npc_states_select_distinct_frames() {
        let table = npc_table();
        let states = [
            AnimState::Idle,
            AnimState::Walk,
            AnimState::Run,
            AnimState::Attack,
            AnimState::Hurt,
            AnimState::Death,
        ];
        let picked: Vec<_> = states
            .iter()
            .map(|s| resolve_baked_frame(&table, "goblin", s.as_str(), 0.1).expect("baked frame"))
            .collect();
        let distinct: std::collections::BTreeSet<_> = picked.iter().copied().collect();
        assert_eq!(distinct.len(), states.len(), "states collapsed onto one frame: {:?}", picked);
        assert_eq!(picked[0], F_IDLE[1]);
        assert_eq!(picked[3], F_ATTACK[1]);
        assert_eq!(picked[5], F_DEATH[1]);
        // A different family with the same state must not reuse the frame.
        let skeleton = resolve_baked_frame(&table, "skeleton", "idle", 0.1).unwrap();
        assert_ne!(skeleton, picked[0]);
        // Name normalisation tolerates asset-style capitalisation/separators.
        assert_eq!(
            resolve_baked_frame(&table, "SKELETON", "Idle", 0.1),
            Some(skeleton)
        );
    }

    #[test]
    fn baked_frames_wrap_in_both_directions() {
        let table = npc_table();
        let at = |t| resolve_baked_frame(&table, "goblin", "idle", t).unwrap();
        // 12 fps, two frames: one full 1/12 s step advances one frame, and the
        // clip loops forever (including negative or very large times).
        assert_eq!(at(0.0), F_IDLE[0]);
        assert_eq!(at(1.0 / 12.0), F_IDLE[1]);
        assert_eq!(at(2.0 / 12.0), F_IDLE[0]);
        assert_eq!(at(25.0 / 12.0), F_IDLE[1]);
        assert_eq!(at(-1.0 / 12.0), F_IDLE[1]);
        // Props use the same resolver with an empty clip name.
        let props = [BakedClip {
            group: "campfire",
            clip: "",
            frames: &["animprop_campfire_00", "animprop_campfire_01", "animprop_campfire_02"],
            fps: 15.0,
        }];
        assert_eq!(
            resolve_baked_frame(&props, "camp_fire", "", 0.0),
            Some("animprop_campfire_00")
        );
        assert_eq!(
            resolve_baked_frame(&props, "campfire", "", 3.0 / 15.0),
            Some("animprop_campfire_00")
        );
        assert_eq!(
            resolve_baked_frame(&props, "campfire", "", 2.0 / 15.0),
            Some("animprop_campfire_02")
        );
        // A prop with multiple clips uses its neutral loop by default.
        let multi = [
            BakedClip { group: "torch", clip: "burst", frames: &["burst_00"], fps: 8.0 },
            BakedClip { group: "torch", clip: "loop", frames: &["loop_00"], fps: 8.0 },
        ];
        assert_eq!(resolve_baked_frame(&multi, "torch", "", 0.0), Some("loop_00"));
    }

    #[test]
    fn missing_manifest_falls_back_to_no_frame() {
        // No manifests -> generated tables are empty -> every lookup is None,
        // which is exactly the signal the renderer uses to keep its static GLB.
        let empty: [BakedClip<'static>; 0] = [];
        for state in [AnimState::Idle, AnimState::Walk, AnimState::Attack] {
            assert_eq!(resolve_baked_frame(&empty, "goblin", state.as_str(), 0.5), None);
        }
        let table = npc_table();
        assert_eq!(resolve_baked_frame(&table, "unknown_family", "idle", 0.1), None);
        assert_eq!(resolve_baked_frame(&table, "goblin", "unknown_clip", 0.1), None);
        let broken = [BakedClip { group: "goblin", clip: "idle", frames: &[], fps: 12.0 }];
        assert_eq!(resolve_baked_frame(&broken, "goblin", "idle", 0.1), None);
        let still = [BakedClip { group: "goblin", clip: "idle", frames: F_IDLE, fps: 0.0 }];
        assert_eq!(resolve_baked_frame(&still, "goblin", "idle", 0.1), None);
        assert_eq!(resolve_baked_frame(&table, "goblin", "idle", f32::NAN), None);
    }

    #[test]
    fn sprite_styles_have_baked_family_aliases() {
        assert!(npc_family_names(SpriteStyle::Goblin).contains(&"goblin"));
        assert!(npc_family_names(SpriteStyle::ScorpionQueen).contains(&"scorpion_queen"));
        assert!(npc_family_names(SpriteStyle::Humanoid).contains(&"villager"));
        assert!(npc_family_names(SpriteStyle::Tree).is_empty());
    }
}
