//! Humanoid: legs, tunic torso, arms, head + hair. `walk` is the movement
//! intensity (0..1): at 0 the figure stands and breathes gently; as it moves
//! the legs and arms swing in opposition and the body bobs. `attack` (0..1) is
//! a strike lunge: the torso leans toward `facing` and the arms extend forward.

use crate::elements::prim::{anim_seed, facing_offset, Part};

pub fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    walk: f32,
    anim_time: f32,
    attack: f32,
) -> Vec<Part> {
    let skin = [0.86, 0.66, 0.52];
    let hair = [0.18, 0.12, 0.08];
    let legs = [0.30, 0.24, 0.18];
    let tunic = color;
    let (hx, hy) = facing_offset(facing, 4.0);
    // Forward lean while attacking (and a tiny bit during a fast walk).
    let (ax, ay) = facing_offset(facing, attack * 6.8 + walk.clamp(0.0, 1.0) * 0.8);
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    let a = attack.clamp(0.0, 1.0);

    // Stride oscillation (-1..1) scaled by movement; gentle breathing when still.
    let phase = anim_time * 7.0 + seed;
    let swing = (phase).sin() * w;
    let bob = (phase * 2.0).sin().abs() * 1.7 * w; // two bobs per stride
    // A small upward "hop" at the peak of a strike so attacks read as lunges.
    let lunge_hop = (a * (1.0 - a) * 4.0) * 4.0;
    let breathe = if w < 0.05 && a < 0.05 { (anim_time * 2.0).sin() * 0.5 } else { 0.0 };
    let yb = bob + breathe + lunge_hop;

    // Legs alternate forward/back; the trailing leg lifts slightly off the ground.
    // During a lunge the rear leg plants and the front leg drives forward.
    let l_lift = (-swing).max(0.0) * 4.0 + a * 2.0;
    let r_lift = (swing).max(0.0) * 4.0;
    let front = ax * a; // forward drive in screen space
    let leg_l = Part::vquad(cx - 4.0 + swing * 5.4 + front, cy - 19.0 - l_lift, 2.0, 19.0 - l_lift, legs, alpha, true);
    let leg_r = Part::vquad(cx + 4.0 - swing * 5.4 + front, cy - 19.0 - r_lift, 2.0, 19.0 - r_lift, legs, alpha, true);

    // Torso (tunic) centered on cx, with the body bob, leaning forward on attack.
    let torso = Part::vquad(cx + ax * 0.6, cy - 36.0 + yb, 5.4, 24.0, tunic, alpha, true);

    // Arms: normally swing opposite the legs; during a strike they snap forward
    // (toward the facing) to sell the hit, with hands reaching out.
    let arm_swing = -swing * 4.0;
    let reach = a * 8.0;
    let arm_l = Part::vquad(cx - 8.0 + arm_swing + ax * reach, cy - 34.0 + yb + ay * reach * 0.5, 1.7, 17.5, tunic, alpha, true);
    let arm_r = Part::vquad(cx + 4.7 - arm_swing + ax * reach, cy - 34.0 + yb + ay * reach * 0.5, 1.7, 17.5, tunic, alpha, true);
    let hand_l = Part::diamond(cx - 8.0 + arm_swing + ax * reach, cy - 20.0 + yb + ay * reach, 2.7, 3.4, 0.0, skin, alpha, true);
    let hand_r = Part::diamond(cx + 4.7 - arm_swing + ax * reach, cy - 20.0 + yb + ay * reach, 2.7, 3.4, 0.0, skin, alpha, true);

    // Head + hair, leaning toward the facing direction (and forward on attack).
    let head = Part::diamond(cx + hx + ax, cy - 50.0 + hy + yb + ay, 9.5, 12.0, 0.0, skin, alpha, true);
    let hairp = Part::diamond(cx + hx + ax, cy - 58.0 + hy + yb + ay, 9.5, 5.4, 0.0, hair, alpha, true);

    vec![leg_l, leg_r, torso, arm_l, arm_r, hand_l, hand_r, head, hairp]
}
