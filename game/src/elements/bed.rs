//! Bed: a wooden frame with a pale mattress and a pillow.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    _color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let frame = [0.50, 0.34, 0.18];
    let sheet = [0.85, 0.85, 0.92];
    vec![
        Part::vquad(cx, cy - 8.0, 18.0, 8.0, frame, alpha, true),
        Part::vquad(cx - 2.0, cy - 11.0, 16.0, 5.0, sheet, alpha, true),
        Part::diamond(cx - 12.0, cy - 11.0, 5.0, 4.0, 0.0, shade(sheet, 1.0), alpha, true),
    ]
}
