//! Village guard: an armored humanoid with a helmet, a thrusting sword and a
//! shield, so defenders read clearly as soldiers rather than plain townsfolk.

use crate::elements::humanoid;
use crate::elements::prim::{facing_offset, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    walk: f32,
    anim_time: f32,
    attack: f32,
) -> Vec<Part> {
    let mut parts = humanoid::build(cx, cy, color, alpha, facing, walk, anim_time, attack);
    let steel = [0.72, 0.75, 0.80];
    let (hx, hy) = facing_offset(facing, 4.0);
    let reach = attack.clamp(0.0, 1.0) * 6.0;
    let (fx, fy) = facing_offset(facing, reach);

    // Helmet over the head.
    let head_cx = cx + hx;
    let head_cy = cy - 41.0 + hy;
    parts.push(Part::diamond(head_cx, head_cy - 1.0, 7.5, 8.0, 0.0, steel, alpha, true));
    parts.push(Part::vquad(head_cx - 4.0, head_cy + 1.0, 4.0, 1.6, [0.15, 0.16, 0.18], alpha, false));

    // Sword in the right hand, thrusting forward on a strike.
    let handx = cx + 4.0 + hx;
    let handy = cy - 16.0 + hy;
    parts.push(Part::vquad(handx - 1.0 + fx, handy - 18.0 + fy, 1.0, 18.0, steel, alpha, true));
    parts.push(Part::vquad(handx - 3.0 + fx, handy - 2.0 + fy, 3.0, 2.0, [0.55, 0.40, 0.18], alpha, true));
    parts.push(Part::vquad(handx - 1.0 + fx, handy + fy, 1.0, 4.0, [0.45, 0.30, 0.16], alpha, true));

    // Shield on the left arm with a colored boss.
    let shx = cx - 6.0 - hx + fx * 0.5;
    let shy = cy - 22.0 + hy;
    parts.push(Part::diamond(shx, shy, 5.0, 7.0, 0.0, shade(steel, 0.9), alpha, true));
    parts.push(Part::diamond(shx, shy, 2.0, 3.0, 0.0, color, alpha, false));
    parts
}
