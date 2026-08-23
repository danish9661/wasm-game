//! Brazier: a metal bowl on legs wreathed in flame. Emits light.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let metal = [0.35, 0.33, 0.36];
    let bowl = [0.45, 0.40, 0.38];
    let seed = anim_seed(cx, cy);
    let flick = 0.82 + 0.18 * (anim_time * 11.0 + seed).sin();
    let flame = shade(color, flick);
    vec![
        Part::vquad(cx - 4.0, cy - 10.0, 2.0, 10.0, metal, alpha, true),
        Part::vquad(cx + 4.0, cy - 10.0, 2.0, 10.0, metal, alpha, true),
        Part::diamond(cx, cy - 12.0, 12.0, 7.0, 0.0, bowl, alpha, true),
        Part::diamond(cx, cy - 18.0, 6.0, 8.0, 0.0, flame, alpha, true),
        Part::diamond(cx, cy - 22.0, 3.0, 5.0, 0.0, shade(flame, 1.3), alpha, true),
    ]
}
