//! BonePile: a low mound of bleached bones, grounded by a contact shadow so
//! it reads on light ground instead of floating.

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
        // soft contact shadow grounds the pile
        Part::diamond(cx, cy + 1.0, 14.0, 5.0, 0.0, [0.12, 0.11, 0.12], alpha * 0.7, false),
        // dark base rim so pale bone separates from bright tiles
        Part::diamond(cx, cy - 2.0, 12.5, 5.5, 0.0, shade(bone, 0.55), alpha, true),
        Part::diamond(cx, cy - 3.0, 12.0, 6.0, 0.0, shade(bone, 0.9), alpha, true),
        Part::vquad(cx - 6.0, cy - 9.0, 1.5, 9.0, bone, alpha, true),
        Part::vquad(cx + 5.0, cy - 7.0, 1.5, 7.0, bone, alpha, true),
        Part::diamond(cx - 3.0, cy - 9.0, 3.0, 2.0, 0.0, bone, alpha, true),
        // rim highlight so bleached bone reads on bright tiles
        Part::diamond(cx + 4.0, cy - 6.0, 2.0, 1.5, 0.0, shade(bone, 1.3), alpha, false),
    ]
}
