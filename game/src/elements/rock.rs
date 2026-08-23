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
        // dark base boulder
        Part::diamond(cx, cy - 1.0, 15.0, 11.0, 0.0, shade(rock, 0.6), alpha, true),
        // mid facet
        Part::diamond(cx, cy - 5.0, 11.0, 9.0, 0.0, shade(rock, 0.85), alpha, true),
        // lit top cap
        Part::diamond(cx, cy - 9.0, 7.0, 6.0, 0.0, shade(rock, 1.2), alpha, true),
        // small highlight glint
        Part::diamond(cx + 3.0, cy - 11.0, 3.0, 3.0, 0.0, shade(rock, 1.45), alpha, true),
    ]
}
