use crate::items::ItemKind;
use crate::render::{Sprite, SpriteStyle};
use crate::weapons::WeaponKind;
use crate::world::TileKind;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};

/// Aggro range: enemies start chasing within this many tiles (chebyshev).
pub const AGGRO_RANGE: f32 = 6.0;
/// Contact damage range for attacks (chebyshev, tile units).
pub const ATTACK_RANGE: f32 = 1.1;
/// Telegraph duration (seconds) before a melee strike lands — the player can
/// dodge during this window. Kept short so combat still feels snappy.
pub const WINDUP: f32 = 0.38;
/// Fraction of max HP at/below which "cowardly" enemies (goblin/imp) break and
/// flee instead of fighting.
pub const FLEE_HP_FRAC: f32 = 0.25;
/// Ranged enemies try to hold this fraction of their shoot range as standoff
/// distance: they back off if the player gets closer than `KITE_MIN_FRAC` of
/// range, and close in if farther than `KITE_MAX_FRAC` of range.
pub const KITE_MIN_FRAC: f32 = 0.55;
pub const KITE_MAX_FRAC: f32 = 0.9;
/// How often (seconds) an enemy replans its A* path.
pub const REPLAN_INTERVAL: f32 = 0.5;
/// Enemy speed in tiles/second (slower than the player, so you can escape).
pub const ENEMY_SPEED: f32 = 2.0;
/// Boss speed — slower than a slime, but it is a wall of hp you must kite.
pub const BOSS_SPEED: f32 = 1.4;
/// Boss aggro — it notices you from much further away.
pub const BOSS_AGGRO_RANGE: f32 = 10.0;
/// Boss melee reach — a wide reach so it cannot be trivially juked.
pub const BOSS_ATTACK_RANGE: f32 = 1.9;
/// Boss attack cooldown (seconds).
pub const BOSS_ATTACK_COOLDOWN: f32 = 1.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyKind {
    Slime,
    /// Forest Warden: the first Crown Fragment guardian (Chapter 3 boss).
    Boss,
    Skeleton,
    Goblin,
    Bat,
    Spider,
    /// Imp: a fast, fragile swamp/forest swarmer.
    Imp,
    /// Ogre: a slow, heavily-armored stone brute.
    Ogre,
    /// Wraith: a flying, phase-shifting spirit that drifts over walls and water.
    Wraith,
    /// Stoneslinger: a stone-biome caster that hurls rocks from range.
    Stoneslinger,
    /// Colossus: an optional bonus elite — a towering stone golem that awakens in
    /// the Stone peaks. Drops treasures but NOT a Crown Fragment.
    Colossus,
    /// Scorpion Queen: the Desert Crown Fragment guardian. A fast, venomous
    /// charger that strikes from range with poison stings.
    ScorpionQueen,
    /// Frost Golem: the Tundra Crown Fragment guardian. A slow, towering wall of
    /// ice that hits like a truck and never stops chasing.
    FrostGolem,
    /// Toad King: the Swamp Crown Fragment guardian. A bloated amphibian that
    /// spits toxic globs from range and lashes out with its tongue.
    ToadKing,
    /// Ocean Leviathan: the Deep Ocean Crown Fragment guardian. A swift tidal
    /// terror that hurls water from afar and surges across the waves.
    OceanLeviathan,
    /// Brute: a hulking melee tank that winds up and charges in a straight
    /// line, dealing heavy contact damage if it connects.
    Brute,
    /// Stormcaller: a flying storm-mage that drifts over walls and hurls
    /// lightning from range. Combines the flying and ranged behaviours.
    Stormcaller,
    /// Wolf: a fast, fragile four-legged melee swarmer that hunts in packs.
    Wolf,
    /// Archer: a ranged humanoid that keeps its distance and looses arrows.
    Archer,
    /// Raider: a night-stalking humanoid bandit that harries your base after
    /// dark. Tough enough to threaten an undefended homestead, so village guards
    /// and War Banners are what keep them at bay.
    Raider,
}

impl EnemyKind {
    /// True for enemies that ignore terrain/structure collision (fly straight
    /// at the player). Used to skip A* pathing.
    pub fn flying(self) -> bool {
        matches!(self, EnemyKind::Wraith | EnemyKind::Stormcaller)
    }

    /// True for enemies that fire projectiles at the player from range.
    pub fn ranged(self) -> bool {
        matches!(
            self,
            EnemyKind::Stoneslinger
                | EnemyKind::Stormcaller
                | EnemyKind::Archer
                | EnemyKind::Boss
                | EnemyKind::ToadKing
                | EnemyKind::OceanLeviathan
        )
    }

    /// True for undead / creatures-of-the-night that shun the sun: they only
    /// prowl after dark and take burning damage while exposed to daylight.
    pub fn nocturnal(self) -> bool {
        matches!(
            self,
            EnemyKind::Skeleton |             EnemyKind::Wraith | EnemyKind::Bat | EnemyKind::Spider | EnemyKind::Imp | EnemyKind::Raider
        )
    }

    /// Range (tiles) at which a ranged enemy will fire.
    pub fn shoot_range(self) -> f32 {
        match self {
            EnemyKind::Stoneslinger => 9.0,
            EnemyKind::Stormcaller => 10.0,
            EnemyKind::Archer => 8.0,
            EnemyKind::Boss => 8.0,
            EnemyKind::ToadKing => 9.0,
            EnemyKind::OceanLeviathan => 10.0,
            _ => 0.0,
        }
    }

    /// Display name for the Bestiary / Codex.
    pub fn name(self) -> &'static str {
        match self {
            EnemyKind::Slime => "Slime",
            EnemyKind::Boss => "Forest Warden",
            EnemyKind::Skeleton => "Skeleton",
            EnemyKind::Goblin => "Goblin",
            EnemyKind::Bat => "Bat",
            EnemyKind::Spider => "Spider",
            EnemyKind::Imp => "Imp",
            EnemyKind::Ogre => "Ogre",
            EnemyKind::Wraith => "Wraith",
            EnemyKind::Stoneslinger => "Stoneslinger",
            EnemyKind::Colossus => "Colossus",
            EnemyKind::Brute => "Brute",
            EnemyKind::Stormcaller => "Stormcaller",
            EnemyKind::Wolf => "Wolf",
            EnemyKind::Archer => "Archer",
            EnemyKind::Raider => "Raider",
            EnemyKind::ScorpionQueen => "Scorpion Queen",
            EnemyKind::FrostGolem => "Frost Golem",
            EnemyKind::ToadKing => "Toad King",
            EnemyKind::OceanLeviathan => "Ocean Leviathan",
        }
    }

    /// One-line behaviour description for the Bestiary / Codex.
    pub fn behavior(self) -> &'static str {
        match self {
            EnemyKind::Slime => "Slow melee; the first thing between you and the ruins.",
            EnemyKind::Boss => "Boss: Forest Warden. Wide reach, high hp, hurls vines. First Crown Fragment.",
            EnemyKind::Skeleton => "Steady melee undead of forest and ruins.",
            EnemyKind::Goblin => "Quick melee ambusher.",
            EnemyKind::Bat => "Fast flyer; weak but hard to pin down.",
            EnemyKind::Spider => "Swift melee lurker.",
            EnemyKind::Imp => "Tiny, very fast melee swarmer.",
            EnemyKind::Ogre => "Slow, heavily-armored bruiser with big hits.",
            EnemyKind::Wraith => "Flying spirit that drifts straight through walls.",
            EnemyKind::Stoneslinger => "Ranged caster; hurls rocks from afar.",
            EnemyKind::Colossus => "Bonus elite: towering stone golem of the peaks. Drops treasure, not a fragment.",
            EnemyKind::Brute => "Tank that winds up and charges in a straight line.",
            EnemyKind::Stormcaller => "Flying storm-mage; drifts over walls and hurls lightning from afar.",
            EnemyKind::Wolf => "Fast pack hunter; tears in and bites before you can react.",
            EnemyKind::Archer => "Ranged marksman; kites and looses arrows from afar.",
            EnemyKind::Raider => "Night raider: harries your base after dark; village guards and War Banners drive them off.",
            EnemyKind::ScorpionQueen => "Boss: Desert. Fast venomous charger that stings from range. Crown Fragment.",
            EnemyKind::FrostGolem => "Boss: Tundra. Slow, towering ice wall with crushing blows. Crown Fragment.",
            EnemyKind::ToadKing => "Boss: Swamp. Bloated amphibian that spits toxic globs from range. Crown Fragment.",
            EnemyKind::OceanLeviathan => "Boss: Ocean. Swift tidal terror hurling water from afar. Crown Fragment.",
        }
    }

    /// True for the Crown Fragment guardian bosses (never respawn). Includes the
    /// optional Colossus elite so it also uses boss-grade stats and behaviour.
    pub fn is_boss(self) -> bool {
        matches!(
            self,
            EnemyKind::Boss
                | EnemyKind::Colossus
                | EnemyKind::ScorpionQueen
                | EnemyKind::FrostGolem
                | EnemyKind::ToadKing
                | EnemyKind::OceanLeviathan
        )
    }

    /// Bit index (0..5) of the Crown Fragment this boss guards, or None for
    /// non-fragment elites (e.g. the bonus Colossus). Used to track which of the
    /// five fragments the player has recovered.
    pub fn fragment_bit(self) -> Option<u8> {
        match self {
            EnemyKind::Boss => Some(0),
            EnemyKind::ScorpionQueen => Some(1),
            EnemyKind::FrostGolem => Some(2),
            EnemyKind::ToadKing => Some(3),
            EnemyKind::OceanLeviathan => Some(4),
            _ => None,
        }
    }

    /// Damage multiplier for a given weapon — certain weapons are strong against
    /// certain foes (the Bestiary calls these out). 1.0 means no bonus.
    pub fn weakness_to(self, w: WeaponKind) -> f32 {
        match (self, w) {
            (EnemyKind::Wraith | EnemyKind::Stormcaller, WeaponKind::Bow) => 1.5,
            (
                EnemyKind::Skeleton
                | EnemyKind::Stoneslinger
                | EnemyKind::Ogre
                | EnemyKind::Colossus
                | EnemyKind::Raider,
                WeaponKind::Hammer,
            ) => 1.5,
            (EnemyKind::Goblin, WeaponKind::Sword) => 1.5,
            (EnemyKind::Wolf, WeaponKind::Spear) => 1.5,
            (EnemyKind::Slime | EnemyKind::Spider | EnemyKind::Imp, WeaponKind::Axe) => 1.5,
            (EnemyKind::ScorpionQueen, WeaponKind::Bow) => 1.5,
            (EnemyKind::FrostGolem | EnemyKind::Colossus, WeaponKind::Hammer) => 1.5,
            (EnemyKind::ToadKing, WeaponKind::Axe) => 1.5,
            (EnemyKind::OceanLeviathan, WeaponKind::Spear) => 1.5,
            _ => 1.0,
        }
    }

    /// What this enemy drops on death (used by the Bestiary / Codex too).
    /// Combat now yields a spread of crafting materials (Iron scrap, Gems, Wood,
    /// Herbs) so the forge economy isn't gated entirely on node farming.
    pub fn drops(self) -> Vec<ItemKind> {
        match self {
            EnemyKind::Slime => vec![ItemKind::Food, ItemKind::Herb, ItemKind::Gold],
            EnemyKind::Boss => vec![ItemKind::Fragment, ItemKind::Gold],
            EnemyKind::Skeleton => vec![ItemKind::Food, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Goblin => vec![ItemKind::Food, ItemKind::Wood, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Bat => vec![ItemKind::Food, ItemKind::Herb, ItemKind::Gold],
            EnemyKind::Spider => vec![ItemKind::Herb, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Imp => vec![ItemKind::Food, ItemKind::Herb, ItemKind::Gold],
            EnemyKind::Ogre => vec![ItemKind::Gem, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Wraith => vec![ItemKind::Herb, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Stoneslinger => vec![ItemKind::Gem, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Colossus => vec![ItemKind::Gem, ItemKind::Herb, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::ScorpionQueen => vec![ItemKind::Fragment, ItemKind::Gold],
            EnemyKind::FrostGolem => vec![ItemKind::Fragment, ItemKind::Gem, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::ToadKing => vec![ItemKind::Fragment, ItemKind::Herb, ItemKind::Gold],
            EnemyKind::OceanLeviathan => vec![ItemKind::Fragment, ItemKind::Gem, ItemKind::Gold],
            EnemyKind::Brute => vec![ItemKind::Gem, ItemKind::Food, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Stormcaller => vec![ItemKind::Gem, ItemKind::Herb, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Wolf => vec![ItemKind::Food, ItemKind::Herb, ItemKind::Wood, ItemKind::Gold],
            EnemyKind::Archer => vec![ItemKind::Food, ItemKind::Wood, ItemKind::Iron, ItemKind::Gold],
            EnemyKind::Raider => vec![ItemKind::Food, ItemKind::Gem, ItemKind::Iron, ItemKind::Wood, ItemKind::Gold],
        }
    }

    /// Experience granted to the player when this enemy is slain.
    pub fn xp(self) -> u32 {
        match self {
            EnemyKind::Slime => 5,
            EnemyKind::Boss => 500,
            EnemyKind::Skeleton => 8,
            EnemyKind::Goblin => 9,
            EnemyKind::Bat => 4,
            EnemyKind::Spider => 7,
            EnemyKind::Imp => 4,
            EnemyKind::Ogre => 30,
            EnemyKind::Wraith => 6,
            EnemyKind::Stoneslinger => 8,
            EnemyKind::Colossus => 1000,
            EnemyKind::ScorpionQueen => 520,
            EnemyKind::FrostGolem => 560,
            EnemyKind::ToadKing => 540,
            EnemyKind::OceanLeviathan => 600,
            EnemyKind::Brute => 40,
            EnemyKind::Stormcaller => 12,
            EnemyKind::Wolf => 7,
            EnemyKind::Archer => 8,
            EnemyKind::Raider => 18,
        }
    }
}

impl EnemyKind {
    pub fn max_hp(self) -> f32 {
        match self {
            EnemyKind::Slime => 12.0,
            EnemyKind::Boss => 70.0,
            EnemyKind::Skeleton => 16.0,
            EnemyKind::Goblin => 18.0,
            EnemyKind::Bat => 8.0,
            EnemyKind::Spider => 14.0,
            EnemyKind::Imp => 6.0,
            EnemyKind::Ogre => 40.0,
            EnemyKind::Wraith => 10.0,
            EnemyKind::Stoneslinger => 16.0,
            EnemyKind::Colossus => 140.0,
            EnemyKind::ScorpionQueen => 80.0,
            EnemyKind::FrostGolem => 110.0,
            EnemyKind::ToadKing => 90.0,
            EnemyKind::OceanLeviathan => 100.0,
            EnemyKind::Brute => 50.0,
            EnemyKind::Stormcaller => 24.0,
            EnemyKind::Wolf => 14.0,
            EnemyKind::Archer => 18.0,
            EnemyKind::Raider => 35.0,
        }
    }

    /// Contact damage dealt per hit.
    pub fn damage(self) -> f32 {
        match self {
            EnemyKind::Slime => 3.0,
            EnemyKind::Boss => 9.0,
            EnemyKind::Skeleton => 4.0,
            EnemyKind::Goblin => 5.0,
            EnemyKind::Bat => 2.0,
            EnemyKind::Spider => 3.5,
            EnemyKind::Imp => 2.0,
            EnemyKind::Ogre => 8.0,
            EnemyKind::Wraith => 3.5,
            EnemyKind::Stoneslinger => 4.0,
            EnemyKind::Colossus => 10.0,
            EnemyKind::ScorpionQueen => 9.0,
            EnemyKind::FrostGolem => 10.0,
            EnemyKind::ToadKing => 8.0,
            EnemyKind::OceanLeviathan => 9.0,
            EnemyKind::Brute => 9.0,
            EnemyKind::Stormcaller => 6.0,
            EnemyKind::Wolf => 3.5,
            EnemyKind::Archer => 4.0,
            EnemyKind::Raider => 6.0,
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            EnemyKind::Slime => [0.30, 0.78, 0.36],
            EnemyKind::Boss => [0.55, 0.18, 0.62],
            EnemyKind::Skeleton => [0.92, 0.90, 0.85],
            EnemyKind::Goblin => [0.45, 0.60, 0.30],
            EnemyKind::Bat => [0.35, 0.30, 0.45],
            EnemyKind::Spider => [0.40, 0.20, 0.20],
            EnemyKind::Imp => [0.88, 0.35, 0.55],
            EnemyKind::Ogre => [0.55, 0.45, 0.35],
            EnemyKind::Wraith => [0.62, 0.45, 0.86],
            EnemyKind::Stoneslinger => [0.45, 0.40, 0.52],
            EnemyKind::Colossus => [0.55, 0.52, 0.50],
            EnemyKind::ScorpionQueen => [0.85, 0.55, 0.20],
            EnemyKind::FrostGolem => [0.62, 0.82, 0.95],
            EnemyKind::ToadKing => [0.45, 0.70, 0.30],
            EnemyKind::OceanLeviathan => [0.20, 0.55, 0.80],
            EnemyKind::Brute => [0.60, 0.30, 0.25],
            EnemyKind::Stormcaller => [0.42, 0.55, 0.86],
            EnemyKind::Wolf => [0.55, 0.50, 0.48],
            EnemyKind::Archer => [0.50, 0.45, 0.62],
            EnemyKind::Raider => [0.45, 0.18, 0.18],
        }
    }

    /// Sprite geometry: slightly larger than the player so it reads clearly.
    /// Alpha fades as hp drops (a visible health telegraph). `facing` leans the
    /// head of humanoid figures toward where they're looking.
    pub fn sprite(self, x: f32, y: f32, hp_frac: f32, facing: (f32, f32)) -> Sprite {
        let (hw, hh) = match self {
            EnemyKind::Slime => (19.0, 19.0),
            EnemyKind::Boss => (30.0, 27.0),
            EnemyKind::Skeleton => (19.0, 24.0),
            EnemyKind::Goblin => (19.0, 24.0),
            EnemyKind::Bat => (16.0, 11.0),
            EnemyKind::Spider => (16.0, 14.0),
            EnemyKind::Imp => (14.0, 16.0),
            EnemyKind::Ogre => (27.0, 30.0),
            EnemyKind::Wraith => (16.0, 23.0),
            EnemyKind::Stoneslinger => (16.0, 24.0),
            EnemyKind::Colossus => (35.0, 43.0),
            EnemyKind::ScorpionQueen => (27.0, 27.0),
            EnemyKind::FrostGolem => (38.0, 46.0),
            EnemyKind::ToadKing => (32.0, 27.0),
            EnemyKind::OceanLeviathan => (30.0, 24.0),
            EnemyKind::Brute => (27.0, 30.0),
            EnemyKind::Stormcaller => (19.0, 24.0),
            EnemyKind::Wolf => (22.0, 16.0),
            EnemyKind::Archer => (19.0, 24.0),
            EnemyKind::Raider => (19.0, 24.0),
        };
        let style = match self {
            EnemyKind::Slime => SpriteStyle::Slime,
            // The player and every humanoid foe share one consistent character
            // rig (legs/torso/arms/head with a walk cycle), tinted per kind, so
            // the cast reads as the same world. Creatures keep bespoke silhouettes.
            EnemyKind::Boss => SpriteStyle::Humanoid,
            EnemyKind::Skeleton => SpriteStyle::Humanoid,
            EnemyKind::Goblin => SpriteStyle::Humanoid,
            EnemyKind::Bat => SpriteStyle::Bat,
            EnemyKind::Spider => SpriteStyle::Spider,
            EnemyKind::Imp => SpriteStyle::Imp,
            EnemyKind::Ogre => SpriteStyle::Humanoid,
            EnemyKind::Wraith => SpriteStyle::Wraith,
            EnemyKind::Stoneslinger => SpriteStyle::Humanoid,
            EnemyKind::Colossus => SpriteStyle::Colossus,
            EnemyKind::ScorpionQueen => SpriteStyle::ScorpionQueen,
            EnemyKind::FrostGolem => SpriteStyle::Colossus,
            EnemyKind::ToadKing => SpriteStyle::ToadKing,
            EnemyKind::OceanLeviathan => SpriteStyle::OceanLeviathan,
            EnemyKind::Brute => SpriteStyle::Brute,
            EnemyKind::Stormcaller => SpriteStyle::Stormcaller,
            EnemyKind::Wolf => SpriteStyle::Wolf,
            EnemyKind::Archer => SpriteStyle::Archer,
            EnemyKind::Raider => SpriteStyle::Raider,
        };
        let mut s = Sprite::new_center(x, y, self.color(), hw, hh, 2.0)
            .with_style(style)
            .with_facing(facing);
        s.alpha = 0.8 + 0.2 * hp_frac;
        s
    }
}

/// Internal AI state of one enemy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiState {
    Idle,
    Chase,
    Attack,
    Flee,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub kind: EnemyKind,
    pub hp: f32,
    pub state: AiState,
    pub facing: (f32, f32),
    pub attack_timer: f32,
    /// Seconds remaining of a wind-up before a melee strike lands. While > 0 the
    /// enemy is telegraphing (caller can render a tell); the hit only fires once
    /// this reaches 0, giving the player a dodge window. 0 = not winding up.
    pub windup: f32,
    /// Seconds remaining of the white "hit" flash (set to 1 on damage, decays).
    pub flash: f32,
    path_timer: f32,
    path: Vec<(i32, i32)>,
    wander: f32,
    /// Seconds until this enemy may fire again (ranged kinds only).
    shoot_timer: f32,
    /// Brute only: seconds left in the current charge dash (0 = not charging).
    charge_t: f32,
    /// Brute only: cooldown until the next charge can start.
    charge_cd: f32,
    /// Set during `update` when the enemy fires: a unit direction toward the
    /// player. The caller (renderer) turns this into an enemy arrow.
    pub pending_shot: Option<(f32, f32)>,
    /// Boss only: latches true once it drops below 40% HP, entering an enraged
    /// second phase (faster, shorter wind-ups). Purely an AI flag.
    pub enraged: bool,
    /// Elite multiplier: scales max HP, contact damage and XP reward. 1.0 for
    /// ordinary spawner enemies; >1 for roaming mini-bosses.
    pub elite: f32,
    /// Speed scaling relative to the player's progression: set each tick from the
    /// player's level so enemies keep pace as you grow stronger (see `set_speed_mult`).
    pub speed_mult: f32,
    /// True for undead / night creatures. They avoid daylight and burn while
    /// exposed to it (see `daylight_burn`). Derived from the kind at spawn.
    pub nocturnal: bool,
    /// Seconds remaining of the sunlight "burning" flash, for rendering tint.
    pub burn: f32,
}

impl Enemy {
    pub fn new(x: f32, y: f32, kind: EnemyKind) -> Self {
        Self {
            x,
            y,
            kind,
            hp: kind.max_hp(),
            state: AiState::Idle,
            facing: (1.0, 0.0),
            attack_timer: 0.0,
            windup: 0.0,
            flash: 0.0,
            path_timer: 0.0,
            path: Vec::new(),
            wander: 0.0,
            shoot_timer: 0.0,
            charge_t: 0.0,
            charge_cd: 0.0,
            pending_shot: None,
            enraged: false,
            elite: 1.0,
            speed_mult: 1.0,
            nocturnal: kind.nocturnal(),
            burn: 0.0,
        }
    }

    /// Returns a copy of this enemy promoted to an elite: boosted max HP (and
    /// current HP) by `elite`, used for roaming mini-bosses.
    pub fn with_elite(mut self, elite: f32) -> Self {
        self.elite = elite;
        self.hp = self.kind.max_hp() * elite;
        self
    }

    pub fn alive(&self) -> bool {
        self.hp > 0.0
    }

    pub fn take_damage(&mut self, dmg: f32) -> bool {
        if !self.alive() {
            return false;
        }
        self.hp -= dmg;
        self.flash = 1.0;
        if self.hp <= 0.0 {
            self.hp = 0.0;
        }
        true
    }

    /// Sunlight damage for nocturnal enemies. `daylight` is the day/night
    /// factor (0 = night, 1 = full noon). Above a threshold the enemy burns,
    /// losing HP each second and flashing a scorch tint. No-op for day creatures
    /// or at night.
    pub fn daylight_burn(&mut self, dt: f32, daylight: f32) {
        if !self.nocturnal || !self.alive() {
            return;
        }
        const BURN_THRESHOLD: f32 = 0.55;
        if daylight <= BURN_THRESHOLD {
            return;
        }
        let intensity = (daylight - BURN_THRESHOLD) / (1.0 - BURN_THRESHOLD);
        let dmg = 9.0 * intensity * dt;
        self.hp -= dmg;
        if self.hp <= 0.0 {
            self.hp = 0.0;
        }
        self.burn = 1.0;
    }

    /// Speed multiplier for enemies relative to the player's progression. As the
    /// player levels up, enemies speed up (capped) so they keep pace instead of
    /// becoming trivial to outrun. 1.0 at level 0, ramping to ~2.2 by high level.
    pub fn speed_scale_for_level(level: u32) -> f32 {
        (1.0 + 0.05 * level as f32).min(2.2)
    }

    /// Drops from a dead enemy.
    pub fn drops(&self) -> Vec<ItemKind> {
        self.kind.drops()
    }

    /// One AI tick. `player` is the target; `is_blocked` mirrors player
    /// collision (tiles + structures + other enemies). Returns Some if the
    /// enemy is close enough to land a hit this tick.
    pub fn update(
        &mut self,
        player: (f32, f32),
        dt: f32,
        mut is_blocked: impl FnMut(i32, i32) -> bool,
    ) -> Option<f32> {
        self.attack_timer = (self.attack_timer - dt).max(0.0);
        self.flash = (self.flash - dt * 5.0).max(0.0);
        self.burn = (self.burn - dt * 2.5).max(0.0);
        self.shoot_timer = (self.shoot_timer - dt).max(0.0);
        self.charge_t = (self.charge_t - dt).max(0.0);
        self.charge_cd = (self.charge_cd - dt).max(0.0);
        self.pending_shot = None;
        let (aggro, atk_range, speed, cooldown) = match self.kind {
            EnemyKind::Slime => (AGGRO_RANGE, ATTACK_RANGE, ENEMY_SPEED, 0.8),
            EnemyKind::Boss => (BOSS_AGGRO_RANGE, BOSS_ATTACK_RANGE, BOSS_SPEED, BOSS_ATTACK_COOLDOWN),
            EnemyKind::Skeleton => (AGGRO_RANGE, ATTACK_RANGE, ENEMY_SPEED, 0.8),
            EnemyKind::Goblin => (AGGRO_RANGE, ATTACK_RANGE, ENEMY_SPEED * 1.1, 0.8),
            EnemyKind::Bat => (AGGRO_RANGE, ATTACK_RANGE, ENEMY_SPEED * 1.6, 0.6),
            EnemyKind::Spider => (AGGRO_RANGE, ATTACK_RANGE, ENEMY_SPEED, 0.9),
            EnemyKind::Imp => (AGGRO_RANGE, ATTACK_RANGE, ENEMY_SPEED * 1.8, 0.5),
            EnemyKind::Ogre => (AGGRO_RANGE, ATTACK_RANGE, ENEMY_SPEED * 0.7, 1.2),
            EnemyKind::Wraith => (AGGRO_RANGE + 2.0, ATTACK_RANGE, ENEMY_SPEED * 1.3, 0.7),
            EnemyKind::Stoneslinger => (AGGRO_RANGE, ATTACK_RANGE, ENEMY_SPEED * 0.9, 1.0),
            EnemyKind::Colossus => (BOSS_AGGRO_RANGE, BOSS_ATTACK_RANGE, BOSS_SPEED, BOSS_ATTACK_COOLDOWN),
            EnemyKind::ScorpionQueen => (BOSS_AGGRO_RANGE, BOSS_ATTACK_RANGE, BOSS_SPEED * 1.25, BOSS_ATTACK_COOLDOWN),
            EnemyKind::FrostGolem => (BOSS_AGGRO_RANGE, BOSS_ATTACK_RANGE * 1.2, BOSS_SPEED * 0.85, BOSS_ATTACK_COOLDOWN * 1.2),
            EnemyKind::ToadKing => (BOSS_AGGRO_RANGE, BOSS_ATTACK_RANGE, BOSS_SPEED, BOSS_ATTACK_COOLDOWN),
            EnemyKind::OceanLeviathan => (BOSS_AGGRO_RANGE, BOSS_ATTACK_RANGE, BOSS_SPEED * 1.15, BOSS_ATTACK_COOLDOWN * 0.9),
            EnemyKind::Brute => (AGGRO_RANGE + 1.0, ATTACK_RANGE, ENEMY_SPEED * 0.8, 1.0),
            EnemyKind::Stormcaller => (AGGRO_RANGE + 3.0, ATTACK_RANGE, ENEMY_SPEED * 1.2, 0.9),
            EnemyKind::Wolf => (AGGRO_RANGE + 2.0, ATTACK_RANGE, ENEMY_SPEED * 1.9, 0.5),
            EnemyKind::Archer => (AGGRO_RANGE + 1.0, ATTACK_RANGE, ENEMY_SPEED * 0.95, 1.1),
            // Raiders are quick, aggressive night skirmishers: a touch faster than
            // a Goblin, with a shorter cooldown so they keep pressure on a base.
            EnemyKind::Raider => (AGGRO_RANGE + 2.0, ATTACK_RANGE, ENEMY_SPEED * 1.15, 0.7),
        };
        // Boss second phase: below 45% HP every Crown Fragment guardian enrages —
        // faster, shorter wind-ups — so a drawn-out fight turns frantic at the end.
        let (speed, cooldown) = if self.kind.is_boss()
            && self.hp < self.kind.max_hp() * 0.45
        {
            self.enraged = true;
            (BOSS_SPEED * 1.6, BOSS_ATTACK_COOLDOWN * 0.6)
        } else {
            (speed, cooldown)
        };
        // Scale movement to the player's progression so enemies stay a credible
        // threat as the player levels up (set each tick via `set_speed_mult`).
        let speed = speed * self.speed_mult;
        let windup_time = if self.enraged { WINDUP * 0.6 } else { WINDUP };
        let d = (player.0 - self.x)
            .abs()
            .max((player.1 - self.y).abs());

        if d <= atk_range {
            self.state = AiState::Attack;
            self.facing = normalize(player.0 - self.x, player.1 - self.y);
            if self.attack_timer > 0.0 {
                // on cooldown after a landed strike
                return None;
            }
            if self.windup > 0.0 {
                // mid-telegraph: count down, then strike when it elapses
                self.windup -= dt;
                if self.windup <= 0.0 {
                    self.attack_timer = cooldown;
                    return Some(self.kind.damage() * self.elite);
                }
                return None;
            }
            // ready to attack: begin a visible wind-up (dodge window)
            self.windup = windup_time;
            return None;
        }

        if d <= aggro {
            // Cowardly enemies (goblin/imp) break and flee once badly hurt,
            // darting away from the player instead of pressing the attack.
            if self.hp < self.kind.max_hp() * FLEE_HP_FRAC
                && matches!(self.kind, EnemyKind::Goblin | EnemyKind::Imp)
            {
                self.state = AiState::Flee;
                let (dx, dy) = normalize(self.x - player.0, self.y - player.1);
                self.facing = (dx, dy);
                self.x += dx * speed * 1.15 * dt;
                self.y += dy * speed * 1.15 * dt;
                return None;
            }

            if self.state != AiState::Chase {
                self.flash = 0.85;
            }
            self.state = AiState::Chase;
            // Brute: wind up, then dash in a straight line toward the player.
            if self.kind == EnemyKind::Brute {
                if self.charge_t > 0.0 {
                    // mid-charge: locked direction, fast dash (ignores pathing)
                    self.x += self.facing.0 * speed * 3.2 * dt;
                    self.y += self.facing.1 * speed * 3.2 * dt;
                    return None;
                } else if d > atk_range * 1.6 && self.charge_cd <= 0.0 {
                    self.facing = normalize(player.0 - self.x, player.1 - self.y);
                    self.charge_t = 0.45;
                    self.charge_cd = 2.5;
                    return None;
                }
            }
            // Ranged enemies keep their distance: back off if the player gets
            // too close, close in if too far, and only fire from standoff.
            if self.kind.ranged() {
                let sr = self.kind.shoot_range();
                let to_p = normalize(player.0 - self.x, player.1 - self.y);
                self.facing = (to_p.0, to_p.1);
                if d <= sr * KITE_MIN_FRAC {
                    // too close — kite away while still able to shoot
                    self.x -= to_p.0 * speed * dt;
                    self.y -= to_p.1 * speed * dt;
                } else if d >= sr * KITE_MAX_FRAC {
                    // too far — close the gap
                    self.x += to_p.0 * speed * dt;
                    self.y += to_p.1 * speed * dt;
                }
                if d <= sr && self.shoot_timer <= 0.0 {
                    self.pending_shot = Some(to_p);
                    self.shoot_timer = 1.8;
                }
                return None;
            }
            // Flying enemies ignore terrain/structures and weave erratically
            // toward the player (can't be juked behind walls).
            if self.kind.flying() {
                let (dx, dy) = normalize(player.0 - self.x, player.1 - self.y);
                // perpendicular sine weave so the flight path isn't a straight line
                let wob = (self.x * 1.7 + self.y * 1.3 + self.path_timer * 4.0).sin() * 0.45;
                let (px, py) = (-dy * wob, dx * wob);
                self.facing = (dx, dy);
                self.x += (dx + px) * speed * dt;
                self.y += (dy + py) * speed * dt;
                return None;
            }
            self.path_timer -= dt;
            if self.path_timer <= 0.0 || self.path.is_empty() {
                self.path = astar(
                    (self.x.floor() as i32, self.y.floor() as i32),
                    (player.0.floor() as i32, player.1.floor() as i32),
                    &mut is_blocked,
                );
                self.path_timer = REPLAN_INTERVAL;
            }
            if let Some(&(tx, ty)) = self.path.first() {
                let to = (tx as f32 + 0.5, ty as f32 + 0.5);
                let (dx, dy) = normalize(to.0 - self.x, to.1 - self.y);
                self.facing = (dx, dy);
                self.x += dx * speed * dt;
                self.y += dy * speed * dt;
                if (self.x.floor() as i32, self.y.floor() as i32) == (tx, ty) {
                    self.path.remove(0);
                }
            }
            return None;
        }

        // idle: gentle drift back toward the spawn tile
        self.state = AiState::Idle;
        self.wander = (self.wander - dt).max(0.0);
        if self.wander <= 0.0 {
            self.wander = 1.0 + (self.x * 7.13 + self.y * 3.71).fract().abs() * 2.0;
        }
        return None;
    }
}

fn normalize(dx: f32, dy: f32) -> (f32, f32) {
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        (0.0, 0.0)
    } else {
        (dx / len, dy / len)
    }
}

/// A* over the tile grid. Returns a path from `start` toward `goal`
/// (excluding the goal tile itself, which may be the player's tile).
/// Falls back to the empty path if unreachable.
pub fn astar(
    start: (i32, i32),
    goal: (i32, i32),
    is_blocked: &mut dyn FnMut(i32, i32) -> bool,
) -> Vec<(i32, i32)> {
    if start == goal {
        return Vec::new();
    }
    let h = |(x, y): (i32, i32)| (x - goal.0).abs() + (y - goal.1).abs();
    let mut open: BinaryHeap<(i32, i32, i32)> = BinaryHeap::new();
    let mut came: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g: HashMap<(i32, i32), i32> = HashMap::new();
    open.push((0, start.0, start.1));
    g.insert(start, 0);
    let mut best_goal: Option<(i32, i32)> = None;
    let mut best_d = i32::MAX;
    let mut budget = 4000;

    while budget > 0 {
        budget -= 1;
        let Some((_neg_f, x, y)) = open.pop() else {
            break;
        };
        let node = (x, y);
        let cost = g[&node];
        let dist = (x - goal.0).abs() + (y - goal.1).abs();
        if dist < best_d {
            best_d = dist;
            best_goal = Some(node);
        }
        if node == goal {
            break;
        }
        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let next = (x + dx, y + dy);
            if is_blocked(next.0, next.1) {
                continue;
            }
            let ng = cost + 1;
            if ng < *g.get(&next).unwrap_or(&i32::MAX) {
                g.insert(next, ng);
                came.insert(next, node);
                open.push((-(ng + h(next)), next.0, next.1));
            }
        }
    }

    let Some(mut node) = best_goal else {
        return Vec::new();
    };
    let mut path = Vec::new();
    while let Some(&prev) = came.get(&node) {
        path.push(node);
        node = prev;
    }
    path.reverse();
    path
}

/// Stateless enemy placement. Swamp tiles carry slimes (1/23); a few more
/// critters are scattered on other biomes so the world feels alive. Enemies
/// are session entities with persistent hp (EnemyRegistry).
  pub fn spawner_on(tx: i32, ty: i32, tile: TileKind) -> Option<EnemyKind> {
      let h = tx.wrapping_mul(73856093) ^ ty.wrapping_mul(19349663) ^ 0x51ab_ce0d;
      // Densities are intentionally sparse: enemies should be a threat you meet
      // occasionally, not a wall. Moduli are large so spawns are rare (~one per
      // few hundred tiles per biome).
      match tile {
          TileKind::Swamp if h.rem_euclid(113) == 0 => Some(EnemyKind::Slime),
          TileKind::Swamp if h.rem_euclid(131) == 0 => Some(EnemyKind::Bat),
          TileKind::Stone if h.rem_euclid(181) == 0 => Some(EnemyKind::Skeleton),
          TileKind::Grass if h.rem_euclid(173) == 0 => Some(EnemyKind::Goblin),
          TileKind::Forest if h.rem_euclid(199) == 0 => Some(EnemyKind::Spider),
          TileKind::Swamp if h.rem_euclid(191) == 0 => Some(EnemyKind::Imp),
           TileKind::Forest if h.rem_euclid(211) == 0 => Some(EnemyKind::Imp),
           TileKind::Forest if h.rem_euclid(233) == 0 => Some(EnemyKind::Wolf),
           TileKind::Forest if h.rem_euclid(277) == 0 => Some(EnemyKind::Archer),
           TileKind::Grass if h.rem_euclid(241) == 0 => Some(EnemyKind::Archer),
           TileKind::Tundra if h.rem_euclid(211) == 0 => Some(EnemyKind::Wolf),
           TileKind::Stone if h.rem_euclid(277) == 0 => Some(EnemyKind::Ogre),
          TileKind::Stone if h.rem_euclid(311) == 0 => Some(EnemyKind::Wraith),
          TileKind::Stone if h.rem_euclid(349) == 0 => Some(EnemyKind::Stoneslinger),
          TileKind::Stone if h.rem_euclid(733) == 0 => Some(EnemyKind::Colossus),
          TileKind::Stone if h.rem_euclid(563) == 0 => Some(EnemyKind::Brute),
          TileKind::Stone if h.rem_euclid(389) == 0 => Some(EnemyKind::Stormcaller),
          TileKind::Tundra if h.rem_euclid(191) == 0 => Some(EnemyKind::Wraith),
          TileKind::Tundra if h.rem_euclid(173) == 0 => Some(EnemyKind::Goblin),
           TileKind::Desert if h.rem_euclid(199) == 0 => Some(EnemyKind::Skeleton),
           TileKind::Desert if h.rem_euclid(277) == 0 => Some(EnemyKind::Spider),
           TileKind::Jungle if h.rem_euclid(151) == 0 => Some(EnemyKind::Spider),
           TileKind::Jungle if h.rem_euclid(181) == 0 => Some(EnemyKind::Bat),
            TileKind::Jungle if h.rem_euclid(257) == 0 => Some(EnemyKind::Imp),
           TileKind::Swamp if h.rem_euclid(257) == 0 => Some(EnemyKind::Wraith),
           TileKind::Volcanic if h.rem_euclid(211) == 0 => Some(EnemyKind::Stoneslinger),
           TileKind::Volcanic if h.rem_euclid(283) == 0 => Some(EnemyKind::Ogre),
           TileKind::Volcanic if h.rem_euclid(337) == 0 => Some(EnemyKind::Imp),
          _ => None,
     }
 }

/// Session enemy registry: keeps per-spawner enemy state (alive hp or
/// respawn timer) keyed by spawn tile.
#[derive(Debug, Default)]
pub struct EnemyRegistry {
    enemies: HashMap<(i32, i32), Enemy>,
    respawn: HashMap<(i32, i32), f32>,
}

impl EnemyRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_respawn_pending(&self, tx: i32, ty: i32) -> bool {
        self.respawn.get(&(tx, ty)).is_some_and(|t| *t > 0.0)
    }

/// Gets a live enemy for the spawn tile, creating it on first contact or
/// resurrecting it after its respawn timer elapses (default: 15s).
pub fn get(&mut self, tx: i32, ty: i32, kind: EnemyKind, dt: f32) -> Option<&mut Enemy> {
    if let Some(rt) = self.respawn.get_mut(&(tx, ty)) {
        *rt -= dt;
        if *rt <= 0.0 {
            self.respawn.remove(&(tx, ty));
        } else {
            return None;
        }
    }
    if !self.enemies.contains_key(&(tx, ty)) {
        let mut e = Enemy::new(tx as f32 + 0.5, ty as f32 + 0.5, kind);
        e.flash = 0.9;
        self.enemies.insert((tx, ty), e);
    }
    self.enemies.get_mut(&(tx, ty))
}

    pub fn enemies_mut(&mut self) -> impl Iterator<Item = &mut Enemy> {
        self.enemies.values_mut()
    }

    /// Mutable enemies with their spawn-tile keys (needed to resolve kills).
    pub fn iter_mut_with_key(&mut self) -> impl Iterator<Item = ((i32, i32), &mut Enemy)> {
        self.enemies.iter_mut().map(|(&k, e)| (k, e))
    }

    /// Spawn a roaming elite (mini-boss) at a free world position. Keyed by its
    /// current tile so it integrates with the normal update/kill pipeline; it
    /// wanders from there under its own AI. `elite` scales HP, damage and XP.
    pub fn spawn_elite(&mut self, kind: EnemyKind, x: f32, y: f32, elite: f32) {
        let key = (x.floor() as i32, y.floor() as i32);
        let mut e = Enemy::new(x, y, kind).with_elite(elite);
        e.flash = 0.9;
        self.enemies.insert(key, e);
    }

    pub fn enemies(&self) -> impl Iterator<Item = &Enemy> {
        self.enemies.values()
    }

    pub fn count(&self) -> usize {
        self.enemies.len()
    }

    /// Replace the entire enemy set with the server's snapshot (multiplayer
    /// client). Keyed by tile so the spawner registry logic keeps working.
    pub fn render_sync(&mut self, enemies: Vec<Enemy>) {
        self.enemies = enemies
            .into_iter()
            .map(|e| ((e.x as i32, e.y as i32), e))
            .collect();
    }

    /// Marks a spawner dead and starts its respawn timer.
    pub fn kill(&mut self, tx: i32, ty: i32, respawn_s: f32) {
        self.enemies.remove(&(tx, ty));
        self.respawn.insert((tx, ty), respawn_s);
    }
}

/// Collects live enemies in the given tile range.
pub fn enemies_in_range<'a>(
    enemies: impl Iterator<Item = &'a Enemy>,
    cx: f32,
    cy: f32,
    range: f32,
) -> Vec<&'a Enemy> {
    enemies
        .filter(|e| (e.x - cx).abs().max((e.y - cy).abs()) <= range)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;
    use crate::world::{ChunkCache, WorldGen, tile_at};

    fn world() -> (WorldGen, ChunkCache) {
        (WorldGen::new(1337), ChunkCache::new(8))
    }

    fn blocked<'a>(world: &'a WorldGen, cache: &'a mut ChunkCache) -> impl FnMut(i32, i32) -> bool + 'a {
        move |tx, ty| !tile_at(world, cache, tx, ty).walkable()
    }

    #[test]
    fn astar_finds_path_around_obstacle() {
        let mut is_blocked = |tx: i32, ty: i32| tx == 3 && (0..=2).contains(&ty);
        let path = astar((2, 1), (4, 1), &mut is_blocked);
        assert!(path.contains(&(2, 2)), "should detour around the wall: {path:?}");
        assert!(path.contains(&(4, 2)) || path.contains(&(4, 1)), "should reach the goal: {path:?}");
    }

    #[test]
    fn astar_empty_when_unreachable() {
        let mut is_blocked = |tx: i32, _ty: i32| tx != 0 && tx != 5;
        let path = astar((0, 0), (10, 0), &mut is_blocked);
        assert!(path.is_empty(), "should fail cleanly: {path:?}");
    }

    #[test]
    fn astar_shortest_around_wall() {
        // wall occupies x=1, rows 0..=4 — the shortest detour is 8 steps
        let mut is_blocked = |tx: i32, ty: i32| tx == 1 && (0..=4).contains(&ty);
        let path = astar((0, 2), (2, 2), &mut is_blocked);
        assert_eq!(path.len(), 8, "detour around the wall: {path:?}");
        assert!(!path.contains(&(1, 2)), "must not pass through the wall");
    }

    #[test]
    fn slime_attacks_in_range() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Slime);
        let mut blocked = blocked(&w, &mut c);
        // The first tick begins a wind-up telegraph; damage lands once it elapses.
        let mut hit = None;
        for _ in 0..20 {
            hit = e.update((6.0, 6.0), 0.05, &mut blocked);
            if hit.is_some() {
                break;
            }
        }
        assert!(hit.is_some(), "adjacent slime must eventually hit after wind-up");
        assert_eq!(e.state, AiState::Attack);
    }

    #[test]
    fn slime_chases_in_aggro_range() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Slime);
        let mut blocked = blocked(&w, &mut c);
        let start = (e.x, e.y);
        for _ in 0..20 {
            e.update((10.5, 5.5), 0.05, &mut blocked);
        }
        assert_eq!(e.state, AiState::Chase);
        assert!(
            (e.x - start.0).abs() > 0.01 || (e.y - start.1).abs() > 0.01,
            "chasing slime must move"
        );
    }

    #[test]
    fn slime_idles_out_of_aggro() {
        let (w, mut c) = world();
        let mut e = Enemy::new(0.5, 0.5, EnemyKind::Slime);
        let mut blocked = blocked(&w, &mut c);
        for _ in 0..10 {
            e.update((40.5, 40.5), 0.05, &mut blocked);
        }
        assert_eq!(e.state, AiState::Idle);
    }

    #[test]
    fn damage_and_death() {
        let mut e = Enemy::new(0.5, 0.5, EnemyKind::Slime);
        let mut taken = 0.0;
        while e.alive() {
            e.take_damage(4.0);
            taken += 1.0;
        }
        assert!(taken >= 3.0, "12 hp / 4 dmg = 3 hits");
        assert_eq!(e.drops(), vec![ItemKind::Food, ItemKind::Herb, ItemKind::Gold]);
    }

    #[test]
    fn registry_respawns_after_timer() {
        let mut reg = EnemyRegistry::new();
        reg.enemies
            .insert((3, 3), Enemy::new(3.5, 3.5, EnemyKind::Slime));
        reg.kill(3, 3, 5.0);
        assert!(reg.get(3, 3, EnemyKind::Slime, 1.0).is_none(), "dead until timer");
        assert!(reg.get(3, 3, EnemyKind::Slime, 4.5).is_some(), "respawned after timer");
    }

    #[test]
    fn registry_creates_enemy_on_first_get() {
        let mut reg = EnemyRegistry::new();
        let e = reg.get(3, 3, EnemyKind::Slime, 0.0);
        assert!(e.is_some(), "first get must spawn the enemy");
        assert_eq!(reg.count(), 1);
        assert!(!reg.is_respawn_pending(3, 3));
    }

    #[test]
    fn spawner_only_on_land_biomes() {
        // no spawners on non-walkable / water biomes
        assert!(spawner_on(0, 0, TileKind::Water).is_none());
        assert!(spawner_on(0, 0, TileKind::Sand).is_none());
        assert!(spawner_on(0, 0, TileKind::Snow).is_none());
        assert!(spawner_on(0, 0, TileKind::DeepWater).is_none());
    }

    #[test]
    fn each_new_biome_has_a_spawner() {
        fn count(kind: TileKind) -> usize {
            (-32..32)
                .flat_map(|tx| (-32..32).map(move |ty| (tx, ty)))
                .filter(|&(tx, ty)| spawner_on(tx, ty, kind).is_some())
                .count()
        }
        assert!(count(TileKind::Stone) > 0, "skeletons should spawn on stone");
        assert!(count(TileKind::Grass) > 0, "goblins should spawn on grass");
        assert!(count(TileKind::Forest) > 0, "spiders should spawn on forest");
    }

    #[test]
    fn boss_stats_are_scaled_up() {
        assert!(EnemyKind::Boss.max_hp() > EnemyKind::Slime.max_hp() * 4.0);
        assert!(EnemyKind::Boss.damage() > EnemyKind::Slime.damage() * 2.0);
        assert_eq!(Enemy::new(0.5, 0.5, EnemyKind::Boss).drops(), vec![ItemKind::Fragment, ItemKind::Gold]);
    }

    #[test]
    fn stormcaller_is_flying_ranged_caster() {
        assert!(EnemyKind::Stormcaller.flying(), "stormcaller drifts over walls");
        assert!(EnemyKind::Stormcaller.ranged(), "stormcaller fires from range");
        assert!(EnemyKind::Stormcaller.shoot_range() > 0.0);
        assert!(!EnemyKind::Stormcaller.is_boss());
        assert_eq!(EnemyKind::Stormcaller.name(), "Stormcaller");
        // it should appear somewhere on the stone peaks
        let found = (-64..64)
            .flat_map(|tx| (-64..64).map(move |ty| (tx, ty)))
            .any(|(tx, ty)| spawner_on(tx, ty, TileKind::Stone) == Some(EnemyKind::Stormcaller));
        assert!(found, "stormcaller must spawn on stone");
    }

    #[test]
    fn boss_attacks_in_wide_reach() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Boss);
        let mut blocked = blocked(&w, &mut c);
        // within the boss melee reach (1.9) but outside the slime reach (1.1)
        let mut hit = None;
        for _ in 0..20 {
            hit = e.update((6.8, 5.5), 0.05, &mut blocked);
            if hit.is_some() {
                break;
            }
        }
        assert!(hit.is_some(), "boss must hit from its wider reach after wind-up");
        assert_eq!(e.state, AiState::Attack);
    }

    #[test]
    fn melee_telegraphs_before_striking() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Slime);
        let mut blocked = blocked(&w, &mut c);
        let first = e.update((6.0, 6.0), 0.05, &mut blocked);
        assert!(first.is_none(), "first tick begins the wind-up, no hit yet");
        assert!(e.windup > 0.0, "wind-up should be active");
        assert_eq!(e.state, AiState::Attack);
    }

    #[test]
    fn goblin_flees_when_low() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Goblin);
        e.hp = e.kind.max_hp() * 0.1;
        let start = (e.x, e.y);
        let mut blocked = blocked(&w, &mut c);
        for _ in 0..12 {
            e.update((9.0, 5.5), 0.05, &mut blocked);
        }
        assert_eq!(e.state, AiState::Flee, "badly hurt goblin should flee");
        assert!(e.x < start.0, "goblin should move away from the player");
    }

    #[test]
    fn boss_chases_from_far_away() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Boss);
        let mut blocked = blocked(&w, &mut c);
        let start = (e.x, e.y);
        for _ in 0..30 {
            e.update((14.5, 5.5), 0.05, &mut blocked);
        }
        assert_eq!(e.state, AiState::Chase);
        assert!((e.x - start.0).abs() > 0.01, "boss must close distance");
    }

    #[test]
    fn boss_dies_after_many_hits_and_drops_fragment() {
        let mut e = Enemy::new(0.5, 0.5, EnemyKind::Boss);
        let mut taken = 0.0;
        while e.alive() {
            e.take_damage(5.0);
            taken += 1.0;
        }
        assert!(taken >= 12.0, "60 hp / 5 dmg = 12 hits");
        assert_eq!(e.drops(), vec![ItemKind::Fragment, ItemKind::Gold]);
    }
    #[test]
    fn swamp_has_some_spawners_in_a_window() {
        let n = (-32..32)
            .flat_map(|tx| (-32..32).map(move |ty| (tx, ty)))
            .filter(|&(tx, ty)| spawner_on(tx, ty, TileKind::Swamp).is_some())
            .count();
        assert!(n > 20, "expected ~1/14 of 4096, got {n}");
    }

    #[test]
    fn colossus_is_a_heavy_bonus_elite_that_does_not_drop_a_fragment() {
        let e = Enemy::new(0.5, 0.5, EnemyKind::Colossus);
        assert!(
            !e.drops().contains(&ItemKind::Fragment),
            "colossus is a bonus elite and must NOT drop a crown fragment"
        );
        assert!(EnemyKind::Colossus.max_hp() > 100.0, "colossus is a heavy boss");
    }

    #[test]
    fn each_fragment_boss_drops_a_fragment() {
        for k in [
            EnemyKind::Boss,
            EnemyKind::ScorpionQueen,
            EnemyKind::FrostGolem,
            EnemyKind::ToadKing,
            EnemyKind::OceanLeviathan,
        ] {
            let e = Enemy::new(0.5, 0.5, k);
            assert!(e.drops().contains(&ItemKind::Fragment), "{k:?} must drop a fragment");
            assert_eq!(k.fragment_bit().is_some(), true, "{k:?} must map to a fragment bit");
        }
    }

    #[test]
    fn stoneslinger_fires_when_player_in_range() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Stoneslinger);
        let mut blocked = blocked(&w, &mut c);
        // player within aggro (6) and shoot range (9) but outside melee reach
        e.update((10.5, 5.5), 0.05, &mut blocked);
        assert!(e.pending_shot.is_some(), "stoneslinger should fire at range");
    }

    #[test]
    fn player_dodge_grants_iframes_and_costs_stamina() {
        let mut p = Player::new(0.5, 0.5);
        p.stamina = 100.0;
        assert!(p.try_dodge((1.0, 0.0)));
        assert!(p.dodge_timer > 0.0, "dodge burst active");
        assert!(p.hurt_timer > 0.0, "dodge grants i-frames");
        assert!(p.stamina < 100.0, "dodge costs stamina");
        // immediately dodging again is blocked by cooldown
        assert!(!p.try_dodge((0.0, 1.0)), "cooldown blocks back-to-back dodges");
    }

    #[test]
    fn brute_winds_up_then_charges() {
        let (w, mut c) = world();
        let mut e = Enemy::new(5.5, 5.5, EnemyKind::Brute);
        let mut blocked = blocked(&w, &mut c);
        // far enough to be in aggro but outside melee; first tick should arm a charge
        e.update((9.5, 5.5), 0.05, &mut blocked);
        assert!(e.charge_t > 0.0, "brute should begin a charge when in range");
        // during the charge it dashes straight toward the player
        let before = (e.x, e.y);
        e.update((9.5, 5.5), 0.05, &mut blocked);
        assert!((e.x - before.0).abs() > 0.05, "brute should lunge during the charge");
    }

    #[test]
    fn enemy_kind_metadata_for_codex() {
        assert_eq!(EnemyKind::Brute.name(), "Brute");
        assert!(EnemyKind::Colossus.is_boss());
        assert!(!EnemyKind::Slime.is_boss());
        assert!(!EnemyKind::Brute.behavior().is_empty());
    }

    #[test]
    fn every_combat_enemy_drops_loot() {
        let kinds = [
            EnemyKind::Slime,
            EnemyKind::Skeleton,
            EnemyKind::Goblin,
            EnemyKind::Bat,
            EnemyKind::Spider,
            EnemyKind::Imp,
            EnemyKind::Ogre,
            EnemyKind::Wraith,
            EnemyKind::Stoneslinger,
            EnemyKind::Colossus,
            EnemyKind::ScorpionQueen,
            EnemyKind::FrostGolem,
            EnemyKind::ToadKing,
            EnemyKind::OceanLeviathan,
            EnemyKind::Brute,
            EnemyKind::Stormcaller,
            EnemyKind::Wolf,
            EnemyKind::Archer,
            EnemyKind::Raider,
        ];
        for k in kinds {
            assert!(!k.drops().is_empty(), "{k:?} must drop at least one item");
        }
    }

    #[test]
    fn loot_spread_supports_the_forge() {
        // After tuning, iron scrap and gems come from many enemy types so the
        // crafting economy isn't gated on node farming alone.
        let iron_sources = [
            EnemyKind::Skeleton,
            EnemyKind::Goblin,
            EnemyKind::Spider,
            EnemyKind::Ogre,
            EnemyKind::Brute,
            EnemyKind::Raider,
        ];
        for k in iron_sources {
            assert!(
                k.drops().contains(&ItemKind::Iron),
                "{k:?} should drop iron scrap"
            );
        }
        let gem_sources = [
            EnemyKind::Ogre,
            EnemyKind::Stoneslinger,
            EnemyKind::Colossus,
            EnemyKind::Brute,
            EnemyKind::Stormcaller,
            EnemyKind::Raider,
        ];
        for k in gem_sources {
            assert!(
                k.drops().contains(&ItemKind::Gem),
                "{k:?} should drop gems"
            );
        }
    }
}