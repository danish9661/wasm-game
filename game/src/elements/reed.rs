//! Reed: a cluster of tall thin marsh stalks with cattail seed-heads.
//! Stalks alternate shade steps so the cluster has volume instead of reading
//! as flat parallel bars.

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
    let tip = [0.45, 0.28, 0.12];
    let s = sway(cx, cy, anim_time, 1.6);
    vec![
        Part::vquad(cx - 5.0 + s, cy - 20.0, 1.5, 20.0, shade(green, 0.85), alpha, true),
        Part::vquad(cx - 1.0 + s, cy - 24.0, 1.5, 24.0, green, alpha, true),
        Part::vquad(cx + 3.0 + s, cy - 18.0, 1.5, 18.0, shade(green, 1.1), alpha, true),
        Part::vquad(cx + 7.0 + s, cy - 22.0, 1.5, 22.0, shade(green, 0.95), alpha, true),
        // Cattail seed-heads crowning the two tallest stalks.
        Part::vquad(cx - 1.0 + s, cy - 28.0, 2.0, 5.0, tip, alpha, true),
        Part::vquad(cx + 7.0 + s, cy - 26.0, 2.0, 4.0, tip, alpha, true),
    ]
}
