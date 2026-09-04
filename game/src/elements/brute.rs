//! Brute: a hulking melee tank with a massive upper body, stubby legs,
//! and a charging stance. Winds up and charges in a straight line.

use crate::elements::prim::{anim_seed, facing_offset, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    walk: f32,
    anim_time: f32,
) -> Vec<Part> {
    let skin = color;
    let dark = shade(skin, 0.7);
    let scar = shade(skin, 1.3);
    let seed = anim_seed(cx, cy);
    let (hx, hy) = facing_offset(facing, 2.0);
    let w = walk.clamp(0.0, 1.0);
    // Heavy breathing sway, widening on the move, plus a ponderous stomp.
    let sway = (anim_time * 2.0 + seed).sin() * (0.5 + 1.5 * w);
    let stomp = (anim_time * (2.5 + 3.5 * w) + seed).sin() * 3.0 * w;

    let mut parts = vec![
        // Two thick, short legs (alternately stomping)
        Part::vquad(cx - 7.0 - stomp, cy - 2.0, 6.0, 12.0, dark, alpha, true),
        Part::vquad(cx + 7.0 + stomp, cy - 2.0, 6.0, 12.0, dark, alpha, true),
        // Massive barrel torso (wide, tall)
        Part::diamond(cx, cy - 16.0, 18.0, 14.0, 0.0, skin, alpha, true),
        // Darker belly band
        Part::diamond(cx, cy - 10.0, 14.0, 6.0, 0.0, dark, alpha, true),
        // Thick arms (wider than the humanoid, hanging at sides)
        Part::vquad(cx - 18.0 + sway, cy - 16.0, 7.0, 18.0, skin, alpha, true),
        Part::vquad(cx + 18.0 - sway, cy - 16.0, 7.0, 18.0, skin, alpha, true),
        // Huge fists
        Part::diamond(cx - 19.0 + sway, cy - 1.0, 5.0, 5.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 19.0 - sway, cy - 1.0, 5.0, 5.0, 0.0, dark, alpha, true),
        // Small head (contrast with massive body)
        Part::diamond(cx + hx, cy - 30.0 + hy, 6.0, 7.0, 0.0, skin, alpha, true),
        // Brow ridge
        Part::diamond(cx + hx, cy - 34.0 + hy, 8.0, 3.0, 0.0, dark, alpha, true),
        // Angry eyes
        Part::diamond(cx - 2.5 + hx, cy - 31.0 + hy, 1.5, 1.5, 0.0, [0.9, 0.2, 0.1], alpha, true),
        Part::diamond(cx + 2.5 + hx, cy - 31.0 + hy, 1.5, 1.5, 0.0, [0.9, 0.2, 0.1], alpha, true),
        // Scar across chest
        Part::diamond(cx + 3.0, cy - 16.0, 1.0, 7.0, 0.0, scar, alpha * 0.7, true),
    ];

    let _ = seed; // used above
    parts
}
