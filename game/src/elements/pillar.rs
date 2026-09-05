//! Pillar: a weathered stone column with a cap and base.

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
        Part::vquad(cx, cy - 30.0, 9.0, 30.0, stone, alpha, true),
        // Lit face so the shaft separates from dark ground at a glance.
        Part::vquad(cx - 2.5, cy - 30.0, 2.5, 30.0, shade(stone, 1.15), alpha, false),
        Part::vquad(cx, cy - 32.0, 10.0, 3.0, shade(stone, 0.8), alpha, true),
        Part::vquad(cx, cy - 2.0, 10.0, 3.0, shade(stone, 0.8), alpha, true),
    ]
}
