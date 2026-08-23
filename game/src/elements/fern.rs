//! Fern: arching fronds of deep green.

use crate::elements::prim::{shade, sway, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let f = color;
    let s = sway(cx, cy, anim_time, 1.8);
    vec![
        Part::diamond(cx - 6.0 + s, cy - 7.0, 6.0, 5.0, 0.0, shade(f, 0.85), alpha, true),
        Part::diamond(cx + 6.0 + s, cy - 7.0, 6.0, 5.0, 0.0, shade(f, 0.85), alpha, true),
        Part::diamond(cx + s, cy - 12.0, 8.0, 8.0, 0.0, f, alpha, true),
        Part::diamond(cx + s, cy - 4.0, 5.0, 4.0, 0.0, shade(f, 1.1), alpha, true),
    ]
}
