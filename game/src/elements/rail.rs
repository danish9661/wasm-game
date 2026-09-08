//! Rail: a railway tie bed with two steel rails running through it.

use crate::elements::prim::Part;

pub(crate) fn build(
    cx: f32,
    cy: f32,
    _color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    hw: f32,
    hh: f32,
) -> Vec<Part> {
    let mut p = Vec::new();
    p.push(Part::diamond(cx, cy, hw, hh.max(2.0), 0.0, [0.22, 0.2, 0.16], alpha, false));
    p.push(Part::diamond(cx, cy - 2.0, hw * 0.85, 1.4, 0.0, [0.6, 0.6, 0.66], alpha, false));
    p.push(Part::diamond(cx, cy + 2.0, hw * 0.85, 1.4, 0.0, [0.6, 0.6, 0.66], alpha, false));
    p
}
