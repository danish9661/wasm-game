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