//! Reed: a cluster of tall thin marsh stalks.

use crate::elements::prim::{sway, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let green = color;
    let s = sway(cx, cy, anim_time, 1.6);
    vec![
        Part::vquad(cx - 5.0 + s, cy - 20.0, 1.5, 20.0, green, alpha, true),
        Part::vquad(cx - 1.0 + s, cy - 24.0, 1.5, 24.0, green, alpha, true),
        Part::vquad(cx + 3.0 + s, cy - 18.0, 1.5, 18.0, green, alpha, true),
        Part::vquad(cx + 7.0 + s, cy - 22.0, 1.5, 22.0, green, alpha, true),
    ]
}
