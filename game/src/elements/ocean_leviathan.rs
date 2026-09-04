//! Ocean Leviathan: a swift tidal terror — a serpentine sea creature with fins,
//! glowing eyes, and a water-trail effect.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    walk: f32,
    anim_time: f32,
) -> Vec<Part> {
    let body = color;
    let dark = shade(body, 0.7);
    let belly = shade(body, 1.25);
    let fin = shade(body, 0.85);
    let eye = [0.85, 0.95, 1.0];
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    // Serpentine undulation, rolling harder while surging.
    let wave1 = (anim_time * (2.5 + 3.0 * w) + seed).sin() * (1.2 + 2.6 * w);
    let wave2 = (anim_time * (2.5 + 3.0 * w) + seed + 1.5).sin() * (1.0 + 2.0 * w);
    // Water trail shimmer
    let shimmer = (anim_time * (5.0 + 6.0 * w) + seed).sin().max(0.0);

    let mut parts = vec![
        // Serpentine body segments (4 segments, each offset by wave)
        Part::diamond(cx, cy - 4.0, 14.0, 6.0, 0.0, body, alpha, true),
        Part::diamond(cx + wave1, cy - 10.0, 11.0, 5.0, 0.0, body, alpha, true),
        Part::diamond(cx + wave2, cy - 16.0, 9.0, 4.0, 0.0, shade(body, 1.05), alpha, true),
        Part::diamond(cx + wave1 * 0.5, cy - 21.0, 7.0, 4.0, 0.0, shade(body, 1.1), alpha, true),
        // Lighter belly strip
        Part::diamond(cx, cy - 1.0, 10.0, 3.0, 0.0, belly, alpha, true),
        // Dorsal fin
        Part::diamond(cx + wave1 * 0.5, cy - 20.0, 3.0, 6.0, 0.0, fin, alpha, true),
        // Side fins
        Part::diamond(cx - 10.0, cy - 8.0, 5.0, 3.0, 0.0, fin, alpha, false),
        Part::diamond(cx + 10.0, cy - 8.0, 5.0, 3.0, 0.0, fin, alpha, false),
        // Tail fluke
        Part::diamond(cx + wave2, cy + 2.0, 8.0, 4.0, 0.0, fin, alpha, true),
        // Head details
        Part::diamond(cx, cy - 24.0, 5.0, 3.0, 0.0, shade(body, 1.1), alpha, true),
        // Glowing eyes
        Part::diamond(cx - 2.0, cy - 25.0, 1.5, 1.5, 0.0, eye, alpha, true),
        Part::diamond(cx + 2.0, cy - 25.0, 1.5, 1.5, 0.0, eye, alpha, true),
        // Open mouth
        Part::diamond(cx, cy - 22.0, 3.0, 2.0, 0.0, dark, alpha, true),
    ];

    // Water trail droplets (shimmer effect)
    if shimmer > 0.3 {
        let a = alpha * shimmer * 0.6;
        parts.push(Part::diamond(cx + wave2 + 4.0, cy + 4.0, 2.0, 2.0, 0.0, [0.4, 0.7, 0.9], a, false));
        parts.push(Part::diamond(cx + wave2 - 3.0, cy + 3.0, 1.5, 1.5, 0.0, [0.5, 0.8, 0.95], a, false));
    }

    parts
}
