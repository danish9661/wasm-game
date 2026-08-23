//! Stoneslinger: a hooded stone-biome caster. A robed body with a shadowed
//! hood and a glowing cyan orb clutched in front (the rock it hurls).

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
    let dark = shade(color, 0.6);
    let orb = [0.55, 0.95, 1.0];
    let pulse = 0.7 + 0.3 * (anim_time * 6.0).sin().max(0.0);
    vec![
        // robe (tapered gown)
        Part::diamond(cx, cy - 1.0, 10.0, 12.0, 0.0, robe, alpha, true),
        Part::diamond(cx, cy - 11.0, 7.0, 8.0, 0.0, robe, alpha, true),
        // hood
        Part::diamond(cx, cy - 18.0, 6.0, 7.0, 0.0, dark, alpha, true),
        // shadowed face
        Part::diamond(cx, cy - 17.0, 3.0, 4.0, 0.0, shade(dark, 0.5), alpha, true),
        // glowing casting orb
        Part::diamond(cx, cy - 3.0, 3.2 * pulse, 3.2 * pulse, 0.0, orb, alpha, true),
    ]
}
