//! Health bar: a dark backplate with a lighter inset (HpBack) and a colored
//! fill (HpFill). Drawn flat (no 2.5D skirt).

use crate::elements::prim::Part;

pub(crate) fn back(
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
    lift: f32,
    _color: [f32; 3],
    alpha: f32,
) -> Vec<Part> {
    let top = cy - lift - hh;
    vec![
        Part::vquad(cx, top, hw, 2.0 * hh, [0.06, 0.06, 0.08], alpha, false),
        Part::vquad(cx, top, hw - 1.0, 2.0 * hh - 1.0, [0.18, 0.18, 0.22], alpha, false),
    ]
}

pub(crate) fn fill(
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
    lift: f32,
    color: [f32; 3],
    alpha: f32,
) -> Vec<Part> {
    let top = cy - lift - hh;
    vec![Part::vquad(cx, top, hw, 2.0 * hh, color, alpha, false)]
}
