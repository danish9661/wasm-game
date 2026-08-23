//! Ore: a rocky cluster studded with bright veins (drops gems).

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let rock = color;
    let vein = [0.95, 0.85, 0.40];
    vec![
        Part::diamond(cx - 5.0, cy - 3.0, 9.0, 6.0, 0.0, shade(rock, 0.9), alpha, true),
        Part::diamond(cx + 5.0, cy - 2.0, 8.0, 6.0, 0.0, rock, alpha, true),
        Part::diamond(cx, cy - 7.0, 10.0, 7.0, 0.0, shade(rock, 1.05), alpha, true),
        Part::diamond(cx - 3.0, cy - 5.0, 2.0, 2.0, 0.0, vein, alpha, true),
        Part::diamond(cx + 4.0, cy - 3.0, 2.0, 2.0, 0.0, vein, alpha, true),
    ]
}
