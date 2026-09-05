//! Mushroom: a cream stem capped by a red, spotted cap. Breathes gently and
//! sheds an occasional glowing spore.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let stem = [0.92, 0.88, 0.80];
    let seed = anim_seed(cx, cy);
    let breathe = (anim_time * 2.2 + seed).sin() * 0.7;
    let mut parts = vec![
        Part::vquad(cx, cy - 6.0, 3.0, 6.0, stem, alpha, true),
        Part::diamond(cx, cy - 8.0 + breathe * 0.4, 9.0, 7.0, 0.0, color, alpha, true),
        Part::diamond(cx - 3.0, cy - 10.0 + breathe * 0.4, 3.0, 2.0, 0.0, shade(color, 1.15), alpha, true),
        Part::diamond(cx + 3.0, cy - 9.0 + breathe * 0.4, 2.0, 2.0, 0.0, shade(color, 1.15), alpha, true),
    ];
    // Drifting spore, looping upward.
    let t = (anim_time * 0.35 + seed).fract();
    if t < 0.5 {
        parts.push(Part::diamond(
            cx + (t * 8.0 - 2.0),
            cy - 10.0 - t * 22.0,
            1.4, 1.4, 0.0,
            [1.0, 0.9, 0.6],
            alpha * (1.0 - t * 2.0) * 0.7,
            false,
        ));
    }
    parts
}
