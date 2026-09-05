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
        // Wide arching side fronds + a tall center spike: spikier and taller
        // than the round bush blob so the two never confuse at a glance.
        Part::diamond(cx - 9.0 + s, cy - 7.0, 7.0, 5.0, 0.0, shade(f, 0.85), alpha, true),
        Part::diamond(cx + 9.0 + s, cy - 7.0, 7.0, 5.0, 0.0, shade(f, 0.85), alpha, true),
        Part::diamond(cx + s, cy - 14.0, 8.0, 9.0, 0.0, f, alpha, true),
        Part::diamond(cx + s, cy - 4.0, 5.0, 4.0, 0.0, shade(f, 1.1), alpha, true),
        Part::vquad(cx + s, cy - 22.0, 1.5, 8.0, shade(f, 1.05), alpha, true),
    ]
}
