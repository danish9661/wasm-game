//! Ocean Leviathan: a swift tidal terror — a serpentine sea creature with fins,
//! back spines, sweeping barbels, glowing eyes, and a water-trail effect.
//! Boss-class mass: longer and taller than before so the name delivers a
//! leviathan, not a small eel.

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
    let spine_tip = [0.95, 0.80, 0.30];
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
        Part::diamond(cx, cy - 6.0, 26.0, 9.0, 0.0, body, alpha, true),
        Part::diamond(cx + wave1, cy - 15.0, 23.0, 8.0, 0.0, body, alpha, true),
        Part::diamond(cx + wave2, cy - 25.0, 17.0, 6.5, 0.0, shade(body, 1.05), alpha, true),
        Part::diamond(cx + wave1 * 0.5, cy - 33.0, 14.0, 6.0, 0.0, shade(body, 1.1), alpha, true),
        // Lighter belly strip
        Part::diamond(cx, cy - 2.0, 17.0, 4.5, 0.0, belly, alpha, true),
        // Dorsal fin + gold-tipped spine ridge (5 spines, contrast in any tint)
        Part::diamond(cx + wave1 * 0.5, cy - 31.0, 5.5, 9.0, 0.0, fin, alpha, true),
        Part::diamond(cx + wave1 * 0.5, cy - 39.0, 3.0, 4.0, 0.0, spine_tip, alpha, true),
        Part::diamond(cx - 8.0 + wave2 * 0.5, cy - 28.0, 2.5, 5.0, 0.0, fin, alpha, true),
        Part::diamond(cx - 8.0 + wave2 * 0.5, cy - 33.0, 2.5, 3.5, 0.0, spine_tip, alpha, true),
        Part::diamond(cx + 8.0 + wave2 * 0.5, cy - 28.0, 2.5, 5.0, 0.0, fin, alpha, true),
        Part::diamond(cx + 8.0 + wave2 * 0.5, cy - 33.0, 2.5, 3.5, 0.0, spine_tip, alpha, true),
        Part::diamond(cx - 13.0 + wave1 * 0.5, cy - 20.0, 2.5, 4.5, 0.0, fin, alpha, true),
        Part::diamond(cx - 13.0 + wave1 * 0.5, cy - 24.0, 2.2, 3.0, 0.0, spine_tip, alpha, true),
        Part::diamond(cx + 13.0 + wave1 * 0.5, cy - 20.0, 2.5, 4.5, 0.0, fin, alpha, true),
        Part::diamond(cx + 13.0 + wave1 * 0.5, cy - 24.0, 2.2, 3.0, 0.0, spine_tip, alpha, true),
        // Side fins
        Part::diamond(cx - 16.5, cy - 12.0, 10.0, 5.5, 0.0, fin, alpha, false),
        Part::diamond(cx + 16.5, cy - 12.0, 10.0, 5.5, 0.0, fin, alpha, false),
        // Broad tail fluke
        Part::diamond(cx + wave2, cy + 3.0, 14.0, 6.5, 0.0, fin, alpha, true),
        // Head with open jaw
        Part::diamond(cx, cy - 37.0, 9.0, 4.5, 0.0, shade(body, 1.1), alpha, true),
        Part::diamond(cx, cy - 34.0, 4.5, 3.0, 0.0, dark, alpha, true),
        // Sweeping barbels (protrude past the jaw silhouette)
        Part::diamond(cx - 16.0, cy - 26.0, 2.0, 5.0, 0.0, [0.93, 0.95, 1.0], alpha, true),
        Part::diamond(cx + 16.0, cy - 26.0, 2.0, 5.0, 0.0, [0.93, 0.95, 1.0], alpha, true),
        // Glowing eyes
        Part::diamond(cx - 3.0, cy - 39.0, 2.2, 2.2, 0.0, eye, alpha, true),
        Part::diamond(cx + 3.0, cy - 39.0, 2.2, 2.2, 0.0, eye, alpha, true),
    ];

    // Water trail droplets (shimmer effect)
    if shimmer > 0.3 {
        let a = alpha * shimmer * 0.6;
        parts.push(Part::diamond(cx + wave2 + 6.0, cy + 6.0, 3.0, 3.0, 0.0, [0.4, 0.7, 0.9], a, false));
        parts.push(Part::diamond(cx + wave2 - 5.0, cy + 5.0, 2.5, 2.5, 0.0, [0.5, 0.8, 0.95], a, false));
    }

    parts
}
