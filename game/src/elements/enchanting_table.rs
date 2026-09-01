//! Enchanting Table: a stone table with floating arcane symbols and a
//! glowing purple arcane core. Used to enchant weapons with gem dust.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let stone = [0.55, 0.50, 0.48];
    let arcane = color; // [0.40, 0.25, 0.70]
    let glow = shade(arcane, 1.4);
    let seed = anim_seed(cx, cy);
    // Pulsing arcane energy
    let pulse = (anim_time * 3.0 + seed).sin();
    let core_size = 3.0 + pulse.abs() * 1.5;
    // Floating symbol orbit
    let orbit = anim_time * 2.0 + seed;
    let sym1_x = orbit.cos() * 8.0;
    let sym1_y = orbit.sin() * 3.0;
    let sym2_x = (orbit + 3.14).cos() * 7.0;
    let sym2_y = (orbit + 3.14).sin() * 2.5;

    let mut parts = vec![
        // Stone base
        Part::vquad(cx, cy - 6.0, 12.0, 6.0, stone, alpha, true),
        // Table surface
        Part::diamond(cx, cy - 8.0, 14.0, 5.0, 0.0, shade(stone, 1.1), alpha, true),
        // Arcane core (pulsing)
        Part::diamond(cx, cy - 14.0, core_size, core_size, 0.0, arcane, alpha, true),
        // Inner glow
        Part::diamond(cx, cy - 14.0, core_size * 0.5, core_size * 0.5, 0.0, glow, alpha, true),
        // Runes carved into table
        Part::diamond(cx - 6.0, cy - 8.0, 1.5, 1.5, 0.0, arcane, alpha * 0.6, true),
        Part::diamond(cx + 6.0, cy - 8.0, 1.5, 1.5, 0.0, arcane, alpha * 0.6, true),
    ];

    // Floating arcane symbols (small diamonds that orbit the core)
    let sym_alpha = alpha * (0.5 + pulse.abs() * 0.5);
    parts.push(Part::diamond(cx + sym1_x, cy - 16.0 + sym1_y, 1.5, 1.5, 0.0, glow, sym_alpha, false));
    parts.push(Part::diamond(cx + sym2_x, cy - 16.0 + sym2_y, 1.5, 1.5, 0.0, glow, sym_alpha, false));

    parts
}
