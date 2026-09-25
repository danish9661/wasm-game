use std::f32::consts::TAU;

/// One full day in real seconds (~10 minutes per the spec).
pub const DAY_LENGTH: f32 = 600.0;
/// Game start time: 0.0 is midnight, 0.5 is noon. Start at ~09:36.
pub const START_TIME: f32 = 0.4;

/// Daylight factor in [0, 1] for time `t` in [0, 1): 1.0 at noon, 0.0 at
/// midnight, smooth cosine ramp through dusk/dawn.
pub fn daylight(t: f32) -> f32 {
    0.5 + 0.5 * ((t - 0.5) * TAU).cos()
}

/// Daylight below this reads as night for pressure systems (raiders,
/// nocturnals, shattered nights).
pub const NIGHTFALL: f32 = 0.3;

/// True every 4th night for the world seed: the Shattered Night, when the
/// wilds send extra elites hunting till dawn. Deterministic from
/// (seed, day) so client and server agree without protocol traffic.
pub fn shattered_night(day_index: u32, seed: u32) -> bool {
    day_index.wrapping_add(seed) % 4 == 3
}

/// "HH:MM" clock string for the HUD.
pub fn clock(t: f32) -> String {
    let minutes = (t.rem_euclid(1.0) * 24.0 * 60.0) as u32;
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// Ambient temperature in °C for time `t`: warmest at ~14:00, coldest at
/// ~02:00. Range roughly [-8, 28] — night can bite.
pub fn temperature(t: f32) -> f32 {
    10.0 + 18.0 * ((t - 14.0 / 24.0) * TAU).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shattered_nights_come_every_fourth() {
        // Seed 1337: days 2, 6, 10... shatter ((d+1337)%4==3).
        assert!(shattered_night(2, 1337));
        assert!(shattered_night(6, 1337));
        assert!(!shattered_night(0, 1337));
        assert!(!shattered_night(1, 1337));
        assert!(!shattered_night(3, 1337));
        // Exactly one shattered night in any 4-day window, any seed.
        for seed in [1, 42, 1337, 99999] {
            for start in [0, 5, 100] {
                let n = (start..start + 4).filter(|d| shattered_night(*d, seed)).count();
                assert_eq!(n, 1, "one shattered night per 4 days (seed {seed})");
            }
        }
    }

    #[test]
    fn daylight_bounds() {
        for i in 0..1000 {
            let d = daylight(i as f32 / 1000.0);
            assert!((0.0..=1.0).contains(&d), "daylight out of range at {i}");
        }
    }

    #[test]
    fn noon_bright_midnight_dark() {
        assert!((daylight(0.5) - 1.0).abs() < 0.001);
        assert!(daylight(0.0) < 0.001);
    }

    #[test]
    fn day_night_symmetry() {
        let day = daylight(0.25);
        let dusk = daylight(0.75);
        assert!((day - dusk).abs() < 0.001);
    }

    #[test]
    fn clock_format() {
        assert_eq!(clock(0.0), "00:00");
        assert_eq!(clock(0.5), "12:00");
        assert_eq!(clock(0.4), "09:36");
        assert_eq!(clock(1.0), "00:00", "time wraps");
    }

    #[test]
    fn temperature_warm_at_afternoon_cold_at_night() {
        let warm = temperature(14.0 / 24.0);
        let cold = temperature(2.0 / 24.0);
        assert!(warm > 20.0, "afternoon should be warm, got {warm}");
        assert!(cold < 0.0, "night should be cold, got {cold}");
        assert!(warm - cold > 30.0, "diurnal range too small");
    }
}