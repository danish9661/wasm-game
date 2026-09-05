//! Humanoid: legs, tunic torso, arms, head + hair. `walk` is the movement
//! intensity (0..1): at 0 the figure stands and breathes gently; as it moves
//! the legs and arms swing in opposition and the body bobs. `attack` (0..1) is
//! a strike lunge: the torso leans toward `facing` and the arms extend forward.
//!
//! [`Variant`] re-skins the same limb math for the humanoid foe cast so walk
//! and attack animation stay identical while silhouettes differ: Goblin
//! (small, eared), Skeleton (gaunt, skull, ribs), Ogre (bulked armor brute),
//! Slinger (hooded, stone bandolier, rock in hand). `build` is the Civilian
//! base used by the player, townsfolk and guards.

use crate::elements::prim::{anim_seed, facing_offset, shade, Part};

/// Foe re-skin of the shared humanoid rig. Every variant runs the exact same
/// stride/lunge math — only proportions, palette and extras change — so the
/// walk-cycle test and facing lean hold for the whole cast.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variant {
    Civilian,
    Goblin,
    Skeleton,
    Ogre,
    Slinger,
}

pub fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    walk: f32,
    anim_time: f32,
    attack: f32,
) -> Vec<Part> {
    build_variant(cx, cy, color, alpha, facing, walk, anim_time, attack, Variant::Civilian)
}

pub fn build_variant(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    walk: f32,
    anim_time: f32,
    attack: f32,
    variant: Variant,
) -> Vec<Part> {
    // (width bulk, height stature, skin, hair/hood, legs)
    let (bulk, stature, skin, hair, legs) = match variant {
        Variant::Civilian => (1.0, 1.0, [0.86, 0.66, 0.52], [0.18, 0.12, 0.08], [0.30, 0.24, 0.18]),
        // Small green ambusher with big pointed ears.
        Variant::Goblin => (0.92, 0.88, [0.45, 0.62, 0.30], [0.10, 0.09, 0.05], [0.24, 0.27, 0.14]),
        // Gaunt undead: bone-white, no hair, visible ribs.
        Variant::Skeleton => (0.85, 1.0, [0.92, 0.90, 0.82], [0.92, 0.90, 0.82], [0.55, 0.52, 0.45]),
        // Slow armored bruiser: wide torso, pauldrons, heavy brow.
        Variant::Ogre => (1.35, 1.12, [0.55, 0.52, 0.48], [0.10, 0.09, 0.08], [0.35, 0.30, 0.25]),
        // Ranged rock-hurler: hood, stone bandolier, rock in the throwing hand.
        Variant::Slinger => (0.95, 1.0, [0.86, 0.66, 0.52], [0.26, 0.21, 0.30], [0.28, 0.22, 0.20]),
    };
    let b = bulk;
    let s = stature;
    let tunic = color;
    let (hx, hy) = facing_offset(facing, 4.0);
    // Forward lean while attacking (and a tiny bit during a fast walk).
    let (ax, ay) = facing_offset(facing, attack * 6.8 + walk.clamp(0.0, 1.0) * 0.8);
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    let a = attack.clamp(0.0, 1.0);

    // Stride oscillation (-1..1) scaled by movement; gentle breathing when still.
    let phase = anim_time * 7.0 + seed;
    let swing = (phase).sin() * w;
    let bob = (phase * 2.0).sin().abs() * 1.7 * w; // two bobs per stride
    // A small upward "hop" at the peak of a strike so attacks read as lunges.
    let lunge_hop = (a * (1.0 - a) * 4.0) * 4.0;
    let breathe = if w < 0.05 && a < 0.05 { (anim_time * 2.0).sin() * 0.5 } else { 0.0 };
    let yb = bob + breathe + lunge_hop;

    // Legs alternate forward/back; the trailing leg lifts slightly off the ground.
    // During a lunge the rear leg plants and the front leg drives forward.
    let l_lift = (-swing).max(0.0) * 4.0 + a * 2.0;
    let r_lift = (swing).max(0.0) * 4.0;
    let front = ax * a; // forward drive in screen space
    let leg_l = Part::vquad(cx - 4.0 * b + swing * 5.4 + front, cy - 19.0 * s - l_lift, 2.0 * b, 19.0 * s - l_lift, legs, alpha, true);
    let leg_r = Part::vquad(cx + 4.0 * b - swing * 5.4 + front, cy - 19.0 * s - r_lift, 2.0 * b, 19.0 * s - r_lift, legs, alpha, true);

    // Torso (tunic) centered on cx, with the body bob, leaning forward on attack.
    let torso = Part::vquad(cx + ax * 0.6, cy - 36.0 * s + yb, 5.4 * b, 24.0 * s, tunic, alpha, true);

    // Arms: normally swing opposite the legs; during a strike they snap forward
    // (toward the facing) to sell the hit, with hands reaching out.
    let arm_swing = -swing * 4.0;
    let reach = a * 8.0;
    let arm_l = Part::vquad(cx - 8.0 * b + arm_swing + ax * reach, cy - 34.0 * s + yb + ay * reach * 0.5, 1.7 * b, 17.5 * s, tunic, alpha, true);
    let arm_r = Part::vquad(cx + 4.7 * b - arm_swing + ax * reach, cy - 34.0 * s + yb + ay * reach * 0.5, 1.7 * b, 17.5 * s, tunic, alpha, true);
    let hand_l = Part::diamond(cx - 8.0 * b + arm_swing + ax * reach, cy - 20.0 * s + yb + ay * reach, 2.7 * b, 3.4 * s, 0.0, skin, alpha, true);
    let hand_r = Part::diamond(cx + 4.7 * b - arm_swing + ax * reach, cy - 20.0 * s + yb + ay * reach, 2.7 * b, 3.4 * s, 0.0, skin, alpha, true);

    // Head + hair, leaning toward the facing direction (and forward on attack).
    let head = Part::diamond(cx + hx + ax, cy - 50.0 * s + hy + yb + ay, 9.5 * b, 12.0 * s, 0.0, skin, alpha, true);

    let mut parts = vec![leg_l, leg_r, torso, arm_l, arm_r, hand_l, hand_r, head];

    match variant {
        Variant::Civilian | Variant::Goblin => {
            // Hair cap; goblins get small dark hair plus pointed ears.
            parts.push(Part::diamond(cx + hx + ax, cy - 58.0 * s + hy + yb + ay, 9.5 * b, 5.4 * s, 0.0, hair, alpha, true));
            if variant == Variant::Goblin {
                let ear = shade(skin, 0.8);
                parts.push(Part::diamond(cx - 10.5 * b + hx * 0.5, cy - 52.0 * s + hy + yb, 3.4 * b, 1.9 * s, 0.0, ear, alpha, true));
                parts.push(Part::diamond(cx + 10.5 * b + hx * 0.5, cy - 52.0 * s + hy + yb, 3.4 * b, 1.9 * s, 0.0, ear, alpha, true));
            }
        }
        Variant::Skeleton => {
            // Bare skull dome, dark eye sockets, exposed ribs.
            parts.push(Part::diamond(cx + hx + ax, cy - 58.0 * s + hy + yb + ay, 9.5 * b, 6.0 * s, 0.0, shade(skin, 1.05), alpha, true));
            let socket = [0.08, 0.07, 0.07];
            parts.push(Part::diamond(cx - 3.2 * b + hx + ax, cy - 50.0 * s + hy + yb + ay, 2.6 * b, 3.0 * s, 0.0, socket, alpha, true));
            parts.push(Part::diamond(cx + 3.2 * b + hx + ax, cy - 50.0 * s + hy + yb + ay, 2.6 * b, 3.0 * s, 0.0, socket, alpha, true));
            let rib = shade(skin, 0.42);
            parts.push(Part::vquad(cx + ax * 0.6, cy - 33.0 * s + yb, 4.6 * b, 1.3, rib, alpha, true));
            parts.push(Part::vquad(cx + ax * 0.6, cy - 29.0 * s + yb, 4.6 * b, 1.3, rib, alpha, true));
        }
        Variant::Ogre => {
            // Low heavy hairline, armor pauldrons, jutting brow.
            parts.push(Part::diamond(cx + hx + ax, cy - 58.0 * s + hy + yb + ay, 9.5 * b, 5.4 * s, 0.0, hair, alpha, true));
            let plate = [0.30, 0.28, 0.26];
            parts.push(Part::diamond(cx - 9.5 * b + arm_swing * 0.5, cy - 37.0 * s + yb, 4.6 * b, 3.6 * s, 0.0, plate, alpha, true));
            parts.push(Part::diamond(cx + 9.5 * b - arm_swing * 0.5, cy - 37.0 * s + yb, 4.6 * b, 3.6 * s, 0.0, plate, alpha, true));
            parts.push(Part::vquad(cx + hx + ax, cy - 55.0 * s + hy + yb + ay, 8.5 * b, 3.0, plate, alpha, true));
        }
        Variant::Slinger => {
            // Deep hood with side flaps, stone bandolier, rock gripped in hand.
            parts.push(Part::diamond(cx + hx + ax, cy - 58.0 * s + hy + yb + ay, 10.5 * b, 6.5 * s, 0.0, hair, alpha, true));
            parts.push(Part::vquad(cx - 8.6 * b + hx + ax, cy - 58.0 * s + hy + yb + ay, 1.8 * b, 10.0 * s, hair, alpha, true));
            parts.push(Part::vquad(cx + 8.6 * b + hx + ax, cy - 58.0 * s + hy + yb + ay, 1.8 * b, 10.0 * s, hair, alpha, true));
            let stone = [0.50, 0.48, 0.45];
            parts.push(Part::diamond(cx - 3.0 * b + ax * 0.6, cy - 33.0 * s + yb, 1.8, 1.8, 0.0, stone, alpha, true));
            parts.push(Part::diamond(cx + ax * 0.6, cy - 30.0 * s + yb, 1.8, 1.8, 0.0, stone, alpha, true));
            parts.push(Part::diamond(cx + 3.0 * b + ax * 0.6, cy - 27.0 * s + yb, 1.8, 1.8, 0.0, stone, alpha, true));
            // Rock in the throwing (right) hand — tracks the arm swing.
            parts.push(Part::diamond(cx + 4.7 * b - arm_swing + ax * reach, cy - 20.0 * s + yb + ay * reach, 3.0, 3.0, 0.0, stone, alpha, true));
        }
    }
    parts
}
