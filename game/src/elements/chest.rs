//! Chest: wooden box + lid + dark lock — faint golden shimmer on the lid.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let wood = [0.45, 0.30, 0.14];
    let lid = color;
    let seed = anim_seed(cx, cy);
    let shimmer = (anim_time * 2.5 + seed).sin() * 0.15 + 0.85;
    vec![
        Part::vquad(cx, cy - 18.0, 14.0, 18.0, wood, alpha, true),
        Part::diamond(cx, cy - 20.0, 14.0, 6.0, 0.0, shade(lid, shimmer), alpha, true),
        Part::diamond(cx, cy - 10.0, 3.0, 3.0, 0.0, [0.10, 0.09, 0.07], alpha, true),
    ]
}
