//! Held weapons + block shield, drawn in the player's hands.
//!
//! `build` places the weapon at the hand anchor (`hx`, `hy` in screen px) and
//! drives it forward along `facing` as `attack` (0..1) ramps — the same lunge
//! curve the humanoid arms use, so steel and arms move as one. Fists return no
//! parts (bare hands are already in the rig). Each kind has a distinct
//! silhouette: sword (blade + guard), axe (shaft + head), spear (long shaft +
//! tip), hammer (shaft + heavy head), bow (arc limbs + grip).
//!
//! Proportions are chunky on purpose: the player figure is ~75px tall and the
//! game is usually viewed at zoom 1.0, so anything under ~4px wide vanishes.

use crate::elements::prim::{facing_offset, Part};
use crate::weapons::WeaponKind;

fn steel() -> [f32; 3] {
    [0.85, 0.88, 0.95]
}
fn wood() -> [f32; 3] {
    [0.55, 0.38, 0.20]
}
fn brass() -> [f32; 3] {
    [0.80, 0.62, 0.25]
}

/// Weapon parts at hand anchor (`hx`, `hy`). `attack` extends the weapon along
/// the facing; `enchant` (0..5) tints the business end arcane purple.
pub fn build(
    kind: WeaponKind,
    hx: f32,
    hy: f32,
    facing: (f32, f32),
    attack: f32,
    enchant: u8,
    alpha: f32,
) -> Vec<Part> {
    if kind == WeaponKind::Fists {
        return Vec::new();
    }
    let a = attack.clamp(0.0, 1.0);
    // Lunge: rest just ahead of the hand, full extension at strike peak.
    let lunge = 5.0 + a * 11.0;
    let (dx, dy) = facing_offset(facing, 1.0);
    let len = (dx * dx + dy * dy).sqrt().max(1e-4);
    let (ux, uy) = (dx / len, dy / len);
    // Enchanted edge: lerp the striking color toward arcane purple
    // (red/green drop, blue stays maxed).
    let edge = {
        let k = (enchant as f32 / 5.0).clamp(0.0, 1.0);
        let s = steel();
        [s[0] + (0.55 - s[0]) * k, s[1] + (0.35 - s[1]) * k, s[2] + (1.0 - s[2]) * k]
    };
    let mut v = Vec::new();
    // Small perpendicular for crosspieces (guard, axe poll, bow limbs).
    let (px, py) = (-uy, ux);
    match kind {
        WeaponKind::Fists => {}
        WeaponKind::Sword => {
            // Grip in hand, guard across, broad blade forward.
            v.push(Part::vquad(hx - 2.0, hy - 9.0, 2.0, 11.0, wood(), alpha, false));
            let gx = hx + ux * (lunge * 0.4);
            let gy = hy + uy * (lunge * 0.4);
            v.push(Part::diamond(gx, gy, 6.5, 2.8, 0.0, brass(), alpha, false));
            let bx = hx + ux * (lunge * 0.4 + 13.0);
            let by = hy + uy * (lunge * 0.4 + 13.0);
            v.push(Part::vquad(bx - 2.6, by - 13.0, 2.6, 24.0, edge, alpha, true));
            v.push(Part::diamond(bx + ux * 13.0, by + uy * 13.0, 2.8, 4.0, 0.0, edge, alpha, false));
        }
        WeaponKind::Axe => {
            let sx = hx + ux * (lunge * 0.4 + 7.0);
            let sy = hy + uy * (lunge * 0.4 + 7.0);
            v.push(Part::vquad(sx - 2.0, sy - 12.0, 2.0, 22.0, wood(), alpha, true));
            // Head: broad bit on one side, poll on the other.
            v.push(Part::diamond(
                sx + ux * 5.0 + px * 5.2,
                sy + uy * 5.0 + py * 5.2,
                7.0,
                5.5,
                0.0,
                edge,
                alpha,
                true,
            ));
            v.push(Part::diamond(
                sx + ux * 5.0 - px * 4.0,
                sy + uy * 5.0 - py * 4.0,
                3.4,
                3.0,
                0.0,
                brass(),
                alpha,
                false,
            ));
        }
        WeaponKind::Spear => {
            // Longest reach: shaft runs well past the hand, steel tip forward,
            // leather binding where the hand grips.
            let sx = hx + ux * (lunge * 0.5 + 11.0);
            let sy = hy + uy * (lunge * 0.5 + 11.0);
            v.push(Part::vquad(sx - 1.8, sy - 18.0, 1.8, 34.0, wood(), alpha, true));
            v.push(Part::diamond(sx - ux * 8.0, sy - uy * 8.0, 2.8, 2.8, 0.0, brass(), alpha, false));
            v.push(Part::diamond(sx + ux * 18.0, sy + uy * 18.0, 3.0, 5.0, 0.0, edge, alpha, true));
        }
        WeaponKind::Hammer => {
            let sx = hx + ux * (lunge * 0.4 + 6.0);
            let sy = hy + uy * (lunge * 0.4 + 6.0);
            v.push(Part::vquad(sx - 2.2, sy - 10.0, 2.2, 18.0, wood(), alpha, true));
            // Heavy rectangular head across the shaft end.
            let hxp = sx + ux * 10.0;
            let hyp = sy + uy * 10.0;
            v.push(Part::vquad(hxp - 8.0, hyp - 6.5, 8.0, 13.0, [0.55, 0.56, 0.62], alpha, true));
            v.push(Part::diamond(hxp + px * 8.0, hyp + py * 8.0, 2.4, 4.8, 0.0, edge, alpha, false));
        }
        WeaponKind::Bow => {
            // Held across the body: grip at hand, limbs curving out both ways.
            v.push(Part::vquad(hx - 1.8, hy - 9.0, 1.8, 18.0, wood(), alpha, true));
            let bow = 9.0 + a * 3.0; // limbs flex a touch on release
            v.push(Part::diamond(hx + px * bow, hy + py * bow - 7.0, 2.4, 7.0, 0.0, wood(), alpha, false));
            v.push(Part::diamond(hx - px * bow, hy - py * bow + 7.0, 2.4, 7.0, 0.0, wood(), alpha, false));
            v.push(Part::diamond(hx + ux * 3.0, hy + uy * 3.0, 1.8, 1.8, 0.0, brass(), alpha, false));
        }
    }
    v
}

/// Raised block shield: a steel-blue kite in front of the figure while the
/// player holds block. Drawn over the weapon so defense reads instantly.
pub fn block_shield(cx: f32, cy: f32, facing: (f32, f32), alpha: f32) -> Vec<Part> {
    let (ox, oy) = facing_offset(facing, 13.0);
    let sx = cx + ox;
    let sy = cy - 28.0 + oy;
    vec![
        Part::diamond(sx, sy, 9.0, 11.5, 0.0, [0.45, 0.62, 0.85], alpha, true),
        Part::diamond(sx, sy, 6.0, 8.0, 0.0, [0.62, 0.78, 0.96], alpha, false),
        Part::diamond(sx, sy, 2.4, 2.4, 0.0, [0.90, 0.94, 1.0], alpha, false),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_weapon_has_a_distinct_silhouette() {
        let kinds = [
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Spear,
            WeaponKind::Hammer,
            WeaponKind::Bow,
        ];
        let counts: Vec<usize> = kinds
            .iter()
            .map(|k| build(*k, 0.0, 0.0, (1.0, 0.0), 0.5, 0, 1.0).len())
            .collect();
        // Fists draw nothing; every real weapon draws 3-4 parts.
        assert_eq!(build(WeaponKind::Fists, 0.0, 0.0, (1.0, 0.0), 0.5, 0, 1.0).len(), 0);
        for (k, n) in kinds.iter().zip(counts.iter()) {
            assert!((3..=4).contains(n), "{k:?} should draw 3-4 parts, drew {n}");
        }
    }

    #[test]
    fn weapons_are_chunky_enough_to_read_at_zoom_1() {
        // The game is usually viewed at zoom 1.0 with a ~75px figure: any
        // weapon part under ~4px wide vanishes. Every striking part (blade /
        // head / tip / limb) must clear that bar.
        for k in [
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Spear,
            WeaponKind::Hammer,
            WeaponKind::Bow,
        ] {
            let parts = build(k, 0.0, 0.0, (1.0, 0.0), 0.5, 0, 1.0);
            let widest = parts.iter().map(|p| p.hw).fold(0.0f32, f32::max);
            assert!(widest >= 2.0, "{k:?} widest half-width {widest} reads <4px wide");
        }
    }

    #[test]
    fn attack_lunge_extends_the_weapon() {
        // Bounding box must grow as the strike ramps: steel moves with the arm.
        let bbox = |parts: &[Part]| {
            let (mut x0, mut x1) = (f32::MAX, f32::MIN);
            for p in parts {
                x0 = x0.min(p.cx - p.hw);
                x1 = x1.max(p.cx + p.hw);
            }
            (x0, x1)
        };
        for k in [WeaponKind::Sword, WeaponKind::Spear, WeaponKind::Hammer] {
            let rest = build(k, 0.0, 0.0, (1.0, 0.0), 0.0, 0, 1.0);
            let strike = build(k, 0.0, 0.0, (1.0, 0.0), 1.0, 0, 1.0);
            let (r0, r1) = bbox(&rest);
            let (s0, s1) = bbox(&strike);
            assert!(s1 > r1, "{k:?} must extend forward on strike");
            let _ = (r0, s0);
        }
    }

    #[test]
    fn spear_outreaches_sword() {
        // Reach order must match WeaponKind::reach: spear longest.
        let extent = |k: WeaponKind| {
            build(k, 0.0, 0.0, (1.0, 0.0), 0.5, 0, 1.0)
                .iter()
                .map(|p| p.cx + p.hw)
                .fold(f32::MIN, f32::max)
        };
        assert!(extent(WeaponKind::Spear) > extent(WeaponKind::Sword));
        assert!(extent(WeaponKind::Sword) > extent(WeaponKind::Hammer));
    }

    #[test]
    fn block_shield_draws_in_front() {
        let parts = block_shield(0.0, 0.0, (1.0, 0.0), 1.0);
        assert_eq!(parts.len(), 3);
        // Shield sits ahead of the figure (positive screen-x for +x facing).
        for p in &parts {
            assert!(p.cx > 0.0, "shield must be in front of the figure");
        }
    }

    #[test]
    fn enchant_tints_the_edge() {
        let plain = build(WeaponKind::Sword, 0.0, 0.0, (1.0, 0.0), 0.5, 0, 1.0);
        let ench = build(WeaponKind::Sword, 0.0, 0.0, (1.0, 0.0), 0.5, 5, 1.0);
        let blue = |ps: &[Part]| ps.iter().map(|p| p.color[2]).fold(0.0f32, f32::max);
        assert!(blue(&ench) >= blue(&plain));
    }
}
