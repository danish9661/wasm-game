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
    let wood = [0.62, 0.45, 0.26];
    let seed = anim_seed(cx, cy);
    let pulse = (0.7 + 0.3 * (anim_time * 3.0 + seed).sin()).max(0.4);
    vec![
        // Tall readable pole with carved bands.
        Part::vquad(cx, cy - 22.0, 3.5, 22.0, wood, alpha, true),
        Part::vquad(cx, cy - 18.0, 4.5, 2.0, shade(wood, 0.7), alpha, true),
        Part::vquad(cx, cy - 10.0, 4.5, 2.0, shade(wood, 0.7), alpha, true),
        // Carved arms.
        Part::vquad(cx - 6.0, cy - 16.0, 2.5, 2.5, shade(wood, 0.9), alpha, true),
        Part::vquad(cx + 6.0, cy - 16.0, 2.5, 2.5, shade(wood, 0.9), alpha, true),
        // Healing cross inlaid on the chest.
        Part::vquad(cx, cy - 14.0, 1.4, 6.0, [0.35, 0.95, 0.50], alpha, false),
        Part::vquad(cx - 2.5, cy - 12.5, 5.0, 1.4, [0.35, 0.95, 0.50], alpha, false),
        // Soft halo so the aura reads at a glance.
        Part::diamond(cx, cy - 14.0, 9.0, 12.0, 0.0, [0.30, 0.85, 0.45], alpha * 0.22 * pulse, false),
        // Glowing green gem (healing aura).
        Part::diamond(cx, cy - 26.0, 4.0, 5.0, 0.0, [0.30, 1.0, 0.50], alpha * pulse, true),
        Part::diamond(cx, cy - 26.0, 2.0, 3.0, 0.0, [0.75, 1.0, 0.80], alpha, true),
    ]
}
