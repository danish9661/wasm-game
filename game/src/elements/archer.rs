//! Archer: a ranged humanoid marksman with a drawn bow and quiver.

use crate::elements::prim::{anim_seed, facing_offset, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let skin = [0.86, 0.66, 0.52];
    let hair = [0.18, 0.12, 0.08];
    let legs = [0.30, 0.24, 0.18];
    let tunic = color;
    let (hx, hy) = facing_offset(facing, 3.0);
    let seed = anim_seed(cx, cy);
    let phase = anim_time * 5.0 + seed;
    let bob = (phase * 2.0).sin().abs() * 0.8;

    // Bow bob — the bow sways slightly when idle
    let bow_sway = (anim_time * 1.8 + seed).sin() * 1.5;
    let (ax, ay) = facing_offset(facing, 1.0);

    vec![
        // Legs
        Part::vquad(cx - 3.0, cy - 14.0, 1.5, 14.0, legs, alpha, true),
        Part::vquad(cx + 3.0, cy - 14.0, 1.5, 14.0, legs, alpha, true),
        // Torso
        Part::vquad(cx, cy - 32.0 + bob, 4.0, 18.0, tunic, alpha, true),
        // Arms — one holds bow, other draws string
        Part::vquad(cx - 6.0, cy - 30.0 + bob, 1.25, 13.0, tunic, alpha, true),
        Part::vquad(cx + 3.5, cy - 30.0 + bob, 1.25, 13.0, tunic, alpha, true),
        // Hands
        Part::diamond(cx - 6.0, cy - 16.0 + bob, 2.0, 2.5, 0.0, skin, alpha, true),
        Part::diamond(cx + 3.5, cy - 16.0 + bob, 2.0, 2.5, 0.0, skin, alpha, true),
        // Head
        Part::diamond(cx + hx, cy - 41.0 + hy + bob, 7.0, 9.0, 0.0, skin, alpha, true),
        // Hair/hood
        Part::diamond(cx + hx, cy - 47.0 + hy + bob, 7.0, 4.0, 0.0, hair, alpha, true),
        // Bow (curved line of thin diamonds, attached to left arm)
        Part::vquad(cx - 10.0 + bow_sway + ax, cy - 32.0 + bob, 1.0, 18.0, [0.40, 0.25, 0.12], alpha, true),
        // Bowstring (thin vertical line)
        Part::vquad(cx - 10.0 + bow_sway + ax, cy - 32.0 + bob, 0.5, 16.0, [0.75, 0.72, 0.65], alpha, true),
        // Quiver on back
        Part::vquad(cx + 5.0, cy - 34.0 + bob, 2.0, 10.0, [0.40, 0.25, 0.12], alpha, false),
        // Arrow tips peeking from quiver
        Part::diamond(cx + 5.0, cy - 40.0 + bob, 1.0, 1.5, 0.0, [0.60, 0.60, 0.65], alpha, false),
    ]
}
