//! Cactus: a green column with two raised arms. Sways faintly in desert wind.

use crate::elements::prim::{anim_seed, sway, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let green = color;
    let s = sway(cx, cy, anim_time, 1.0);
    vec![
        Part::vquad(cx + s * 0.4, cy - 22.0, 3.0, 22.0, green, alpha, true),
        Part::vquad(cx - 9.0 + s, cy - 14.0, 3.0, 4.0, green, alpha, true),
        Part::vquad(cx - 9.0 + s, cy - 20.0, 3.0, 8.0, green, alpha, true),
        Part::vquad(cx + 6.0 + s, cy - 10.0, 3.0, 4.0, green, alpha, true),
        Part::vquad(cx + 6.0 + s, cy - 16.0, 3.0, 8.0, green, alpha, true),
    ]
}
