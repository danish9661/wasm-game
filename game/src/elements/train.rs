//! Train: a rusted old-world locomotive car — long body, roof, lit window
//! strip, wheels.

use crate::elements::prim::Part;

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    hw: f32,
    _hh: f32,
) -> Vec<Part> {
    let mut p = Vec::new();
    // Bogies peek below the body so the wheels read instead of hiding.
    p.push(Part::diamond(cx - hw * 0.7, cy + 1.0, 3.6, 3.6, -2.0, [0.08, 0.08, 0.09], alpha, false));
    p.push(Part::diamond(cx + hw * 0.7, cy + 1.0, 3.6, 3.6, -2.0, [0.08, 0.08, 0.09], alpha, false));
    p.push(Part::vquad(cx, cy - 18.0, hw, 18.0, color, alpha, true));
    p.push(Part::vquad(
        cx,
        cy - 34.0,
        hw,
        8.0,
        [color[0] * 1.1, color[1] * 1.1, color[2] * 1.1],
        alpha,
        true,
    ));
    p.push(Part::vquad(cx, cy - 22.0, hw * 0.7, 6.0, [0.7, 0.8, 0.9], alpha, false));
    // Roof line, panel seams and an engine cab break the long box.
    p.push(Part::vquad(cx - hw, cy - 35.0, hw, 1.6, [color[0] * 1.35, color[1] * 1.35, color[2] * 1.35], alpha, false));
    for fx in [-0.5, 0.0, 0.5] {
        p.push(Part::vquad(cx + hw * fx - 0.5, cy - 26.0, 0.5, 14.0, [color[0] * 0.8, color[1] * 0.8, color[2] * 0.8], alpha, false));
    }
    p.push(Part::diamond(cx + hw * 0.85, cy - 28.0, 3.0, 5.0, 0.0, [0.75, 0.82, 0.88], alpha, false));
    p
}
