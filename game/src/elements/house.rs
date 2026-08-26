//! Default world buildings (House / Cabin / Hut) sprinkled across the grasslands
//! so the map reads as a lived-in world rather than empty wilderness. Each is a
//! simple silhouette: a wall block, a pitched roof, a door and a lit window.

use crate::elements::prim::{shade, Part};

/// `kind`: 0 = house (stone, tall), 1 = cabin (wood, medium), 2 = hut (small).
pub(crate) fn build(
    kind: u8,
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    _anim_time: f32,
) -> Vec<Part> {
    let (wall_w, wall_h, roof_h, roof_col, door_w) = match kind {
        0 => (22.0, 28.0, 16.0, shade(color, 0.7), 5.0),
        1 => (18.0, 22.0, 13.0, [0.45, 0.28, 0.16], 4.0),
        _ => (14.0, 16.0, 11.0, [0.55, 0.40, 0.22], 3.0),
    };
    let wall = color;
    let dark = shade(wall, 0.65);
    let lit = [0.98, 0.86, 0.45];
    let mut parts = Vec::new();

    // Wall block (base sits on the tile).
    parts.push(Part::vquad(cx - wall_w / 2.0, cy - wall_h, wall_w / 2.0, wall_h, wall, alpha, true));
    // Slight base plinth for grounding.
    parts.push(Part::vquad(cx - wall_w / 2.0 - 2.0, cy - 4.0, wall_w / 2.0 + 2.0, 4.0, dark, alpha, true));
    // Pitched roof: a band then an apex diamond.
    let roof_base_y = cy - wall_h;
    parts.push(Part::vquad(cx - wall_w / 2.0 - 3.0, roof_base_y - roof_h * 0.5, wall_w / 2.0 + 3.0, roof_h * 0.5, roof_col, alpha, true));
    parts.push(Part::diamond(cx, roof_base_y - roof_h, wall_w / 2.0 + 4.0, roof_h * 0.65, 0.0, roof_col, alpha, true));
    // Door.
    parts.push(Part::vquad(cx - door_w / 2.0, cy - 11.0, door_w / 2.0, 11.0, dark, alpha, true));
    // Lit window.
    let wy = roof_base_y + wall_h * 0.42;
    parts.push(Part::vquad(cx + wall_w * 0.22, wy - 4.0, 4.5, 5.0, lit, alpha, true));
    parts
}
