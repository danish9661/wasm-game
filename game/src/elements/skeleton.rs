//! Skeleton: a bony humanoid — pale legs, a ribbed torso, and a skull.

use crate::elements::prim::{facing_offset, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let bone = color;
    let (hx, hy) = facing_offset(facing, 3.0);
    vec![
        Part::vquad(cx - 3.0, cy - 14.0, 3.0, 14.0, shade(bone, 0.9), alpha, true),
        Part::vquad(cx + 3.0, cy - 14.0, 3.0, 14.0, shade(bone, 0.9), alpha, true),
        Part::vquad(cx, cy - 30.0, 7.0, 16.0, bone, alpha, true),
        Part::vquad(cx - 4.0, cy - 32.0, 1.5, 12.0, shade(bone, 0.7), alpha, true),
        Part::vquad(cx + 4.0, cy - 32.0, 1.5, 12.0, shade(bone, 0.7), alpha, true),
        Part::diamond(cx + hx, cy - 38.0 + hy, 6.0, 7.0, 0.0, bone, alpha, true),
    ]
}
