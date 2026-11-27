use serde::{Deserialize, Serialize};

use crate::items::ItemKind;

/// Non-hostile townsfolk that populate villages. They wander, chat when you
/// approach, and give the world a lived-in feel. Guards can hint at danger;
/// merchants can (later) trade. Enemies never target them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcKind {
    Villager,
    Guard,
    Merchant,
    Golem,
}

/// What a townsperson is doing right now. Drives movement, facing and the
/// little walk-bob used by the renderer, so behaviour reads in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NpcState {
    #[default]
    Idle,
    Wander,
    /// Turned toward the player, waiting for a conversation.
    Greet,
    /// Guard/golem circuit around their home point.
    Patrol,
    /// Guard/golem moving toward a nearby hostile.
    Alert,
    /// Civilian retreating from danger.
    Flee,
    /// Merchant standing at their stall.
    Trade,
    /// At home after dark; only guards and golems stay on watch.
    Sleep,
}

impl NpcState {
    pub fn label(self) -> &'static str {
        match self {
            NpcState::Idle => "idle",
            NpcState::Wander => "wander",
            NpcState::Greet => "greet",
            NpcState::Patrol => "patrol",
            NpcState::Alert => "alert",
            NpcState::Flee => "flee",
            NpcState::Trade => "trade",
            NpcState::Sleep => "sleep",
        }
    }
}

/// World facts an NPC needs for a single AI tick. The renderer fills this
/// from the live game; tests and headless sims can use `Default`.
#[derive(Debug, Clone, Copy)]
pub struct NpcContext {
    pub player: (f32, f32),
    /// Nearest hostile creature, if one is close enough to matter.
    pub danger: Option<(f32, f32)>,
    pub night: bool,
    /// Distance at which the NPC notices and greets the player.
    pub talk_range: f32,
}

impl Default for NpcContext {
    fn default() -> Self {
        Self {
            player: (f32::NAN, f32::NAN),
            danger: None,
            night: false,
            talk_range: 3.4,
        }
    }
}

impl NpcKind {
    pub fn label(self) -> &'static str {
        match self {
            NpcKind::Villager => "Villager",
            NpcKind::Guard => "Guard",
            NpcKind::Merchant => "Merchant",
            NpcKind::Golem => "Golem",
        }
    }

    /// Body tint for the humanoid sprite.
    pub fn color(self) -> [f32; 3] {
        match self {
            NpcKind::Villager => [0.80, 0.66, 0.48],
            NpcKind::Guard => [0.55, 0.58, 0.66],
            NpcKind::Merchant => [0.80, 0.62, 0.78],
            NpcKind::Golem => [0.60, 0.62, 0.66],
        }
    }

    fn is_watch(self) -> bool {
        matches!(self, NpcKind::Guard | NpcKind::Golem)
    }

    fn leash(self) -> f32 {
        match self {
            NpcKind::Guard => 11.0,
            NpcKind::Golem => 8.0,
            _ => 5.0,
        }
    }

    fn speed(self) -> f32 {
        match self {
            NpcKind::Guard => 2.2,
            NpcKind::Golem => 1.5,
            NpcKind::Merchant => 1.0,
            NpcKind::Villager => 1.35,
        }
    }

    /// A little flavor line shown when you talk to them.
    pub fn line(self, seed: u32) -> &'static str {
        let lines: &[&[&str]] = &[
            &[
                "The nights grow colder. Keep a fire lit.",
                "Mind the wolves in the woods.",
                "I heard the old ruins still hold a fragment.",
                "Trade? Not today, friend.",
            ],
            &[
                "Stay within the walls after dark.",
                "The Crown's shards are cursed — and priceless.",
                "I've slain things you wouldn't believe.",
                "Report trouble to the watch.",
            ],
            &[
                "Fine wares for a fair price, traveler.",
                "Gems fetch the best coin.",
                "An anvil and a dream — that's all it takes.",
                "Restock comes with the next caravan.",
            ],
        ];
        let pool = match self {
            NpcKind::Villager => lines[0],
            NpcKind::Guard => lines[1],
            NpcKind::Merchant => lines[2],
            NpcKind::Golem => &["I guard this place with stone and silence."],
        };
        pool[(seed as usize) % pool.len()]
    }
}

/// A single townsperson.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub kind: NpcKind,
    pub x: f32,
    pub y: f32,
    pub name: String,
    pub facing: (f32, f32),
    pub walk: f32,
    /// Wander target; when reached, a new one is picked after a pause.
    target: (f32, f32),
    retarget: f32,
    /// Home anchor so they don't drift too far from the village.
    pub home: (f32, f32),
    /// An optional fetch quest this townsperson has given the player:
    /// (need item, need count, reward xp, reward item, reward count). When the
    /// player fulfills it (via talk), the reward is granted and this clears.
    pub quest: Option<(ItemKind, u32, u32, ItemKind, u32)>,
    /// Current AI state (defaults keep old save files loadable).
    #[serde(default)]
    pub state: NpcState,
    #[serde(default)]
    state_timer: f32,
    #[serde(default)]
    patrol_step: u8,
}

impl Npc {
    pub fn new(kind: NpcKind, x: f32, y: f32, name: String, home: (f32, f32)) -> Self {
        Self {
            kind,
            x,
            y,
            name,
            facing: (0.0, 1.0),
            walk: 0.0,
            target: (x, y),
            retarget: 0.0,
            home,
            quest: None,
            state: NpcState::Idle,
            state_timer: 0.0,
            patrol_step: 0,
        }
    }

    /// Gentle wander around `home` with no external context (tests / legacy).
    pub fn update(&mut self, dt: f32, is_blocked: impl FnMut(f32, f32) -> bool) {
        self.update_with_context(dt, is_blocked, NpcContext::default());
    }

    /// Full behaviour tick. `is_blocked` reports whether a tile is solid
    /// (water/wall) so NPCs stay on paths. The state machine is deterministic:
    /// night sends civilians home, guards patrol/chase, civilians flee danger,
    /// and everyone greets the player when close.
    pub fn update_with_context(
        &mut self,
        dt: f32,
        mut is_blocked: impl FnMut(f32, f32) -> bool,
        ctx: NpcContext,
    ) {
        self.retarget -= dt;
        self.state_timer -= dt;

        let home_d = (self.x - self.home.0).hypot(self.y - self.home.1);
        let danger_d = ctx
            .danger
            .map(|(dx, dy)| (dx - self.x).hypot(dy - self.y))
            .unwrap_or(f32::INFINITY);
        let player_d = if ctx.player.0.is_nan() {
            f32::INFINITY
        } else {
            (ctx.player.0 - self.x).hypot(ctx.player.1 - self.y)
        };

        // --- choose state ---------------------------------------------------
        let next = if ctx.night && !self.kind.is_watch() {
            NpcState::Sleep
        } else if self.kind.is_watch() && danger_d < 8.0 && home_d < self.kind.leash() {
            NpcState::Alert
        } else if danger_d < 4.5 && !self.kind.is_watch() {
            NpcState::Flee
        } else if player_d < ctx.talk_range {
            if self.kind == NpcKind::Merchant {
                NpcState::Trade
            } else {
                NpcState::Greet
            }
        } else if self.kind.is_watch() {
            NpcState::Patrol
        } else if self.retarget <= 0.0 {
            NpcState::Wander
        } else {
            self.state
        };
        if next != self.state {
            self.state = next;
            self.state_timer = 0.6;
        }

        // --- choose destination / facing ------------------------------------
        match self.state {
            NpcState::Sleep => {
                self.target = self.home;
                if home_d < 0.4 {
                    self.walk *= 0.85;
                }
            }
            NpcState::Alert => {
                let (dx, dy) = ctx.danger.unwrap();
                self.target = (dx, dy);
            }
            NpcState::Flee => {
                let (dx, dy) = ctx.danger.unwrap();
                let ax = self.x - dx;
                let ay = self.y - dy;
                let l = ax.hypot(ay).max(1e-3);
                self.target = (self.x + ax / l * 4.0, self.y + ay / l * 4.0);
                if home_d > self.kind.leash() {
                    self.target = self.home;
                }
            }
            NpcState::Greet | NpcState::Trade => {
                self.facing = (ctx.player.0 - self.x, ctx.player.1 - self.y);
                let l = self.facing.0.hypot(self.facing.1).max(1e-3);
                self.facing = (self.facing.0 / l, self.facing.1 / l);
                self.walk = self.walk + dt * 1.5 % 1000.0;
                return; // stop moving while interacting
            }
            NpcState::Patrol => {
                if self.retarget <= 0.0 || (self.target.0 - self.x).hypot(self.target.1 - self.y) < 0.4 {
                    let a = self.patrol_step as f32 * std::f32::consts::FRAC_PI_2
                        + (self.home.0 * 0.7 + self.home.1 * 1.3);
                    let r = 3.0;
                    let tx = (self.home.0 + a.cos() * r).round();
                    let ty = (self.home.1 + a.sin() * r).round();
                    self.patrol_step = self.patrol_step.wrapping_add(1);
                    if !is_blocked(tx, ty) {
                        self.target = (tx, ty);
                    }
                    self.retarget = 4.0;
                }
            }
            NpcState::Wander | NpcState::Idle => {
                if self.retarget <= 0.0
                    || (self.target.0 - self.x).hypot(self.target.1 - self.y) < 0.15
                {
                    let ang = (self.x * 12.9 + self.y * 78.2).fract()
                        * std::f32::consts::TAU
                        + self.retarget;
                    let r = 1.5 + (self.x * 3.1).fract().abs() * 3.0;
                    let tx = (self.home.0 + ang.cos() * r).round();
                    let ty = (self.home.1 + ang.sin() * r).round();
                    if !is_blocked(tx, ty) {
                        self.target = (tx, ty);
                    }
                    self.retarget = 1.5 + self.y.fract().abs() * 2.0;
                }
            }
        }
        if home_d > self.kind.leash() && self.state != NpcState::Flee {
            self.target = self.home;
        }

        // --- move, with a simple slide along blocked tiles ------------------
        let dx = self.target.0 - self.x;
        let dy = self.target.1 - self.y;
        let d = dx.hypot(dy);
        if d > 0.05 {
            let nx = dx / d;
            let ny = dy / d;
            self.facing = (nx, ny);
            let speed = self.kind.speed() * if self.state == NpcState::Alert { 1.15 } else { 1.0 };
            let step = speed * dt;
            let tx = self.x + nx * step;
            let ty = self.y + ny * step;
            if !is_blocked(tx, ty) {
                self.x = tx;
                self.y = ty;
            } else if !is_blocked(self.x + nx * step, self.y) {
                self.x += nx * step;
            } else if !is_blocked(self.x, self.y + ny * step) {
                self.y += ny * step;
            } else {
                self.retarget = 0.0; // pick a different target next tick
            }
            self.walk = (self.walk + dt * 5.0) % 1000.0;
        } else {
            self.walk *= 0.9;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn npc(kind: NpcKind) -> Npc {
        Npc::new(kind, 0.0, 0.0, "Test".into(), (0.0, 0.0))
    }

    #[test]
    fn greets_player_and_stops() {
        let mut n = npc(NpcKind::Villager);
        let ctx = NpcContext { player: (1.0, 0.0), ..Default::default() };
        n.update_with_context(0.1, |_, _| false, ctx);
        assert_eq!(n.state, NpcState::Greet);
        assert_eq!(n.facing.0, 1.0);
        assert_eq!(n.x, 0.0);
    }

    #[test]
    fn civilians_flee_danger() {
        let mut n = npc(NpcKind::Villager);
        let ctx = NpcContext { danger: Some((2.0, 0.0)), ..Default::default() };
        for _ in 0..10 { n.update_with_context(0.1, |_, _| false, ctx); }
        assert_eq!(n.state, NpcState::Flee);
        assert!(n.x < 0.0, "should move away from danger, x={}", n.x);
    }

    #[test]
    fn guards_chase_danger_but_stay_leashed() {
        let mut n = npc(NpcKind::Guard);
        let ctx = NpcContext { danger: Some((4.0, 0.0)), ..Default::default() };
        for _ in 0..10 { n.update_with_context(0.1, |_, _| false, ctx); }
        assert_eq!(n.state, NpcState::Alert);
        assert!(n.x > 0.0);
        let far = NpcContext { danger: Some((40.0, 0.0)), ..Default::default() };
        n.update_with_context(0.1, |_, _| false, far);
        assert_ne!(n.state, NpcState::Alert);
    }

    #[test]
    fn civilians_sleep_at_night_but_guards_watch() {
        let mut v = npc(NpcKind::Villager);
        let mut g = npc(NpcKind::Guard);
        let ctx = NpcContext { night: true, ..Default::default() };
        v.update_with_context(0.1, |_, _| false, ctx);
        g.update_with_context(0.1, |_, _| false, ctx);
        assert_eq!(v.state, NpcState::Sleep);
        assert_eq!(g.state, NpcState::Patrol);
    }

    #[test]
    fn blocked_tile_does_not_teleport() {
        let mut n = npc(NpcKind::Villager);
        n.target = (3.0, 0.0);
        n.retarget = 5.0;
        n.update_with_context(0.5, |x, _| x > 0.5, NpcContext::default());
        assert!(n.x <= 0.5, "blocked NPC moved to {}", n.x);
    }
}
