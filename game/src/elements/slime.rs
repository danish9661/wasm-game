//! Slime: a living blob that hops on a sine cycle and squashes on landing.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let body = color; // green from EnemyKind::Slime
    let seed = anim_seed(cx, cy);
    let ph = anim_time * 5.0 + seed;
    let hop = (ph.sin() * 0.5 + 0.5).clamp(0.0, 1.0); // 0 grounded .. 1 peak
    let lift_y = -hop * 7.0; // rise up to 7px
    let wx = 16.0 * (1.0 + 0.18 * (1.0 - hop)); // wider when grounded
    let wy = 12.0 * (1.0 - 0.12 * (1.0 - hop)); // shorter when grounded
    let center_y = cy - wy + lift_y;
    vec![
        Part::diamond(cx, center_y, wx, wy, 0.0, body, alpha, true),
        Part::diamond(cx - 4.0, center_y - wy * 0.5, 5.0, 4.0, 0.0, shade(body, 1.4), alpha, true),
    ]
}
