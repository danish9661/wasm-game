//! Scorpion Queen: a fast, venomous desert boss — a wide, low-slung arachnid
//! with two massive front claws, a segmented body, a curled tail stinger, and
//! a golden crown of spikes marking her royalty. Boss-class mass: broader and
//! taller than the Brute so she reads as a boss, not a minion.

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
    let claw = shade(body, 1.15);
    let stinger = [0.95, 0.85, 0.20];
    let crown = [0.95, 0.80, 0.30];
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    // Venom pulse + claws snap wider while charging.
    let pulse = (anim_time * (3.0 + 4.0 * w) + seed).sin() * 0.5 + 0.5;
    // Claws swing slightly
    let claw_swing = (anim_time * (2.0 + 3.0 * w) + seed).sin() * (1.0 + 2.5 * w);

    let mut parts = vec![
        // Wide segmented body — three overlapping horizontal diamonds
        Part::diamond(cx, cy - 6.0, 25.0, 10.0, 0.0, dark, alpha, true),
        Part::diamond(cx, cy - 14.0, 20.0, 10.0, 0.0, body, alpha, true),
        Part::diamond(cx, cy - 22.0, 15.0, 8.0, 0.0, shade(body, 1.1), alpha, true),
        // Eight splayed legs (4 per side), thick enough to read at distance
        Part::diamond(cx - 22.0, cy - 1.0, 9.0, 4.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 22.0, cy - 1.0, 9.0, 4.0, 0.0, dark, alpha, true),
        Part::diamond(cx - 17.0, cy - 8.0, 8.0, 4.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 17.0, cy - 8.0, 8.0, 4.0, 0.0, dark, alpha, true),
        Part::diamond(cx - 14.0, cy - 14.0, 6.0, 4.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 14.0, cy - 14.0, 6.0, 4.0, 0.0, dark, alpha, true),
        // Two massive front claws
        Part::diamond(cx - 19.0 + claw_swing, cy - 25.0, 11.0, 8.0, 0.0, claw, alpha, true),
        Part::diamond(cx + 19.0 - claw_swing, cy - 25.0, 11.0, 8.0, 0.0, claw, alpha, true),
        // Claw pincers
        Part::diamond(cx - 24.0 + claw_swing, cy - 22.0, 4.5, 3.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 24.0 - claw_swing, cy - 22.0, 4.5, 3.0, 0.0, dark, alpha, true),
        // Curled tail with stinger
        Part::diamond(cx, cy + 6.0, 6.0, 4.5, 0.0, body, alpha, true),
        Part::diamond(cx, cy + 12.0, 4.5, 4.0, 0.0, dark, alpha, true),
        Part::diamond(cx, cy + 17.0, 3.0, 3.0, 0.0, shade(body, 1.1), alpha, true),
        // Glowing stinger tip
        Part::diamond(cx, cy + 20.0, 2.5 + pulse * 0.8, 2.5 + pulse * 0.8, 0.0, stinger, alpha, true),
        // Queen's crown: golden spikes on the head segment
        Part::diamond(cx, cy - 30.0, 2.5, 5.0, 0.0, crown, alpha, true),
        Part::diamond(cx - 7.0, cy - 28.0, 2.0, 4.0, 0.0, crown, alpha, true),
        Part::diamond(cx + 7.0, cy - 28.0, 2.0, 4.0, 0.0, crown, alpha, true),
    ];

    // Two large eyes
    parts.push(Part::diamond(cx - 6.0, cy - 25.0, 2.5, 2.5, 0.0, [0.95, 0.90, 0.30], alpha, true));
    parts.push(Part::diamond(cx + 6.0, cy - 25.0, 2.5, 2.5, 0.0, [0.95, 0.90, 0.30], alpha, true));

    parts
}
