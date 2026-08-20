use crate::{TILE_HEIGHT, TILE_WIDTH};

pub const HALF_W: f32 = TILE_WIDTH / 2.0;
pub const HALF_H: f32 = TILE_HEIGHT / 2.0;

pub fn world_to_iso(x: f32, y: f32) -> (f32, f32) {
    let sx = (x - y) * HALF_W;
    let sy = (x + y) * HALF_H;
    (sx, sy)
}

pub fn iso_to_world(sx: f32, sy: f32) -> (f32, f32) {
    let x = (sy / HALF_H + sx / HALF_W) / 2.0;
    let y = (sy / HALF_H - sx / HALF_W) / 2.0;
    (x, y)
}

pub fn tile_screen_corner(tx: i32, ty: i32) -> (f32, f32) {
    world_to_iso(tx as f32, ty as f32)
}

pub fn depth_order(tx: i32, ty: i32) -> i32 {
    tx + ty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for (x, y) in [(0.0, 0.0), (3.0, -2.0), (-5.5, 7.25)] {
            let (sx, sy) = world_to_iso(x, y);
            let (rx, ry) = iso_to_world(sx, sy);
            assert!((rx - x).abs() < 0.001);
            assert!((ry - y).abs() < 0.001);
        }
    }

    #[test]
    fn orthogonality() {
        let (ax, ay) = world_to_iso(1.0, 0.0);
        let (bx, by) = world_to_iso(0.0, 1.0);
        assert!((ax + bx).abs() < 0.001, "x+1 and y+1 must mirror in sx");
        assert!((ay - by).abs() < 0.001, "x+1 and y+1 must share sy");
        assert!((ax).abs() > 0.001, "sx must change with x");
    }
}