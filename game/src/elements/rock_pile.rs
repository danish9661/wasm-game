//! Rock pile: a few gray stones stacked loosely.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let r = color;
    vec![
        // Three separated lobes with notches between them + a glint per lobe
        // so the pile reads as a stack, not one single rock.
        Part::diamond(cx - 7.0, cy - 1.0, 8.0, 6.0, 0.0, shade(r, 0.7), alpha, true),
        Part::diamond(cx + 7.0, cy - 3.0, 7.0, 5.0, 0.0, shade(r, 0.9), alpha, true),
        Part::diamond(cx, cy - 8.0, 6.0, 5.0, 0.0, shade(r, 1.1), alpha, true),
        Part::diamond(cx - 9.0, cy - 3.0, 2.0, 2.0, 0.0, shade(r, 1.4), alpha, true),
        Part::diamond(cx + 5.0, cy - 10.0, 2.0, 2.0, 0.0, shade(r, 1.4), alpha, true),
    ]
}
