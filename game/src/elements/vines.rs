//! Vines: trailing green strands hanging from above.

use crate::elements::prim::{shade, sway, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let green = color;
    let s = sway(cx, cy, anim_time, 1.4);
    vec![
        Part::vquad(cx - 6.0 + s, cy - 24.0, 2.0, 24.0, green, alpha, true),
        Part::vquad(cx + 2.0 + s, cy - 20.0, 2.0, 20.0, green, alpha, true),
        Part::vquad(cx + 7.0 + s, cy - 26.0, 2.0, 26.0, green, alpha, true),
        Part::diamond(cx - 6.0 + s, cy - 2.0, 3.0, 2.0, 0.0, shade(green, 0.8), alpha, true),
        Part::diamond(cx + 7.0 + s, cy - 2.0, 3.0, 2.0, 0.0, shade(green, 0.8), alpha, true),
    ]
}
