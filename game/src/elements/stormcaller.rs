//! Stormcaller: a floating storm-mage. A hooded, tapering robe with a
//! crackling lightning orb held before it and two pale eyes. Semi-transparent
//! drifting body so it reads as airborne, like the wraith.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let robe = color;
    let pale = shade(color, 1.3);
    let bright = [0.80, 0.92, 1.0];
    let spark = [0.95, 0.98, 1.0];
    let seed = (cx * 2.1 + cy * 1.3).fract().abs();
    let drift = (anim_time * 2.0 + seed * 6.28).sin() * 2.0;
    let flick = (anim_time * 9.0 + seed * 12.0).sin().max(0.0);
    let mut parts = vec![
        // tapering robe, wide at the shoulders, narrowing to a wisp below
        Part::diamond(cx, cy - 3.0, 10.0, 8.0, 0.0, robe, alpha, true),
        Part::diamond(cx, cy - 11.0, 8.0, 7.0, 0.0, robe, alpha, true),
        Part::diamond(cx, cy - 18.0, 5.0, 6.0, 0.0, pale, alpha, true),
        Part::diamond(cx - 4.0 + drift, cy + 9.0, 5.0, 6.0, 0.0, robe, alpha * 0.7, true),
        // hooded head
        Part::diamond(cx, cy - 24.0, 6.0, 6.0, 0.0, pale, alpha, true),
    ];
    // pale glowing eyes
    parts.push(Part::diamond(cx - 2.5, cy - 24.0, 1.6, 1.6, 0.0, bright, alpha, true));
    parts.push(Part::diamond(cx + 2.5, cy - 24.0, 1.6, 1.6, 0.0, bright, alpha, true));
    // crackling lightning orb held before it
    parts.push(Part::diamond(cx, cy - 6.0, 4.0 + flick * 1.5, 4.0 + flick * 1.5, 0.0, spark, alpha, true));
    parts.push(Part::diamond(cx, cy - 6.0, 2.0, 2.0, 0.0, bright, alpha, true));
    parts
}
