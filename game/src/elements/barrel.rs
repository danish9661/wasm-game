//! Barrel: a wooden cask with two dark metal hoops.

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
        Part::vquad(cx, cy - 16.0, 8.0, 16.0, wood, alpha, true),
        Part::vquad(cx - 8.0, cy - 12.0, 8.0, 2.5, shade(wood, 0.7), alpha, true),
        Part::vquad(cx - 8.0, cy - 4.0, 8.0, 2.5, shade(wood, 0.7), alpha, true),
    ]
}
