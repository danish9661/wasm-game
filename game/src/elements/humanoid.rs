//! Humanoid: legs, tunic torso, head + hair. `color` is the tunic color.
//! Leans toward `facing` for a touch of motion and breathes gently.

use crate::elements::prim::{anim_seed, facing_offset, Part};

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
    let bob = (anim_time * 2.0 + anim_seed(cx, cy)).sin() * 0.8;
    vec![
        Part::vquad(cx - 3.0, cy - 14.0, 3.0, 14.0, legs, alpha, true),
        Part::vquad(cx + 3.0, cy - 14.0, 3.0, 14.0, legs, alpha, true),
        Part::vquad(cx, cy - 32.0 + bob, 8.0, 18.0, tunic, alpha, true),
        Part::diamond(cx + hx, cy - 41.0 + hy + bob, 7.0, 9.0, 0.0, skin, alpha, true),
        Part::diamond(cx + hx, cy - 47.0 + hy + bob, 7.0, 4.0, 0.0, hair, alpha, true),
    ]
}
