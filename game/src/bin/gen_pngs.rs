//! Offline tool: render every game element to a transparent PNG, plus a
//! combined sprite-sheet montage.
//!
//! Run with: `cargo run -p game --bin gen_pngs`
//! Writes into `../web/static/element_previews/` (relative to this crate's
//! dir): one PNG per element, and `spritesheet.png`.
//!
//! Vertex buffers come from `game::elements::preview_elements`, which calls
//! each element's `build()` and flattens the triangles (x, y, r, g, b, a).

use std::fs;
use std::io::BufWriter;
use std::path::Path;

fn main() {
    let out_dir = Path::new("element_previews");
    fs::create_dir_all(out_dir).expect("create previews dir");

    let elements = game::elements::preview_elements();
    let mut imgs: Vec<(String, usize, usize, Vec<u8>)> = Vec::new();
    let mut written = 0;
    for (name, verts) in elements {
        if let Some((w, h, rgba)) = rasterize_png(&verts) {
            let path = out_dir.join(format!("{name}.png"));
            write_png(&path, w, h, &rgba);
            imgs.push((name.clone(), w, h, rgba));
            written += 1;
            println!("  {name}: {}x{}", w, h);
        } else {
            eprintln!("  {name}: empty (skipped)");
        }
    }
    println!("Wrote {written} element PNGs to {}", out_dir.display());

    let sheet = build_montage(&imgs);
    let sheet_path = out_dir.join("spritesheet.png");
    write_png(&sheet_path, sheet.0, sheet.1, &sheet.2);
    println!(
        "Wrote spritesheet.png ({}x{}) — {} elements",
        sheet.0, sheet.1, imgs.len()
    );
}

/// Triangle-fill the vertex buffer into an RGBA image cropped to its bbox.
fn rasterize_png(verts: &[f32]) -> Option<(usize, usize, Vec<u8>)> {
    if verts.len() % 6 != 0 {
        return None;
    }
    let nv = verts.len() / 6;
    if nv < 3 {
        return None;
    }
    let mut minx = f32::INFINITY;
    let mut miny = f32::INFINITY;
    let mut maxx = f32::NEG_INFINITY;
    let mut maxy = f32::NEG_INFINITY;
    for v in 0..nv {
        let x = verts[v * 6];
        let y = verts[v * 6 + 1];
        minx = minx.min(x);
        miny = miny.min(y);
        maxx = maxx.max(x);
        maxy = maxy.max(y);
    }
    if !minx.is_finite() || !maxx.is_finite() {
        return None;
    }
    let pad = 6.0;
    let w = (maxx - minx + 2.0 * pad).ceil() as usize;
    let h = (maxy - miny + 2.0 * pad).ceil() as usize;
    if w == 0 || h == 0 {
        return None;
    }
    let ox = minx - pad;
    let oy = miny - pad;
    let mut buf: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 0.0]; w * h];

    let tri = nv / 3;
    for t in 0..tri {
        let a = t * 3;
        let (ax, ay, ar, ag, ab, aa) = v(verts, a);
        let (bx, by, _br, _bg, _bb, _ba) = v(verts, a + 1);
        let (cx2, cy2, _cr, _cg, _cb, _ca) = v(verts, a + 2);
        let ax = ax - ox;
        let ay = ay - oy;
        let bx = bx - ox;
        let by = by - oy;
        let cx2 = cx2 - ox;
        let cy2 = cy2 - oy;
        let min_tx = (ax.min(bx).min(cx2).floor() as i32).max(0).min(w as i32);
        let max_tx = (ax.max(bx).max(cx2).ceil() as i32).max(0).min(w as i32);
        let min_ty = (ay.min(by).min(cy2).floor() as i32).max(0).min(h as i32);
        let max_ty = (ay.max(by).max(cy2).ceil() as i32).max(0).min(h as i32);
        let denom = (by - cy2) * (ax - cx2) + (cx2 - bx) * (ay - cy2);
        if denom.abs() < 1e-6 {
            continue;
        }
        for py in min_ty..max_ty {
            for px in min_tx..max_tx {
                let px = px as f32 + 0.5;
                let py = py as f32 + 0.5;
                let l1 = ((by - cy2) * (px - cx2) + (cx2 - bx) * (py - cy2)) / denom;
                let l2 = ((cy2 - ay) * (px - cx2) + (ax - cx2) * (py - cy2)) / denom;
                let l3 = 1.0 - l1 - l2;
                if l1 < -0.001 || l2 < -0.001 || l3 < -0.001 {
                    continue;
                }
                let idx = py as usize * w + px as usize;
                blend(&mut buf[idx], ar, ag, ab, aa);
            }
        }
    }

    let mut rgba = Vec::with_capacity(w * h * 4);
    for p in &buf {
        rgba.push((p[0].clamp(0.0, 1.0) * 255.0).round() as u8);
        rgba.push((p[1].clamp(0.0, 1.0) * 255.0).round() as u8);
        rgba.push((p[2].clamp(0.0, 1.0) * 255.0).round() as u8);
        rgba.push((p[3].clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    Some((w, h, rgba))
}

/// Pack every element into a single transparent grid image.
fn build_montage(imgs: &[(String, usize, usize, Vec<u8>)]) -> (usize, usize, Vec<u8>) {
    let cols = 9;
    let rows = (imgs.len() + cols - 1) / cols;
    let cell = 80usize; // square cell; elements are centred within
    let w = cols * cell;
    let h = rows * cell;
    let mut buf: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 0.0]; w * h];
    for (i, (_, iw, ih, data)) in imgs.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let ox = col * cell + (cell.saturating_sub(*iw)) / 2;
        let oy = row * cell + (cell.saturating_sub(*ih)) / 2;
        for y in 0..*ih {
            for x in 0..*iw {
                let sa = data[(y * *iw + x) * 4 + 3] as f32 / 255.0;
                if sa <= 0.0 {
                    continue;
                }
                let dx = ox + x;
                let dy = oy + y;
                if dx >= w || dy >= h {
                    continue;
                }
                let sr = data[(y * *iw + x) * 4] as f32 / 255.0;
                let sg = data[(y * *iw + x) * 4 + 1] as f32 / 255.0;
                let sb = data[(y * *iw + x) * 4 + 2] as f32 / 255.0;
                blend(&mut buf[dy * w + dx], sr, sg, sb, sa);
            }
        }
    }
    let mut rgba = Vec::with_capacity(w * h * 4);
    for p in &buf {
        rgba.push((p[0].clamp(0.0, 1.0) * 255.0).round() as u8);
        rgba.push((p[1].clamp(0.0, 1.0) * 255.0).round() as u8);
        rgba.push((p[2].clamp(0.0, 1.0) * 255.0).round() as u8);
        rgba.push((p[3].clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    (w, h, rgba)
}

#[inline]
fn v(v: &[f32], i: usize) -> (f32, f32, f32, f32, f32, f32) {
    (
        v[i * 6],
        v[i * 6 + 1],
        v[i * 6 + 2],
        v[i * 6 + 3],
        v[i * 6 + 4],
        v[i * 6 + 5],
    )
}

/// Alpha-composite `src` over `dst` (both straight alpha).
#[inline]
fn blend(dst: &mut [f32; 4], r: f32, g: f32, b: f32, a: f32) {
    let a = a.clamp(0.0, 1.0);
    let out_a = a + dst[3] * (1.0 - a);
    if out_a <= 0.0 {
        *dst = [0.0, 0.0, 0.0, 0.0];
        return;
    }
    dst[0] = (r * a + dst[0] * dst[3] * (1.0 - a)) / out_a;
    dst[1] = (g * a + dst[1] * dst[3] * (1.0 - a)) / out_a;
    dst[2] = (b * a + dst[2] * dst[3] * (1.0 - a)) / out_a;
    dst[3] = out_a;
}

fn write_png(path: &Path, w: usize, h: usize, rgba: &[u8]) {
    let file = fs::File::create(path).expect("create png");
    let wtr = BufWriter::new(file);
    let mut enc = png::Encoder::new(wtr, w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().expect("png header");
    writer.write_image_data(rgba).expect("png data");
}
