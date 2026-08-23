//! Wall: stone block — dark body + lighter cap.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let stone = color; // tan-gray from StructureKind::Wall
    vec![
        Part::vquad(cx, cy - 24.0, 20.0, 24.0, shade(stone, 0.7), alpha, true),
        Part::diamond(cx, cy - 26.0, 20.0, 12.0, 0.0, shade(stone, 1.05), alpha, true),
    ]
}
