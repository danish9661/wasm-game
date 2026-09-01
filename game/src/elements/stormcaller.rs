//! Stormcaller: a flying storm-mage that drifts over walls. A robed figure
//! with crackling lightning wisps and glowing storm eyes.

use crate::elements::prim::{anim_seed, shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    let robe = color;
    let dark = shade(robe, 0.7);
    let lightning = [0.70, 0.85, 1.0];
    let eye = [0.60, 0.80, 1.0];
    let seed = anim_seed(cx, cy);
    // Floating bob
    let bob = (anim_time * 2.5 + seed).sin() * 3.0;
    // Lightning flicker
    let flicker = (anim_time * 8.0 + seed).sin().max(0.0);

    let mut parts = vec![
        // Tapered robe body (wide at bottom, narrow at shoulders)
        Part::diamond(cx, cy - 2.0 + bob, 10.0, 8.0, 0.0, dark, alpha, true),
        Part::diamond(cx, cy - 10.0 + bob, 8.0, 8.0, 0.0, robe, alpha, true),
        Part::diamond(cx, cy - 18.0 + bob, 6.0, 5.0, 0.0, shade(robe, 1.1), alpha, true),
        // Hooded head
        Part::diamond(cx, cy - 22.0 + bob, 6.0, 5.0, 0.0, robe, alpha, true),
        // Hood point
        Part::diamond(cx, cy - 27.0 + bob, 4.0, 3.0, 0.0, dark, alpha, true),
        // Glowing eyes
        Part::diamond(cx - 2.5, cy - 22.0 + bob, 1.5, 1.5, 0.0, eye, alpha, true),
        Part::diamond(cx + 2.5, cy - 22.0 + bob, 1.5, 1.5, 0.0, eye, alpha, true),
        // Outstretched arms
        Part::vquad(cx - 10.0, cy - 18.0 + bob, 2.0, 10.0, robe, alpha, true),
        Part::vquad(cx + 10.0, cy - 18.0 + bob, 2.0, 10.0, robe, alpha, true),
    ];

    // Crackling lightning wisps (appear and disappear with flicker)
    if flicker > 0.2 {
        let a = alpha * flicker;
        parts.push(Part::diamond(cx - 13.0, cy - 10.0 + bob, 3.0, 2.0, 0.0, lightning, a, false));
        parts.push(Part::diamond(cx + 13.0, cy - 10.0 + bob, 3.0, 2.0, 0.0, lightning, a, false));
    }
    if flicker > 0.5 {
        let a = alpha * (flicker - 0.5) * 2.0;
        parts.push(Part::diamond(cx - 10.0, cy - 14.0 + bob, 2.0, 1.5, 0.0, lightning, a, false));
        parts.push(Part::diamond(cx + 10.0, cy - 14.0 + bob, 2.0, 1.5, 0.0, lightning, a, false));
    }
    // Storm cloud halo above head
    parts.push(Part::diamond(cx, cy - 29.0 + bob, 8.0, 3.0, 0.0, shade(lightning, 0.7), alpha * 0.5, false));

    parts
}
