//! Dungeon: a crumbling stone archway entrance. Dark interior visible through
//! the arch, with ominous flickering light from within.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let stone = color;
    let dark = shade(stone, 0.6);
    let door = [0.08, 0.06, 0.04];
    let glow = [0.60, 0.20, 0.10];
    let seed = anim_seed(cx, cy);
    let flicker = (anim_time * 5.0 + seed).sin().max(0.0) * 0.3;

    let mut parts = vec![
        // Left pillar
        Part::vquad(cx - 10.0, cy - 20.0, 5.0, 20.0, stone, alpha, true),
        // Right pillar
        Part::vquad(cx + 10.0, cy - 20.0, 5.0, 20.0, stone, alpha, true),
        // Arch top
        Part::diamond(cx, cy - 24.0, 15.0, 5.0, 0.0, shade(stone, 1.1), alpha, true),
        // Dark doorway opening
        Part::diamond(cx, cy - 10.0, 8.0, 10.0, 0.0, door, alpha, true),
        // Stone texture details
        Part::diamond(cx - 10.0, cy - 8.0, 2.0, 3.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 10.0, cy - 14.0, 2.0, 3.0, 0.0, dark, alpha, true),
        // Crumbled top detail
        Part::diamond(cx - 6.0, cy - 26.0, 3.0, 2.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 8.0, cy - 25.0, 2.0, 2.0, 0.0, dark, alpha, true),
    ];

    // Ominous glow from within
    if flicker > 0.05 {
        parts.push(Part::diamond(cx, cy - 8.0, 6.0, 6.0, 0.0, glow, flicker, false));
    }

    parts
}
