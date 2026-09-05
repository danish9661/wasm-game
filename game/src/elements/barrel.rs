//! Barrel: a wooden cask with two dark metal hoops — subtle creak sway.
//! `color` is the wood tint (callers pass browns); hoops are shaded darker.
//! Hoops are centered on the body and slightly wider so the silhouette stays
//! symmetric on its tile (see CENTER-vquad convention).

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let wood = color;
    let seed = anim_seed(cx, cy);
    let creak = (anim_time * 0.8 + seed).sin() * 0.3;
    vec![
        Part::vquad(cx + creak, cy - 16.0, 8.0, 16.0, wood, alpha, true),
        Part::vquad(cx + creak * 0.5, cy - 12.5, 8.7, 3.0, shade(wood, 0.55), alpha, true),
        Part::vquad(cx + creak * 0.5, cy - 4.5, 8.7, 3.0, shade(wood, 0.55), alpha, true),
    ]
}
