//! Spider: a round body, splayed legs, a head and two glowing eyes.

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
    let eye = [0.90, 0.90, 0.30];
    let w = walk.clamp(0.0, 1.0);
    let seed = anim_seed(cx, cy);
    // Legs shuffle faster and wider while hunting; idle is near-still.
    let sh = (anim_time * (3.0 + 7.0 * w) + seed).sin() * (0.6 + 2.6 * w);
    let bob = (anim_time * (6.0 + 8.0 * w) + seed).sin().abs() * 1.2 * w;
    vec![
        Part::diamond(cx - 8.0 - sh, cy - 3.0, 8.0, 2.0, 0.0, shade(body, 0.8), alpha, true),
        Part::diamond(cx + 8.0 + sh, cy - 3.0, 8.0, 2.0, 0.0, shade(body, 0.8), alpha, true),
        Part::diamond(cx, cy - 8.0 - bob, 11.0, 9.0, 0.0, body, alpha, true),
        Part::diamond(cx, cy - 16.0 - bob, 6.0, 5.0, 0.0, shade(body, 1.1), alpha, true),
        Part::diamond(cx - 3.0, cy - 20.0 - bob, 2.0, 2.0, 0.0, eye, alpha, true),
        Part::diamond(cx + 3.0, cy - 20.0 - bob, 2.0, 2.0, 0.0, eye, alpha, true),
    ]
}
