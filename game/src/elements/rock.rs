//! Rock: faceted gray stone — dark base + lighter top cap.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let rock = color; // gray from ResourceKind::Rock
    vec![
        Part::diamond(cx, cy - 2.0, 14.0, 11.0, 0.0, shade(rock, 0.7), alpha, true),
        Part::diamond(cx, cy - 7.0, 9.0, 7.0, 0.0, shade(rock, 1.15), alpha, true),
    ]
}
