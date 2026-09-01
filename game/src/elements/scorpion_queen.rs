//! Scorpion Queen: a fast, venomous desert boss — a wide, low-slung arachnid
//! with two massive front claws, a segmented body, and a curled tail stinger.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let body = color;
    let dark = shade(body, 0.7);
    let claw = shade(body, 1.15);
    let stinger = [0.95, 0.85, 0.20];
    let seed = anim_seed(cx, cy);
    let pulse = (anim_time * 4.0 + seed).sin() * 0.5 + 0.5;
    // Claws swing slightly
    let claw_swing = (anim_time * 2.0 + seed).sin() * 2.0;

    let mut parts = vec![
        // Wide segmented body — three overlapping horizontal diamonds
        Part::diamond(cx, cy - 4.0, 16.0, 5.0, 0.0, dark, alpha, true),
        Part::diamond(cx, cy - 9.0, 13.0, 5.0, 0.0, body, alpha, true),
        Part::diamond(cx, cy - 14.0, 10.0, 4.0, 0.0, shade(body, 1.1), alpha, true),
        // Eight splayed legs (4 per side)
        Part::diamond(cx - 14.0, cy - 1.0, 6.0, 1.5, 0.0, dark, alpha, false),
        Part::diamond(cx + 14.0, cy - 1.0, 6.0, 1.5, 0.0, dark, alpha, false),
        Part::diamond(cx - 11.0, cy - 5.0, 5.0, 1.5, 0.0, dark, alpha, false),
        Part::diamond(cx + 11.0, cy - 5.0, 5.0, 1.5, 0.0, dark, alpha, false),
        Part::diamond(cx - 9.0, cy - 9.0, 4.0, 1.5, 0.0, dark, alpha, false),
        Part::diamond(cx + 9.0, cy - 9.0, 4.0, 1.5, 0.0, dark, alpha, false),
        // Two massive front claws
        Part::diamond(cx - 12.0 + claw_swing, cy - 16.0, 7.0, 5.0, 0.0, claw, alpha, true),
        Part::diamond(cx + 12.0 - claw_swing, cy - 16.0, 7.0, 5.0, 0.0, claw, alpha, true),
        // Claw pincers
        Part::diamond(cx - 15.0 + claw_swing, cy - 14.0, 3.0, 2.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 15.0 - claw_swing, cy - 14.0, 3.0, 2.0, 0.0, dark, alpha, true),
        // Curled tail with stinger
        Part::diamond(cx, cy + 4.0, 4.0, 3.0, 0.0, body, alpha, true),
        Part::diamond(cx, cy + 8.0, 3.0, 2.5, 0.0, dark, alpha, true),
        Part::diamond(cx, cy + 11.0, 2.0, 2.0, 0.0, shade(body, 1.1), alpha, true),
        // Glowing stinger tip
        Part::diamond(cx, cy + 13.0, 1.5 + pulse * 0.5, 1.5 + pulse * 0.5, 0.0, stinger, alpha, true),
    ];

    // Two small eyes
    parts.push(Part::diamond(cx - 4.0, cy - 16.0, 1.5, 1.5, 0.0, [0.95, 0.90, 0.30], alpha, true));
    parts.push(Part::diamond(cx + 4.0, cy - 16.0, 1.5, 1.5, 0.0, [0.95, 0.90, 0.30], alpha, true));

    parts
}
