//! Rubble: a scatter of broken stones.

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
    vec![
        Part::diamond(cx - 5.0, cy - 3.0, 7.0, 4.0, 0.0, stone, alpha, true),
        Part::diamond(cx + 5.0, cy - 2.0, 6.0, 4.0, 0.0, shade(stone, 0.85), alpha, true),
        Part::diamond(cx + 1.0, cy - 5.0, 5.0, 3.0, 0.0, shade(stone, 0.7), alpha, true),
    ]
}
