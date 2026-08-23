//! Tree: brown trunk + layered green canopy.

use crate::elements::prim::{shade, sway, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let trunk = [0.35, 0.22, 0.10];
    let canopy = color; // green from ResourceKind::Tree
    let s = sway(cx, cy, anim_time, 2.5);
    vec![
        Part::vquad(cx, cy - 16.0, 4.0, 16.0, trunk, alpha, true),
        Part::diamond(cx + s, cy - 20.0, 18.0, 20.0, 0.0, shade(canopy, 0.8), alpha, true),
        Part::diamond(cx + s, cy - 34.0, 14.0, 16.0, 0.0, canopy, alpha, true),
    ]
}
