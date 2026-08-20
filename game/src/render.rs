use crate::iso::{depth_order, iso_to_world, world_to_iso, HALF_H, HALF_W};
use crate::player::Player;
use crate::world::{ChunkCache, WorldGen};
use crate::TILE_HEIGHT;

/// Floats per vertex: position x, y + color r, g, b.
pub const VERTEX_FLOATS: usize = 5;
pub const VERTEX_STRIDE_BYTES: usize = VERTEX_FLOATS * 4;

/// Player quad color (warm orange so it stands out from all biomes).
pub const PLAYER_COLOR: [f32; 3] = [1.0, 0.55, 0.15];

/// A small diamond drawn on top of a tile (resource node, structure, ...).
/// Centered on the world position (fractional for moving entities),
/// optionally lifted `lift` pixels (tall sprites).
#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub color: [f32; 3],
    pub half_w: f32,
    pub half_h: f32,
    pub lift: f32,
    pub alpha: f32,
}

impl Sprite {
    /// Sprite centered on tile center.
    pub fn new(tx: i32, ty: i32, color: [f32; 3], half_w: f32, half_h: f32, lift: f32) -> Self {
        Self::new_center(tx as f32 + 0.5, ty as f32 + 0.5, color, half_w, half_h, lift)
    }

    /// Sprite centered on an arbitrary world position.
    pub fn new_center(x: f32, y: f32, color: [f32; 3], half_w: f32, half_h: f32, lift: f32) -> Self {
        Self {
            x,
            y,
            color,
            half_w,
            half_h,
            lift,
            alpha: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DrawKind {
    Tile,
    Sprite,
    Player,
}

#[derive(Debug, Clone, Copy)]
struct Draw {
    depth: f32,
    kind: DrawKind,
    tx: i32,
    ty: i32,
    sx: f32,
    sy: f32,
    half_w: f32,
    half_h: f32,
    lift: f32,
    color: [f32; 3],
    alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
}

impl Camera {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Camera position that puts the player's sprite diamond at the viewport
/// center. The camera maps to the screen origin, so we back off by the
/// tile-center offset: `iso(cam) = iso(player) - (vw/2, vh/2 - HALF_H)`.
pub fn focus_target(player: &Player, viewport: (f32, f32)) -> (f32, f32) {
    let (sx, sy) = world_to_iso(player.x, player.y);
    iso_to_world(sx - viewport.0 / 2.0, sy + HALF_H - viewport.1 / 2.0)
}

/// Tiles visible from `camera` given `viewport` (pixels), sorted in draw
/// order (painter's algorithm: ascending x+y depth).
pub fn visible_tiles(camera: Camera, viewport: (f32, f32)) -> Vec<(i32, i32)> {
    let vw = viewport.0;
    let vh = viewport.1;
    let rx = (vw / (2.0 * HALF_W)).ceil() as i32 + 2;
    let ry = (vh / (2.0 * HALF_H)).ceil() as i32 + 2;
    let mut quads: Vec<(i32, i32, i32)> = Vec::new();

    for ty in camera.y as i32 - ry..=camera.y as i32 + ry {
        for tx in camera.x as i32 - rx..=camera.x as i32 + rx {
            let (sx, sy) = world_to_iso(tx as f32 - camera.x, ty as f32 - camera.y);
            if sx + HALF_W < 0.0 || sx - HALF_W > vw || sy + HALF_H < 0.0 || sy - HALF_H > vh {
                continue;
            }
            quads.push((depth_order(tx, ty), tx, ty));
        }
    }
    quads.sort_unstable_by_key(|q| q.0);
    quads.into_iter().map(|(_, tx, ty)| (tx, ty)).collect()
}

/// Emits the mesh for the visible area into `out`: depth-sorted tiles, then
/// sprites (each on top of its own tile, at tile depth + 0.5), then the
/// player (always last, never occluded). Layout: 6 vertices per quad, each
/// `[x, y, r, g, b]`. Returns the number of quads emitted.
pub fn build_tile_mesh(
    world: &WorldGen,
    cache: &mut ChunkCache,
    camera: Camera,
    viewport: (f32, f32),
    sprites: &[Sprite],
    player: Option<&Player>,
    out: &mut Vec<f32>,
) -> u32 {
    out.clear();
    let vw = viewport.0;
    let vh = viewport.1;
    let mut draws: Vec<Draw> = Vec::with_capacity(1024);

    for (tx, ty) in visible_tiles(camera, viewport) {
        let (sx, sy) = world_to_iso(tx as f32 - camera.x, ty as f32 - camera.y);
        draws.push(Draw {
            depth: (tx + ty) as f32,
            kind: DrawKind::Tile,
            tx,
            ty,
            sx,
            sy,
            half_w: 0.0,
            half_h: 0.0,
            lift: 0.0,
            color: [0.0; 3],
            alpha: 1.0,
        });
    }

    for s in sprites {
        let (cx, cy) = world_to_iso(s.x - camera.x, s.y - camera.y);
        if cx + s.half_w < 0.0
            || cx - s.half_w > vw
            || cy + s.half_h + s.lift < 0.0
            || cy - s.half_h + s.lift > vh
        {
            continue;
        }
        let tx = s.x.floor() as i32;
        let ty = s.y.floor() as i32;
        draws.push(Draw {
            depth: (tx + ty) as f32 + 0.5,
            kind: DrawKind::Sprite,
            tx,
            ty,
            sx: cx,
            sy: cy,
            half_w: s.half_w,
            half_h: s.half_h,
            lift: s.lift,
            color: s.color,
            alpha: s.alpha,
        });
    }

    if let Some(p) = player {
        let (sx, sy) = world_to_iso(p.x - camera.x, p.y - camera.y);
        draws.push(Draw {
            depth: f32::MAX,
            kind: DrawKind::Player,
            tx: 0,
            ty: 0,
            sx,
            sy,
            half_w: 0.0,
            half_h: 0.0,
            lift: 0.0,
            color: PLAYER_COLOR,
            alpha: 1.0,
        });
    }

    draws.sort_by(|a, b| a.depth.total_cmp(&b.depth));

    for d in draws {
        match d.kind {
            DrawKind::Tile => {
                let mut base = tile_kind_at(world, cache, d.tx, d.ty).color();
                let v = 0.06 * (((d.tx * 7 + d.ty * 13) % 9) as f32 - 4.0);
                for c in base.iter_mut() {
                    *c = (*c + v).clamp(0.0, 1.0);
                }
                push_quad(out, d.sx, d.sy, base);
            }
            DrawKind::Sprite => push_center_quad(
                out,
                d.sx,
                d.sy,
                d.half_w,
                d.half_h,
                d.lift,
                d.color,
                d.alpha,
            ),
            DrawKind::Player => push_quad(out, d.sx, d.sy, PLAYER_COLOR),
        }
    }
    (out.len() / (6 * VERTEX_FLOATS)) as u32
}

fn tile_kind_at(world: &WorldGen, cache: &mut ChunkCache, tx: i32, ty: i32) -> crate::world::TileKind {
    let chunk = cache.get(world, tx, ty);
    chunk.tiles[ty.rem_euclid(crate::world::CHUNK_SIZE) as usize]
        [tx.rem_euclid(crate::world::CHUNK_SIZE) as usize]
        .kind
}

fn push_quad(out: &mut Vec<f32>, ox: f32, oy: f32, color: [f32; 3]) {
    let top = (ox, oy);
    let right = (ox + HALF_W, oy + HALF_H);
    let bottom = (ox, oy + TILE_HEIGHT);
    let left = (ox - HALF_W, oy + HALF_H);
    let verts = [
        top, right, bottom, top, bottom, left, // two triangles
    ];
    for (vx, vy) in verts {
        out.push(vx);
        out.push(vy);
        out.extend_from_slice(&color);
    }
}

/// Diamond centered at (cx, cy + lift): top/bottom on the vertical axis,
/// left/right on the horizontal, sized by half_w / half_h.
fn push_center_quad(
    out: &mut Vec<f32>,
    cx: f32,
    cy: f32,
    half_w: f32,
    half_h: f32,
    lift: f32,
    color: [f32; 3],
    alpha: f32,
) {
    let top = (cx, cy - half_h + lift);
    let right = (cx + half_w, cy + lift);
    let bottom = (cx, cy + half_h + lift);
    let left = (cx - half_w, cy + lift);
    let verts = [
        top, right, bottom, top, bottom, left, // two triangles
    ];
    // alpha blends the sprite toward the underlying world color (cheap
    // transparency without a blend pipeline; sprites are drawn over tiles)
    let tint = [color[0] * alpha, color[1] * alpha, color[2] * alpha];
    for (vx, vy) in verts {
        out.push(vx);
        out.push(vy);
        out.extend_from_slice(&tint);
    }
}

/// Vertex list of one quad's 6 vertices for assertions.
pub fn quad_vertices<'a>(mesh: &'a [f32], index: usize) -> &'a [f32] {
    let start = index * 6 * VERTEX_FLOATS;
    &mesh[start..start + 6 * VERTEX_FLOATS]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::WorldGen;

    fn cam() -> Camera {
        Camera::new(0.0, 0.0)
    }

    #[test]
    fn visible_tiles_centered_and_sorted() {
        let viewport = (1280.0, 720.0);
        let tiles = visible_tiles(cam(), viewport);
        assert!(!tiles.is_empty(), "viewport must see tiles");
        // contains origin
        assert!(tiles.contains(&(0, 0)));
        // strictly ascending depth (painter's order)
        for pair in tiles.windows(2) {
            assert!(
                depth_order(pair[0].0, pair[0].1) <= depth_order(pair[1].0, pair[1].1),
                "draw order violated at {:?} -> {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn visible_tiles_inside_viewport() {
        let viewport = (640.0, 360.0);
        let cam_pos = Camera::new(3.5, -2.25);
        for (tx, ty) in visible_tiles(cam_pos, viewport) {
            let (sx, sy) = world_to_iso(tx as f32 - cam_pos.x, ty as f32 - cam_pos.y);
            assert!(sx + HALF_W >= -0.001 && sx - HALF_W <= viewport.0 + 0.001);
            assert!(sy + HALF_H >= -0.001 && sy - HALF_H <= viewport.1 + 0.001);
        }
    }

    #[test]
    fn tile_geometry_is_diamond() {
        let world = WorldGen::new(1);
        let mut cache = ChunkCache::new(64);
        let mut mesh = Vec::new();
        let quads = build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &[], None, &mut mesh);
        assert!(quads > 0);
        // find the quad whose top corner sits at world (0,0) (tile 0,0)
        let mut found = None;
        for i in 0..quads as usize {
            let v = quad_vertices(&mesh, i);
            if v[0] == 0.0 && v[1] == 0.0 {
                found = Some(v);
                break;
            }
        }
        let v = found.expect("tile (0,0) must be in the mesh");
        // vertex layout per vertex: [x, y, r, g, b]
        // v0=top, v1=right, v2=bottom, v3=top, v4=bottom, v5=left
        assert!((v[5] - HALF_W).abs() < 0.001, "right corner x");
        assert!((v[6] - HALF_H).abs() < 0.001, "right corner y");
        assert!((v[10] - 0.0).abs() < 0.001, "bottom corner x");
        assert!((v[11] - TILE_HEIGHT).abs() < 0.001, "bottom corner y");
        assert!((v[25] + HALF_W).abs() < 0.001, "left corner x");
        assert!((v[26] - HALF_H).abs() < 0.001, "left corner y");
    }

    #[test]
    fn mesh_quads_have_valid_colors() {
        let world = WorldGen::new(7);
        let mut cache = ChunkCache::new(64);
        let mut mesh = Vec::new();
        let quads = build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &[], None, &mut mesh);
        assert_eq!(mesh.len(), quads as usize * 6 * VERTEX_FLOATS);
        for i in (2..mesh.len()).step_by(VERTEX_FLOATS) {
            assert!((0.0..=1.0).contains(&mesh[i]), "r out of range at {i}");
            assert!((0.0..=1.0).contains(&mesh[i + 1]), "g out of range at {i}");
            assert!((0.0..=1.0).contains(&mesh[i + 2]), "b out of range at {i}");
        }
    }

    #[test]
    fn chunk_cache_negative_coords() {
        let world = WorldGen::new(42);
        let mut cache = ChunkCache::new(64);
        cache.get(&world, -1, -1);
        assert!(cache.contains(-1, -1), "chunk (-1,-1) must be cached");
        cache.get(&world, -33, 0);
        cache.get(&world, 0, -33);
        cache.get(&world, 31, 31);
        cache.get(&world, 32, 32);
        assert_eq!(cache.len(), 5);
        for (cx, cy) in [(-1, -1), (-2, 0), (0, -2), (0, 0), (1, 1)] {
            assert!(cache.contains(cx, cy), "chunk ({cx},{cy}) missing");
        }
    }

    #[test]
    fn chunk_cache_evicts_over_capacity() {
        let world = WorldGen::new(42);
        let mut cache = ChunkCache::new(4);
        for i in 0..10 {
            cache.get(&world, i * 33, 0);
        }
        assert!(cache.len() <= 4, "cache must stay under capacity");
    }

    #[test]
    fn focus_centers_player_on_viewport() {
        use crate::player::Player;
        let viewport = (1280.0, 720.0);
        let player = Player::new(0.5, 0.5);
        let (fx, fy) = focus_target(&player, viewport);
        // camera at focus maps the player's diamond center to the screen center
        let (sx, sy) = world_to_iso(player.x - fx, player.y - fy);
        assert!((sx - viewport.0 / 2.0).abs() < 0.01, "player center x");
        assert!((sy + HALF_H - viewport.1 / 2.0).abs() < 0.01, "player center y");
    }

    #[test]
    fn player_quad_in_mesh_depth_sorted() {
        use crate::player::Player;
        let world = WorldGen::new(3);
        let mut cache = ChunkCache::new(64);
        let mut mesh = Vec::new();
        let player = Player::new(0.5, 0.5);
        let quads = build_tile_mesh(
            &world,
            &mut cache,
            cam(),
            (640.0, 360.0),
            &[],
            Some(&player),
            &mut mesh,
        );
        // one quad more than the plain tile mesh
        let mut plain = Vec::new();
        let plain_quads =
            build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &[], None, &mut plain);
        assert_eq!(quads, plain_quads + 1);

        // find the player quad (warm orange color)
        let mut player_quad = None;
        for i in 0..quads as usize {
            let v = quad_vertices(&mesh, i);
            if v[2] == PLAYER_COLOR[0] && v[3] == PLAYER_COLOR[1] && v[4] == PLAYER_COLOR[2] {
                player_quad = Some(v);
                break;
            }
        }
        let v = player_quad.expect("player quad must be in the mesh");
        let (ox, oy) = world_to_iso(0.5, 0.5);
        assert!((v[0] - ox).abs() < 0.001, "player top x");
        assert!((v[1] - oy).abs() < 0.001, "player top y");

        // depth ordering: tile (0,0) (depth 0) must come before the player (depth 1)
        let mut idx = 0;
        for i in 0..quads as usize {
            let q = quad_vertices(&mesh, i);
            if q[2] == PLAYER_COLOR[0] {
                idx = i;
                break;
            }
        }
        // tile (0,0)'s top corner sits at screen (0,0) when camera is at origin
        let mut tile_zero_seen = false;
        for i in 0..idx {
            let q = quad_vertices(&mesh, i);
            if q[0] == 0.0 && q[1] == 0.0 {
                tile_zero_seen = true;
                break;
            }
        }
        assert!(tile_zero_seen, "tile (0,0) must render before the player");
    }

    #[test]
    fn sprites_depth_sorted_between_tiles() {
        let world = WorldGen::new(3);
        let mut cache = ChunkCache::new(64);
        let mut mesh = Vec::new();
        // a tree on tile (0,0): depth 0.5 — after tile (0,0), before depth-1 tiles
        let tree = Sprite::new(0, 0, [0.06, 0.30, 0.12], 14.0, 20.0, 8.0);
        let quads = build_tile_mesh(
            &world,
            &mut cache,
            cam(),
            (640.0, 360.0),
            &[tree],
            None,
            &mut mesh,
        );
        let tree_idx = (0..quads as usize)
            .find(|&i| quad_vertices(&mesh, i)[2] == 0.06)
            .expect("tree sprite must be in the mesh");

        // its center sits at the tile center: iso(0.5, 0.5) = (0, 16)
        let v = quad_vertices(&mesh, tree_idx);
        assert!((v[0] - 0.0).abs() < 0.001, "tree top x (center column)");
        assert!((v[1] - (16.0 - 20.0 + 8.0)).abs() < 0.001, "tree top y (lifted)");

        // tile (0,0) must render before the tree
        assert!(
            (0..tree_idx).any(|i| {
                let q = quad_vertices(&mesh, i);
                q[0] == 0.0 && q[1] == 0.0
            }),
            "tile (0,0) must render before the tree on it"
        );
        // and the tree must render before tiles of depth 1 (e.g. tile (1,0))
        assert!(
            (tree_idx + 1..quads as usize).any(|i| {
                let q = quad_vertices(&mesh, i);
                (q[0] - 32.0).abs() < 0.001 && (q[1] - 16.0).abs() < 0.001
            }),
            "tile (1,0) must render after the tree"
        );
    }

    #[test]
    fn offscreen_sprite_is_culled() {
        let world = WorldGen::new(3);
        let mut cache = ChunkCache::new(64);
        let mut mesh = Vec::new();
        // sprite far outside the 640x360 viewport
        let far = Sprite::new(1000, 1000, [0.9, 0.1, 0.1], 14.0, 20.0, 8.0);
        let quads = build_tile_mesh(
            &world,
            &mut cache,
            cam(),
            (640.0, 360.0),
            &[far],
            None,
            &mut mesh,
        );
        assert!(
            !(0..quads as usize).any(|i| quad_vertices(&mesh, i)[2] == 0.9),
            "off-screen sprite must be culled"
        );
    }

    #[test]
    fn world_generation_is_deterministic() {
        let a = WorldGen::new(1337).generate_chunk(3, -2);
        let b = WorldGen::new(1337).generate_chunk(3, -2);
        for ty in 0..crate::world::CHUNK_SIZE {
            for tx in 0..crate::world::CHUNK_SIZE {
                assert_eq!(a.tiles[ty as usize][tx as usize].kind, b.tiles[ty as usize][tx as usize].kind);
            }
        }
        let c = WorldGen::new(1338).generate_chunk(3, -2);
        let same = (0..crate::world::CHUNK_SIZE)
            .flat_map(|ty| {
                (0..crate::world::CHUNK_SIZE).map(move |tx| {
                    a.tiles[ty as usize][tx as usize].kind == c.tiles[ty as usize][tx as usize].kind
                })
            })
            .all(|eq| eq);
        assert!(!same, "different seeds should produce different terrain");
    }
}