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

/// House / Cabin / Hut / Inn: a walled building with a lit timber frame,
/// glowing windows, a hearth-lit doorway, a pitched roof with ridge shadow
/// and a smoking chimney. Walls carry vertical timber posts + a warm
/// lamplight wash spilling from the windows so facades read as shelter at
/// night, not flat color slabs.
fn cottage(
    kind: u8,
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    anim_time: f32,
) -> Vec<Part> {
    // (wall_w, wall_h, roof_h, roof_col, door_w, win_per, floors, lit_ratio)
    // `lit_ratio` is the share of windows burning warm at any moment: houses
    // glow in patchwork (some panes dark) so the village reads inhabited,
    // not floodlit. Roofs run dark against brightened walls so each house
    // reads as wall + lid instead of one mud slab; shutters below pick out
    // each home's color.
    let (wall_w, wall_h, roof_h, roof_col, door_w, win_per, floors, lit_ratio) = match kind {
        0 => (92.0, 110.0, 67.0, [0.30, 0.19, 0.13], 19.0, 2, 2, 0.7),
        1 => (74.0, 88.0, 56.0, [0.34, 0.20, 0.11], 15.0, 1, 1, 0.6),
        2 => (56.0, 65.0, 45.0, [0.46, 0.34, 0.15], 11.0, 1, 1, 0.5),
        // Inn: a broad, warm two-storey tavern — nearly every pane lit.
        _ => (110.0, 102.0, 50.0, [0.26, 0.15, 0.12], 22.0, 2, 2, 0.9),
    };
    // Shutter / door accent per home, so houses tell apart at a glance.
    let accent = match kind {
        0 => [0.55, 0.18, 0.14], // house: deep red
        1 => [0.20, 0.38, 0.20], // cabin: forest green
        2 => [0.20, 0.35, 0.55], // hut: ocean blue
        _ => [0.55, 0.38, 0.14], // inn: brass gold
    };
    let wall = color;
    let dark = shade(wall, 0.62);
    let lit = [1.0, 0.86, 0.46];
    // Hearth ember breathes: the doorway glows a touch brighter on a slow
    // cycle so entered homes feel occupied. Shared by door + threshold.
    let ember = 0.9 + 0.1 * (anim_time * 1.7).sin();
    let mut parts = Vec::new();

    // Foundation plinth.
    parts.push(Part::vquad(
        cx,
        cy - 5.0,
        wall_w / 2.0 + 2.0,
        5.0,
        dark,
        alpha,
        true,
    ));
    // Main wall block.
    parts.push(Part::vquad(
        cx,
        cy - wall_h,
        wall_w / 2.0,
        wall_h,
        wall,
        alpha,
        true,
    ));
    // Stone/timber courses: horizontal bands plus vertical timber posts at
    // the corners and center so the facade reads as framed construction.
    let courses = 5;
    for i in 1..courses {
        let yy = cy - wall_h + (wall_h * i as f32 / courses as f32);
        parts.push(Part::vquad(
            cx,
            yy,
            wall_w / 2.0 - 1.0,
            1.6,
            shade(wall, 0.82),
            alpha * 0.7,
            false,
        ));
    }
    let post_col = shade(wall, 0.55);
    for px in [-wall_w / 2.0 + 2.0, 0.0, wall_w / 2.0 - 2.0] {
        parts.push(Part::vquad(
            cx + px,
            cy - wall_h,
            1.6,
            wall_h,
            post_col,
            alpha * 0.75,
            false,
        ));
    }
    // Windows in patchwork: a deterministic per-pane roll against `lit_ratio`
    // decides burning vs dark, so each house keeps its own lived-in pattern
    // across frames (no flicker) while the street reads varied.
    // Lit panes get shutters + crossbars + a wash; dark panes get a cold
    // blue-grey glass that catches a little skylight.
    let dark_glass = [0.24, 0.30, 0.38];
    let mut pane: u32 = (kind as u32).wrapping_mul(97).wrapping_add(13);
    for f in 0..floors {
        let fy = cy - wall_h * (0.42 + f as f32 * 0.40);
        for _ in 0..win_per {
            for side in [-1.0, 1.0] {
                pane = pane.wrapping_mul(1664525).wrapping_add(1013904223);
                let lit_pane = (pane % 100) < (lit_ratio * 100.0) as u32;
                let wx = cx + side * wall_w * 0.27;
                parts.push(Part::vquad(wx - 5.7, fy - 4.0, 1.8, 8.0, accent, alpha, false));
                parts.push(Part::vquad(wx + 5.7, fy - 4.0, 1.8, 8.0, accent, alpha, false));
                parts.push(Part::vquad(wx, fy - 4.0, 4.0, 8.0, [0.22, 0.16, 0.10], alpha, false));
                if lit_pane {
                    parts.push(Part::vquad(wx, fy - 3.0, 3.0, 6.0, lit, alpha, false));
                    parts.push(Part::vquad(wx, fy - 3.0, 0.4, 6.0, [0.22, 0.16, 0.10], alpha, false));
                    parts.push(Part::vquad(wx, fy - 0.4, 3.0, 0.4, [0.22, 0.16, 0.10], alpha, false));
                    // Lamplight wash: soft warm pool beneath each burning window.
                    parts.push(Part::diamond(wx, fy + 8.0, 6.5, 4.5, 0.0, [1.0, 0.72, 0.32], alpha * 0.16, false));
                } else {
                    parts.push(Part::vquad(wx, fy - 3.0, 3.0, 6.0, dark_glass, alpha, false));
                    parts.push(Part::vquad(wx, fy - 3.0, 3.0, 1.2, shade(dark_glass, 1.5), alpha * 0.8, false));
                }
            }
        }
    }
    // Glowing doorway with an arched cap, a brighter hearth ember inside,
    // and a warm threshold wash pooling at the step.
    let door_y = cy - door_w * 1.8;
    parts.push(Part::vquad(
        cx,
        door_y,
        door_w / 2.0,
        door_w * 1.8,
        shade(wall, 0.4),
        alpha,
        true,
    ));
    parts.push(Part::diamond(
        cx,
        door_y - 1.0,
        door_w / 2.0 + 1.0,
        4.0,
        0.0,
        shade(wall, 0.55),
        alpha,
        false,
    ));
    parts.push(Part::vquad(
        cx,
        door_y + 2.0,
        door_w / 2.0 - 1.5,
        door_w * 1.8 - 4.0,
        [0.48, 0.32, 0.13],
        alpha * 0.9,
        false,
    ));
    parts.push(Part::diamond(
        cx,
        door_y + door_w * 0.9,
        4.2,
        4.2,
        0.0,
        [1.0 * ember, 0.68 * ember, 0.26 * ember],
        alpha,
        false,
    ));
    // Threshold wash: hearth light spilling out of the open door.
    parts.push(Part::diamond(
        cx,
        cy + 2.0,
        door_w / 2.0 + 4.0,
        4.5,
        0.0,
        [1.0 * ember, 0.70 * ember, 0.30 * ember],
        alpha * 0.18,
        false,
    ));
    // A hanging inn sign for kind 3: stout post, broad board with a gold
    // fascia so it reads at distance; tucked close to the wall.
    if kind == 3 {
        parts.push(Part::vquad(cx + wall_w / 2.0 - 0.5, cy - wall_h * 0.62, 1.5, 10.0, [0.30, 0.20, 0.12], alpha, true));
        parts.push(Part::vquad(cx + wall_w / 2.0 + 8.0, cy - wall_h * 0.58, 11.0, 9.0, [0.45, 0.30, 0.18], alpha, true));
        parts.push(Part::vquad(cx + wall_w / 2.0 + 7.0, cy - wall_h * 0.55, 9.0, 2.5, [0.95, 0.80, 0.40], alpha, false));
    }
    // Pitched roof, set off from the wall by a deep eave shadow so the lid
    // reads separately from the facade. A ridge highlight runs down the lit
    // slope and staggered shingle rows step down the dark slope so the roof
    // reads as tiled, not a flat triangle.
    let roof_base_y = cy - wall_h;
    parts.push(Part::vquad(
        cx,
        roof_base_y - 1.0,
        wall_w / 2.0 + 4.0,
        5.0,
        shade(wall, 0.35),
        alpha,
        false,
    ));
    parts.push(Part::vquad(
        cx,
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
        cx,
        roof_base_y - roof_h * 0.95,
        1.0,
        roof_h * 0.5,
        shade(roof_col, 1.25),
        alpha * 0.8,
        false,
    ));
    // Shingle rows: short staggered dashes stepping down the roof slope.
    let shingle = shade(roof_col, 0.72);
    let rows = 3;
    for r in 0..rows {
        let ry = roof_base_y - roof_h * (0.30 + r as f32 * 0.20);
        let half = (wall_w / 2.0 + 5.0) * (0.55 + r as f32 * 0.18);
        let off = if r % 2 == 0 { 0.0 } else { 3.0 };
        let mut x = -half + off;
        while x < half {
            parts.push(Part::vquad(cx + x, ry, 2.6, 1.4, shingle, alpha * 0.65, false));
            x += 6.5;
        }
    }
    // Chimney + drifting smoke.
    let chx = cx + wall_w * 0.22;
    let chy = roof_base_y - roof_h * 0.7;
    parts.push(Part::vquad(chx, chy - roof_h * 0.5, 3.0, roof_h * 0.5, [0.5, 0.3, 0.22], alpha, true));
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

/// Red gambrel-roofed barn with big double doors, a hayloft window, timber
/// framing and a warm loft glow.
fn barn(
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    _anim_time: f32,
) -> Vec<Part> {
    let wall_w = 116.0;
    let wall_h = 82.0;
    let wall = color;
    let dark = shade(wall, 0.62);
    let roof_col = [0.38, 0.16, 0.14];
    let mut parts = Vec::new();

    parts.push(Part::vquad(cx, cy - 5.0, wall_w / 2.0 + 2.0, 5.0, dark, alpha, true));
    parts.push(Part::vquad(cx, cy - wall_h, wall_w / 2.0, wall_h, wall, alpha, true));
    // White trim boards + vertical timber posts so the red wall isn't flat.
    parts.push(Part::vquad(cx - wall_w / 2.0, cy - wall_h, 2.0, wall_h, [0.92, 0.88, 0.82], alpha, false));
    parts.push(Part::vquad(cx + wall_w / 2.0, cy - wall_h, 2.0, wall_h, [0.92, 0.88, 0.82], alpha, false));
    for px in [-wall_w / 4.0, 0.0, wall_w / 4.0] {
        parts.push(Part::vquad(cx + px, cy - wall_h, 1.8, wall_h, shade(wall, 0.7), alpha * 0.7, false));
    }

    // Big double doors.
    let dw = 25.0;
    parts.push(Part::vquad(cx, cy - 36.0, dw / 2.0, 36.0, shade(wall, 0.5), alpha, true));
    parts.push(Part::vquad(cx, cy - 34.0, dw / 2.0 - 2.0, 34.0, [0.40, 0.16, 0.14], alpha * 0.9, false));
    parts.push(Part::vquad(cx, cy - 36.0, 0.8, 36.0, [0.25, 0.10, 0.08], alpha, false));
    // Hayloft window with a warm lamplight wash spilling down the wall.
    parts.push(Part::vquad(cx, cy - wall_h + 8.0, 5.5, 11.0, [0.22, 0.16, 0.10], alpha, false));
    parts.push(Part::vquad(cx, cy - wall_h + 9.5, 4.2, 8.5, [1.0, 0.86, 0.46], alpha, false));
    parts.push(Part::diamond(cx, cy - wall_h + 26.0, 8.0, 6.0, 0.0, [1.0, 0.72, 0.32], alpha * 0.16, false));

    // Gambrel roof: an eave slab plus a wide apex diamond in barn-red, with
    // shingle rows stepping down both faces.
    let roof_base_y = cy - wall_h;
    parts.push(Part::vquad(cx, roof_base_y - 17.0, wall_w / 2.0 + 5.5, 17.0, roof_col, alpha, true));
    parts.push(Part::diamond(cx, roof_base_y - 36.0, wall_w / 2.0 + 7.0, 31.0, 0.0, roof_col, alpha, true));
    let shingle = shade(roof_col, 0.7);
    let mut x = -wall_w / 2.0;
    while x < wall_w / 2.0 {
        parts.push(Part::vquad(cx + x, roof_base_y - 22.0, 2.6, 1.4, shingle, alpha * 0.65, false));
        x += 7.0;
    }
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
    let wall_w = 45.0;
    let wall_h = 218.0;
    let wall = color;
    let dark = shade(wall, 0.62);
    let mut parts = Vec::new();

    parts.push(Part::vquad(cx, cy - 5.0, wall_w / 2.0 + 2.0, 5.0, dark, alpha, true));
    parts.push(Part::vquad(cx, cy - wall_h, wall_w / 2.0, wall_h, wall, alpha, true));
    // Lit side seam: a warm vertical wash down the lit edge so the shaft has
    // volume instead of one flat stone slab.
    parts.push(Part::vquad(
        cx - wall_w / 2.0 + 2.5,
        cy - wall_h,
        2.2,
        wall_h,
        [1.0, 0.78, 0.42],
        alpha * 0.10,
        false,
    ));
    // Course bands.
    let courses = 9;
    for i in 1..courses {
        let yy = cy - wall_h + (wall_h * i as f32 / courses as f32);
        parts.push(Part::vquad(cx, yy, wall_w / 2.0 - 1.0, 1.4, shade(wall, 0.82), alpha * 0.7, false));
    }
    // Arrow-slit windows up the shaft.
    for f in 0..3 {
        let fy = cy - wall_h * (0.30 + f as f32 * 0.22);
        parts.push(Part::vquad(cx, fy - 5.0, 1.5, 10.0, [0.20, 0.18, 0.16], alpha, false));
    }
    // Stout door with a warm seam of interior light around the frame.
    parts.push(Part::vquad(cx, cy - 22.0, 7.0, 22.0, shade(wall, 0.45), alpha, true));
    parts.push(Part::vquad(cx, cy - 20.0, 5.0, 20.0, [0.35, 0.22, 0.14], alpha * 0.9, false));
    parts.push(Part::diamond(cx, cy + 2.0, 9.0, 4.0, 0.0, [1.0, 0.70, 0.30], alpha * 0.15, false));
    // Battlements (merlons) at the top.
    let top_y = cy - wall_h;
    for m in 0..4 {
        let mx = cx - wall_w / 2.0 + 2.5 + m as f32 * (wall_w - 5.0) / 3.0;
        parts.push(Part::vquad(mx, top_y - 11.0, 2.5, 11.0, shade(wall, 1.1), alpha, true));
    }
    // Crenellated cap + glowing beacon.
    parts.push(Part::vquad(cx, top_y - 8.5, wall_w / 2.0 + 2.5, 8.5, shade(wall, 0.9), alpha, true));
    let beacon = (anim_time * 2.0).sin() * 0.5 + 0.5;
    parts.push(Part::diamond(cx, top_y - 17.0, 7.0, 8.5, 0.0, [1.0, 0.85, 0.45], alpha, false));
    parts.push(Part::diamond(cx, top_y - 17.0, 4.2 + beacon * 2.8, 5.6 + beacon * 2.8, 0.0, [1.0, 0.95, 0.7], alpha * (0.6 + 0.4 * beacon), false));
    parts
}
