//! Banner: a war banner on a tall pole — empowers nearby guards. The banner
//! cloth flutters in the wind, and the pole has a glowing gem topper.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let cloth = color;
    let dark = shade(cloth, 0.7);
    let pole = [0.50, 0.34, 0.18];
    let gem = [0.90, 0.85, 0.30];
    let seed = anim_seed(cx, cy);
    // Banner flutter
    let flutter = (anim_time * 3.0 + seed).sin() * 2.0;
    let pulse = (anim_time * 2.5 + seed).sin() * 0.3 + 0.7;

    vec![
        // Tall pole
        Part::vquad(cx, cy - 26.0, 2.0, 26.0, pole, alpha, true),
        // Banner cloth (fluttering diamond)
        Part::diamond(cx + flutter, cy - 20.0, 10.0, 8.0, 0.0, cloth, alpha, true),
        // Darker stripe on banner
        Part::diamond(cx + flutter, cy - 18.0, 8.0, 3.0, 0.0, dark, alpha * 0.8, true),
        // Pole cap / gem
        Part::diamond(cx, cy - 28.0, 3.0, 3.0, 0.0, gem, pulse, false),
        // Ground plate
        Part::diamond(cx, cy - 1.0, 5.0, 2.5, 0.0, [0.30, 0.24, 0.18], alpha, true),
    ]
}
