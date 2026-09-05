//! Lilypad: a flat pad resting on water, with a small bloom. Bobs gently.

use crate::elements::prim::{anim_seed, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let pad = color;
    let seed = anim_seed(cx, cy);
    let bob = (anim_time * 1.8 + seed).sin() * 1.2;
    vec![
        Part::diamond(cx, cy - 2.0 + bob, 14.0, 6.0, 0.0, pad, alpha, true),
        Part::diamond(cx + 4.0, cy - 5.0 + bob, 3.0, 3.0, 0.0, [0.90, 0.70, 0.85], alpha, true),
    ]
}
