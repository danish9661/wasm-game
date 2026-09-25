//! Wall: a timbered room block — plastered face, dark timber sole plate,
//! and a pale limewashed cap that catches the room light. Interiors reuse
//! the surface wall art, so the ring reads as the same construction.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let plaster = color;
    let timber = shade(color, 0.52);
    let lime = [shade(color, 1.35)[0].min(0.95), shade(color, 1.35)[1].min(0.95), shade(color, 1.35)[2].min(0.95)];
    vec![
        // Timber sole plate grounding the block.
        Part::vquad(cx, cy - 4.0, 20.0, 4.0, timber, alpha, true),
        Part::vquad(cx, cy - 24.0, 20.0, 20.0, shade(plaster, 0.85), alpha, true),
        // Vertical stud so long runs read as framed bays, not one slab.
        Part::vquad(cx, cy - 24.0, 1.8, 20.0, timber, alpha * 0.8, false),
        // Limewashed cap catching the light.
        Part::diamond(cx, cy - 26.0, 20.0, 12.0, 0.0, lime, alpha, true),
        Part::diamond(cx, cy - 28.0, 12.0, 5.0, 0.0, shade(lime, 1.08), alpha * 0.9, false),
    ]
}
