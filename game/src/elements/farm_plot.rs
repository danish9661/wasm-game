//! Farm Plot: a tilled soil bed with a few sprouts. Renewable food source —
//! the player harvests it (E) once the crops are grown; it regrows on a timer.
//! Drawn low so it sits on the ground like a floor feature.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let soil = color;
    let soil_dk = shade(color, 0.6);
    let leaf = [0.35, 0.70, 0.30];
    let sway = (anim_time * 1.5).sin() * 1.0;
    let mut parts = vec![
        Part::diamond(cx, cy, 14.0, 6.0, 0.0, soil_dk, alpha, true),
        Part::diamond(cx, cy - 1.0, 11.0, 4.0, 0.0, soil, alpha, true),
    ];
    // rows of sprouts
    for (dx, dy) in [(-7.0, 0.0), (-2.0, 1.0), (3.0, 0.0), (8.0, 1.0)] {
        parts.push(Part::diamond(cx + dx, cy + dy - 4.0 + sway, 3.0, 4.0, 0.0, leaf, alpha, true));
        parts.push(Part::diamond(cx + dx + 1.5, cy + dy - 6.0 - sway, 2.0, 3.0, 0.0, shade(leaf, 1.2), alpha, true));
    }
    parts
}
