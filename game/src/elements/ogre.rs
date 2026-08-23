//! Ogre: a hulking, armored brute — broad shoulders, a heavy brow, a club.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let hide = color;
    let loin = shade(hide, 0.7);
    let (hx, hy) = {
        let nx = facing.0 - facing.1;
        let ny = facing.0 + facing.1;
        let len = (nx * nx + ny * ny).sqrt();
        if len < 1e-4 {
            (0.0, 0.0)
        } else {
            (nx / len * 3.0, ny / len * 3.0)
        }
    };
    vec![
        Part::vquad(cx - 6.0, cy - 16.0, 6.0, 16.0, loin, alpha, true),
        Part::vquad(cx + 6.0, cy - 16.0, 6.0, 16.0, loin, alpha, true),
        // heavy torso
        Part::vquad(cx, cy - 38.0, 12.0, 22.0, hide, alpha, true),
        // head + brow
        Part::diamond(cx + hx, cy - 48.0 + hy, 9.0, 11.0, 0.0, shade(hide, 1.1), alpha, true),
        Part::diamond(cx + hx - 6.0, cy - 50.0 + hy, 3.0, 3.0, 0.0, shade(hide, 0.7), alpha, true),
        Part::diamond(cx + hx + 6.0, cy - 50.0 + hy, 3.0, 3.0, 0.0, shade(hide, 0.7), alpha, true),
        // club in the leading hand
        Part::vquad(cx + hx + 10.0, cy - 30.0, 3.0, 18.0, shade(hide, 0.6), alpha, true),
        Part::diamond(cx + hx + 10.0, cy - 32.0, 6.0, 6.0, 0.0, shade(hide, 0.85), alpha, true),
    ]
}
