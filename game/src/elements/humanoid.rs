//! Humanoid: legs, tunic torso, arms, head + hair. `walk` is the movement
//! intensity (0..1): at 0 the figure stands and breathes gently; as it moves
//! the legs and arms swing in opposition and the body bobs.

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
    let tunic = color;
    let (hx, hy) = facing_offset(facing, 3.0);
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);

    // Stride oscillation (-1..1) scaled by movement; gentle breathing when still.
    let phase = anim_time * 7.0 + seed;
    let swing = (phase).sin() * w;
    let bob = (phase * 2.0).sin().abs() * 1.3 * w; // two bobs per stride
    let breathe = if w < 0.05 { (anim_time * 2.0).sin() * 0.5 } else { 0.0 };
    let yb = bob + breathe;

    // Legs alternate forward/back; the trailing leg lifts slightly off the ground.
    let l_lift = (-swing).max(0.0) * 3.0;
    let r_lift = (swing).max(0.0) * 3.0;
    let leg_l = Part::vquad(cx - 3.0 + swing * 4.0, cy - 14.0 - l_lift, 1.5, 14.0 - l_lift, legs, alpha, true);
    let leg_r = Part::vquad(cx + 3.0 - swing * 4.0, cy - 14.0 - r_lift, 1.5, 14.0 - r_lift, legs, alpha, true);

    // Torso (tunic) centered on cx, with the body bob.
    let torso = Part::vquad(cx, cy - 32.0 + yb, 4.0, 18.0, tunic, alpha, true);

    // Arms swing opposite to the legs, with little hands at the ends.
    let arm_swing = -swing * 3.0;
    let arm_l = Part::vquad(cx - 6.0 + arm_swing, cy - 30.0 + yb, 1.25, 13.0, tunic, alpha, true);
    let arm_r = Part::vquad(cx + 3.5 - arm_swing, cy - 30.0 + yb, 1.25, 13.0, tunic, alpha, true);
    let hand_l = Part::diamond(cx - 6.0 + arm_swing, cy - 16.0 + yb, 2.0, 2.5, 0.0, skin, alpha, true);
    let hand_r = Part::diamond(cx + 3.5 - arm_swing, cy - 16.0 + yb, 2.0, 2.5, 0.0, skin, alpha, true);

    // Head + hair, leaning toward the facing direction.
    let head = Part::diamond(cx + hx, cy - 41.0 + hy + yb, 7.0, 9.0, 0.0, skin, alpha, true);
    let hairp = Part::diamond(cx + hx, cy - 47.0 + hy + yb, 7.0, 4.0, 0.0, hair, alpha, true);

    vec![leg_l, leg_r, torso, arm_l, arm_r, hand_l, hand_r, head, hairp]
}
