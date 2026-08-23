//! Crate: a wooden box with plank seams.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let wood = color;
    vec![
        Part::vquad(cx - 10.0, cy - 12.0, 10.0, 12.0, wood, alpha, true),
        Part::vquad(cx - 10.0, cy - 12.0, 10.0, 1.5, shade(wood, 0.7), alpha, true),
        Part::vquad(cx - 10.0, cy - 2.0, 10.0, 1.5, shade(wood, 0.7), alpha, true),
        Part::vquad(cx - 1.0, cy - 12.0, 1.0, 12.0, shade(wood, 0.7), alpha, true),
    ]
}
