//! Turret: a stone emplacement with a crossbow barrel. Auto-fires at enemies
//! within range (gameplay handled by the world; this only draws the silhouette).

use crate::elements::prim::shade;
use crate::elements::prim::Part;

pub(crate) fn build(
    cx: f32,
    cy: f32,
    _color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let stone = [0.50, 0.50, 0.55];
    let metal = [0.50, 0.52, 0.58];
    let edge = [0.82, 0.84, 0.90];
    vec![
        // wide stone base with a lighter cap ring
        Part::vquad(cx, cy - 6.0, 10.0, 7.0, shade(stone, 0.9), alpha, true),
        Part::diamond(cx, cy - 7.0, 10.0, 3.0, 0.0, shade(stone, 1.15), alpha, true),
        Part::vquad(cx, cy - 12.0, 6.0, 6.0, shade(stone, 1.05), alpha, true),
        // barrel pointing up with bright edges both sides
        Part::vquad(cx, cy - 22.0, 3.0, 12.0, metal, alpha, true),
        Part::vquad(cx - 3.0, cy - 22.0, 0.8, 12.0, edge, alpha, false),
        Part::vquad(cx + 2.2, cy - 22.0, 0.8, 12.0, edge, alpha, false),
        // muzzle tip
        Part::diamond(cx, cy - 30.0, 3.4, 3.4, 0.0, shade(metal, 1.3), alpha, true),
        // glowing sight
        Part::diamond(cx, cy - 14.0, 3.0, 3.0, 0.0, [1.0, 0.85, 0.40], alpha, true),
    ]
}
