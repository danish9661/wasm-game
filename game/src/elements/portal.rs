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
    // Twin pillar feet root the gate so it reads as architecture, not a gem.
    parts.push(Part::vquad(cx - 11.0, cy - 8.0, 3.0, 10.0, shade(t, 0.5), alpha, true));
    parts.push(Part::vquad(cx + 11.0, cy - 8.0, 3.0, 10.0, shade(t, 0.5), alpha, true));
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
    // Rune dots orbiting the core mark it as a worked gate.
    let r1 = (anim_time * 1.2).sin() * 6.0;
    parts.push(Part::diamond(cx - 7.0, cy - 14.0 + r1, 1.6, 1.6, 0.0, [0.9, 0.98, 1.0], alpha, false));
    parts.push(Part::diamond(cx + 7.0, cy - 14.0 - r1, 1.6, 1.6, 0.0, [0.9, 0.98, 1.0], alpha, false));
    // Ground glow so it's visible from afar.
    parts.push(Part::diamond(cx, cy, 14.0, 6.0, 0.0, shade(t, 1.2), alpha * 0.55, false));
    parts
}
