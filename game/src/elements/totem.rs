//! Totem: a tall carved post with three stacked, striped faces.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let t = color;
    vec![
        Part::vquad(cx, cy - 26.0, 6.0, 26.0, shade(t, 0.9), alpha, true),
        Part::diamond(cx, cy - 26.0, 8.0, 7.0, 0.0, t, alpha, true),
        Part::diamond(cx, cy - 16.0, 7.0, 6.0, 0.0, shade(t, 1.15), alpha, true),
        Part::diamond(cx, cy - 7.0, 6.0, 5.0, 0.0, shade(t, 0.8), alpha, true),
    ]
}
