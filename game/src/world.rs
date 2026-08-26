use noise::{Fbm, NoiseFn, Perlin};
use std::collections::HashMap;

use crate::building::{decor_on, StructureKind};
use crate::resources::{resource_on, ResourceKind};

pub const CHUNK_SIZE: i32 = 32;
pub const RENDER_RADIUS: i32 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileKind {
    DeepWater,
    Water,
    ShallowWater,
    Sand,
    Grass,
    Forest,
    Swamp,
    Snow,
    Stone,
    Tundra,
    Desert,
    /// Lush, humid lowland forest — hot but vegetated. A new, distinct biome
    /// with its own palette, denser foliage, and its own critters.
    Jungle,
    /// Scorched volcanic highlands: cracked basalt and lava seams. A hostile
    /// biome that slowly burns the player unless they keep moving or shelter.
    Volcanic,
}

impl TileKind {
    pub fn color(self) -> [f32; 3] {
        match self {
            TileKind::DeepWater => [0.04, 0.16, 0.34],
            TileKind::Water => [0.09, 0.33, 0.52],
            TileKind::ShallowWater => [0.17, 0.52, 0.60],
            TileKind::Sand => [0.86, 0.78, 0.55],
            TileKind::Grass => [0.35, 0.58, 0.27],
            TileKind::Forest => [0.22, 0.44, 0.19],
            TileKind::Swamp => [0.30, 0.40, 0.24],
            TileKind::Snow => [0.92, 0.95, 0.98],
            TileKind::Stone => [0.50, 0.50, 0.52],
            TileKind::Tundra => [0.66, 0.70, 0.62],
            TileKind::Desert => [0.91, 0.80, 0.50],
            TileKind::Jungle => [0.16, 0.40, 0.18],
            TileKind::Volcanic => [0.22, 0.10, 0.09],
        }
    }

    pub fn walkable(self) -> bool {
        !matches!(self, TileKind::DeepWater | TileKind::Water)
    }

    /// True for the crossable shallow water (wading) tiles.
    pub fn wadable(self) -> bool {
        matches!(self, TileKind::ShallowWater)
    }
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub kind: TileKind,
    /// Integer terrain height level. 0 = sea-level baseline; positive = hills /
    /// mountains; negative = basins (water sits below the land). Rendering
    /// multiplies this by `HEIGHT_STEP` pixels to extrude each tile, giving the
    /// world real up/down relief. Purely visual — movement ignores it for now.
    pub height: i8,
}

/// Pixels of vertical screen offset per `Tile::height` level. Controls how
/// dramatic the relief reads.
pub const HEIGHT_STEP: f32 = 9.0;

pub struct Chunk {
    pub cx: i32,
    pub cy: i32,
    pub tiles: [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    /// Resource nodes present in this chunk (world coords + kind), resolved
    /// once at generation so the per-frame sprite pass never re-hashes tiles.
    pub resources: Vec<(i32, i32, ResourceKind)>,
    /// Decorative props present in this chunk (world coords + kind).
    pub decor: Vec<(i32, i32, StructureKind)>,
}

pub struct WorldGen {
    seed: u32,
    elevation: Fbm<Perlin>,
    moisture: Fbm<Perlin>,
    temperature: Fbm<Perlin>,
    river: Fbm<Perlin>,
}

/// LRU-ish chunk cache: generates chunks on demand, clears wholesale when
/// over capacity (a new camera region invalidates old chunks anyway).
pub struct ChunkCache {
    chunks: HashMap<(i32, i32), Chunk>,
    order: std::collections::VecDeque<(i32, i32)>,
    capacity: usize,
}

impl ChunkCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            order: std::collections::VecDeque::new(),
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn contains(&self, cx: i32, cy: i32) -> bool {
        self.chunks.contains_key(&(cx, cy))
    }

    /// Evict the least-recently-used chunk (kept near the camera so scrolling
    /// back doesn't force a full regenerate-and-pop like the old wholesale
    /// clear did).
    fn evict_one(&mut self) {
        if let Some((cx, cy)) = self.order.pop_front() {
            self.chunks.remove(&(cx, cy));
        }
    }

    pub fn get(&mut self, world: &WorldGen, tx: i32, ty: i32) -> &Chunk {
        let cx = tx.div_euclid(CHUNK_SIZE);
        let cy = ty.div_euclid(CHUNK_SIZE);
        if !self.chunks.contains_key(&(cx, cy)) {
            while self.chunks.len() >= self.capacity {
                self.evict_one();
            }
            let chunk = world.generate_chunk(cx, cy);
            self.chunks.insert((cx, cy), chunk);
            self.order.push_back((cx, cy));
        } else {
            // Mark as most-recently-used.
            if let Some(pos) = self.order.iter().position(|&k| k == (cx, cy)) {
                self.order.remove(pos);
            }
            self.order.push_back((cx, cy));
        }
        &self.chunks[&(cx, cy)]
    }
}

/// Tile kind at a world tile coordinate (generates the chunk on demand).
pub fn tile_at(world: &WorldGen, cache: &mut ChunkCache, tx: i32, ty: i32) -> TileKind {
    let chunk = cache.get(world, tx, ty);
    chunk.tiles[ty.rem_euclid(CHUNK_SIZE) as usize][tx.rem_euclid(CHUNK_SIZE) as usize].kind
}

/// Terrain height level at a world tile coordinate (generates the chunk on
/// demand). Multiplied by `HEIGHT_STEP` by the renderer to extrude relief.
pub fn tile_height(world: &WorldGen, cache: &mut ChunkCache, tx: i32, ty: i32) -> i8 {
    let chunk = cache.get(world, tx, ty);
    chunk.tiles[ty.rem_euclid(CHUNK_SIZE) as usize][tx.rem_euclid(CHUNK_SIZE) as usize].height
}

impl WorldGen {
    pub fn new(seed: u32) -> Self {
        let mut elevation = Fbm::<Perlin>::new(seed);
        elevation.frequency = 0.008;
        let mut moisture = Fbm::<Perlin>::new(seed ^ 0x9E3779B9);
        moisture.frequency = 0.015;
        let mut temperature = Fbm::<Perlin>::new(seed ^ 0x85EBCA6B);
        temperature.frequency = 0.006;
        let mut river = Fbm::<Perlin>::new(seed ^ 0xC2B2AE35);
        river.frequency = 0.012;
        Self {
            seed,
            elevation,
            moisture,
            temperature,
            river,
        }
    }

    /// The seed this world was generated from (for display / determinism).
    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn generate_chunk(&self, cx: i32, cy: i32) -> Chunk {
        let mut tiles = [[Tile { kind: TileKind::DeepWater, height: 0 }; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        let mut resources = Vec::new();
        let mut decor = Vec::new();
        for ty in 0..CHUNK_SIZE {
            for tx in 0..CHUNK_SIZE {
                let gx = cx * CHUNK_SIZE + tx;
                let gy = cy * CHUNK_SIZE + ty;
                let wx = gx as f64;
                let wy = gy as f64;
                let e = self.elevation.get([wx, wy]) as f32;
                let m = self.moisture.get([wx, wy]) as f32;
                let t = self.temperature.get([wx, wy]) as f32;
                let mut kind = self.classify(e, m, t);
                // Intra-biome detail: deterministic swaps between walkable
                // sibling tiles so regions aren't flat. Never introduces water
                // or changes overall biome presence, so spawn/POI logic holds.
                let h = (gx.wrapping_mul(73856093) ^ gy.wrapping_mul(19349663)) as f64;
                kind = match kind {
                    TileKind::Grass if h.rem_euclid(11.0) == 0.0 => TileKind::Forest,
                    TileKind::Forest if h.rem_euclid(13.0) == 0.0 => TileKind::Grass,
                    TileKind::Stone if h.rem_euclid(17.0) == 0.0 => TileKind::Snow,
                    TileKind::Snow if h.rem_euclid(19.0) == 0.0 => TileKind::Stone,
                    TileKind::Tundra if h.rem_euclid(23.0) == 0.0 => TileKind::Snow,
                    TileKind::Snow if h.rem_euclid(29.0) == 0.0 => TileKind::Tundra,
                    TileKind::Desert if h.rem_euclid(31.0) == 0.0 => TileKind::Sand,
                    TileKind::Sand if h.rem_euclid(37.0) == 0.0 => TileKind::Desert,
                    other => other,
                };
                tiles[ty as usize][tx as usize].kind = kind;
                // Terrain relief: water sits in a basin below land; land rises
                // with elevation so hills/peaks emerge from the plains. Gentle
                // steps (integer levels) keep the extruded walls readable.
                let height = if matches!(kind, TileKind::DeepWater) {
                    -2
                } else if matches!(kind, TileKind::Water) {
                    -1
                } else if matches!(kind, TileKind::ShallowWater) {
                    0
                } else {
                    let h = ((e + 0.05) * 7.0).round().clamp(0.0, 16.0) as i8;
                    h
                };
                tiles[ty as usize][tx as usize].height = height;
                // Cache resource/decor placement for this tile (deterministic,
                // so it only needs computing once, at generation time).
                if let Some(rk) = resource_on(gx, gy, kind) {
                    resources.push((gx, gy, rk));
                }
                if let Some(dk) = decor_on(gx, gy, kind) {
                    decor.push((gx, gy, dk));
                }
            }
        }
        Chunk { cx, cy, tiles, resources, decor }
    }

    fn classify(&self, elevation: f32, moisture: f32, temperature: f32) -> TileKind {
        if elevation < -0.30 {
            if elevation < -0.45 {
                TileKind::DeepWater
            } else {
                TileKind::Water
            }
        } else if elevation < -0.05 {
            // Lowlands: arid ones become desert, the rest sandy shore.
            if moisture < -0.30 {
                TileKind::Desert
            } else {
                TileKind::Sand
            }
        } else if elevation > 0.45 {
            // Very hot, arid peaks become scorched volcanic highlands; temperate
            // peaks stay as normal Stone.
            if temperature > 0.45 && moisture < 0.10 {
                TileKind::Volcanic
            } else {
                TileKind::Stone
            }
        } else if temperature < -0.45 {
            TileKind::Snow
        } else if temperature < -0.25 {
            TileKind::Tundra
        } else if moisture > 0.55 && temperature > 0.15 {
            TileKind::Jungle
        } else if moisture > 0.35 {
            if elevation < 0.15 {
                TileKind::Swamp
            } else {
                TileKind::Forest
            }
        } else if moisture < -0.30 {
            TileKind::Desert
        } else {
            TileKind::Grass
        }
    }
}