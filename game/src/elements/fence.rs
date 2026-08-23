//! Fence: two posts with two horizontal rails between them.

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
        Part::vquad(cx - 12.0, cy - 16.0, 3.0, 16.0, wood, alpha, true),
        Part::vquad(cx + 12.0, cy - 16.0, 3.0, 16.0, wood, alpha, true),
        Part::vquad(cx - 15.0, cy - 12.0, 15.0, 3.0, shade(wood, 0.9), alpha, true),
        Part::vquad(cx - 15.0, cy - 6.0, 15.0, 3.0, shade(wood, 0.9), alpha, true),
    ]
}
