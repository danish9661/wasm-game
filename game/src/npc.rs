use serde::{Deserialize, Serialize};

/// Non-hostile townsfolk that populate villages. They wander, chat when you
/// approach, and give the world a lived-in feel. Guards can hint at danger;
/// merchants can (later) trade. Enemies never target them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcKind {
    Villager,
    Guard,
    Merchant,
}

impl NpcKind {
    pub fn label(self) -> &'static str {
        match self {
            NpcKind::Villager => "Villager",
            NpcKind::Guard => "Guard",
            NpcKind::Merchant => "Merchant",
        }
    }

    /// Body tint for the humanoid sprite.
    pub fn color(self) -> [f32; 3] {
        match self {
            NpcKind::Villager => [0.80, 0.66, 0.48],
            NpcKind::Guard => [0.55, 0.58, 0.66],
            NpcKind::Merchant => [0.80, 0.62, 0.78],
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
    home: (f32, f32),
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
        }
    }

    /// Gentle wander around `home`. `is_blocked` reports whether a tile is solid
    /// (water/wall) so NPCs stay on paths.
    pub fn update(&mut self, dt: f32, mut is_blocked: impl FnMut(f32, f32) -> bool) {
        self.retarget -= dt;
        let dx = self.target.0 - self.x;
        let dy = self.target.1 - self.y;
        let d = (dx * dx + dy * dy).sqrt();
        if d < 0.15 || self.retarget <= 0.0 {
            let ang = (self.x * 12.9 + self.y * 78.2).fract() * std::f32::consts::TAU + self.retarget;
            let r = 1.5 + (self.x as f32 * 3.1).fract().abs() * 3.0;
            let tx = (self.home.0 + ang.cos() * r).round() as f32;
            let ty = (self.home.1 + ang.sin() * r).round() as f32;
            if !is_blocked(tx, ty) {
                self.target = (tx, ty);
            }
            self.retarget = 1.5 + (self.y.fract().abs()) * 2.0;
        }
        if d > 0.05 {
            let nx = dx / d;
            let ny = dy / d;
            self.facing = (nx, ny);
            let sp = 1.6 * dt; // slow stroll
            let nx2 = self.x + nx * sp;
            let ny2 = self.y + ny * sp;
            if !is_blocked(nx2, ny2) {
                self.x = nx2;
                self.y = ny2;
                self.walk = (self.walk + dt * 4.0) % 1000.0;
            }
        } else {
            self.walk = (self.walk * 0.9).max(0.0);
        }
    }
}
