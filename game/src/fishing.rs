//! Shoreline fishing: a two-press timing mini-game with no randomness.
//!
//! Cast from wadable shoreline water (`TileKind::ShallowWater` next to open
//! water). After a short, fixed delay the bobber dips for a brief window; a
//! reel inside that window yields one raw `Food`. Reeling early or late simply
//! loses the cast. State is a single float so it round-trips through saves and
//! behaves identically in the browser and the headless server.

use serde::{Deserialize, Serialize};

use crate::items::{Inventory, ItemKind};

/// Seconds from cast until the bite window opens.
pub const BITE_DELAY: f32 = 0.8;
/// Length of the bite window in seconds.
pub const BITE_WINDOW: f32 = 0.7;
/// Casts expire this long after starting (delay + window + grace).
pub const CAST_TIMEOUT: f32 = BITE_DELAY + BITE_WINDOW + 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fishing {
    /// Negative = idle; otherwise seconds since the cast began.
    #[serde(default = "idle_cast")]
    cast_t: f32,
}

fn idle_cast() -> f32 {
    -1.0
}

impl Default for Fishing {
    fn default() -> Self {
        Self::new()
    }
}

impl Fishing {
    pub fn new() -> Self {
        Self { cast_t: idle_cast() }
    }

    pub fn active(&self) -> bool {
        self.cast_t >= 0.0
    }

    /// True only while the bite window is open.
    pub fn bite_open(&self) -> bool {
        self.active()
            && self.cast_t >= BITE_DELAY
            && self.cast_t <= BITE_DELAY + BITE_WINDOW
    }

    /// Begin a cast. No-op (false) if a cast is already in the water.
    pub fn cast(&mut self) -> bool {
        if self.active() {
            return false;
        }
        self.cast_t = 0.0;
        true
    }

    /// Advance the bobber; late casts are discarded.
    pub fn tick(&mut self, dt: f32) {
        if !self.active() || dt <= 0.0 {
            return;
        }
        self.cast_t += dt;
        if self.cast_t >= CAST_TIMEOUT {
            self.cast_t = idle_cast();
        }
    }

    /// Reel in. A bite pays one raw Food; a mistimed pull wastes the cast.
    pub fn reel(&mut self, inv: &mut Inventory) -> bool {
        let caught = self.bite_open();
        self.cast_t = idle_cast();
        if caught {
            inv.add(ItemKind::Food, 1);
        }
        caught
    }

    /// HUD hint for the current cast state.
    pub fn hint(&self) -> &'static str {
        if !self.active() {
            "E: cast line from the shallows"
        } else if self.bite_open() {
            "BITE! Press E now to reel"
        } else if self.cast_t < BITE_DELAY {
            "Wait for the bobber to dip…"
        } else {
            "Missed the bite — cast again"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bite_opens_after_a_delay_and_closes() {
        let mut f = Fishing::new();
        assert!(!f.active());
        assert!(f.cast());
        assert!(!f.cast(), "one line at a time");
        f.tick(BITE_DELAY - 0.1);
        assert!(!f.bite_open(), "too early");
        f.tick(0.1);
        assert!(f.bite_open(), "bite window opens at BITE_DELAY");
        f.tick(BITE_WINDOW + 0.01);
        assert!(!f.bite_open(), "bite window closed");
    }

    #[test]
    fn reeling_on_the_bite_yields_food() {
        let mut f = Fishing::new();
        let mut inv = Inventory::new();
        assert!(!f.reel(&mut inv), "cannot reel an idle line");
        assert_eq!(inv.count(ItemKind::Food), 0);
        f.cast();
        f.tick(BITE_DELAY - 0.01);
        assert!(!f.reel(&mut inv), "early pull catches nothing");
        assert_eq!(inv.count(ItemKind::Food), 0);
        f.cast();
        f.tick(BITE_DELAY + 0.02);
        assert!(f.reel(&mut inv), "timed pull lands a fish");
        assert_eq!(inv.count(ItemKind::Food), 1);
        assert!(!f.active(), "reel clears the cast");
    }

    #[test]
    fn late_cast_is_discarded() {
        let mut f = Fishing::new();
        let mut inv = Inventory::new();
        f.cast();
        f.tick(CAST_TIMEOUT);
        assert!(!f.active(), "stale bobber expires");
        assert!(!f.reel(&mut inv));
        assert_eq!(inv.count(ItemKind::Food), 0);
    }

    #[test]
    fn serde_default_restores_an_idle_line() {
        let f: Fishing = serde_json::from_str("{}").expect("old saves default to idle");
        assert_eq!(f, Fishing::new());
        assert!(f.hint().contains("cast"));
    }
}
