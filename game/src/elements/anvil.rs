//! Anvil: a dark metal block with a flared top and a small foot.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let metal = color;
    vec![
        Part::vquad(cx - 3.0, cy - 6.0, 4.0, 6.0, shade(metal, 0.8), alpha, true),
        Part::vquad(cx, cy - 10.0, 12.0, 10.0, metal, alpha, true),
        Part::diamond(cx, cy - 12.0, 14.0, 5.0, 0.0, shade(metal, 1.15), alpha, true),
    ]
}
