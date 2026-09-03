use crate::elements::prim::{facing_offset, flashed, rasterize, rasterize_flash, sway, Part};
use crate::iso::{depth_order, iso_to_world, world_to_iso, HALF_H, HALF_W};
use crate::player::Player;
use crate::weapons::WeaponKind;
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
    /// Villager guard: armored figure with helmet, sword and shield.
    Guard,
    /// Stone golem protector: bulky, glowing eyes, heavy club.
    Golem,
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
    /// Buried treasure cache: rendered as a small chest so it reads as loot.
    Treasure,
    // ---- Buildable structures --------------------------------------------
    Fence,
    Torch,
    Anvil,
    Bed,
    Well,
    Spike,
    FarmPlot,
    Turret,
    HealingTotem,
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
    RuinTower,
    /// Arcane village portal that teleports the player to the walled town.
    Portal,
    /// War Banner: fluttering cloth on a tall pole with glowing gem topper.
    Banner,
    /// Enchanting Table: stone table with floating arcane symbols.
    EnchantingTable,
    /// Dungeon entrance: crumbling stone archway with ominous glow.
    Dungeon,
    // ---- Default world buildings (decor, scattered by worldgen) -----------
    House,
    Cabin,
    Hut,
    Inn,
    Barn,
    Watchtower,
    // ---- Enemies ----------------------------------------------------------
    // Humanoid foes (Skeleton, Goblin, Ogre, Brute, Stormcaller, Stoneslinger,
    // Boss) all share SpriteStyle::Humanoid for a consistent cast; only the
    // non-humanoid creatures below keep bespoke silhouettes.
    Bat,
    Spider,
    Imp,
    Wraith,
    Colossus,
    /// Scorpion Queen: wide arachnid with claws and stinger.
    ScorpionQueen,
    /// Toad King: bloated amphibian with tongue flick.
    ToadKing,
    /// Brute: hulking melee tank with massive upper body.
    Brute,
    /// Stormcaller: flying storm-mage with lightning wisps.
    Stormcaller,
    /// Ocean Leviathan: serpentine sea creature.
    OceanLeviathan,
    /// Lupine melee swarmer: a low, fast four-legged silhouette.
    Wolf,
    /// Archer: ranged marksman with drawn bow and quiver.
    Archer,
    /// Raider: night-stalking bandit with dark cloak and dagger.
    Raider,
    /// Old-world vehicles and railway props scattered through towns.
    Car,
    Train,
    Rail,
    /// Flat ground diamond drawn at the sprite's base — used for building interiors
    /// (floors) so a room reads as an enclosed space rather than open terrain.
    Floor,
}

/// Short label used by the headless visualizer (`get_frame_dump`) so sprites
/// can be rendered as ASCII art without needing a real screenshot.
pub fn style_label(s: SpriteStyle) -> &'static str {
    match s {
        SpriteStyle::Generic => "?",
        SpriteStyle::Tree => "t",
        SpriteStyle::Rock => "R",
        SpriteStyle::Bush => "b",
        SpriteStyle::Wall => "#",
        SpriteStyle::Chest => "$",
        SpriteStyle::Campfire => "*",
        SpriteStyle::Altar => "A",
        SpriteStyle::Slime => "s",
        SpriteStyle::Humanoid => "P",
        SpriteStyle::Guard => "G",
        SpriteStyle::Golem => "O",
        SpriteStyle::HpBack => "_",
        SpriteStyle::HpFill => "+",
        SpriteStyle::Arrow => "/",
        SpriteStyle::Mushroom => "m",
        SpriteStyle::Crystal => "c",
        SpriteStyle::Flower => "f",
        SpriteStyle::GrassTuft => "g",
        SpriteStyle::Fern => "n",
        SpriteStyle::Ore => "o",
        SpriteStyle::Treasure => "$",
        SpriteStyle::Fence => "|",
        SpriteStyle::Torch => "t",
        SpriteStyle::Anvil => "v",
        SpriteStyle::Bed => "B",
        SpriteStyle::Well => "W",
        SpriteStyle::Spike => "x",
        SpriteStyle::FarmPlot => "u",
        SpriteStyle::Turret => "Y",
        SpriteStyle::HealingTotem => "M",
        SpriteStyle::Sign => "i",
        SpriteStyle::Barrel => "=",
        SpriteStyle::Totem => "Z",
        SpriteStyle::RockPile => "r",
        SpriteStyle::Statue => "S",
        SpriteStyle::Lantern => "l",
        SpriteStyle::Brazier => "z",
        SpriteStyle::Crate => "[]",
        SpriteStyle::Pillar => "|",
        SpriteStyle::BonePile => "X",
        SpriteStyle::Cactus => "K",
        SpriteStyle::Vines => "N",
        SpriteStyle::Lilypad => "L",
        SpriteStyle::Reed => "q",
        SpriteStyle::Rubble => "..",
        SpriteStyle::RuinTower => "^",
        SpriteStyle::Portal => "@",
        SpriteStyle::Banner => "Bn",
        SpriteStyle::EnchantingTable => "Eq",
        SpriteStyle::Dungeon => "Dn",
        SpriteStyle::House => "H",
        SpriteStyle::Cabin => "C",
        SpriteStyle::Hut => "U",
        SpriteStyle::Inn => "I",
        SpriteStyle::Barn => "E",
        SpriteStyle::Watchtower => "T",
        SpriteStyle::Bat => ":",
        SpriteStyle::Spider => "y",
        SpriteStyle::Imp => "j",
        SpriteStyle::Wraith => "w",
        SpriteStyle::Colossus => "Q",
        SpriteStyle::ScorpionQueen => "sq",
        SpriteStyle::ToadKing => "tk",
        SpriteStyle::Brute => "bt",
        SpriteStyle::Stormcaller => "sc",
        SpriteStyle::OceanLeviathan => "ol",
        SpriteStyle::Wolf => "Q",
        SpriteStyle::Archer => "ar",
        SpriteStyle::Raider => "rd",
        SpriteStyle::Car => "V",
        SpriteStyle::Train => "J",
        SpriteStyle::Rail => "-",
        SpriteStyle::Floor => ".",
    }
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
    /// Movement intensity 0..1 for walk-cycle animation (humanoid figures).
    pub walk: f32,
    /// Hit-flash 0..1: lerps the figure toward white for a few frames on damage.
    pub flash: f32,
    /// Attack lunge 0..1: humanoid figures lean/extend toward the facing on strike.
    pub attack: f32,
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
            walk: 0.0,
            flash: 0.0,
            attack: 0.0,
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

    /// Set the movement intensity (0..1) driving the walk-cycle animation.
    pub fn with_walk(mut self, walk: f32) -> Self {
        self.walk = walk;
        self
    }

    /// Set the hit-flash intensity (0..1) for damage telegraph.
    pub fn with_flash(mut self, flash: f32) -> Self {
        self.flash = flash;
        self
    }

    /// Set the attack-lunge intensity (0..1) for the strike animation.
    pub fn with_attack(mut self, attack: f32) -> Self {
        self.attack = attack;
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
    walk: f32,
    flash: f32,
    attack: f32,
    /// Remaining dodge-roll time (for ghost trail rendering).
    dodge_timer: f32,
    /// Weapon enchantment level (0..5) for glow effect.
    enchant: u8,
    /// Equipped weapon, drawn in the player's hands (Fists = bare hands).
    weapon: WeaponKind,
    /// True while holding block: a shield is raised in front of the figure.
    blocking: bool,
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
    tiles: &[(i32, i32)],
    sprites: &[Sprite],
    player: Option<&Player>,
    out: &mut Vec<f32>,
    anim_time: f32,
    player_walk: f32,
) -> u32 {
    out.clear();
    let vw = viewport.0;
    let vh = viewport.1;
    let mut draws: Vec<Draw> = Vec::with_capacity(1024);

    for &(tx, ty) in tiles {
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
            walk: 0.0,
            flash: 0.0,
            attack: 0.0,
            dodge_timer: 0.0,
            enchant: 0,
            weapon: WeaponKind::Fists,
            blocking: false,
        });
    }

    for s in sprites {
        let (cx, cy0) = world_to_iso(s.x - camera.x, s.y - camera.y);
        // Lift the sprite onto the terrain height under its tile so props,
        // trees and characters stand on hills instead of floating at z=0.
        let z = tile_height_at(world, cache, s.x.floor() as i32, s.y.floor() as i32) as f32
            * crate::world::HEIGHT_STEP;
        let cy = cy0 - z;
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
            walk: s.walk,
            flash: s.flash,
            attack: s.attack,
            dodge_timer: 0.0,
            enchant: 0,
            weapon: WeaponKind::Fists,
            blocking: false,
        });
    }

    if let Some(p) = player {
        let (sx, sy0) = world_to_iso(p.x - camera.x, p.y - camera.y);
        let z = tile_height_at(world, cache, p.x.floor() as i32, p.y.floor() as i32) as f32
            * crate::world::HEIGHT_STEP;
        let sy = sy0 - z;
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
            walk: player_walk,
            flash: if p.hurt_timer > 0.0 {
                (p.hurt_timer / 0.6).clamp(0.0, 1.0)
            } else {
                0.0
            },
            // Lunge out and back over the swing: peaks mid-swing where the hit
            // lands, then recovers — reads as a committed strike.
            attack: (p.swing_t * (1.0 - p.swing_t) * 4.0).clamp(0.0, 1.0),
            dodge_timer: p.dodge_timer,
            enchant: p.enchant,
            weapon: p.weapon,
            blocking: p.blocking,
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
                // Deeper water (lower terrain height) reads darker / more teal, so
                // ocean basins and shallow shelves are visually distinct.
                if matches!(kind, TileKind::Water | TileKind::DeepWater | TileKind::ShallowWater) {
                    let depth = tile_height_at(world, cache, d.tx, d.ty);
                    let f = match depth {
                        -2 => 0.72,
                        -1 => 0.88,
                        _ => 1.0,
                    };
                    for c in base.iter_mut() {
                        *c = (*c * f).clamp(0.0, 1.0);
                    }
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
                // Water: depth-dependent swells + a finer sparkle ripple, a slow
                // traveling wave so the surface rolls, and foam crests on corners
                // that border land (so shorelines read). Foam gently pulses.
                if matches!(kind, TileKind::Water | TileKind::DeepWater | TileKind::ShallowWater) {
                    let is_water = |k: TileKind| {
                        matches!(k, TileKind::Water | TileKind::DeepWater | TileKind::ShallowWater)
                    };
                    // Slower, calmer swells in deep water; quick sparkle in shallows.
                    let (speed, amp, sparkle) = match kind {
                        TileKind::DeepWater => (1.3, 0.09, 0.03),
                        TileKind::Water => (2.1, 0.12, 0.07),
                        TileKind::ShallowWater => (3.0, 0.14, 0.13),
                        _ => (2.0, 0.1, 0.05),
                    };
                    let sh = (anim_time * speed + d.tx as f32 * 0.7 + d.ty as f32 * 0.5).sin() * amp;
                    let sp = (anim_time * speed * 2.3
                        + d.tx as f32 * 1.9
                        - d.ty as f32 * 1.3)
                        .sin()
                        * sparkle;
                    // Slow rolling wave travelling diagonally across the surface.
                    let wave = (anim_time * speed * 0.6 + (d.tx + d.ty) as f32 * 0.5).sin() * amp * 0.7;
                    let crest = (anim_time * speed * 1.7 + (d.tx - d.ty) as f32 * 0.8).sin().max(0.0)
                        * sparkle
                        * 0.6;
                    for c in [&mut c_n, &mut c_e, &mut c_s, &mut c_w] {
                        let a = (sh + sp + wave + crest).clamp(-0.3, 0.3);
                        c[0] = (c[0] + a).clamp(0.0, 1.0);
                        c[2] = (c[2] + a * 0.85).clamp(0.0, 1.0);
                    }
                    // Foam on each corner whose neighbour is land (not water),
                    // gently pulsing so the shoreline shimmers.
                    let foam_pulse = 0.18 + 0.06 * (anim_time * 2.0 + d.tx as f32 * 0.7).sin();
                    let foam = |nb: TileKind| -> f32 {
                        if is_water(kind) && !is_water(nb) {
                            foam_pulse
                        } else {
                            0.0
                        }
                    };
                    let foams = [foam(nk), foam(ek), foam(sk), foam(wk)];
                    for (c, fa) in [&mut c_n, &mut c_e, &mut c_s, &mut c_w]
                        .iter_mut()
                        .zip(foams.iter())
                    {
                        if *fa > 0.0 {
                            c[0] = (c[0] + fa * 0.9).clamp(0.0, 1.0);
                            c[1] = (c[1] + fa * 0.95).clamp(0.0, 1.0);
                            c[2] = (c[2] + fa).clamp(0.0, 1.0);
                        }
                    }
                }
                // Emit the tile itself (lifted by its height), plus vertical
                // "cliff" walls on the two front edges so raised terrain reads as
                // solid blocks with real depth. Walls drop to the neighbour's
                // height, so a hill shows its side and lower ground shows beneath.
                let z = tile_height_at(world, cache, d.tx, d.ty) as f32 * crate::world::HEIGHT_STEP;
                let zE = tile_height_at(world, cache, d.tx + 1, d.ty) as f32 * crate::world::HEIGHT_STEP;
                let zS = tile_height_at(world, cache, d.tx, d.ty + 1) as f32 * crate::world::HEIGHT_STEP;
                let mut wall_col = base;
                for c in wall_col.iter_mut() { *c = (*c * 0.62).clamp(0.0, 1.0); }
                if zE < z {
                    push_wall(out, d.sx + HALF_W, d.sy + HALF_H, d.sx, d.sy + TILE_HEIGHT, z, zE, wall_col);
                }
                if zS < z {
                    push_wall(out, d.sx, d.sy + TILE_HEIGHT, d.sx - HALF_W, d.sy + HALF_H, z, zS, wall_col);
                }
                push_quad_blended(out, d.sx, d.sy, z, c_n, c_e, c_s, c_w);
                if !matches!(kind, TileKind::Water | TileKind::DeepWater | TileKind::ShallowWater) {
                    tile_detail(out, kind, d.sx, d.sy, z, d.tx, d.ty, anim_time);
                }
            }
            DrawKind::Sprite => {
                // Fake 2.5D: ground shadow first — offset southeast for tall
                // buildings so roofs cast a readable shadow.
                let is_building = matches!(
                    d.style,
                    SpriteStyle::House
                        | SpriteStyle::Cabin
                        | SpriteStyle::Hut
                        | SpriteStyle::Inn
                        | SpriteStyle::Barn
                        | SpriteStyle::Watchtower
                );
                if is_building {
                    // Larger, offset shadow for depth; alpha scales with height.
                    let ox = d.half_h * 0.22;
                    let oy = d.half_h * 0.14;
                    push_shadow(out, d.sx + ox, d.sy + oy, d.half_w * 1.35, d.half_h * 0.85);
                    // Second soft shadow for roof overhang.
                    push_shadow_soft(out, d.sx + ox * 0.6, d.sy + oy * 0.6, d.half_w * 0.9, d.half_h * 0.6);
                } else {
                    push_shadow(out, d.sx, d.sy, d.half_w, d.half_h);
                }
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
                        d.facing, d.walk, anim_time, d.flash, d.attack,
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

                // Dodge ghost trail: draw fading copies behind the player
                // during a dodge roll, giving a motion-blur feel.
                if d.dodge_timer > 0.01 {
                    let ghost_count = 3;
                    for i in 1..=ghost_count {
                        let t = i as f32 / ghost_count as f32;
                        let offset = d.dodge_timer * 4.0 * t; // trail behind
                        let gx = d.sx - d.facing.0 * offset * 12.0;
                        let gy2 = gy - d.facing.1 * offset * 12.0;
                        let ga = 0.35 * (1.0 - t); // fade out
                        let ghost_parts = crate::elements::humanoid::build(
                            gx, gy2, PLAYER_COLOR, ga, d.facing, 0.0, 0.0, 0.0,
                        );
                        rasterize(&ghost_parts, out);
                    }
                }

                push_shadow(out, d.sx, gy, HALF_W * 1.05, HALF_H * 1.05);
                let parts = crate::elements::humanoid::build(
                    d.sx,
                    gy,
                    tunic,
                    1.0,
                    d.facing,
                    d.walk,
                    anim_time,
                    d.attack,
                );
                rasterize(&crate::elements::prim::flashed(&parts, d.flash), out);

                // Held weapon: anchored at the forward hand so steel tracks the
                // arm lunge, extended by the strike curve. Flash applies too.
                if d.weapon != WeaponKind::Fists {
                    let (ox, oy) = facing_offset(d.facing, 5.0 + d.attack * 8.0);
                    let wparts = crate::elements::weapon::build(
                        d.weapon,
                        d.sx + ox,
                        gy - 22.0 + oy,
                        d.facing,
                        d.attack,
                        d.enchant,
                        1.0,
                    );
                    rasterize(&crate::elements::prim::flashed(&wparts, d.flash), out);
                }
                // Raised block shield while holding block (drawn over weapon).
                if d.blocking {
                    let sparts =
                        crate::elements::weapon::block_shield(d.sx, gy, d.facing, 0.95);
                    rasterize(&sparts, out);
                }

                // Enchantment glow: a pulsing arcane ring around the player
                // when the weapon is enchanted (levels 1-5).
                if d.enchant > 0 {
                    let intensity = d.enchant as f32 / 5.0;
                    let pulse = (anim_time * 4.0).sin() * 0.3 + 0.7;
                    let glow_alpha = intensity * pulse * 0.35;
                    let glow_color = [0.40, 0.25, 0.70]; // arcane purple
                    let glow_size = 10.0 + intensity * 6.0;
                    rasterize(
                        &[
                            crate::elements::prim::Part::diamond(d.sx, gy - 16.0, glow_size, glow_size * 0.6, 0.0, glow_color, glow_alpha, false),
                            crate::elements::prim::Part::diamond(d.sx, gy - 16.0, glow_size * 0.5, glow_size * 0.3, 0.0, [0.60, 0.45, 0.90], glow_alpha * 0.5, false),
                        ],
                        out,
                    );
                }
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

/// Terrain height level at a tile (mirrors `world::tile_height`).
pub fn tile_height_at(world: &WorldGen, cache: &mut ChunkCache, tx: i32, ty: i32) -> i8 {
    let chunk = cache.get(world, tx, ty);
    chunk.tiles[ty.rem_euclid(crate::world::CHUNK_SIZE) as usize]
        [tx.rem_euclid(crate::world::CHUNK_SIZE) as usize]
        .height
}

/// Draw a tile diamond with per-corner colors. Each corner blends toward the
/// neighbouring tile in that direction, so biome edges become smooth gradients
/// instead of a hard checkerboard. The whole diamond is lifted `z` pixels to
/// sit on its terrain height. Vertex order matches `push_quad` so the geometry
/// tests (positions) stay valid.
fn push_quad_blended(
    out: &mut Vec<f32>,
    ox: f32,
    oy: f32,
    z: f32,
    north: [f32; 3],
    east: [f32; 3],
    south: [f32; 3],
    west: [f32; 3],
) {
    let top = (ox, oy - z);
    let right = (ox + HALF_W, oy + HALF_H - z);
    let bottom = (ox, oy + TILE_HEIGHT - z);
    let left = (ox - HALF_W, oy + HALF_H - z);
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

/// A vertical quad (a "cliff" wall) along the screen segment (x1,y1)->(x2,y2),
/// extruded from height `z_top` (top edge) down to `z_bottom` (bottom edge).
/// Used to connect a raised tile to its lower neighbour so terrain has depth.
fn push_wall(
    out: &mut Vec<f32>,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    z_top: f32,
    z_bottom: f32,
    color: [f32; 3],
) {
    let tl = (x1, y1 - z_top);
    let tr = (x2, y2 - z_top);
    let bl = (x1, y1 - z_bottom);
    let br = (x2, y2 - z_bottom);
    // Two triangles: tl, tr, br  and  tl, br, bl.
    for (vx, vy) in [tl, tr, br, tl, br, bl] {
        out.push(vx);
        out.push(vy);
        out.extend_from_slice(&color);
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
        out.push(0.42);
    }
}

fn push_shadow_soft(out: &mut Vec<f32>, cx: f32, cy: f32, half_w: f32, half_h: f32) {
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
        out.push(0.18);
    }
}

/// Deterministic per-tile pseudo-random so scattered detail is stable across
/// frames (no flicker) but varies from tile to tile.
fn tile_hash(tx: i32, ty: i32) -> u32 {
    let mut h = (tx as u32).wrapping_mul(374761393).wrapping_add((ty as u32).wrapping_mul(668265263));
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    h
}

/// Scatter small procedural detail onto a ground tile so it no longer reads as
/// a flat colour sheet: grass blades (with a gentle sway), dirt/stone speckles,
/// sand grains, snow sparkle. Detail is confined to the tile diamond.
fn tile_detail(out: &mut Vec<f32>, kind: TileKind, sx: f32, sy: f32, z: f32, tx: i32, ty: i32, anim_time: f32) {
    let cx = sx;
    let cy = sy + HALF_H - z; // tile center, lifted to terrain height
    let h = tile_hash(tx, ty);
    let mut parts: Vec<Part> = Vec::new();
    match kind {
        TileKind::Grass | TileKind::Forest => {
            let n = 2 + (h % 2);
            for i in 0..n {
                let r1 = (((h >> (i * 5)) & 31) as f32) / 31.0;
                let r2 = (((h >> (i * 5 + 2)) & 31) as f32) / 31.0;
                let ox = (r1 - 0.5) * (HALF_W * 0.9);
                let oy = (r2 - 0.5) * (HALF_H * 0.7);
                let s = sway(cx + ox, cy + oy, anim_time, 1.2);
                let bh = 4.0 + r2 * 3.0;
                let g = [0.18 + r1 * 0.06, 0.40 + r2 * 0.12, 0.15];
                parts.push(Part::vquad(cx + ox + s - 0.6, cy + oy - bh, 1.2, bh, g, 1.0, false));
            }
            if kind == TileKind::Grass && (h & 7) == 0 {
                let fx = cx + ((h % 17) as f32 - 8.0);
                let fy = cy + ((h % 11) as f32 - 5.0);
                parts.push(Part::diamond(fx, fy - 2.0, 1.6, 1.6, 0.0, [0.92, 0.82, 0.32], 1.0, false));
            }
        }
        TileKind::Swamp => {
            let n = 3 + (h % 3);
            for i in 0..n {
                let r1 = (((h >> (i * 4)) & 15) as f32) / 15.0;
                let r2 = (((h >> (i * 4 + 2)) & 15) as f32) / 15.0;
                let ox = (r1 - 0.5) * (HALF_W * 1.2);
                let oy = (r2 - 0.5) * (HALF_H * 0.9);
                let sh = 0.12 + r2 * 0.10;
                parts.push(Part::diamond(cx + ox, cy + oy, 1.6, 1.0, 0.0, [sh, sh * 0.8, sh * 0.6], 1.0, false));
            }
        }
        TileKind::Jungle => {
            // Dense, layered foliage with the occasional bright flower.
            let n = 4 + (h % 3);
            for i in 0..n {
                let r1 = (((h >> (i * 5)) & 31) as f32) / 31.0;
                let r2 = (((h >> (i * 5 + 2)) & 31) as f32) / 31.0;
                let ox = (r1 - 0.5) * (HALF_W * 1.3);
                let oy = (r2 - 0.5) * (HALF_H * 1.0);
                let s = sway(cx + ox, cy + oy, anim_time, 1.6);
                let bh = 6.0 + r2 * 5.0;
                let g = [0.10 + r1 * 0.06, 0.34 + r2 * 0.14, 0.14];
                parts.push(Part::vquad(cx + ox + s - 0.8, cy + oy - bh, 1.6, bh, g, 1.0, false));
            }
            if (h & 31) == 0 {
                let fx = cx + ((h % 19) as f32 - 9.0);
                let fy = cy + ((h % 13) as f32 - 6.0);
                parts.push(Part::diamond(fx, fy - 2.0, 1.8, 1.8, 0.0, [0.95, 0.85, 0.30], 1.0, false));
            }
        }
        TileKind::Sand | TileKind::Desert => {
            let n = 3 + (h % 3);
            for i in 0..n {
                let r1 = (((h >> (i * 4)) & 15) as f32) / 15.0;
                let r2 = (((h >> (i * 4 + 2)) & 15) as f32) / 15.0;
                let ox = (r1 - 0.5) * (HALF_W * 1.3);
                let oy = (r2 - 0.5) * (HALF_H * 1.0);
                let c = if r2 > 0.5 { [0.86, 0.78, 0.55] } else { [0.62, 0.54, 0.34] };
                parts.push(Part::diamond(cx + ox, cy + oy, 1.3, 0.8, 0.0, c, 1.0, false));
            }
        }
        TileKind::Snow => {
            let n = 2 + (h % 3);
            for i in 0..n {
                let r1 = (((h >> (i * 4)) & 15) as f32) / 15.0;
                let r2 = (((h >> (i * 4 + 2)) & 15) as f32) / 15.0;
                let ox = (r1 - 0.5) * (HALF_W * 1.2);
                let oy = (r2 - 0.5) * (HALF_H * 0.9);
                let tw = 0.7 + 0.3 * (anim_time * 1.5 + (i as f32) + r1 * 6.0).sin().max(0.0);
                parts.push(Part::diamond(cx + ox, cy + oy, 1.2, 1.0, 0.0, [0.95 * tw, 0.97 * tw, 1.0], 1.0, false));
            }
        }
        TileKind::Stone | TileKind::Tundra => {
            let n = 1 + (h % 2);
            for i in 0..n {
                let r1 = (((h >> (i * 4)) & 15) as f32) / 15.0;
                let ox = (r1 - 0.5) * (HALF_W * 0.8);
                parts.push(Part::vquad(cx + ox, cy - 8.0, 0.8, 16.0, [0.12, 0.12, 0.14], 0.6, false));
            }
        }
        TileKind::Volcanic => {
            // Cracked basalt with a glowing lava seam that flickers.
            let n = 2 + (h % 2);
            for i in 0..n {
                let r1 = (((h >> (i * 5)) & 31) as f32) / 31.0;
                let ox = (r1 - 0.5) * (HALF_W * 1.1);
                parts.push(Part::vquad(cx + ox, cy - 9.0, 0.7, 18.0, [0.10, 0.09, 0.10], 0.7, false));
            }
            let gx = cx + ((h % 21) as f32 - 10.0);
            let flick = 0.6 + 0.4 * (anim_time * 3.0 + (h % 7) as f32).sin().max(0.0);
            parts.push(Part::diamond(gx, cy - 2.0, 2.4, 1.4, 0.0, [1.0 * flick, 0.4 * flick, 0.08], 1.0, false));
        }
        _ => {}
    }
    if !parts.is_empty() {
        rasterize(&parts, out);
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
    walk: f32,
    anim_time: f32,
    flash: f32,
    attack: f32,
) {
    use crate::elements::{
        altar, anvil, archer, arrow, banner, barrel, bat, bed, bone_pile, brazier, brute, bush, cactus, campfire, chest,
        crate_box, crystal, dungeon, enchanting_table, fence, fern, flower,
        grass_tuft, healing_totem, humanoid, hpbar, imp, lantern,
        lilypad, mushroom, ocean_leviathan, ore, pillar, portal, raider, reed, rock, rock_pile, rubble, ruin_tower, scorpion_queen, sign,
        slime, spider, stormcaller, statue, toad_king, torch, totem, tree, turret, vines, wall, well, wraith, wolf,
        colossus, spike, farm_plot, house, guard, golem,
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
        SpriteStyle::Slime => {
            let parts = slime::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Humanoid => {
            let parts = humanoid::build(cx, cy, color, alpha, facing, walk, anim_time, attack);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Guard => {
            let parts = guard::build(cx, cy, color, alpha, facing, walk, anim_time, attack);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Golem => {
            let parts = golem::build(cx, cy, color, alpha, facing, walk, anim_time, attack);
            rasterize_flash(&parts, out, flash);
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
        SpriteStyle::Treasure => rasterize(&chest::build(cx, cy, color, alpha, facing, anim_time), out),
        // Buildable structures
        SpriteStyle::Fence => rasterize(&fence::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Torch => rasterize(&torch::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Anvil => rasterize(&anvil::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Bed => rasterize(&bed::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Well => rasterize(&well::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Spike => rasterize(&spike::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::FarmPlot => rasterize(&farm_plot::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Turret => rasterize(&turret::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::HealingTotem => {
            rasterize(&healing_totem::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        // Decorative props
        SpriteStyle::Sign => rasterize(&sign::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Barrel => rasterize(&barrel::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Totem => rasterize(&totem::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::RockPile => {
            rasterize(&rock_pile::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        SpriteStyle::Statue => rasterize(&statue::build(cx, cy, color, alpha, facing, anim_time), out),
        // Enemies (humanoid foes render via SpriteStyle::Humanoid above)
        SpriteStyle::Bat => rasterize(&bat::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Spider => rasterize(&spider::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Imp => rasterize(&imp::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Wraith => rasterize(&wraith::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Colossus => rasterize(&colossus::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::ScorpionQueen => {
            let parts = scorpion_queen::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::ToadKing => {
            let parts = toad_king::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Brute => {
            let parts = brute::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Stormcaller => {
            let parts = stormcaller::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::OceanLeviathan => {
            let parts = ocean_leviathan::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Wolf => {
            let parts = wolf::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Archer => {
            let parts = archer::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Raider => {
            let parts = raider::build(cx, cy, color, alpha, facing, anim_time);
            rasterize_flash(&parts, out, flash);
        }
        SpriteStyle::Car => {
            // Abandoned old-world automobile: body + cabin + two wheels.
            let mut p = Vec::new();
            p.push(Part::diamond(cx - hw * 0.6, cy - 2.0, 3.2, 3.2, -2.0, [0.08, 0.08, 0.09], alpha, false));
            p.push(Part::diamond(cx + hw * 0.6, cy - 2.0, 3.2, 3.2, -2.0, [0.08, 0.08, 0.09], alpha, false));
            p.push(Part::vquad(cx, cy - 14.0, hw, 14.0, color, alpha, true));
            p.push(Part::vquad(
                cx,
                cy - 26.0,
                hw * 0.62,
                12.0,
                [color[0] * 1.15, color[1] * 1.15, color[2] * 1.15],
                alpha,
                true,
            ));
            rasterize(&p, out);
        }
        SpriteStyle::Train => {
            // A rusted locomotive car: long body, roof, lit window strip, wheels.
            let mut p = Vec::new();
            p.push(Part::diamond(cx - hw * 0.7, cy - 2.0, 3.5, 3.5, -2.0, [0.08, 0.08, 0.09], alpha, false));
            p.push(Part::diamond(cx + hw * 0.7, cy - 2.0, 3.5, 3.5, -2.0, [0.08, 0.08, 0.09], alpha, false));
            p.push(Part::vquad(cx, cy - 18.0, hw, 18.0, color, alpha, true));
            p.push(Part::vquad(
                cx,
                cy - 34.0,
                hw,
                8.0,
                [color[0] * 1.1, color[1] * 1.1, color[2] * 1.1],
                alpha,
                true,
            ));
            p.push(Part::vquad(cx, cy - 22.0, hw * 0.7, 6.0, [0.7, 0.8, 0.9], alpha, false));
            rasterize(&p, out);
        }
        SpriteStyle::Rail => {
            // A railway tie bed with two steel rails running through it.
            let mut p = Vec::new();
            p.push(Part::diamond(cx, cy, hw, hh.max(2.0), 0.0, [0.22, 0.2, 0.16], alpha, false));
            p.push(Part::diamond(cx, cy - 2.0, hw * 0.85, 1.4, 0.0, [0.6, 0.6, 0.66], alpha, false));
            p.push(Part::diamond(cx, cy + 2.0, hw * 0.85, 1.4, 0.0, [0.6, 0.6, 0.66], alpha, false));
            rasterize(&p, out);
        }
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
        SpriteStyle::RuinTower => rasterize(&ruin_tower::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Portal => rasterize(&portal::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Banner => rasterize(&banner::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::EnchantingTable => {
            rasterize(&enchanting_table::build(cx, cy, color, alpha, facing, anim_time), out)
        }
        SpriteStyle::Dungeon => rasterize(&dungeon::build(cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::House => rasterize(&house::build(0, cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Cabin => rasterize(&house::build(1, cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Hut => rasterize(&house::build(2, cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Inn => rasterize(&house::build(3, cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Barn => rasterize(&house::build(4, cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Watchtower => rasterize(&house::build(5, cx, cy, color, alpha, facing, anim_time), out),
        SpriteStyle::Floor => {
            rasterize(&[Part::diamond(cx, cy, hw.max(1.0), hh.max(1.0), 0.0, color, alpha, false)], out)
        }
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
        let quads = build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &visible_tiles(cam(), (640.0, 360.0)), &[], None, &mut mesh, 0.0, 0.0);
        assert!(quads > 0);
        // Tile (0,0) is lifted by its terrain height.
        let z0 = tile_height_at(&world, &mut cache, 0, 0) as f32 * crate::world::HEIGHT_STEP;
        // find the quad whose top corner sits at world (0, -z0) (tile 0,0)
        let mut found = None;
        for i in 0..quads as usize {
            let v = quad_vertices(&mesh, i);
            if v[0] == 0.0 && (v[1] + z0).abs() < 0.001 {
                found = Some(v);
                break;
            }
        }
        let v = found.expect("tile (0,0) must be in the mesh");
        // vertex layout per vertex: [x, y, r, g, b, a]
        // v0=top, v1=right, v2=bottom, v3=top, v4=bottom, v5=left
        assert!((v[6] - HALF_W).abs() < 0.001, "right corner x");
        assert!((v[7] - (HALF_H - z0)).abs() < 0.001, "right corner y");
        assert!((v[12] - 0.0).abs() < 0.001, "bottom corner x");
        assert!((v[13] - (TILE_HEIGHT - z0)).abs() < 0.001, "bottom corner y");
        assert!((v[30] + HALF_W).abs() < 0.001, "left corner x");
        assert!((v[31] - (HALF_H - z0)).abs() < 0.001, "left corner y");
    }

    #[test]
    fn mesh_quads_have_valid_colors() {
        let world = WorldGen::new(7);
        let mut cache = ChunkCache::new(64);
        let mut mesh = Vec::new();
        let quads = build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &visible_tiles(cam(), (640.0, 360.0)), &[], None, &mut mesh, 0.0, 0.0);
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
            &visible_tiles(cam(), (640.0, 360.0)),
            &[],
            Some(&player),
            &mut mesh,
            0.0,
            0.0,
        );
        // player branch emits: ground shadow (1) + humanoid (2 legs, torso, 2
        // arms, 2 hands, head, hair = 9 parts) each drawn as bright + dark-skirt
        // + bright = 9*3 + 1 = 28 quads
        let mut plain = Vec::new();
        let plain_quads =
            build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &visible_tiles(cam(), (640.0, 360.0)), &[], None, &mut plain, 0.0, 0.0);
        assert_eq!(quads, plain_quads + 28);

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
        let z0 = tile_height_at(&world, &mut cache, 0, 0) as f32 * crate::world::HEIGHT_STEP;
        // The torso (PLAYER_COLOR) is centered on the player's screen x (ox).
        let cx_avg = (0..6).map(|k| v[k * 6]).sum::<f32>() / 6.0;
        assert!((cx_avg - ox).abs() < 0.001, "player center x");
        // and it stands on the tile center (gy = oy + HALF_H), lifted by terrain
        // height, so its bottom edge sits below the lifted tile top corner.
        let cy_max = (0..6).map(|k| v[k * 6 + 1]).fold(f32::MIN, f32::max);
        assert!(cy_max > oy - z0, "player torso must be below the lifted tile top corner");

        // depth ordering: tile (0,0) (depth 0) must come before the player (depth 1)
        let mut idx = 0;
        for i in 0..quads as usize {
            let q = quad_vertices(&mesh, i);
            if q[2] == PLAYER_COLOR[0] {
                idx = i;
                break;
            }
        }
        // tile (0,0)'s top corner sits at screen (0, -z0) when camera is at origin
        let mut tile_zero_seen = false;
        for i in 0..idx {
            let q = quad_vertices(&mesh, i);
            if q[0] == 0.0 && (q[1] + z0).abs() < 0.001 {
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
            &visible_tiles(cam(), (640.0, 360.0)),
            &[tree],
            None,
            &mut mesh,
            0.0,
            0.0,
        );
        let tree_idx = (0..quads as usize)
            .find(|&i| quad_vertices(&mesh, i)[2] == 0.06)
            .expect("tree sprite must be in the mesh");

        let z0 = tile_height_at(&world, &mut cache, 0, 0) as f32 * crate::world::HEIGHT_STEP;
        // its center sits at the tile center: iso(0.5, 0.5) = (0, 16)
        let v = quad_vertices(&mesh, tree_idx);
        assert!((v[0] - 0.0).abs() < 0.001, "tree top x (center column)");
        assert!((v[1] - (16.0 - 20.0 + 8.0 - z0)).abs() < 0.001, "tree top y (lifted)");

        // tile (0,0) must render before the tree
        assert!(
            (0..tree_idx).any(|i| {
                let q = quad_vertices(&mesh, i);
                q[0] == 0.0 && (q[1] + z0).abs() < 0.001
            }),
            "tile (0,0) must render before the tree on it"
        );
        // and the tree must render before tiles of depth 1 (e.g. tile (1,0))
        let z1 = tile_height_at(&world, &mut cache, 1, 0) as f32 * crate::world::HEIGHT_STEP;
        assert!(
            (tree_idx + 1..quads as usize).any(|i| {
                let q = quad_vertices(&mesh, i);
                (q[0] - 32.0).abs() < 0.001 && (q[1] - (16.0 - z1)).abs() < 0.001
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
            &visible_tiles(cam(), (640.0, 360.0)),
            &[far],
            None,
            &mut mesh,
            0.0,
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
        build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &visible_tiles(cam(), (640.0, 360.0)), &[slime], None, &mut m0, 0.0, 0.0);
        let mut m1 = Vec::new();
        build_tile_mesh(&world, &mut cache, cam(), (640.0, 360.0), &visible_tiles(cam(), (640.0, 360.0)), &[slime], None, &mut m1, 0.3, 0.0);

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