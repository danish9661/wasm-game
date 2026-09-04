//! Wolf: a fast, low quadruped — four legs, lean body, pointed ears, and
//! glowing eyes. Pack hunter with a running gait.

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
    let body = color;
    let dark = shade(body, 0.75);
    let light = shade(body, 1.2);
    let eye = [0.85, 0.80, 0.30];
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    // Leg swing for a running gait: quicker and wider at full stride.
    let phase = anim_time * (5.0 + 5.0 * w) + seed;
    let leg1 = (phase).sin() * (1.2 + 2.6 * w);
    let leg2 = (phase + 3.14).sin() * (1.2 + 2.6 * w);
    // Body bobs over the planted legs while running.
    let wbob = (phase * 2.0).sin().abs() * 1.4 * w;

    vec![
        // Lean body
        Part::vquad(cx - 7.0, cy - 11.0 - wbob, 14.0, 9.0, body, alpha, true),
        // Head (forward, pointed snout)
        Part::vquad(cx + 6.0, cy - 14.0 - wbob, 7.0, 7.0, body, alpha, true),
        Part::diamond(cx + 10.0, cy - 13.0 - wbob, 3.0, 2.5, 0.0, light, alpha, true),
        // Pointed ears
        Part::diamond(cx + 4.0, cy - 20.0 - wbob, 2.0, 3.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 8.0, cy - 20.0 - wbob, 2.0, 3.0, 0.0, dark, alpha, true),
        // Eyes
        Part::diamond(cx + 5.0, cy - 14.0 - wbob, 1.2, 1.2, 0.0, eye, alpha, true),
        Part::diamond(cx + 9.0, cy - 14.0 - wbob, 1.2, 1.2, 0.0, eye, alpha, true),
        // Four legs with running animation
        Part::vquad(cx - 5.0 + leg1, cy - 3.0, 2.5, 6.0, dark, alpha, false),
        Part::vquad(cx - 1.0 + leg2, cy - 3.0, 2.5, 6.0, dark, alpha, false),
        Part::vquad(cx + 3.0 + leg2, cy - 3.0, 2.5, 6.0, dark, alpha, false),
        Part::vquad(cx + 7.0 + leg1, cy - 3.0, 2.5, 6.0, dark, alpha, false),
        // Tail (slightly raised, curved)
        Part::diamond(cx - 10.0, cy - 12.0 - wbob, 2.0, 4.0, 0.0, dark, alpha, true),
    ]
}
