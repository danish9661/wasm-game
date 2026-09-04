//! Imp: a small, wiry swamp/forest swarmer with a forked tail and horns.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    walk: f32,
    anim_time: f32,
) -> Vec<Part> {
    let skin = color;
    let horn = shade(skin, 1.3);
    let w = walk.clamp(0.0, 1.0);
    let seed = anim_seed(cx, cy);
    // Skittish hop while swarming; legs shuffle beneath.
    let hop = (anim_time * (5.0 + 5.0 * w) + seed).sin().abs() * 4.0 * w;
    let shuf = (anim_time * (4.0 + 6.0 * w) + seed).sin() * 2.0 * w;
    let (hx, hy) = {
        let nx = facing.0 - facing.1;
        let ny = facing.0 + facing.1;
        let len = (nx * nx + ny * ny).sqrt();
        if len < 1e-4 {
            (0.0, 0.0)
        } else {
            (nx / len * 2.0, ny / len * 2.0)
        }
    };
    vec![
        Part::vquad(cx - 3.0 - shuf, cy - 8.0 - hop, 3.0, 8.0, shade(skin, 0.7), alpha, true),
        Part::vquad(cx + 3.0 + shuf, cy - 8.0 - hop, 3.0, 8.0, shade(skin, 0.7), alpha, true),
        Part::diamond(cx + hx, cy - 14.0 + hy - hop, 6.0, 8.0, 0.0, skin, alpha, true),
        Part::diamond(cx - 4.0 + hx, cy - 18.0 + hy - hop, 2.0, 3.0, 0.0, horn, alpha, true),
        Part::diamond(cx + 4.0 + hx, cy - 18.0 + hy - hop, 2.0, 3.0, 0.0, horn, alpha, true),
    ]
}
