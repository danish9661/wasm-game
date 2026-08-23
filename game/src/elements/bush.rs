//! Bush: small leafy clumps.

use crate::elements::prim::{shade, sway, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let bush = color; // green from ResourceKind::Bush
    let s = sway(cx, cy, anim_time, 1.5);
    vec![
        Part::diamond(cx - 5.0 + s, cy - 2.0, 8.0, 7.0, 0.0, shade(bush, 0.85), alpha, true),
        Part::diamond(cx + 5.0 + s, cy - 2.0, 8.0, 7.0, 0.0, shade(bush, 0.85), alpha, true),
        Part::diamond(cx + s, cy - 7.0, 9.0, 8.0, 0.0, bush, alpha, true),
    ]
}
