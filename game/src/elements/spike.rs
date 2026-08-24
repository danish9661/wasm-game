//! Spike Trap: a low ring of iron spikes on the ground. Purely a hazard tile —
//! it does not block movement, but enemies (and the player) crossing it take
//! damage. Drawn flat so it reads as a floor hazard under the sprite shadow.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let base = color;
    let tip = shade(color, 1.4);
    let mut parts = vec![
        // flat base plate
        Part::diamond(cx, cy, 14.0, 5.0, 0.0, shade(base, 0.7), alpha, true),
    ];
    // four spikes poking up
    for (dx, dy) in [(-8.0, -1.0), (-3.0, -2.0), (3.0, -2.0), (8.0, -1.0)] {
        parts.push(Part::diamond(cx + dx, cy + dy - 4.0, 2.2, 5.0, 0.0, base, alpha, true));
        parts.push(Part::diamond(cx + dx, cy + dy - 7.0, 1.0, 2.5, 0.0, tip, alpha, true));
    }
    parts
}
