//! Sign: a wooden post with a board and two bolt dots.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let post = [0.50, 0.34, 0.18];
    let board = color;
    vec![
        Part::vquad(cx, cy - 14.0, 2.0, 14.0, post, alpha, true),
        Part::diamond(cx, cy - 18.0, 12.0, 7.0, 0.0, board, alpha, true),
        Part::diamond(cx - 4.0, cy - 18.0, 2.0, 2.0, 0.0, shade(board, 0.7), alpha, true),
        Part::diamond(cx + 4.0, cy - 18.0, 2.0, 2.0, 0.0, shade(board, 0.7), alpha, true),
    ]
}
