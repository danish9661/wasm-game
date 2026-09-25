//! Archer: a ranged humanoid marksman with a drawn bow and quiver.

use crate::elements::prim::{anim_seed, facing_offset, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    walk: f32,
    anim_time: f32,
) -> Vec<Part> {
    let skin = [0.86, 0.66, 0.52];
    let hair = [0.18, 0.12, 0.08];
    let legs = [0.30, 0.24, 0.18];
    let boots = [0.22, 0.17, 0.13];
    let belt = [0.30, 0.22, 0.14];
    let tunic = color;
    let (hx, hy) = facing_offset(facing, 3.0);
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    // Marching stride: legs scissor in opposition with a two-bobs-per-stride
    // bounce; at rest the feet settle slightly apart and the body breathes.
    let phase = anim_time * 7.0 + seed;
    let swing = phase.sin() * w;
    let bob = (phase * 2.0).sin().abs() * 1.5 * w;
    let breathe = if w < 0.05 { (anim_time * 2.0).sin() * 0.4 } else { 0.0 };
    let yb = bob + breathe;
    let stance = if w < 0.05 { 1.0 } else { 0.0 };

    // Bow bob — the bow sways slightly when idle
    let bow_sway = (anim_time * 1.8 + seed).sin() * (0.8 + 1.4 * w);
    // Horizontal facing lean only; the bow stays vertical by design.
    let (ax, _ay) = facing_offset(facing, 1.0);

    vec![
        // Legs stride; boot feet plant opposite the shin swing.
        Part::vquad(cx - 3.0 + swing * 4.5, cy - 14.0 - (-swing).max(0.0) * 2.5, 1.5, 14.0 - (-swing).max(0.0) * 2.5, legs, alpha, true),
        Part::vquad(cx + 3.0 - swing * 4.5, cy - 14.0 - swing.max(0.0) * 2.5, 1.5, 14.0 - swing.max(0.0) * 2.5, legs, alpha, true),
        Part::diamond(cx - 3.0 + swing * 5.2 + stance * 1.2, cy - 1.2, 2.4, 1.8, 0.0, boots, alpha, true),
        Part::diamond(cx + 3.0 - swing * 5.2 - stance * 1.0, cy - 1.2, 2.4, 1.8, 0.0, boots, alpha, true),
        // Torso with belt + shaded side seam.
        Part::vquad(cx, cy - 32.0 + yb, 4.0, 18.0, tunic, alpha, true),
        Part::vquad(cx + 2.6, cy - 32.0 + yb, 1.1, 18.0, belt, alpha * 0.55, false),
        Part::vquad(cx, cy - 20.0 + yb, 4.0, 1.6, belt, alpha * 0.9, false),
        // Arms ride the stride (opposite the legs); hands track the arms.
        Part::vquad(cx - 6.0 - swing * 3.0, cy - 30.0 + yb, 1.25, 13.0, tunic, alpha, true),
        Part::vquad(cx + 3.5 + swing * 3.0, cy - 30.0 + yb, 1.25, 13.0, tunic, alpha, true),
        // Hands
        Part::diamond(cx - 6.0 - swing * 3.0, cy - 16.0 + yb, 2.0, 2.5, 0.0, skin, alpha, true),
        Part::diamond(cx + 3.5 + swing * 3.0, cy - 16.0 + yb, 2.0, 2.5, 0.0, skin, alpha, true),
        // Head with a shaded jaw so the face has a front.
        Part::diamond(cx + hx, cy - 41.0 + hy + yb, 7.0, 9.0, 0.0, skin, alpha, true),
        Part::diamond(cx + hx - hx.signum() * 2.0, cy - 38.5 + hy + yb, 3.4, 4.2, 0.0, [0.70, 0.53, 0.42], alpha * 0.8, false),
        // Hair/hood rides the head bob.
        Part::diamond(cx + hx, cy - 47.0 + hy + yb, 7.0, 4.0, 0.0, hair, alpha, true),
        // Bow rides the left hand (attached to the striding arm, not the
        // torso) so it never detaches mid-stride; the string stays with it.
        Part::vquad(cx - 10.0 + bow_sway + ax - swing * 3.0, cy - 32.0 + yb, 1.0, 18.0, [0.40, 0.25, 0.12], alpha, true),
        // Bowstring (thin vertical line)
        Part::vquad(cx - 10.0 + bow_sway + ax - swing * 3.0, cy - 32.0 + yb, 0.5, 16.0, [0.75, 0.72, 0.65], alpha, true),
        // Quiver on back
        Part::vquad(cx + 5.0, cy - 34.0 + yb, 2.0, 10.0, [0.40, 0.25, 0.12], alpha, false),
        // Arrow tips peeking from quiver
        Part::diamond(cx + 5.0, cy - 40.0 + yb, 1.0, 1.5, 0.0, [0.60, 0.60, 0.65], alpha, false),
    ]
}
