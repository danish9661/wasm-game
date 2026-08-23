//! Well: a stone ring with a dark water pool in the middle.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let stone = color;
    let water = [0.25, 0.45, 0.70];
    vec![
        Part::vquad(cx, cy - 14.0, 16.0, 14.0, shade(stone, 0.8), alpha, true),
        Part::diamond(cx, cy - 16.0, 16.0, 9.0, 0.0, shade(stone, 1.1), alpha, true),
        Part::diamond(cx, cy - 15.0, 9.0, 5.0, 0.0, water, alpha, true),
    ]
}
