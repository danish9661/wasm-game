//! Brute: a hulking, heavily-armored melee tank. Big shoulders, a horned
//! helm, and a slab of a body — built to read as "charging danger" even at a
//! distance. Its silhouette is wider and squatter than the Ogre's.

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
    let dark = shade(stone, 0.65);
    let light = shade(stone, 1.2);
    let horn = [0.95, 0.86, 0.55];
    let glow = (anim_time * 2.0).sin().max(0.0);
    let lean = (anim_time * 1.1).sin() * 1.0;
    vec![
        // legs
        Part::vquad(cx - 10.0, cy - 2.0, 8.0, 13.0, dark, alpha, true),
        Part::vquad(cx + 2.0, cy - 2.0, 8.0, 13.0, dark, alpha, true),
        // wide torso
        Part::diamond(cx, cy - 12.0, 19.0, 15.0, 0.0, stone, alpha, true),
        Part::diamond(cx, cy - 12.0, 12.0, 10.0, 0.0, shade(stone, 0.9), alpha, true),
        // big shoulders
        Part::diamond(cx - 16.0 + lean, cy - 18.0, 8.0, 7.0, 0.0, light, alpha, true),
        Part::diamond(cx + 16.0 - lean, cy - 18.0, 8.0, 7.0, 0.0, light, alpha, true),
        // arms
        Part::vquad(cx - 19.0 + lean, cy - 22.0, 7.0, 18.0, dark, alpha, true),
        Part::vquad(cx + 12.0 - lean, cy - 22.0, 7.0, 18.0, dark, alpha, true),
        // head + horned helm
        Part::diamond(cx, cy - 28.0, 8.0, 8.0, 0.0, stone, alpha, true),
        Part::diamond(cx - 5.0 - lean * 0.5, cy - 33.0, 1.8, 5.0, 0.0, horn, alpha, true),
        Part::diamond(cx + 5.0 + lean * 0.5, cy - 33.0, 1.8, 5.0, 0.0, horn, alpha, true),
        // glowing eyes
        Part::diamond(cx - 3.0, cy - 29.0, 1.4, 1.4, 0.0, [1.0, 0.5 + glow * 0.5, 0.3], alpha, true),
        Part::diamond(cx + 3.0, cy - 29.0, 1.4, 1.4, 0.0, [1.0, 0.5 + glow * 0.5, 0.3], alpha, true),
    ]
}
