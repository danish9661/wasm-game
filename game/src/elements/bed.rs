//! Bed: a wooden frame with a tucked blanket, folded sheet and a pillow.
//! The blanket carries the room's accent so beds read as furniture, not
//! white slabs, at interior scale.

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
    let blanket = [0.62, 0.24, 0.18];
    vec![
        // Frame + headboard.
        Part::vquad(cx, cy - 8.0, 18.0, 8.0, frame, alpha, true),
        Part::vquad(cx - 17.0, cy - 14.0, 1.8, 12.0, shade(frame, 0.7), alpha, true),
        // Mattress sheet.
        Part::vquad(cx - 2.0, cy - 11.0, 16.0, 5.0, sheet, alpha, true),
        // Folded blanket across the foot half with a tucked shadow line.
        Part::vquad(cx + 5.0, cy - 11.0, 9.0, 5.0, blanket, alpha, true),
        Part::vquad(cx + 5.0, cy - 7.5, 9.0, 1.2, shade(blanket, 0.6), alpha * 0.9, false),
        // Pillow plumped at the head.
        Part::diamond(cx - 12.0, cy - 11.0, 5.0, 4.0, 0.0, shade(sheet, 1.0), alpha, true),
        Part::diamond(cx - 12.0, cy - 12.0, 3.0, 2.2, 0.0, [1.0, 1.0, 1.0], alpha * 0.7, false),
    ]
}
