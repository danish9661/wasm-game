//! Lantern: a post with a warm glowing glass box. Emits light (see building.rs).

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let post = [0.30, 0.22, 0.14];
    let seed = anim_seed(cx, cy);
    let flick = 0.85 + 0.15 * (anim_time * 12.0 + seed).sin();
    let glass = shade(color, flick);
    vec![
        Part::vquad(cx, cy - 18.0, 2.0, 14.0, post, alpha, true),
        Part::diamond(cx, cy - 20.0, 5.0, 7.0, 0.0, glass, alpha, true),
        Part::diamond(cx, cy - 20.0, 2.5, 4.0, 0.0, shade(glass, 1.25), alpha, true),
    ]
}
