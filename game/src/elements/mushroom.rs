//! Mushroom: a cream stem capped by a red, spotted cap.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let stem = [0.92, 0.88, 0.80];
    vec![
        Part::vquad(cx, cy - 6.0, 3.0, 6.0, stem, alpha, true),
        Part::diamond(cx, cy - 8.0, 9.0, 7.0, 0.0, color, alpha, true),
        Part::diamond(cx - 3.0, cy - 10.0, 3.0, 2.0, 0.0, shade(color, 1.15), alpha, true),
        Part::diamond(cx + 3.0, cy - 9.0, 2.0, 2.0, 0.0, shade(color, 1.15), alpha, true),
    ]
}
