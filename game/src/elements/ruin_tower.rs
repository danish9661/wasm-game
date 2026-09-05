//! RuinTower: a tall, broken stone watchtower — crenellated crown, a dark
//! window, and a jagged snap where the upper floors crumbled away. Atmospheric
//! world-gen flavor for the Stone biome.

use crate::elements::prim::{shade, Part};

pub(crate) fn build(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let stone = color;
    let dark = shade(stone, 0.6);
    let cap = shade(stone, 1.1);
    let parts = vec![
        // main shaft
        Part::vquad(cx, cy - 34.0, 11.0, 34.0, stone, alpha, true),
        // base plinth
        Part::vquad(cx, cy - 4.0, 13.0, 4.0, dark, alpha, true),
        // crenellated crown (broken — alternating merlons)
        Part::vquad(cx, cy - 40.0, 24.0, 6.0, cap, alpha, true),
        Part::vquad(cx - 7.0, cy - 46.0, 5.0, 6.0, cap, alpha, true),
        Part::vquad(cx + 2.0, cy - 46.0, 6.0, 6.0, cap, alpha, true),
        Part::vquad(cx + 12.0, cy - 46.0, 5.0, 6.0, cap, alpha, true),
        // dark window
        Part::vquad(cx, cy - 22.0, 4.0, 10.0, dark, alpha, true),
        // jagged crumble line partway up the shaft
        Part::diamond(cx + 9.0, cy - 18.0, 4.0, 5.0, 0.0, shade(stone, 0.85), alpha, true),
    ];
    parts
}
