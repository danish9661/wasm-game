//! Anvil: a dark metal block with a flared top — occasional sparkle on hit surface.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let metal = color;
    let seed = anim_seed(cx, cy);
    // Occasional glint on the anvil surface
    let glint = {
        let t = (anim_time * 0.5 + seed).fract();
        if t > 0.92 { ((t - 0.92) / 0.08).min(1.0) } else { 0.0 }
    };
    let mut parts = vec![
        Part::vquad(cx - 3.0, cy - 6.0, 4.0, 6.0, shade(metal, 0.8), alpha, true),
        Part::vquad(cx, cy - 10.0, 12.0, 10.0, metal, alpha, true),
        Part::diamond(cx, cy - 12.0, 14.0, 5.0, 0.0, shade(metal, 1.15), alpha, true),
    ];
    if glint > 0.01 {
        parts.push(Part::diamond(cx + 4.0, cy - 12.0, 2.0, 2.0, 0.0, [0.95, 0.95, 0.90], glint * 0.8, false));
    }
    parts
}
