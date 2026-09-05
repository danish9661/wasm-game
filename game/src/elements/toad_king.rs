//! Toad King: a bloated swamp boss — a wide, low amphibian with bulging eyes,
//! a lolling tongue, warty bumpy skin, and a golden crown marking his reign.
//! Boss-class mass: far broader and taller than before so he reads as a king,
//! not a common swarmer.

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
    let eye = [0.90, 0.85, 0.20];
    let tongue = [0.85, 0.20, 0.20];
    let crown = [0.95, 0.80, 0.30];
    let jewel = [0.85, 0.15, 0.20];
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    // Breathing pulse — the body swells gently, harder on the move.
    let breath = (anim_time * (2.0 + 2.0 * w) + seed).sin() * (0.8 + 1.4 * w);
    // Tongue flicks out occasionally (more often while hunting).
    let tongue_out = {
        let t = (anim_time * (0.6 + 1.0 * w) + seed).fract();
        if t < 0.15 { t / 0.15 } else if t < 0.3 { 1.0 - (t - 0.15) / 0.15 } else { 0.0 }
    };

    let mut parts = vec![
        // Wide squat body
        Part::diamond(cx, cy - 10.0, 27.0 + breath, 13.0, 0.0, body, alpha, true),
        // Lighter belly
        Part::diamond(cx, cy - 5.0, 20.0, 6.5, 0.0, belly, alpha, true),
        // Royal belly jewel
        Part::diamond(cx, cy - 5.0, 3.0, 4.0, 0.0, jewel, alpha, true),
        // Two stubby front legs
        Part::vquad(cx - 16.5, cy - 13.0, 6.5, 13.0, dark, alpha, true),
        Part::vquad(cx + 16.5, cy - 13.0, 6.5, 13.0, dark, alpha, true),
        // Two stubby back legs (wider, tucked)
        Part::diamond(cx - 20.0, cy - 3.0, 8.0, 5.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 20.0, cy - 3.0, 8.0, 5.0, 0.0, dark, alpha, true),
        // Head — wide, flat
        Part::diamond(cx, cy - 23.0, 16.0, 10.0, 0.0, body, alpha, true),
        // Two bulging eyes on top of head
        Part::diamond(cx - 8.0, cy - 33.0, 5.5, 5.5, 0.0, shade(body, 1.1), alpha, true),
        Part::diamond(cx + 8.0, cy - 33.0, 5.5, 5.5, 0.0, shade(body, 1.1), alpha, true),
        // Pupils
        Part::diamond(cx - 8.0, cy - 33.0, 2.5, 2.5, 0.0, eye, alpha, true),
        Part::diamond(cx + 8.0, cy - 33.0, 2.5, 2.5, 0.0, eye, alpha, true),
        // King's crown: golden band with three spikes and a center jewel
        Part::vquad(cx, cy - 42.0, 12.0, 5.0, crown, alpha, true),
        Part::diamond(cx, cy - 45.0, 3.0, 5.0, 0.0, crown, alpha, true),
        Part::diamond(cx - 8.0, cy - 43.0, 2.5, 4.0, 0.0, crown, alpha, true),
        Part::diamond(cx + 8.0, cy - 43.0, 2.5, 4.0, 0.0, crown, alpha, true),
        Part::diamond(cx, cy - 40.0, 2.0, 2.5, 0.0, jewel, alpha, true),
    ];

    // Tongue (flicks out)
    if tongue_out > 0.01 {
        parts.push(Part::diamond(
            cx,
            cy - 13.0 + tongue_out * 6.0,
            3.0,
            tongue_out * 8.0,
            0.0,
            tongue,
            alpha,
            true,
        ));
    }

    // Warty bumps on the back
    parts.push(Part::diamond(cx - 5.0, cy - 16.0, 2.5, 2.5, 0.0, shade(body, 0.8), alpha, true));
    parts.push(Part::diamond(cx + 6.5, cy - 13.0, 2.5, 2.5, 0.0, shade(body, 0.8), alpha, true));
    parts.push(Part::diamond(cx, cy - 19.0, 2.5, 2.5, 0.0, shade(body, 0.8), alpha, true));
    parts.push(Part::diamond(cx - 12.0, cy - 10.0, 2.0, 2.0, 0.0, shade(body, 0.8), alpha, true));
    parts.push(Part::diamond(cx + 12.0, cy - 10.0, 2.0, 2.0, 0.0, shade(body, 0.8), alpha, true));

    parts
}
