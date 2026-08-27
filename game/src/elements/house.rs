//! Default world buildings (House / Cabin / Hut / Inn / Barn / Watchtower)
//! scattered through villages and the countryside so the world reads as
//! lived-in. Each is a substantial, proper building rather than a flat
//! silhouette: stone/wood walls, pitched or gambrel roofs, chimneys with
//! drifting smoke, lit windows and doorways that glow with a hint of the warm
//! interior beyond.

use crate::elements::prim::{shade, Part};

/// `kind`: 0 = house (stone, two storeys), 1 = cabin (wood), 2 = hut (small,
/// thatched), 3 = inn (broad tavern with a sign), 4 = barn (gambrel red barn),
/// 5 = watchtower (tall stone tower with a beacon).
pub(crate) fn build(
    kind: u8,
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _facing: (f32, f32),
    anim_time: f32,
) -> Vec<Part> {
    match kind {
        4 => barn(cx, cy, color, alpha, anim_time),
        5 => watchtower(cx, cy, color, alpha, anim_time),
        _ => cottage(kind, cx, cy, color, alpha, anim_time),
    }
}

/// House / Cabin / Hut / Inn: a walled building with course bands, lit windows,
/// a glowing doorway, a pitched roof and a smoking chimney.
fn cottage(
    kind: u8,
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    anim_time: f32,
) -> Vec<Part> {
    // (wall_w, wall_h, roof_h, roof_col, door_w, windows_per_floor, floors)
    let (wall_w, wall_h, roof_h, roof_col, door_w, win_per, floors) = match kind {
        0 => (46.0, 58.0, 36.0, [0.40, 0.27, 0.19], 10.0, 2, 2),
        1 => (38.0, 46.0, 30.0, [0.45, 0.28, 0.16], 8.0, 1, 1),
        2 => (30.0, 34.0, 24.0, [0.55, 0.45, 0.22], 6.0, 1, 1),
        // Inn: a broad, warm two-storey tavern.
        _ => (58.0, 52.0, 26.0, [0.34, 0.20, 0.16], 12.0, 2, 2),
    };
    let wall = color;
    let dark = shade(wall, 0.62);
    let lit = [1.0, 0.86, 0.46];
    let mut parts = Vec::new();

    // Foundation plinth.
    parts.push(Part::vquad(
        cx - wall_w / 2.0 - 2.0,
        cy - 5.0,
        wall_w / 2.0 + 2.0,
        5.0,
        dark,
        alpha,
        true,
    ));
    // Main wall block.
    parts.push(Part::vquad(
        cx - wall_w / 2.0,
        cy - wall_h,
        wall_w / 2.0,
        wall_h,
        wall,
        alpha,
        true,
    ));
    // Stone/timber courses.
    let courses = 5;
    for i in 1..courses {
        let yy = cy - wall_h + (wall_h * i as f32 / courses as f32);
        parts.push(Part::vquad(
            cx - wall_w / 2.0 + 1.0,
            yy,
            wall_w / 2.0 - 1.0,
            1.6,
            shade(wall, 0.82),
            alpha * 0.7,
            false,
        ));
    }
    // Lit windows per floor.
    for f in 0..floors {
        let fy = cy - wall_h * (0.42 + f as f32 * 0.40);
        for _ in 0..win_per {
            for side in [-1.0, 1.0] {
                let wx = cx + side * wall_w * 0.27;
                parts.push(Part::vquad(wx - 4.0, fy - 4.0, 4.0, 8.0, [0.22, 0.16, 0.10], alpha, false));
                parts.push(Part::vquad(wx - 3.0, fy - 3.0, 3.0, 6.0, lit, alpha, false));
                parts.push(Part::vquad(wx - 0.4, fy - 3.0, 0.4, 6.0, [0.22, 0.16, 0.10], alpha, false));
                parts.push(Part::vquad(wx - 3.0, fy - 0.4, 3.0, 0.4, [0.22, 0.16, 0.10], alpha, false));
            }
        }
    }
    // Glowing doorway with a hearth ember inside.
    let door_y = cy - door_w * 1.8;
    parts.push(Part::vquad(
        cx - door_w / 2.0,
        door_y,
        door_w / 2.0,
        door_w * 1.8,
        shade(wall, 0.4),
        alpha,
        true,
    ));
    parts.push(Part::vquad(
        cx - door_w / 2.0 + 1.5,
        door_y + 2.0,
        door_w / 2.0 - 1.5,
        door_w * 1.8 - 4.0,
        [0.45, 0.30, 0.12],
        alpha * 0.9,
        false,
    ));
    parts.push(Part::diamond(
        cx,
        door_y + door_w * 0.9,
        3.0,
        3.0,
        0.0,
        [1.0, 0.62, 0.22],
        alpha,
        false,
    ));
    // A hanging inn sign for kind 3.
    if kind == 3 {
        parts.push(Part::vquad(cx + wall_w / 2.0 - 2.0, cy - wall_h * 0.62, 1.5, 10.0, [0.30, 0.20, 0.12], alpha, true));
        parts.push(Part::vquad(cx + wall_w / 2.0 + 1.0, cy - wall_h * 0.58, 9.0, 7.0, [0.45, 0.30, 0.18], alpha, true));
        parts.push(Part::vquad(cx + wall_w / 2.0 + 2.0, cy - wall_h * 0.55, 7.0, 2.0, [0.95, 0.80, 0.40], alpha, false));
    }
    // Pitched roof.
    let roof_base_y = cy - wall_h;
    parts.push(Part::vquad(
        cx - wall_w / 2.0 - 5.0,
        roof_base_y - roof_h * 0.45,
        wall_w / 2.0 + 5.0,
        roof_h * 0.45,
        roof_col,
        alpha,
        true,
    ));
    parts.push(Part::diamond(
        cx,
        roof_base_y - roof_h,
        wall_w / 2.0 + 6.0,
        roof_h * 0.72,
        0.0,
        roof_col,
        alpha,
        true,
    ));
    parts.push(Part::vquad(
        cx - 1.0,
        roof_base_y - roof_h * 0.95,
        1.0,
        roof_h * 0.5,
        shade(roof_col, 1.25),
        alpha * 0.8,
        false,
    ));
    // Chimney + drifting smoke.
    let chx = cx + wall_w * 0.22;
    let chy = roof_base_y - roof_h * 0.7;
    parts.push(Part::vquad(chx - 3.0, chy - roof_h * 0.5, 3.0, roof_h * 0.5, [0.5, 0.3, 0.22], alpha, true));
    for i in 0..3 {
        let t = (anim_time * 0.6 + i as f32 * 0.5).fract();
        let sy = chy - 6.0 - t * 18.0;
        let sc = 2.0 + t * 3.5;
        parts.push(Part::diamond(
            chx + (t * 5.0 - 2.5),
            sy,
            sc,
            sc,
            0.0,
            [0.82, 0.82, 0.85],
            alpha * (1.0 - t) * 0.5,
            false,
        ));
    }
    parts
}

/// Red gambrel-roofed barn with big double doors and a hayloft window.
fn barn(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _anim_time: f32,
) -> Vec<Part> {
    let wall_w = 62.0;
    let wall_h = 44.0;
    let wall = color;
    let dark = shade(wall, 0.62);
    let roof_col = [0.38, 0.16, 0.14];
    let mut parts = Vec::new();

    parts.push(Part::vquad(cx - wall_w / 2.0 - 2.0, cy - 5.0, wall_w / 2.0 + 2.0, 5.0, dark, alpha, true));
    parts.push(Part::vquad(cx - wall_w / 2.0, cy - wall_h, wall_w / 2.0, wall_h, wall, alpha, true));
    // White trim boards.
    parts.push(Part::vquad(cx - wall_w / 2.0, cy - wall_h, 2.0, wall_h, [0.92, 0.88, 0.82], alpha, false));
    parts.push(Part::vquad(cx + wall_w / 2.0 - 2.0, cy - wall_h, 2.0, wall_h, [0.92, 0.88, 0.82], alpha, false));

    // Big double doors.
    let dw = 18.0;
    parts.push(Part::vquad(cx - dw / 2.0, cy - 26.0, dw / 2.0, 26.0, shade(wall, 0.5), alpha, true));
    parts.push(Part::vquad(cx - dw / 2.0 + 1.5, cy - 24.0, dw / 2.0 - 1.5, 24.0, [0.40, 0.16, 0.14], alpha * 0.9, false));
    parts.push(Part::vquad(cx - 0.6, cy - 26.0, 0.6, 26.0, [0.25, 0.10, 0.08], alpha, false));
    // Hayloft window.
    parts.push(Part::vquad(cx - 4.0, cy - wall_h + 6.0, 4.0, 8.0, [0.22, 0.16, 0.10], alpha, false));
    parts.push(Part::vquad(cx - 3.0, cy - wall_h + 7.0, 3.0, 6.0, [1.0, 0.86, 0.46], alpha, false));

    // Gambrel roof: an eave slab plus a wide apex diamond in barn-red.
    let roof_base_y = cy - wall_h;
    parts.push(Part::vquad(cx - wall_w / 2.0 - 4.0, roof_base_y - 12.0, wall_w / 2.0 + 4.0, 12.0, roof_col, alpha, true));
    parts.push(Part::diamond(cx, roof_base_y - 26.0, wall_w / 2.0 + 5.0, 22.0, 0.0, roof_col, alpha, true));
    parts
}

/// Tall stone watchtower: battlements, arrow slits, a stout door and a glowing
/// beacon at the top (it also emits a real light via `emits_light`).
fn watchtower(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    anim_time: f32,
) -> Vec<Part> {
    let wall_w = 26.0;
    let wall_h = 120.0;
    let wall = color;
    let dark = shade(wall, 0.62);
    let mut parts = Vec::new();

    parts.push(Part::vquad(cx - wall_w / 2.0 - 2.0, cy - 5.0, wall_w / 2.0 + 2.0, 5.0, dark, alpha, true));
    parts.push(Part::vquad(cx - wall_w / 2.0, cy - wall_h, wall_w / 2.0, wall_h, wall, alpha, true));
    // Course bands.
    let courses = 9;
    for i in 1..courses {
        let yy = cy - wall_h + (wall_h * i as f32 / courses as f32);
        parts.push(Part::vquad(cx - wall_w / 2.0 + 1.0, yy, wall_w / 2.0 - 1.0, 1.4, shade(wall, 0.82), alpha * 0.7, false));
    }
    // Arrow-slit windows up the shaft.
    for f in 0..3 {
        let fy = cy - wall_h * (0.30 + f as f32 * 0.22);
        parts.push(Part::vquad(cx - 1.5, fy - 5.0, 1.5, 10.0, [0.20, 0.18, 0.16], alpha, false));
    }
    // Stout door.
    parts.push(Part::vquad(cx - 5.0, cy - 16.0, 5.0, 16.0, shade(wall, 0.45), alpha, true));
    parts.push(Part::vquad(cx - 3.5, cy - 14.0, 3.5, 14.0, [0.35, 0.22, 0.14], alpha * 0.9, false));
    // Battlements (merlons) at the top.
    let top_y = cy - wall_h;
    for m in 0..4 {
        let mx = cx - wall_w / 2.0 + 2.0 + m as f32 * (wall_w - 4.0) / 3.0;
        parts.push(Part::vquad(mx - 2.0, top_y - 8.0, 2.0, 8.0, shade(wall, 1.1), alpha, true));
    }
    // Crenellated cap + glowing beacon.
    parts.push(Part::vquad(cx - wall_w / 2.0 - 2.0, top_y - 6.0, wall_w / 2.0 + 2.0, 6.0, shade(wall, 0.9), alpha, true));
    let beacon = (anim_time * 2.0).sin() * 0.5 + 0.5;
    parts.push(Part::diamond(cx, top_y - 12.0, 5.0, 6.0, 0.0, [1.0, 0.85, 0.45], alpha, false));
    parts.push(Part::diamond(cx, top_y - 12.0, 3.0 + beacon * 2.0, 4.0 + beacon * 2.0, 0.0, [1.0, 0.95, 0.7], alpha * (0.6 + 0.4 * beacon), false));
    parts
}
