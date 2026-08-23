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
        // trunk (dark base, lit upper)
        Part::vquad(cx, cy - 4.0, 4.0, 4.0, shade(trunk, 0.7), alpha, true),
        Part::vquad(cx, cy - 18.0, 3.5, 14.0, trunk, alpha, true),
        // three canopy tiers for a rounded, full crown
        Part::diamond(cx + s, cy - 16.0, 19.0, 18.0, 0.0, shade(canopy, 0.7), alpha, true),
        Part::diamond(cx + s, cy - 30.0, 15.0, 16.0, 0.0, shade(canopy, 0.95), alpha, true),
        Part::diamond(cx + s, cy - 42.0, 10.0, 12.0, 0.0, canopy, alpha, true),
        // sunlit highlight on the upper-left of the crown
        Part::diamond(cx - 5.0 + s, cy - 44.0, 5.0, 7.0, 0.0, shade(canopy, 1.25), alpha, true),
    ]
}
