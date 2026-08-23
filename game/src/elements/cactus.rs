//! Cactus: a green column with two raised arms.

use crate::elements::prim::Part;

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let green = color;
    vec![
        Part::vquad(cx - 3.0, cy - 22.0, 3.0, 22.0, green, alpha, true),
        Part::vquad(cx - 9.0, cy - 14.0, 3.0, 4.0, green, alpha, true),
        Part::vquad(cx - 9.0, cy - 20.0, 3.0, 8.0, green, alpha, true),
        Part::vquad(cx + 6.0, cy - 10.0, 3.0, 4.0, green, alpha, true),
        Part::vquad(cx + 6.0, cy - 16.0, 3.0, 8.0, green, alpha, true),
    ]
}
