//! Stone golem: a bulky, slow protector with a heavy club and glowing eyes.
//! Used as a village defender alongside the guards.

use crate::elements::prim::{anim_seed, facing_offset, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    _color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    walk: f32,
    anim_time: f32,
    attack: f32,
) -> Vec<Part> {
    let stone = [0.60, 0.62, 0.66];
    let dark = shade(stone, 0.7);
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    let phase = anim_time * 5.0 + seed;
    let swing = (phase).sin() * w * 3.0;
    let (fx, fy) = facing_offset(facing, attack.clamp(0.0, 1.0) * 6.0);
    let mut parts = Vec::new();

    // Thick legs.
    parts.push(Part::vquad(cx - 5.0 + swing, cy - 18.0, 3.0, 18.0, dark, alpha, true));
    parts.push(Part::vquad(cx + 5.0 - swing, cy - 18.0, 3.0, 18.0, dark, alpha, true));
    // Broad stone torso with a crack.
    parts.push(Part::vquad(cx - 9.0, cy - 44.0, 9.0, 26.0, stone, alpha, true));
    parts.push(Part::vquad(cx - 1.0, cy - 42.0, 1.0, 18.0, dark, alpha * 0.7, false));
    // Thick arms.
    parts.push(Part::vquad(cx - 12.0 - swing, cy - 42.0, 3.5, 20.0, stone, alpha, true));
    parts.push(Part::vquad(cx + 9.0 + swing, cy - 42.0, 3.5, 20.0, stone, alpha, true));
    // Stone head with glowing eyes.
    parts.push(Part::diamond(cx, cy - 52.0, 7.0, 8.0, 0.0, stone, alpha, true));
    let glow = (anim_time * 3.0).sin() * 0.3 + 0.7;
    parts.push(Part::diamond(cx - 3.0, cy - 52.0, 1.5, 1.5, 0.0, [0.3, 0.9, 1.0], alpha * glow, false));
    parts.push(Part::diamond(cx + 3.0, cy - 52.0, 1.5, 1.5, 0.0, [0.3, 0.9, 1.0], alpha * glow, false));
    // Heavy club in the right hand.
    let hx = cx + 9.0 + swing + fx;
    let hy = cy - 30.0 + fy;
    parts.push(Part::vquad(hx - 2.0, hy - 16.0, 2.0, 16.0, dark, alpha, true));
    parts.push(Part::diamond(hx, hy - 18.0, 6.0, 6.0, 0.0, stone, alpha, true));
    parts
}
