//! Sign: a wooden post with a board — the board wobbles slightly in wind.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let post = [0.50, 0.34, 0.18];
    let board = color;
    let seed = anim_seed(cx, cy);
    let wobble = (anim_time * 1.8 + seed).sin() * 0.8;
    vec![
        Part::vquad(cx, cy - 14.0, 2.0, 14.0, post, alpha, true),
        Part::diamond(cx + wobble, cy - 18.0, 12.0, 7.0, 0.0, board, alpha, true),
        Part::diamond(cx - 4.0 + wobble, cy - 18.0, 2.0, 2.0, 0.0, shade(board, 0.7), alpha, true),
        Part::diamond(cx + 4.0 + wobble, cy - 18.0, 2.0, 2.0, 0.0, shade(board, 0.7), alpha, true),
    ]
}
