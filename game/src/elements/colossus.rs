//! Colossus: a towering stone golem — the second Crown Fragment guardian.
//! A blocky, heavyset body with a glowing core and eyes, shoulder boulders,
//! and jagged cracks of brighter stone.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let stone = color;
    let dark = shade(stone, 0.7);
    let light = shade(stone, 1.25);
    let core = [0.95, 0.85, 0.45];
    let glow = (anim_time * 3.0).sin().max(0.0);
    let sway = (anim_time * 1.2).sin() * 1.5;
    vec![
        // legs
        Part::vquad(cx - 11.0, cy - 2.0, 9.0, 14.0, dark, alpha, true),
        Part::vquad(cx + 2.0, cy - 2.0, 9.0, 14.0, dark, alpha, true),
        // torso
        Part::diamond(cx, cy - 12.0, 20.0, 18.0, 0.0, stone, alpha, true),
        Part::diamond(cx, cy - 12.0, 13.0, 12.0, 0.0, shade(stone, 0.9), alpha, true),
        // glowing core
        Part::diamond(cx, cy - 12.0, 5.0 + glow * 1.5, 6.0 + glow * 1.5, 0.0, core, alpha, true),
        // arms
        Part::vquad(cx - 21.0 + sway, cy - 20.0, 7.0, 20.0, dark, alpha, true),
        Part::vquad(cx + 14.0 - sway, cy - 20.0, 7.0, 20.0, dark, alpha, true),
        // shoulder boulders
        Part::diamond(cx - 22.0, cy - 22.0, 7.0, 7.0, 0.0, light, alpha, true),
        Part::diamond(cx + 22.0, cy - 22.0, 7.0, 7.0, 0.0, light, alpha, true),
        // head
        Part::diamond(cx, cy - 30.0, 9.0, 9.0, 0.0, stone, alpha, true),
        // eyes
        Part::diamond(cx - 3.5, cy - 31.0, 1.8, 1.8, 0.0, [1.0, 0.9, 0.5], alpha, true),
        Part::diamond(cx + 3.5, cy - 31.0, 1.8, 1.8, 0.0, [1.0, 0.9, 0.5], alpha, true),
        // cracks
        Part::diamond(cx + 6.0, cy - 6.0, 2.0, 9.0, 0.0, light, alpha * 0.8, true),
        Part::diamond(cx - 7.0, cy - 16.0, 2.0, 7.0, 0.0, light, alpha * 0.8, true),
    ]
}
