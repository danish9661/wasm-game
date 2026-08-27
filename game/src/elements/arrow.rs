//! Arrow: a shaft oriented along `facing` (world dir) with an arrowhead.
//! Emitted directly (flat style — no 2.5D skirt).

pub(crate) fn draw(
    out: &mut Vec<f32>,
    cx: f32,
    cy: f32,
    color: [f32; 3],
    alpha: f32,
    facing: (f32, f32),
) {
    // world dir -> screen dir: (dx-dy, dx+dy); y is up so negate.
    let sx = facing.0 - facing.1;
    let sy = -(facing.0 + facing.1);
    let len = (sx * sx + sy * sy).sqrt();
    let (ux, uy) = if len < 1e-4 { (1.0, 0.0) } else { (sx / len, sy / len) };
    let shaft = 11.0;
    let tail = (cx - ux * shaft, cy - uy * shaft);
    let head = (cx + ux * shaft, cy + uy * shaft);
    let nx = -uy;
    let ny = ux;
    let hw = 2.0;
    // Motion trail: a faint streak trailing behind the arrow so shots read as
    // fast-moving even in a still frame.
    let trail_tail = (cx - ux * (shaft + 12.0), cy - uy * (shaft + 12.0));
    let tverts = [
        (trail_tail.0 - nx * (hw * 0.5), trail_tail.1 - ny * (hw * 0.5)),
        (trail_tail.0 + nx * (hw * 0.5), trail_tail.1 + ny * (hw * 0.5)),
        (tail.0 + nx * hw, tail.1 + ny * hw),
        (trail_tail.0 - nx * (hw * 0.5), trail_tail.1 - ny * (hw * 0.5)),
        (tail.0 + nx * hw, tail.1 + ny * hw),
        (tail.0 - nx * hw, tail.1 - ny * hw),
    ];
    for (vx, vy) in tverts {
        out.push(vx);
        out.push(vy);
        out.extend_from_slice(&color);
        out.push(alpha * 0.22);
    }
    // shaft as a thin quad (two triangles)
    let verts = [
        (tail.0 - nx * hw, tail.1 - ny * hw),
        (tail.0 + nx * hw, tail.1 + ny * hw),
        (head.0 + nx * hw, head.1 + ny * hw),
        (tail.0 - nx * hw, tail.1 - ny * hw),
        (head.0 + nx * hw, head.1 + ny * hw),
        (head.0 - nx * hw, head.1 - ny * hw),
    ];
    for (vx, vy) in verts {
        out.push(vx);
        out.push(vy);
        out.extend_from_slice(&color);
        out.push(alpha);
    }
    // arrowhead: a small diamond at the tip
    let hcx = head.0 + ux * 4.0;
    let hcy = head.1 + uy * 4.0;
    let tip = [
        (hcx, hcy - 5.0),
        (hcx + 4.0, hcy),
        (hcx, hcy + 5.0),
        (hcx, hcy - 5.0),
        (hcx, hcy + 5.0),
        (hcx - 4.0, hcy),
    ];
    for (vx, vy) in tip {
        out.push(vx);
        out.push(vy);
        out.extend_from_slice(&color);
        out.push(alpha);
    }
}
