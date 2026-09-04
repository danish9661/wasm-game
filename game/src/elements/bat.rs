//! Bat: a small body with flapping wings and two ear nubs.

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
    let wing = shade(color, 0.85);
    let body = color;
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    // Wings beat faster and deeper in flight; folded-ish glide at rest.
    let amp = 0.08 + 0.12 * w;
    let flap = (1.0 - amp) + amp * (anim_time * (7.0 + 8.0 * w) + seed).sin();
    vec![
        Part::diamond(cx - 10.0 * flap, cy - 4.0, 10.0 * flap, 6.0, 0.0, wing, alpha, true),
        Part::diamond(cx + 10.0 * flap, cy - 4.0, 10.0 * flap, 6.0, 0.0, wing, alpha, true),
        Part::diamond(cx, cy - 6.0, 5.0, 7.0, 0.0, body, alpha, true),
        Part::diamond(cx - 2.0, cy - 13.0, 2.0, 2.0, 0.0, shade(body, 1.2), alpha, true),
        Part::diamond(cx + 2.0, cy - 13.0, 2.0, 2.0, 0.0, shade(body, 1.2), alpha, true),
    ]
}
