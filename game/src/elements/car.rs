//! Car: an abandoned old-world automobile — body + cabin + two wheels.

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
    // Wheels peek below the body (drawn first so the body overlaps their tops).
    p.push(Part::diamond(cx - hw * 0.6, cy + 1.0, 3.4, 3.4, -2.0, [0.08, 0.08, 0.09], alpha, false));
    p.push(Part::diamond(cx + hw * 0.6, cy + 1.0, 3.4, 3.4, -2.0, [0.08, 0.08, 0.09], alpha, false));
    p.push(Part::vquad(cx, cy - 14.0, hw, 14.0, color, alpha, true));
    p.push(Part::vquad(
        cx,
        cy - 26.0,
        hw * 0.62,
        12.0,
        [color[0] * 1.15, color[1] * 1.15, color[2] * 1.15],
        alpha,
        true,
    ));
    // Windshield + side windows catch the sky.
    p.push(Part::diamond(cx, cy - 24.0, 4.5, 3.0, 0.0, [0.62, 0.72, 0.80], alpha, false));
    // Roof highlight and bumpers break the flat red fill.
    p.push(Part::diamond(cx, cy - 30.0, hw * 0.4, 2.0, 0.0, [color[0] * 1.3, color[1] * 1.3, color[2] * 1.3], alpha, false));
    p.push(Part::vquad(cx - hw - 1.0, cy - 4.0, 1.5, 3.0, [0.25, 0.25, 0.28], alpha, true));
    p.push(Part::vquad(cx + hw - 0.5, cy - 4.0, 1.5, 3.0, [0.25, 0.25, 0.28], alpha, true));
    p
}
