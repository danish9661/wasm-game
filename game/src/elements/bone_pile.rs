//! BonePile: a low mound of bleached bones.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let bone = color;
    vec![
        Part::diamond(cx, cy - 3.0, 12.0, 6.0, 0.0, shade(bone, 0.9), alpha, true),
        Part::vquad(cx - 6.0, cy - 9.0, 1.5, 9.0, bone, alpha, true),
        Part::vquad(cx + 5.0, cy - 7.0, 1.5, 7.0, bone, alpha, true),
        Part::diamond(cx - 3.0, cy - 9.0, 3.0, 2.0, 0.0, bone, alpha, true),
    ]
}
