//! Campfire: crossed dark logs + flickering orange/yellow flame.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let log = [0.30, 0.20, 0.12];
    let seed = anim_seed(cx, cy);
    let flick = 0.82 + 0.18 * (anim_time * 11.0 + seed).sin();
    let flame = shade(color, flick); // orange from StructureKind::Campfire
    vec![
        Part::vquad(cx - 8.0, cy - 4.0, 4.0, 6.0, log, alpha, true),
        Part::vquad(cx + 8.0, cy - 4.0, 4.0, 6.0, log, alpha, true),
        Part::diamond(cx, cy - 14.0, 8.0, 12.0, 0.0, flame, alpha, true),
        Part::diamond(cx, cy - 18.0, 5.0, 7.0, 0.0, shade([1.0, 0.85, 0.30], flick), alpha, true),
    ]
}
