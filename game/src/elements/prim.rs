//! Shared primitive that turns element [`Part`]s into vertices.
//!
//! This is the ONLY module that knows about pixels, so a future texture atlas
//! (option B) is a localized change here: when `Part::uv` is `Some` we would
//! sample the atlas instead of using `color`.

use std::f32::consts::TAU;

/// One drawn piece of an element.
#[derive(Clone, Copy)]
pub struct Part {
    pub cx: f32,
    /// Vertical center (screen px). For a `VQuad` this is `top_y + height/2`.
    pub cy: f32,
    pub hw: f32,
    pub hh: f32,
    /// How far the piece's base is lifted off the ground (screen px).
    pub lift: f32,
    pub color: [f32; 3],
    pub alpha: f32,
    pub shape: Shape,
    /// Emit the dark 2.5D "skirt" copy for this piece.
    pub skirt: bool,
    /// Future texture-atlas rect (x, y, w, h). `None` = solid `color`. B-ready.
    pub uv: Option<[f32; 4]>,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Shape {
    /// Isometric diamond (matches the old `push_center_quad`).
    Diamond,
    /// Axis-aligned vertical rectangle (matches the old `push_vquad`).
    VQuad,
}

impl Part {
    pub(crate) fn diamond(
        cx: f32,
        cy: f32,
        hw: f32,
        hh: f32,
        lift: f32,
        color: [f32; 3],
        alpha: f32,
        skirt: bool,
    ) -> Self {
        Self {
            cx,
            cy,
            hw,
            hh,
            lift,
            color,
            alpha,
            shape: Shape::Diamond,
            skirt,
            uv: None,
        }
    }

    /// Build a `VQuad` from the old `(cx, top_y, hw, height)` convention.
    pub(crate) fn vquad(
        cx: f32,
        top_y: f32,
        hw: f32,
        height: f32,
        color: [f32; 3],
        alpha: f32,
        skirt: bool,
    ) -> Self {
        Self {
            cx,
            cy: top_y + height / 2.0,
            hw,
            hh: height / 2.0,
            lift: 0.0,
            color,
            alpha,
            shape: Shape::VQuad,
            skirt,
            uv: None,
        }
    }
}

/// Darken a color (used for the 2.5D skirt and shaded bases).
pub(crate) fn shade(color: [f32; 3], k: f32) -> [f32; 3] {
    [color[0] * k, color[1] * k, color[2] * k].map(|c| c.clamp(0.0, 1.0))
}

/// Convert a world-space facing into a screen-space lean offset.
pub(crate) fn facing_offset(facing: (f32, f32), amt: f32) -> (f32, f32) {
    let nx = facing.0 - facing.1;
    let ny = facing.0 + facing.1;
    let len = (nx * nx + ny * ny).sqrt();
    if len < 1e-4 {
        (0.0, 0.0)
    } else {
        (nx / len * amt, ny / len * amt)
    }
}

/// Turn parts into vertices. Emits the bright pass, then (for skirted parts)
/// the dark skirt pass followed by a second bright pass — matching the
/// original `push_styled_sprite` silhouette exactly.
pub(crate) fn rasterize(parts: &[Part], out: &mut Vec<f32>) {
    for p in parts {
        emit(out, p, false);
    }
    if parts.iter().any(|p| p.skirt) {
        for p in parts {
            if p.skirt {
                emit(out, p, true);
            }
        }
        for p in parts {
            emit(out, p, false);
        }
    }
}

fn emit(out: &mut Vec<f32>, p: &Part, dark: bool) {
    let _ = p.uv; // B-ready hook: ignored until atlas support lands.
    let color = if dark {
        [p.color[0] * 0.28, p.color[1] * 0.28, p.color[2] * 0.28]
    } else {
        p.color
    };
    // Thicker outline: dark skirt is slightly enlarged.
    let hw = if dark { p.hw * 1.22 } else { p.hw };
    let hh = if dark { p.hh * 1.22 } else { p.hh };
    match p.shape {
        Shape::Diamond => {
            let top = (p.cx, p.cy - hh + p.lift);
            let right = (p.cx + hw, p.cy + p.lift);
            let bottom = (p.cx, p.cy + hh + p.lift);
            let left = (p.cx - hw, p.cy + p.lift);
            let verts = [top, right, bottom, top, bottom, left];
            for (vx, vy) in verts {
                out.push(vx);
                out.push(vy);
                out.extend_from_slice(&color);
                out.push(p.alpha);
            }
        }
        Shape::VQuad => {
            let top = p.cy - hh + p.lift;
            let bot = p.cy + hh + p.lift;
            let verts = [
                (p.cx - hw, top),
                (p.cx + hw, top),
                (p.cx + hw, bot),
                (p.cx - hw, top),
                (p.cx + hw, bot),
                (p.cx - hw, bot),
            ];
            for (vx, vy) in verts {
                out.push(vx);
                out.push(vy);
                out.extend_from_slice(&color);
                out.push(p.alpha);
            }
        }
    }
}

/// Convenience for animation phase seeds (was inlined in the old code).
pub(crate) fn anim_seed(cx: f32, cy: f32) -> f32 {
    ((cx * 0.31 + cy * 0.17).fract().abs()) * TAU
}

/// Return a copy of `parts` with each part's color lerped toward white by `f`
/// (0..1) — used for the hit-flash that telegraphs damage on any figure.
pub(crate) fn flashed(parts: &[Part], f: f32) -> Vec<Part> {
    let f = f.clamp(0.0, 1.0);
    if f <= 0.0 {
        return parts.to_vec();
    }
    parts
        .iter()
        .map(|p| Part {
            color: [
                p.color[0] + (1.0 - p.color[0]) * f,
                p.color[1] + (1.0 - p.color[1]) * f,
                p.color[2] + (1.0 - p.color[2]) * f,
            ],
            ..*p
        })
        .collect()
}

/// Rasterize `parts` after applying a hit-flash (see `flashed`).
pub(crate) fn rasterize_flash(parts: &[Part], out: &mut Vec<f32>, f: f32) {
    rasterize(&flashed(parts, f), out);
}

/// Horizontal wind-sway offset (px) for foliage, phased per-instance so a
/// whole field doesn't lean in unison.
pub(crate) fn sway(cx: f32, cy: f32, anim_time: f32, amp: f32) -> f32 {
    (anim_time * 1.4 + anim_seed(cx, cy)).sin() * amp
}
