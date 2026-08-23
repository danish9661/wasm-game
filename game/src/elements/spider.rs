//! Spider: a round body, splayed legs, a head and two glowing eyes.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let body = color;
    let eye = [0.90, 0.90, 0.30];
    vec![
        Part::diamond(cx - 8.0, cy - 3.0, 8.0, 2.0, 0.0, shade(body, 0.8), alpha, true),
        Part::diamond(cx + 8.0, cy - 3.0, 8.0, 2.0, 0.0, shade(body, 0.8), alpha, true),
        Part::diamond(cx, cy - 8.0, 11.0, 9.0, 0.0, body, alpha, true),
        Part::diamond(cx, cy - 16.0, 6.0, 5.0, 0.0, shade(body, 1.1), alpha, true),
        Part::diamond(cx - 3.0, cy - 20.0, 2.0, 2.0, 0.0, eye, alpha, true),
        Part::diamond(cx + 3.0, cy - 20.0, 2.0, 2.0, 0.0, eye, alpha, true),
    ]
}
