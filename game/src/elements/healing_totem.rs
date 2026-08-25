//! Healing Totem: a carved post that radiates a soothing green aura, slowly
//! regenerating the player while they stay nearby (world applies the heal).

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    _color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let wood = [0.50, 0.35, 0.20];
    let seed = anim_seed(cx, cy);
    let pulse = (0.7 + 0.3 * (anim_time * 3.0 + seed).sin()).max(0.4);
    vec![
        Part::vquad(cx, cy - 12.0, 4.0, 12.0, shade(wood, 1.0), alpha, true),
        // carved arms
        Part::vquad(cx - 6.0, cy - 12.0, 2.5, 2.5, shade(wood, 0.9), alpha, true),
        Part::vquad(cx + 6.0, cy - 12.0, 2.5, 2.5, shade(wood, 0.9), alpha, true),
        // glowing green gem (healing aura)
        Part::diamond(cx, cy - 21.0, 4.0, 5.0, 0.0, [0.30, 1.0, 0.50], alpha * pulse, true),
        Part::diamond(cx, cy - 21.0, 2.0, 3.0, 0.0, [0.75, 1.0, 0.80], alpha, true),
    ]
}
