//! Rock pile: a few gray stones stacked loosely.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let r = color;
    vec![
        Part::diamond(cx - 3.0, cy - 1.0, 8.0, 6.0, 0.0, shade(r, 0.7), alpha, true),
        Part::diamond(cx + 4.0, cy - 3.0, 6.0, 5.0, 0.0, shade(r, 0.9), alpha, true),
        Part::diamond(cx, cy - 6.0, 6.0, 4.0, 0.0, shade(r, 1.1), alpha, true),
    ]
}
