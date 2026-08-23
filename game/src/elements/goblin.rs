//! Goblin: a squat green humanoid with big pointed ears.

use crate::elements::prim::{facing_offset, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let skin = color;
    let loin = [0.35, 0.28, 0.18];
    let (hx, hy) = facing_offset(facing, 3.0);
    vec![
        Part::vquad(cx - 3.0, cy - 14.0, 3.0, 14.0, shade(skin, 0.8), alpha, true),
        Part::vquad(cx + 3.0, cy - 14.0, 3.0, 14.0, shade(skin, 0.8), alpha, true),
        Part::vquad(cx, cy - 30.0, 7.0, 16.0, loin, alpha, true),
        Part::diamond(cx + hx, cy - 38.0 + hy, 6.0, 8.0, 0.0, skin, alpha, true),
        Part::diamond(cx + hx - 6.0, cy - 40.0 + hy, 4.0, 2.0, 0.0, shade(skin, 1.1), alpha, true),
        Part::diamond(cx + hx + 6.0, cy - 40.0 + hy, 4.0, 2.0, 0.0, shade(skin, 1.1), alpha, true),
    ]
}
