//! Grass tuft: a few thin blades of light green.

use crate::elements::prim::{shade, sway, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let blade = color;
    let s = sway(cx, cy, anim_time, 2.0);
    vec![
        Part::diamond(cx - 4.0 + s, cy - 6.0, 3.0, 8.0, 0.0, shade(blade, 0.9), alpha, true),
        Part::diamond(cx + s, cy - 9.0, 3.0, 11.0, 0.0, blade, alpha, true),
        Part::diamond(cx + 4.0 + s, cy - 6.0, 3.0, 8.0, 0.0, shade(blade, 0.8), alpha, true),
    ]
}
