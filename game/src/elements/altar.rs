//! Altar: tall stone pillar with a pulsing gold cap.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let stone = [0.55, 0.52, 0.50];
    let pulse = 0.8 + 0.2 * (anim_time * 3.0).sin();
    let glow = shade(color, pulse); // gold from StructureKind::Altar
    vec![
        Part::vquad(cx, cy - 30.0, 16.0, 30.0, shade(stone, 0.8), alpha, true),
        Part::diamond(cx, cy - 32.0, 18.0, 9.0, 0.0, shade(stone, 1.1), alpha, true),
        // carved rune inset on the front face
        Part::diamond(cx, cy - 20.0, 4.0, 7.0, 0.0, shade(stone, 0.55), alpha, false),
        Part::diamond(cx, cy - 20.0, 1.8, 4.5, 0.0, glow, alpha * 0.9, false),
        // bright cap edge so the gold crown reads at a glance
        Part::diamond(cx, cy - 36.0, 8.0, 2.2, 0.0, shade(glow, 1.25), alpha, false),
        Part::diamond(cx, cy - 35.0, 8.0, 6.0, 0.0, glow, alpha, true),
    ]
}
