//! Stormcaller: a flying storm-mage boss that drifts over walls. A broad robed
//! figure with a lightning-charged staff, crackling wisps, a storm halo and
//! glowing storm eyes. Boss-class mass: wider robe and taller presence than a
//! common cultist, with the staff as the boss tell.
//!
//! The halo/wisps keep a soft (semi-transparent, unskirted) glow as the
//! storm-faction trait; the robe itself stays hard-edged like every other rig.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    walk: f32,
    anim_time: f32,
) -> Vec<Part> {
    let robe = color;
    let dark = shade(robe, 0.7);
    let lightning = [0.70, 0.85, 1.0];
    let orb = [0.85, 0.95, 1.0];
    let eye = [0.60, 0.80, 1.0];
    let wood = [0.30, 0.22, 0.15];
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    // Floating bob, deeper while drifting; lightning crackles faster aloft.
    let bob = (anim_time * (2.0 + 2.0 * w) + seed).sin() * (1.5 + 3.0 * w);
    // Lightning flicker
    let flicker = (anim_time * (6.0 + 8.0 * w) + seed).sin().max(0.0);

    let mut parts = vec![
        // Broad tapered robe body (wide at bottom, narrow at shoulders)
        Part::diamond(cx, cy - 3.0 + bob, 16.0, 12.0, 0.0, dark, alpha, true),
        Part::diamond(cx, cy - 15.0 + bob, 13.0, 12.0, 0.0, robe, alpha, true),
        Part::diamond(cx, cy - 27.0 + bob, 9.0, 7.5, 0.0, shade(robe, 1.1), alpha, true),
        // Hooded head
        Part::diamond(cx, cy - 33.0 + bob, 9.0, 7.5, 0.0, robe, alpha, true),
        // Hood point
        Part::diamond(cx, cy - 40.0 + bob, 6.0, 4.5, 0.0, dark, alpha, true),
        // Glowing eyes
        Part::diamond(cx - 3.75, cy - 33.0 + bob, 2.2, 2.2, 0.0, eye, alpha, true),
        Part::diamond(cx + 3.75, cy - 33.0 + bob, 2.2, 2.2, 0.0, eye, alpha, true),
        // Outstretched arms (right hand grips the staff)
        Part::vquad(cx - 15.0, cy - 27.0 + bob, 3.0, 15.0, robe, alpha, true),
        Part::vquad(cx + 14.0, cy - 27.0 + bob, 3.0, 15.0, robe, alpha, true),
        Part::diamond(cx + 17.5, cy - 16.0 + bob, 2.8, 2.8, 0.0, shade(robe, 1.1), alpha, true),
        // Storm staff: thick dark shaft beside the figure, glowing halo + orb
        Part::vquad(cx + 19.0, cy - 36.0 + bob, 3.0, 44.0, wood, alpha, true),
        Part::diamond(cx + 19.0, cy - 38.0 + bob, 7.0 + flicker * 1.5, 8.0 + flicker * 1.5, 0.0, lightning, alpha * 0.35, false),
        Part::diamond(cx + 19.0, cy - 38.0 + bob, 4.5 + flicker * 1.5, 5.5 + flicker * 1.5, 0.0, orb, alpha, true),
    ];

    // Crackling lightning wisps (appear and disappear with flicker)
    if flicker > 0.2 {
        let a = alpha * flicker;
        parts.push(Part::diamond(cx - 19.0, cy - 15.0 + bob, 4.5, 3.0, 0.0, lightning, a, false));
        parts.push(Part::diamond(cx + 13.0, cy - 15.0 + bob, 3.0, 2.5, 0.0, lightning, a, false));
    }
    if flicker > 0.5 {
        let a = alpha * (flicker - 0.5) * 2.0;
        parts.push(Part::diamond(cx - 15.0, cy - 21.0 + bob, 3.0, 2.0, 0.0, lightning, a, false));
        parts.push(Part::diamond(cx + 15.0, cy - 21.0 + bob, 3.0, 2.0, 0.0, lightning, a, false));
    }
    // Storm cloud halo above head
    parts.push(Part::diamond(cx, cy - 43.0 + bob, 12.0, 4.5, 0.0, shade(lightning, 0.7), alpha * 0.5, false));

    parts
}
