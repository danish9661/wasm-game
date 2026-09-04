//! Slime: a living blob that hops on a sine cycle and squashes on landing.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    walk: f32,
    anim_time: f32,
) -> Vec<Part> {
    let body = color; // green from EnemyKind::Slime
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    // Hop cycle quickens and rises while hunting; idle is a slow jelly wobble.
    let ph = anim_time * (4.0 + 5.0 * w) + seed;
    let hop = (ph.sin() * 0.5 + 0.5).clamp(0.0, 1.0); // 0 grounded .. 1 peak
    let lift_y = -hop * (5.0 + 5.0 * w); // rise up to 10px at full stride
    // Side-to-side jiggle so the blob feels like jelly, not a bouncing ball.
    let wob = (anim_time * 9.0 + seed).sin() * (1.2 + 1.6 * w);
    let wx = 16.0 * (1.0 + 0.24 * (1.0 - hop)) + wob.abs() * 0.25; // wider when grounded
    let wy = 12.0 * (1.0 - 0.18 * (1.0 - hop)); // shorter when grounded
    let jx = cx + wob;
    let center_y = cy - wy + lift_y;
    vec![
        Part::diamond(jx, center_y, wx, wy, 0.0, body, alpha, true),
        Part::diamond(jx - 4.0, center_y - wy * 0.5, 5.0, 4.0, 0.0, shade(body, 1.4), alpha, true),
    ]
}
