//! Humanoid rig: boot feet, striding legs, belted tunic torso, jointed arms,
//! and a faced head. `walk` is the movement intensity (0..1): at 0 the figure
//! settles into a relaxed stance and breathes gently; as it moves the legs
//! scissor, hips and shoulders counter-rotate, and the body bobs. `attack`
//! (0..1) is a strike lunge: the torso leans toward `facing` and both arms
//! extend forward.
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

/// The clamped tunic tint the rig actually draws for a `color`. The renderer
/// clamps every tunic just under white so the hit-flash can always brighten
/// it; tests that hunt the torso quad must match this, not the raw tint.
pub fn civilian_tunic(color: [f32; 3]) -> [f32; 3] {
    if color == [0.92, 0.90, 0.82] {
        color
    } else {
        [
            color[0].min(0.92),
            color[1].min(0.92),
            color[2].min(0.92),
        ]
    }
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
    // Clamp the tunic tint so *some* channel stays below 1.0: `flashed()`
    // lerps each channel toward white, and a fully-maxed color cannot
    // brighten (the flash audit would see no change on a pale tunic).
    // Pure-bone white (skeleton) is exempted below by dulling the skull
    // cap one step instead.
    let tunic = civilian_tunic(color);
    let boots = shade(legs, 0.72);
    let belt = shade(tunic, 0.5);
    // Collar trim: brightened but capped below 1.0 so the hit-flash can
    // always brighten the figure (a maxed channel cannot lerp to white).
    let trim = {
        let t = shade(tunic, 1.35);
        [t[0].min(0.9), t[1].min(0.9), t[2].min(0.9)]
    };
    // Keep cloth below the white ceiling so the hit-flash can always
    // brighten the figure: a color already at 1.0 cannot lerp to white.
    let (hx, hy) = facing_offset(facing, 4.0);
    // Forward lean while attacking (and a tiny bit during a fast walk).
    let (ax, ay) = facing_offset(facing, attack * 6.8 + walk.clamp(0.0, 1.0) * 0.8);
    let seed = anim_seed(cx, cy);
    let w = walk.clamp(0.0, 1.0);
    let a = attack.clamp(0.0, 1.0);

    // Stride oscillation: legs scissor in opposition. At rest the figure
    // settles into a relaxed contrapposto stance (one knee soft) instead of
    // standing bolt upright; the torso counter-rotates against the hips so
    // walking reads as a gait rather than a slide.
    let phase = anim_time * 7.0 + seed;
    let swing = (phase).sin() * w;
    let hip_sway = (phase).cos() * 1.6 * w; // hips shift side to side
    let shoulder_sway = -(phase).cos() * 1.2 * w; // shoulders counter the hips
    let bob = (phase * 2.0).sin().abs() * 1.7 * w; // two bobs per stride
    // Idle stance: soft knee, one foot slightly forward, weight settled.
    let stance = if w < 0.05 { 1.0 } else { 0.0 };
    // A small upward "hop" at the peak of a strike so attacks read as lunges.
    let lunge_hop = (a * (1.0 - a) * 4.0) * 4.0;
    let breathe = if w < 0.05 && a < 0.05 { (anim_time * 2.0).sin() * 0.5 } else { 0.0 };
    let yb = bob + breathe + lunge_hop;

    // Legs alternate forward/back; the trailing leg lifts slightly off the ground.
    // During a lunge the rear leg plants and the front leg drives forward.
    // Boots are split into foot + shin so the stride bends at the ankle.
    let l_lift = (-swing).max(0.0) * 4.0 + a * 2.0;
    let r_lift = (swing).max(0.0) * 4.0;
    let front = ax * a; // forward drive in screen space
    let rest_l = stance * 1.6; // relaxed left knee forward at rest
    let rest_r = -stance * 1.2;
    let leg_l = Part::vquad(cx - 4.0 * b + swing * 5.4 + front + hip_sway + rest_l, cy - 19.0 * s - l_lift, 2.0 * b, 19.0 * s - l_lift, legs, alpha, true);
    let leg_r = Part::vquad(cx + 4.0 * b - swing * 5.4 + front - hip_sway + rest_r, cy - 19.0 * s - r_lift, 2.0 * b, 19.0 * s - r_lift, legs, alpha, true);
    // Boot feet: dark toes that plant opposite the shin swing.
    let foot_l = Part::diamond(cx - 4.0 * b + swing * 6.4 + front + hip_sway * 0.5 + rest_l, cy - 1.5 - l_lift * 0.6, 3.0 * b, 2.2 * s, 0.0, boots, alpha, true);
    let foot_r = Part::diamond(cx + 4.0 * b - swing * 6.4 + front - hip_sway * 0.5 + rest_r, cy - 1.5 - r_lift * 0.6, 3.0 * b, 2.2 * s, 0.0, boots, alpha, true);

    // Torso (tunic) centered on cx, with the body bob, leaning forward on attack.
    // A shaded side seam + belt + collar trim give the chest volume and break
    // the flat slab of color.
    let torso = Part::vquad(cx + ax * 0.6 + shoulder_sway * 0.4, cy - 36.0 * s + yb, 5.4 * b, 24.0 * s, tunic, alpha, true);
    let torso_shade = Part::vquad(cx + ax * 0.6 + shoulder_sway * 0.4 + 3.4 * b, cy - 36.0 * s + yb, 1.6 * b, 24.0 * s, shade(tunic, 0.62), alpha * 0.85, false);
    let belt_band = Part::vquad(cx + ax * 0.6, cy - 22.0 * s + yb, 5.4 * b, 2.0, belt, alpha * 0.95, false);
    let collar = Part::diamond(cx + hx * 0.4 + ax * 0.6, cy - 37.0 * s + yb + ay * 0.6, 4.4 * b, 2.2 * s, 0.0, trim, alpha * 0.9, false);

    // Arms: shoulders hang from the counter-rotated shoulder line; the swing
    // arm leads with the elbow slightly bent (hand offset sideways from the
    // arm so the limb reads as jointed, not a stick). During a strike both
    // arms snap forward toward the facing to sell the hit.
    let arm_swing = -swing * 4.0;
    let reach = a * 8.0;
    let bend = 2.2 * b; // elbow bend at rest
    let arm_l = Part::vquad(cx - 8.0 * b + arm_swing + ax * reach + shoulder_sway, cy - 34.0 * s + yb + ay * reach * 0.5, 1.7 * b, 17.5 * s, tunic, alpha, true);
    let arm_r = Part::vquad(cx + 4.7 * b - arm_swing + ax * reach - shoulder_sway, cy - 34.0 * s + yb + ay * reach * 0.5, 1.7 * b, 17.5 * s, tunic, alpha, true);
    // Sleeve cuffs where arm meets hand.
    let cuff_l = Part::vquad(cx - 8.0 * b + arm_swing + ax * reach + shoulder_sway, cy - 21.5 * s + yb + ay * reach * 0.5, 1.7 * b, 1.8, shade(tunic, 0.6), alpha * 0.9, false);
    let cuff_r = Part::vquad(cx + 4.7 * b - arm_swing + ax * reach - shoulder_sway, cy - 21.5 * s + yb + ay * reach * 0.5, 1.7 * b, 1.8, shade(tunic, 0.6), alpha * 0.9, false);
    let hand_l = Part::diamond(cx - 8.0 * b + arm_swing + ax * reach + bend * (1.0 - a) + shoulder_sway, cy - 20.0 * s + yb + ay * reach, 2.7 * b, 3.4 * s, 0.0, skin, alpha, true);
    let hand_r = Part::diamond(cx + 4.7 * b - arm_swing + ax * reach - bend * (1.0 - a) - shoulder_sway, cy - 20.0 * s + yb + ay * reach, 2.7 * b, 3.4 * s, 0.0, skin, alpha, true);

    // Head + hair, leaning toward the facing direction (and forward on attack).
    // A shaded jaw on the far side + a single catch-light eye give the face a
    // front that tracks the facing instead of a blank oval.
    let head = Part::diamond(cx + hx + ax, cy - 50.0 * s + hy + yb + ay, 9.5 * b, 12.0 * s, 0.0, skin, alpha, true);
    let jaw = Part::diamond(cx + hx + ax - hx.signum() * 2.5 * b, cy - 46.0 * s + hy + yb + ay, 4.5 * b, 5.5 * s, 0.0, shade(skin, 0.82), alpha * 0.8, false);
    let eye = Part::diamond(cx + hx * 1.6 + ax, cy - 51.0 * s + hy + yb + ay, 1.6 * b, 2.0 * s, 0.0, [0.12, 0.09, 0.08], alpha * 0.9, false);

    let mut parts = vec![foot_l, foot_r, leg_l, leg_r, torso, torso_shade, belt_band, collar, arm_l, arm_r, cuff_l, cuff_r, hand_l, hand_r, head, jaw, eye];

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
            // Bare skull dome (one step off pure white so the hit-flash can
            // still brighten it), dark eye sockets, exposed ribs.
            parts.push(Part::diamond(cx + hx + ax, cy - 58.0 * s + hy + yb + ay, 9.5 * b, 6.0 * s, 0.0, shade(skin, 0.97), alpha, true));
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
