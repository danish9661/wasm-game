//! Portal: a standing ring of arcane light you step through to reach the town.
//! It pulses gently so players notice it as a landmark.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let t = color;
    // Soft breathing pulse so the gate reads as "alive".
    let pulse = 0.5 + 0.5 * (anim_time * 2.0).sin();
    let mut parts = Vec::new();
    // Outer frame.
    parts.push(Part::diamond(cx, cy - 14.0, 13.0, 26.0, 0.0, shade(t, 0.65), alpha * 0.9, true));
    // Mid band.
    parts.push(Part::diamond(cx, cy - 14.0, 9.0, 20.0, 0.0, shade(t, 1.05), alpha, true));
    // Glowing core that brightens with the pulse.
    parts.push(Part::diamond(
        cx,
        cy - 14.0,
        5.0,
        13.0,
        0.0,
        [t[0] * 1.3, t[1] * 1.3, t[2] * 1.3],
        alpha.min(1.0) * (0.55 + 0.45 * pulse),
        true,
    ));
    // Ground glow so it's visible from afar.
    parts.push(Part::diamond(cx, cy, 11.0, 5.0, 0.0, shade(t, 1.2), alpha * 0.5, false));
    parts
}
