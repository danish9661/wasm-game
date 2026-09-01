//! Raider: a night-stalking humanoid bandit with a dark cloak and dagger.

use crate::elements::prim::{anim_seed, facing_offset, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let skin = [0.75, 0.58, 0.45];
    let cloak = color;
    let dark = shade(cloak, 0.6);
    let legs = [0.20, 0.15, 0.12];
    let (hx, hy) = facing_offset(facing, 3.0);
    let seed = anim_seed(cx, cy);
    let phase = anim_time * 6.0 + seed;
    let swing = (phase).sin() * 0.3;
    let bob = (phase * 2.0).sin().abs() * 0.8;
    // Cloak flutters
    let flutter = (anim_time * 4.0 + seed).sin() * 1.5;
    let (ax, ay) = facing_offset(facing, 2.0);

    let mut parts = vec![
        // Legs
        Part::vquad(cx - 3.0 + swing * 3.0, cy - 14.0, 1.5, 14.0, legs, alpha, true),
        Part::vquad(cx + 3.0 - swing * 3.0, cy - 14.0, 1.5, 14.0, legs, alpha, true),
        // Torso
        Part::vquad(cx, cy - 32.0 + bob, 4.0, 18.0, cloak, alpha, true),
        // Arms
        Part::vquad(cx - 6.0 + swing * 2.0, cy - 30.0 + bob, 1.25, 13.0, cloak, alpha, true),
        Part::vquad(cx + 3.5 - swing * 2.0, cy - 30.0 + bob, 1.25, 13.0, cloak, alpha, true),
        // Hands
        Part::diamond(cx - 6.0, cy - 16.0 + bob, 2.0, 2.5, 0.0, skin, alpha, true),
        Part::diamond(cx + 3.5, cy - 16.0 + bob, 2.0, 2.5, 0.0, skin, alpha, true),
        // Head (masked, only lower face visible)
        Part::diamond(cx + hx, cy - 41.0 + hy + bob, 7.0, 9.0, 0.0, skin, alpha, true),
        // Hood/mask
        Part::diamond(cx + hx, cy - 46.0 + hy + bob, 7.5, 5.0, 0.0, dark, alpha, true),
        Part::diamond(cx + hx, cy - 40.0 + hy + bob, 7.0, 3.0, 0.0, dark, alpha, true),
        // Dagger in hand
        Part::vquad(cx + 5.0 + ax * 3.0, cy - 16.0 + bob + ay * 2.0, 1.0, 8.0, [0.60, 0.60, 0.65], alpha, true),
        Part::diamond(cx + 5.0 + ax * 3.0, cy - 12.0 + bob + ay * 2.0, 1.5, 1.5, 0.0, [0.70, 0.70, 0.75], alpha, true),
    ];

    // Trailing cloak edges
    parts.push(Part::diamond(cx - 4.0 - flutter, cy - 28.0 + bob, 4.0, 6.0, 0.0, dark, alpha * 0.8, true));
    parts.push(Part::diamond(cx + 4.0 + flutter, cy - 26.0 + bob, 3.0, 5.0, 0.0, dark, alpha * 0.7, true));

    parts
}
