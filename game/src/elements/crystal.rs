//! Crystal: a cluster of faceted cyan shards.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let dark = shade(color, 0.7);
    vec![
        Part::diamond(cx - 5.0, cy - 4.0, 4.0, 8.0, 0.0, dark, alpha, true),
        Part::diamond(cx + 5.0, cy - 4.0, 4.0, 8.0, 0.0, dark, alpha, true),
        Part::diamond(cx, cy - 10.0, 5.0, 14.0, 0.0, color, alpha, true),
        Part::diamond(cx, cy - 16.0, 2.0, 6.0, 0.0, shade(color, 1.2), alpha, true),
    ]
}
