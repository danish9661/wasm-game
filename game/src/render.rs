use crate::elements::prim::{rasterize, Part};
use crate::iso::{depth_order, iso_to_world, world_to_iso, HALF_H, HALF_W};
use crate::player::Player;
use crate::world::{ChunkCache, TileKind, WorldGen};
use crate::TILE_HEIGHT;

/// Floats per vertex: position x, y + color r, g, b, a.
pub const VERTEX_FLOATS: usize = 6;
pub const VERTEX_STRIDE_BYTES: usize = VERTEX_FLOATS * 4;

/// Player quad color (warm orange so it stands out from all biomes).
pub const PLAYER_COLOR: [f32; 3] = [1.0, 0.55, 0.15];

/// A small diamond drawn on top of a tile (resource node, structure, ...).
/// Centered on the world position (fractional for moving entities),
/// optionally lifted `lift` pixels (tall sprites).
/// How a `Sprite` should be drawn. `Generic` is the flat lifted-diamond look;
/// everything else routes to a kind-aware silhouette so trees, rocks, walls,
/// enemies, etc. read as distinct objects instead of colored diamonds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteStyle {
    Generic,
    Tree,
    Rock,
    Bush,
    Wall,
    Chest,
    Campfire,
    Altar,
    Slime,
    Humanoid,
    /// HP bar backing plate (dark frame), drawn just above an enemy.
    HpBack,
    /// HP bar fill (colored, width encodes hp fraction).
    HpFill,
    /// Projectile (arrow): a shaft oriented along `facing` with an arrowhead.
    Arrow,
    // ---- Harvestable resources -------------------------------------------
    Mushroom,
    Crystal,
    Flower,
    GrassTuft,
    Fern,
    Ore,
    // ---- Buildable structures --------------------------------------------
    Fence,
    Torch,
    Anvil,
    Bed,
    Well,
    // ---- Decorative props (world-gen only, non-interactive) --------------
    Sign,
    Barrel,
    Totem,
    RockPile,
    Statue,
    Lantern,
    Brazier,
    Crate,
    Pillar,
    BonePile,
    Cactus,
    Vines,
    Lilypad,
    Reed,
    Rubble,
    // ---- Enemies ----------------------------------------------------------
    Skeleton,
    Goblin,
    Bat,
    Spider,
    Imp,
    Ogre,
}

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub color: [f32; 3],
    pub half_w: f32,
    pub half_h: f32,
    pub lift: f32,
    pub alpha: f32,
    pub style: SpriteStyle,
    /// Facing direction in world coords (also the attack facing). Used to lean
    /// the head of humanoid figures toward where they're looking/moving.
    pub facing: (f32, f32),
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
            style: SpriteStyle::Generic,
            facing: (1.0, 0.0),
        }
    }

    /// Tag this sprite with a distinct silhouette style.
    pub fn with_style(mut self, style: SpriteStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the facing direction (world coords) for humanoid figures.
    pub fn with_facing(mut self, facing: (f32, f32)) -> Self {
        self.facing = facing;
        self
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
    style: SpriteStyle,
    facing: (f32, f32),
    hurt: bool,
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
/// Fraction of viewport height where the player is drawn. 0.5 = dead-centre;
/// 0.62 places the player in the lower third so movement is clearly visible
/// on screen (the character actually travels in the pressed direction instead
/// of being pinned to the centre while only the world scrolls).
const PLAYER_SCREEN_Y: f32 = 0.62;

pub fn focus_target(player: &Player, viewport: (f32, f32)) -> (f32, f32) {
    let (sx, sy) = world_to_iso(player.x, player.y);
    iso_to_world(sx - viewport.0 / 2.0, sy + HALF_H - viewport.1 * PLAYER_SCREEN_Y)
}

/// Tiles visible from `camera` given `viewport` (pixels), sorted in draw
/// order (painter's algorithm: ascending x+y depth).
pub fn visible_tiles(camera: Camera, viewport: (f32, f32)) -> Vec<(i32, i32)> {
    let vw = viewport.0;
    let vh = viewport.1;
    // In isometric projection both world axes contribute to both screen axes,
    // so the world-space iteration range must cover the full diagonal extent
    // of the viewport.  The screen extents are:
    //   screen_x = (tx - ty) * HALF_W  → needs |tx-ty| up to  vw / HALF_W
    //   screen_y = (tx + ty) * HALF_H  → needs |tx+ty| up to  vh / HALF_H
    // Solving for the max |tx| and |ty| gives (vw/HALF_W + vh/HALF_H) / 2.
    let r = ((vw / HALF_W + vh / HALF_H) / 2.0).ceil() as i32 + 2;
    let mut quads: Vec<(i32, i32, i32)> = Vec::new();

    for ty in camera.y as i32 - r..=camera.y as i32 + r {
        for tx in camera.x as i32 - r..=camera.x as i32 + r {
            let (sx, sy) = world_to_iso(tx as f32 - camera.x, ty as f32 - camera.y);
            // The tile diamond spans [sx - HALF_W, sx + HALF_W] horizontally
            // and [sy, sy + TILE_HEIGHT] vertically.
            if sx + HALF_W < 0.0 || sx - HALF_W > vw || sy + TILE_HEIGHT < 0.0 || sy > vh {
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
    anim_time: f32,
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
            style: SpriteStyle::Generic,
            facing: (1.0, 0.0),
            hurt: false,
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
            style: s.style,
            facing: s.facing,
            hurt: false,
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
            style: SpriteStyle::Generic,
            facing: p.facing,
            hurt: p.hurt_timer > 0.0,
        });
    }

    draws.sort_by(|a, b| a.depth.total_cmp(&b.depth));

    for d in draws {
        match d.kind {
            DrawKind::Tile => {
                let kind = tile_kind_at(world, cache, d.tx, d.ty);
                let mut base = kind.color();
                // Per-tile value variation so grass/forest/spawned-detail don't
                // read as a flat uniform sheet. Extra jitter on grass/forest.
                let mut v = 0.05 * (((d.tx * 7 + d.ty * 13) % 9) as f32 - 4. as f32);
                if matches!(kind, TileKind::Grass | TileKind::Forest | TileKind::Swamp) {
                    v += 0.04 * (((d.tx * 31 + d.ty * 17) % 7) as f32 - 3. as f32);
                }
                for c in base.iter_mut() {
                    *c = (*c + v).clamp(0.0, 1.0);
                }
                // Sample the 4 cardinal neighbours and blend each tile corner
                // toward them so biome boundaries become smooth gradients.
                let nk = tile_kind_at(world, cache, d.tx, d.ty - 1);
                let ek = tile_kind_at(world, cache, d.tx + 1, d.ty);
                let sk = tile_kind_at(world, cache, d.tx, d.ty + 1);
                let wk = tile_kind_at(world, cache, d.tx - 1, d.ty);
                let mut c_n = corner_blend(base, kind, nk);
                let mut c_e = corner_blend(base, kind, ek);
                let mut c_s = corner_blend(base, kind, sk);
                let mut c_w = corner_blend(base, kind, wk);
                // subtle top-lit bevel: north (top) corner catches light, south
                // (bottom) corner falls into shadow — gives each tile gentle 2.5D
                // definition without re-introducing hard seams.
                for i in 0..3 {
                    c_n[i] = (c_n[i] * 1.06).clamp(0.0, 1.0);
                    c_s[i] = (c_s[i] * 0.92).clamp(0.0, 1.0);
                }
                // Water shimmer: a travelling wave of brightness, now strong
                // enough to clearly read on screen (corner-agnostic).
                if matches!(kind, TileKind::Water | TileKind::DeepWater | TileKind::ShallowWater) {
                    let sh = (anim_time * 2.2 + d.tx as f32 * 0.7 + d.ty as f32 * 0.5).sin() * 0.12;
                    for c in [&mut c_n, &mut c_e, &mut c_s, &mut c_w] {
                        c[0] = (c[0] + sh).clamp(0.0, 1.0);
                        c[2] = (c[2] + sh * 0.85).clamp(0.0, 1.0);
                    }
                }
                push_quad_blended(out, d.sx, d.sy, c_n, c_e, c_s, c_w);
            }
            DrawKind::Sprite => {
                // Fake 2.5D: ground shadow first, then a kind-aware silhouette.
                push_shadow(out, d.sx, d.sy, d.half_w, d.half_h);
                match d.style {
                    SpriteStyle::Generic => {
                        let mut g = Vec::new();
                        g.push(Part::diamond(
                            d.sx, d.sy, d.half_w, d.half_h, d.lift, d.color, d.alpha, d.lift > 0.0,
                        ));
                        rasterize(&g, out);
                    }
                    other => push_styled_sprite(
                        out, d.sx, d.sy, other, d.color, d.half_w, d.half_h, d.lift, d.alpha,
                        d.facing, anim_time,
                    ),
                }
            }
            DrawKind::Player => {
                // Player stands on the tile center (sx, sy + HALF_H): shadow
                // then the humanoid figure built upward from the ground point.
                // Flash red briefly after taking a hit.
                let gy = d.sy + HALF_H;
                let tunic = if d.hurt {
                    [1.0, 0.32, 0.30]
                } else {
                    PLAYER_COLOR
                };
                push_shadow(out, d.sx, gy, HALF_W * 0.7, HALF_H * 0.7);
                let parts = crate::elements::humanoid::build(d.sx, gy, tunic, 1.0, d.facing, anim_time);
                rasterize(&parts, out);
            }
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

/// Draw a tile diamond with per-corner colors. Each corner blends toward the
/// neighbouring tile in that direction, so biome edges become smooth gradients
/// instead of a hard checkerboard. Vertex order matches `push_quad` so the
/// geometry tests (positions) stay valid.
fn push_quad_blended(
    out: &mut Vec<f32>,
    ox: f32,
    oy: f32,
    north: [f32; 3],
    east: [f32; 3],
    south: [f32; 3],
    west: [f32; 3],
) {
    let top = (ox, oy);
    let right = (ox + HALF_W, oy + HALF_H);
    let bottom = (ox, oy + TILE_HEIGHT);
    let left = (ox - HALF_W, oy + HALF_H);
    // Matching vertex order: top, right, bottom, top, bottom, left.
    let verts: [(f32, f32); 6] = [top, right, bottom, top, bottom, left];
    // North/Top, East/Right, South/Bottom, West/Left — one color per corner.
    let colors = [north, east, south, north, south, west];
    for i in 0..6 {
        let (vx, vy) = verts[i];
        out.push(vx);
        out.push(vy);
        out.extend_from_slice(&colors[i]);
        out.push(1.0);
    }
}

/// Blend a tile corner toward a neighbouring tile's color (50% when the biome
/// differs, full self color when they match). Keeps uniform regions solid but
/// softens boundaries into gradients.
fn corner_blend(self_color: [f32; 3], self_kind: TileKind, neighbor_kind: TileKind) -> [f32; 3] {
    if self_kind == neighbor_kind {
        return self_color;
    }
    let n_color = neighbor_kind.color();
    let mut c = [0.0f32; 3];
    for i in 0..3 {
        c[i] = (self_color[i] + n_color[i]) * 0.5;
    }
    c
}

/// Flat ground shadow: a darkened diamond on the tile (no lift), alpha-blended
/// over the tile beneath it so entities read as standing on the ground.
fn push_shadow(out: &mut Vec<f32>, cx: f32, cy: f32, half_w: f32, half_h: f32) {
    let top = (cx, cy - half_h);
    let right = (cx + half_w, cy);
    let bottom = (cx, cy + half_h);
    let left = (cx - half_w, cy);
    let verts = [top, right, bottom, top, bottom, left];
    for (vx, vy) in verts {
        out.push(vx);
        out.push(vy);
        out.push(0.0);
        out.push(0.0);
        out.push(0.0);
        out.push(0.30);
    }
}

/// Draw a sprite using its kind-aware silhouette. `(cx, cy)` is the ground
/// point (the tile top where the shadow sits); shapes are built upward.
///
/// Each element returns a list of [`Part`]s via its own module in
/// `crate::elements`; the uniform `rasterize` call emits them (and the fake
/// 2.5D dark "skirt") so the renderer never hard-codes any artwork.
fn push_styled_sprite(
    out: &mut Vec<f32>,
    cx: f32,
    cy: f32,
    style: SpriteStyle,
    color: [f32; 3],
    hw: f32,
    hh: f32,
    lift: f32,
    alpha: f32,
    facing: (f32, f32),
    anim_time: f32,
) {
    use crate::elements::{
        altar, anvil, arrow, barrel, bat, bed, bone_pile, brazier, bush, cactus, campfire, chest,
        crate_box, crystal, fence, fern, flower, goblin, grass_tuft, humanoid, hpbar, imp, lantern,
        lilypad, mushroom, ogre, ore, pillar, reed, rock, rock_pile, rubble, sign, skeleton, slime,
        spider, statue, torch, totem, tree, vines, wall, well,
    };
    match style {
        SpriteStyle::Generic => {
            // handled by caller (DrawKind::Sprite branch) — never reaches here.
            let _ = (cx, cy, color, hw, hh, lift, alpha, facing, anim_time);
        }
        SpriteStyle::Tree => rasterize(&tree::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Rock => rasterize(&rock::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Bush => rasterize(&bush::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Wall => rasterize(&wall::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Chest => rasterize(&chest::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Campfire => {
            rasterize(&campfire::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        SpriteStyle::Altar => rasterize(&altar::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Arrow => arrow::draw(out, cx, cy, color, alpha, facing),
        SpriteStyle::Slime => rasterize(&slime::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Humanoid => {
            rasterize(&humanoid::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        SpriteStyle::HpBack => rasterize(&hpbar::back(cx, cy, hw, hh, lift, color, alpha), out),
        SpriteStyle::HpFill => rasterize(&hpbar::fill(cx, cy, hw, hh, lift, color, alpha), out),
        // Harvestable resources
        SpriteStyle::Mushroom => {
            rasterize(&mushroom::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        SpriteStyle::Crystal => rasterize(&crystal::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Flower => rasterize(&flower::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::GrassTuft => {
            rasterize(&grass_tuft::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        SpriteStyle::Fern => rasterize(&fern::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Ore => rasterize(&ore::build(cx, cy, color, alpha, facing, anim_time), out),
        // Buildable structures
        SpriteStyle::Fence => rasterize(&fence::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Torch => rasterize(&torch::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Anvil => rasterize(&anvil::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Bed => rasterize(&bed::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Well => rasterize(&well::build(cx, cy, color, alpha, facing, anim_time), out),
        // Decorative props
        SpriteStyle::Sign => rasterize(&sign::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Barrel => rasterize(&barrel::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Totem => rasterize(&totem::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::RockPile => {
            rasterize(&rock_pile::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        SpriteStyle::Statue => rasterize(&statue::build(cx, cy, color, alpha, facing, anim_time), out),
        // Enemies
        SpriteStyle::Skeleton => {
            rasterize(&skeleton::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        SpriteStyle::Goblin => rasterize(&goblin::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Bat => rasterize(&bat::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Spider => rasterize(&spider::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Imp => rasterize(&imp::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Ogre => rasterize(&ogre::build(cx, cy, color, alpha, facing, anim_time), out),
        // New decorative props
        SpriteStyle::Lantern => rasterize(&lantern::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Brazier => rasterize(&brazier::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Crate => rasterize(&crate_box::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Pillar => rasterize(&pillar::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::BonePile => rasterize(&bone_pile::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Cactus => rasterize(&cactus::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Vines => rasterize(&vines::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Lilypad => rasterize(&lilypad::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Reed => rasterize(&reed::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Rubble => rasterize(&rubble::build(cx, cy, color, alpha, facing, anim_time), out),
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
            // Diamond spans [sx - HALF_W, sx + HALF_W] x [sy, sy + TILE_HEIGHT]
            assert!(sx + HALF_W >= -0.001 && sx - HALF_W <= viewport.0 + 0.001);
            assert!(sy + TILE_HEIGHT >= -0.001 && sy <= viewport.1 + 0.001);
        }
    }

    #[test]
    fn tile_geometry_is_diamond() {
        let world = WorldGen::new(1);
        let mut cache = ChunkCache::new(64);
        let mut mesh = Vec::new();
        let quads = build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &[], None, &mut mesh, 0.0);
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
        // vertex layout per vertex: [x, y, r, g, b, a]
        // v0=top, v1=right, v2=bottom, v3=top, v4=bottom, v5=left
        assert!((v[6] - HALF_W).abs() < 0.001, "right corner x");
        assert!((v[7] - HALF_H).abs() < 0.001, "right corner y");
        assert!((v[12] - 0.0).abs() < 0.001, "bottom corner x");
        assert!((v[13] - TILE_HEIGHT).abs() < 0.001, "bottom corner y");
        assert!((v[30] + HALF_W).abs() < 0.001, "left corner x");
        assert!((v[31] - HALF_H).abs() < 0.001, "left corner y");
    }

    #[test]
    fn mesh_quads_have_valid_colors() {
        let world = WorldGen::new(7);
        let mut cache = ChunkCache::new(64);
        let mut mesh = Vec::new();
        let quads = build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &[], None, &mut mesh, 0.0);
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
        // camera at focus maps the player's diamond center to (screen-center-x,
        // PLAYER_SCREEN_Y of the viewport height) so the character sits in the
        // lower third and visibly travels when moving.
        let (sx, sy) = world_to_iso(player.x - fx, player.y - fy);
        assert!((sx - viewport.0 / 2.0).abs() < 0.01, "player center x");
        assert!((sy + HALF_H - viewport.1 * PLAYER_SCREEN_Y).abs() < 0.01, "player screen y");
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
            0.0,
        );
        // player branch emits: ground shadow + humanoid (2 legs, torso, head,
        // hair) each drawn as bright + dark-skirt + bright = 5*3 + 1 = 16 quads
        let mut plain = Vec::new();
        let plain_quads =
            build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &[], None, &mut plain, 0.0);
        assert_eq!(quads, plain_quads + 16);

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
        // The torso (PLAYER_COLOR) is centered on the player's screen x (ox).
        let cx_avg = (0..6).map(|k| v[k * 6]).sum::<f32>() / 6.0;
        assert!((cx_avg - ox).abs() < 0.001, "player center x");
        // and it stands on the tile center (gy = oy + HALF_H), so its bottom
        // edge sits at roughly the tile center, not the tile's top corner.
        let cy_max = (0..6).map(|k| v[k * 6 + 1]).fold(f32::MIN, f32::max);
        assert!(cy_max > oy, "player torso must be below the tile top corner");

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
            0.0,
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
            0.0,
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

    #[test]
    fn slime_bobs_with_anim_time() {
        // The slime's blob must move vertically as anim_time advances (hop/squash),
        // and the two sampled frames must differ — otherwise the animation is dead.
        use crate::render::SpriteStyle;
        let world = WorldGen::new(1);
        let mut cache = ChunkCache::new(64);
        let slime = Sprite::new(0, 0, [0.30, 0.78, 0.36], 14.0, 14.0, 2.0).with_style(SpriteStyle::Slime);

        let mut m0 = Vec::new();
        build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &[slime], None, &mut m0, 0.0);
        let mut m1 = Vec::new();
        build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &[slime], None, &mut m1, 0.3);

        // slime green signature
        let is_slime = |v: &[f32]| v[2] > 0.25 && v[2] < 0.45 && v[3] > 0.7 && v[4] < 0.45;
        let y_extent = |m: &[f32]| -> (f32, f32) {
            let mut top = f32::MAX;
            let mut bot = f32::MIN;
            for q in 0..m.len() / (6 * VERTEX_FLOATS) {
                let v = quad_vertices(m, q);
                if is_slime(v) {
                    for k in 0..6 {
                        let y = v[k * 6 + 1];
                        top = top.min(y);
                        bot = bot.max(y);
                    }
                }
            }
            (top, bot)
        };
        let (t0, b0) = y_extent(&m0);
        let (t1, b1) = y_extent(&m1);
        assert!(t0 < f32::MAX, "slime must be present in mesh");
        assert!(((t0 - t1).abs() + (b0 - b1).abs()) > 1.0, "slime blob must move between frames");
    }
}