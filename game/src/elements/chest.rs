//! Chest: wooden box + lid + dark lock.

use crate::elements::prim::Part;

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let wood = [0.45, 0.30, 0.14];
    let lid = color; // gold-brown from StructureKind::Chest
    vec![
        Part::vquad(cx, cy - 18.0, 14.0, 18.0, wood, alpha, true),
        Part::diamond(cx, cy - 20.0, 14.0, 6.0, 0.0, lid, alpha, true),
        Part::diamond(cx, cy - 10.0, 3.0, 3.0, 0.0, [0.10, 0.09, 0.07], alpha, true),
    ]
}
