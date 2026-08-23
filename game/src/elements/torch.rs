//! Torch: a wooden post topped with a flickering flame.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let post = [0.45, 0.30, 0.16];
    let seed = anim_seed(cx, cy);
    let flick = 0.82 + 0.18 * (anim_time * 13.0 + seed).sin();
    let flame = shade(color, flick);
    vec![
        Part::vquad(cx, cy - 18.0, 2.5, 18.0, post, alpha, true),
        Part::diamond(cx, cy - 22.0, 4.0, 7.0, 0.0, flame, alpha, true),
        Part::diamond(cx, cy - 25.0, 2.5, 4.0, 0.0, shade([1.0, 0.85, 0.40], flick), alpha, true),
    ]
}
