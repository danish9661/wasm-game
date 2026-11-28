//! Deterministic weather → fire interactions.
//!
//! The renderer owns structures, so this module deliberately owns no world
//! state: callers hand it the current weather, the tiles carrying campfires,
//! and a shelter predicate. Everything it returns is plain integer/f32 data
//! that can be applied in the same order on client and server. No RNG: the
//! lightning candidate comes from a fixed LCG keyed by the simulation tick.

use serde::{Deserialize, Serialize};

/// Weather codes shared with the renderer / HUD (`weather` 0..4).
pub const CLEAR: u8 = 0;
pub const RAIN: u8 = 1;
pub const SNOW: u8 = 2;
pub const STORM: u8 = 3;
pub const HEAT_WAVE: u8 = 4;

/// Seconds of rain exposure needed to douse an unsheltered campfire.
pub const DOUSE_SECONDS: f32 = 12.0;
/// Storms soak fires faster than a gentle rain.
pub const STORM_DOUSE_MULT: f32 = 1.6;
/// Seconds between lightning strikes while a storm rages.
pub const LIGHTNING_INTERVAL: f32 = 22.0;
/// Max tile offset of a lightning strike from the player.
pub const STRIKE_RANGE: i32 = 6;

/// True when weather actively wets exposed fires (rain or storm).
pub fn is_wet(weather: u8) -> bool {
    weather == RAIN || weather == STORM
}

/// One fire-related weather tick's output, applied by the caller.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WeatherEvents {
    /// Campfire tiles that just burnt out in the rain.
    pub doused: Vec<(i32, i32)>,
    /// Candidate tile for a lightning-ignited fire (caller validates terrain).
    pub lightning: Option<(i32, i32)>,
}

fn lightning_interval() -> f32 {
    LIGHTNING_INTERVAL
}

/// Named fields instead of a tuple key so JSON saves round-trip as cleanly as
/// bincode; every field defaults for old save files.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct FireExposure {
    #[serde(default)]
    tx: i32,
    #[serde(default)]
    ty: i32,
    #[serde(default)]
    wet: f32,
}

impl FireExposure {
    fn tile(&self) -> (i32, i32) {
        (self.tx, self.ty)
    }
}

/// Session state for rain exposure and the lightning cooldown. All fields are
/// serde-defaulted so old saves and snapshot adapters keep loading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherFireState {
    /// Accumulated wetness per active campfire tile (seconds).
    #[serde(default)]
    exposure: Vec<FireExposure>,
    /// Seconds until the next storm may strike.
    #[serde(default = "lightning_interval")]
    lightning_timer: f32,
}

impl Default for WeatherFireState {
    fn default() -> Self {
        Self {
            exposure: Vec::new(),
            lightning_timer: LIGHTNING_INTERVAL,
        }
    }
}

impl WeatherFireState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current accumulated wetness for a tile (0 when unknown).
    pub fn exposure(&self, tile: (i32, i32)) -> f32 {
        self.exposure
            .iter()
            .find(|e| e.tile() == tile)
            .map_or(0.0, |e| e.wet)
    }

    /// Advance rain dousing and lightning. `sheltered` describes the *player*;
    /// `covered` describes each fire tile (a roof over the fire blocks rain).
    /// `tick` is any monotonically increasing simulation counter — the same
    /// (tick, player tile) always yields the same strike candidate.
    pub fn tick(
        &mut self,
        dt: f32,
        weather: u8,
        fires: &[(i32, i32)],
        covered: impl Fn((i32, i32)) -> bool,
        player_sheltered: bool,
        player_tile: (i32, i32),
        tick: u32,
    ) -> WeatherEvents {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let mut events = WeatherEvents::default();
        let wet = is_wet(weather);
        let rate = if weather == STORM { STORM_DOUSE_MULT } else { 1.0 };

        // Retire stale entries so a long session cannot grow this vector
        // without bound. Deterministic iteration order: insertion order.
        self.exposure.retain(|e| fires.contains(&e.tile()));

        for &tile in fires {
            let entry = self.exposure.iter_mut().find(|e| e.tile() == tile);
            if covered(tile) {
                // Shelter blocks rain: a covered fire even dries out.
                if let Some(e) = entry {
                    e.wet = (e.wet - dt * 2.0).max(0.0);
                }
                continue;
            }
            let value = if wet {
                entry.map_or(dt * rate, |e| {
                    e.wet = (e.wet + dt * rate).min(DOUSE_SECONDS);
                    e.wet
                })
            } else if let Some(e) = entry {
                e.wet = (e.wet - dt).max(0.0);
                e.wet
            } else {
                0.0
            };
            if value > 0.0 && !self.exposure.iter().any(|e| e.tile() == tile) {
                self.exposure.push(FireExposure {
                    tx: tile.0,
                    ty: tile.1,
                    wet: value,
                });
            }
            if value >= DOUSE_SECONDS {
                events.doused.push(tile);
            }
        }
        self.exposure.retain(|e| {
            e.wet > 0.0 && fires.contains(&e.tile()) && !events.doused.contains(&e.tile())
        });

        // Lightning only during storms, and only for a player in the open.
        self.lightning_timer = (self.lightning_timer - dt).max(0.0);
        if weather == STORM && self.lightning_timer <= 0.0 {
            self.lightning_timer = LIGHTNING_INTERVAL + (tick % 5) as f32 * 2.0;
            if !player_sheltered {
                events.lightning = Some(strike_candidate(player_tile, tick));
            }
        }
        events
    }
}

/// Fixed LCG candidate tile, deterministic across machines (u32 wrapping
/// arithmetic). The caller decides whether the tile is actually flammable.
pub fn strike_candidate(player_tile: (i32, i32), tick: u32) -> (i32, i32) {
    let mut h = tick
        .wrapping_mul(2654435761)
        ^ (player_tile.0 as u32).wrapping_mul(73856093)
        ^ (player_tile.1 as u32).wrapping_mul(19349663);
    h = h.wrapping_mul(1664525).wrapping_add(1013904223);
    let span = (STRIKE_RANGE * 2 + 1) as u32;
    let dx = ((h >> 9) % span) as i32 - STRIKE_RANGE;
    let dy = ((h >> 21) % span) as i32 - STRIKE_RANGE;
    (player_tile.0 + dx, player_tile.1 + dy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_saves_default_to_a_sane_cooldown() {
        let state: WeatherFireState = serde_json::from_str("{}").unwrap();
        assert_eq!(state.lightning_timer, LIGHTNING_INTERVAL);
        let round: WeatherFireState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(round, state);
    }

    #[test]
    fn exposure_round_trips_through_json() {
        let mut state = WeatherFireState::new();
        state.tick(3.0, RAIN, &[(2, -1)], |_| false, false, (0, 0), 0);
        let json = serde_json::to_string(&state).unwrap();
        let back: WeatherFireState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
        assert_eq!(back.exposure((2, -1)), 3.0);
    }

    #[test]
    fn rain_douses_only_unsheltered_fires() {
        let mut state = WeatherFireState::new();
        let fires = [(3, 4), (9, 9)];
        let covered = |tile| tile == (3, 4);
        let mut doused = Vec::new();
        for _ in 0..13 {
            let e = state.tick(1.0, RAIN, &fires, covered, false, (0, 0), 0);
            doused.extend(e.doused);
        }
        assert_eq!(doused, vec![(9, 9)], "roofed fire never douses");
        assert!(state.exposure((3, 4)) <= 0.0, "covered fire dries out");
        assert!(state.exposure((9, 9)) <= DOUSE_SECONDS);
    }

    #[test]
    fn storm_douses_faster_than_rain_and_clear_weather_dries() {
        let rain = {
            let mut s = WeatherFireState::new();
            for _ in 0..5 {
                s.tick(1.0, RAIN, &[(0, 0)], |_| false, false, (0, 0), 0);
            }
            s.exposure((0, 0))
        };
        let storm = {
            let mut s = WeatherFireState::new();
            for _ in 0..5 {
                s.tick(1.0, STORM, &[(0, 0)], |_| false, false, (0, 0), 0);
            }
            s.exposure((0, 0))
        };
        assert!(storm > rain, "storm must soak faster");
        let mut s = WeatherFireState::new();
        s.tick(100.0, RAIN, &[(0, 0)], |_| false, false, (0, 0), 0);
        for _ in 0..10 {
            s.tick(1.0, CLEAR, &[(0, 0)], |_| false, false, (0, 0), 0);
        }
        assert!(s.exposure((0, 0)) < DOUSE_SECONDS - 9.0, "clear weather dries fires");
        assert_eq!(s.exposure((1, 1)), 0.0);
    }

    #[test]
    fn lightning_needs_a_storm_and_an_exposed_player() {
        let mut state = WeatherFireState::new();
        // Rain and clear weather never strike, even long enough to recharge.
        for w in [RAIN, CLEAR, SNOW, HEAT_WAVE] {
            for i in 0..30 {
                let e = state.tick(1.0, w, &[], |_| false, false, (0, 0), i);
                assert_eq!(e.lightning, None);
            }
        }
        // Shelter blocks the strike, but the cooldown still resets.
        let e = state.tick(LIGHTNING_INTERVAL, STORM, &[], |_| false, true, (0, 0), 7);
        assert_eq!(e.lightning, None, "shelter blocks lightning");
        let e = state.tick(LIGHTNING_INTERVAL + 10.0, STORM, &[], |_| false, false, (0, 0), 7);
        let tile = e.lightning.expect("storm must eventually strike");
        assert!(tile.0.abs() <= STRIKE_RANGE && tile.1.abs() <= STRIKE_RANGE);
        // Same tick + tile always yields the same candidate.
        assert_eq!(tile, strike_candidate((0, 0), 7));
    }

    #[test]
    fn lightning_candidate_stays_near_the_player_and_is_deterministic() {
        for tick in 0..200u32 {
            let a = strike_candidate((12, -4), tick);
            let b = strike_candidate((12, -4), tick);
            assert_eq!(a, b);
            assert!((a.0 - 12).abs() <= STRIKE_RANGE && (a.1 + 4).abs() <= STRIKE_RANGE);
        }
        // Invalid/negative dt cannot poison the state.
        let mut s = WeatherFireState::new();
        let e = s.tick(-5.0, STORM, &[], |_| false, false, (0, 0), 0);
        assert_eq!(e.lightning, None);
        assert_eq!(s.lightning_timer, LIGHTNING_INTERVAL);
    }
}
