//! Wraith: a floating, tattered spirit. A tapering ghostly body with two
//! glowing eyes and a trailing lower wisp. Drawn semi-transparent by the
//! caller's sprite alpha.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let body = color;
    let pale = shade(color, 1.25);
    let glow = [0.85, 1.0, 0.9];
    let seed = (cx * 1.7 + cy * 2.3).fract().abs();
    let drift = (anim_time * 1.6 + seed * 6.28).sin() * 2.0;
    // Tapering body: wide near the middle, narrowing to a wisp at the bottom.
    let mut parts = vec![
        Part::diamond(cx, cy - 2.0, 11.0, 9.0, 0.0, body, alpha, true),
        Part::diamond(cx, cy - 11.0, 8.0, 7.0, 0.0, body, alpha, true),
        Part::diamond(cx, cy - 19.0, 5.0, 6.0, 0.0, pale, alpha, true),
        // tattered lower wisp
        Part::diamond(cx - 4.0 + drift, cy + 9.0, 5.0, 6.0, 0.0, body, alpha * 0.7, true),
        Part::diamond(cx + 4.0 - drift, cy + 11.0, 4.0, 5.0, 0.0, body, alpha * 0.6, true),
    ];
    // Glowing eyes.
    parts.push(Part::diamond(cx - 3.0, cy - 9.0, 1.8, 1.8, 0.0, glow, alpha, true));
    parts.push(Part::diamond(cx + 3.0, cy - 9.0, 1.8, 1.8, 0.0, glow, alpha, true));
    parts
}
