//! Flower: a thin green stem with three colored petals and a yellow center.

use crate::elements::prim::{shade, sway, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let stem = [0.25, 0.50, 0.20];
    let center = [0.98, 0.85, 0.25];
    let s = sway(cx, cy, anim_time, 1.2);
    vec![
        Part::vquad(cx, cy - 7.0, 1.5, 7.0, stem, alpha, true),
        Part::diamond(cx - 4.0 + s, cy - 10.0, 4.0, 3.0, 0.0, color, alpha, true),
        Part::diamond(cx + 4.0 + s, cy - 10.0, 4.0, 3.0, 0.0, color, alpha, true),
        Part::diamond(cx + s, cy - 13.0, 4.0, 3.0, 0.0, shade(color, 0.9), alpha, true),
        Part::diamond(cx + s, cy - 10.0, 3.0, 3.0, 0.0, center, alpha, true),
    ]
}
