//! Statue: a pale stone figure on a plinth with a calm head.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let s = color;
    vec![
        Part::vquad(cx, cy - 26.0, 9.0, 26.0, shade(s, 0.85), alpha, true),
        Part::diamond(cx, cy - 30.0, 7.0, 8.0, 0.0, s, alpha, true),
        Part::diamond(cx, cy - 40.0, 6.0, 6.0, 0.0, shade(s, 1.05), alpha, true),
    ]
}
