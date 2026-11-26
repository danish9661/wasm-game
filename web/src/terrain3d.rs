//! Terrain heightfield for the 3D vertical slice.
//!
//! Reads the *same* procedural world state the 2.5D path uses (`WorldGen`,
//! `ChunkCache`, `TileEdits`) and emits a flat-shaded XZ grid with per-biome
//! vertex colours plus short side skirts where a tile sits above a neighbour.
//! Output uses `model::MODEL_VERTEX_FLOATS` (pos/normal/colour) so it can be
//! uploaded through `model::upload_geometry()` and drawn by `pipeline3d`.

use crate::model::ParsedGeometry;
use game::render::tile_height_at;
use game::world::{edited_tile, ChunkCache, TileEdits, WorldGen};

/// World units of vertical relief per terrain height level. `1 tile = 1 unit`
/// in XZ; keeping hills modest keeps props readable at the isometric angle.
pub const HEIGHT_SCALE: f32 = 0.22;

fn height_at(world: &WorldGen, chunks: &mut ChunkCache, tx: i32, ty: i32) -> f32 {
    tile_height_at(world, chunks, tx, ty) as f32 * HEIGHT_SCALE
}

fn variation(tx: i32, ty: i32) -> f32 {
    // Deterministic small lightness jitter so large biomes are not a flat wash.
    let h = tx.wrapping_mul(73856093).wrapping_add(ty.wrapping_mul(19349663));
    let n = h.unsigned_abs() % 100;
    0.92 + (n as f32) / 100.0 * 0.12
}

/// Build a square heightfield centred on `(cx, cy)` in tile coordinates.
/// `radius` is the half-extent in tiles (e.g. 14 -> a 29x29 patch).
pub fn build_terrain(
    world: &WorldGen,
    chunks: &mut ChunkCache,
    edits: &TileEdits,
    (cx, cy): (i32, i32),
    radius: i32,
) -> ParsedGeometry {
    let mut vertices: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut base_colours: Vec<[f32; 3]> = Vec::new();
    let r = radius.max(1);

    for ty in cy - r..=cy + r {
        for tx in cx - r..=cx + r {
            let kind = edited_tile(edits, world, chunks, tx, ty);
            let base = kind.color();
            let shade = variation(tx, ty);
            let top = [
                base[0] * shade,
                base[1] * shade,
                base[2] * shade,
            ];
            base_colours.push(top);
            let h = height_at(world, chunks, tx, ty);

            // Top face. Y-up: tile (tx, ty) spans [tx, tx+1] x [ty, ty+1].
            let base_index = (vertices.len() / crate::model::MODEL_VERTEX_FLOATS) as u32;
            for (x, z) in [
                (tx as f32, ty as f32),
                (tx as f32 + 1.0, ty as f32),
                (tx as f32 + 1.0, ty as f32 + 1.0),
                (tx as f32, ty as f32 + 1.0),
            ] {
                vertices.extend_from_slice(&[
                    x, h, z, // position
                    0.0, 1.0, 0.0, // up normal
                    top[0], top[1], top[2], // biome colour
                ]);
            }
            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);

            // Front-facing skirts so a raised tile reads as solid ground, not
            // a floating card. Only emit where the neighbour is lower.
            let side = [top[0] * 0.82, top[1] * 0.82, top[2] * 0.82];
            let h_east = height_at(world, chunks, tx + 1, ty);
            if h > h_east + 0.001 {
                let s = (vertices.len() / crate::model::MODEL_VERTEX_FLOATS) as u32;
                for (x, z, y) in [
                    (tx as f32 + 1.0, ty as f32, h),
                    (tx as f32 + 1.0, ty as f32 + 1.0, h),
                    (tx as f32 + 1.0, ty as f32 + 1.0, h_east),
                    (tx as f32 + 1.0, ty as f32, h_east),
                ] {
                    vertices.extend_from_slice(&[
                        x, y, z, // position
                        1.0, 0.0, 0.0, // east normal
                        side[0], side[1], side[2],
                    ]);
                }
                indices.extend_from_slice(&[s, s + 1, s + 2, s, s + 2, s + 3]);
            }
            let h_south = height_at(world, chunks, tx, ty + 1);
            if h > h_south + 0.001 {
                let s = (vertices.len() / crate::model::MODEL_VERTEX_FLOATS) as u32;
                for (x, z, y) in [
                    (tx as f32, ty as f32 + 1.0, h),
                    (tx as f32 + 1.0, ty as f32 + 1.0, h),
                    (tx as f32 + 1.0, ty as f32 + 1.0, h_south),
                    (tx as f32, ty as f32 + 1.0, h_south),
                ] {
                    vertices.extend_from_slice(&[
                        x, y, z, // position
                        0.0, 0.0, 1.0, // south normal
                        side[0], side[1], side[2],
                    ]);
                }
                indices.extend_from_slice(&[s, s + 1, s + 2, s, s + 2, s + 3]);
            }
        }
    }

    ParsedGeometry {
        name: "terrain".to_string(),
        vertices,
        indices,
        base_colours,
    }
}
