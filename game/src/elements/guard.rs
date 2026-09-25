//! Village guard: an armored humanoid with a helmet, a thrusting sword and a
//! shield, so defenders read clearly as soldiers rather than plain townsfolk.

use crate::elements::humanoid;
use crate::elements::prim::{anim_seed, facing_offset, shade, Part};

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
    let a = attack.clamp(0.0, 1.0);
    let w = walk.clamp(0.0, 1.0);
    let (ax, ay) = facing_offset(facing, a * 6.8 + w * 0.8);
    let reach = a * 6.0;
    let (fx, fy) = facing_offset(facing, reach);
    // Mirror the rig's stride math so held gear never detaches: the right
    // hand sits at (cx + 4.7 - arm_swing + ax*reach - bend*(1-a) -
    // shoulder_sway, cy - 20 + yb + ay*reach), the left arm at
    // (cx - 8.0 + arm_swing + ax*reach + shoulder_sway). `yb` carries the
    // walk bob, idle breath and strike lunge-hop, so gear rides them all.
    let seed = anim_seed(cx, cy);
    let phase = anim_time * 7.0 + seed;
    let swing = phase.sin() * w;
    let shoulder_sway = -phase.cos() * 1.2 * w;
    let bob = (phase * 2.0).sin().abs() * 1.7 * w;
    let lunge_hop = (a * (1.0 - a) * 4.0) * 4.0;
    let breathe = if w < 0.05 && a < 0.05 { (anim_time * 2.0).sin() * 0.5 } else { 0.0 };
    let yb = bob + breathe + lunge_hop;
    let arm_swing = -swing * 4.0;
    let bend = 2.2; // elbow bend at rest (civilian bulk)

    // Helmet over the head: tracks the head lean (hx + ax), the facing lift
    // (hy + ay) and the body bob so it never floats off mid-stride.
    let head_cx = cx + hx + ax;
    let head_cy = cy - 41.0 + hy + yb + ay;
    parts.push(Part::diamond(head_cx, head_cy - 1.0, 7.5, 8.0, 0.0, steel, alpha, true));
    parts.push(Part::vquad(head_cx - 4.0, head_cy + 1.0, 4.0, 1.6, [0.15, 0.16, 0.18], alpha, false));

    // Sword in the right hand, thrusting forward on a strike. Anchored to
    // the rig's right-hand math (arm + elbow bend + shoulder sway + lunge).
    let handx = cx + 4.7 - arm_swing + ax * reach - bend * (1.0 - a) - shoulder_sway;
    let handy = cy - 20.0 + yb + ay * reach;
    parts.push(Part::vquad(handx - 1.0, handy - 22.0, 1.0, 18.0, steel, alpha, true));
    parts.push(Part::vquad(handx - 3.0 + fx, handy - 2.0 + fy, 3.0, 2.0, [0.55, 0.40, 0.18], alpha, true));
    parts.push(Part::vquad(handx - 1.0 + fx, handy + fy, 1.0, 4.0, [0.45, 0.30, 0.16], alpha, true));

    // Shield on the left arm with a colored boss. Anchored to the rig's
    // left-hand math so it rides the arm swing, shoulder sway and lunge.
    let shx = cx - 8.0 + arm_swing + ax * reach - bend * (1.0 - a) + shoulder_sway;
    let shy = cy - 20.0 + yb + ay * reach;
    parts.push(Part::diamond(shx, shy, 5.0, 7.0, 0.0, shade(steel, 0.65), alpha, true));
    parts.push(Part::diamond(shx, shy, 2.0, 3.0, 0.0, color, alpha, false));
    parts
}
