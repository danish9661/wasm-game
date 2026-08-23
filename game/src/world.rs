use noise::{Fbm, NoiseFn, Perlin};
use std::collections::HashMap;

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
}

impl TileKind {
    pub fn color(self) -> [f32; 3] {
        match self {
            TileKind::DeepWater => [0.08, 0.20, 0.38],
            TileKind::Water => [0.15, 0.35, 0.55],
            TileKind::ShallowWater => [0.22, 0.48, 0.52],
            TileKind::Sand => [0.86, 0.78, 0.55],
            TileKind::Grass => [0.35, 0.58, 0.27],
            TileKind::Forest => [0.22, 0.44, 0.19],
            TileKind::Swamp => [0.30, 0.40, 0.24],
            TileKind::Snow => [0.92, 0.95, 0.98],
            TileKind::Stone => [0.50, 0.50, 0.52],
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
}

pub struct Chunk {
    pub cx: i32,
    pub cy: i32,
    pub tiles: [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
}

pub struct WorldGen {
    elevation: Fbm<Perlin>,
    moisture: Fbm<Perlin>,
    temperature: Fbm<Perlin>,
    river: Fbm<Perlin>,
}

/// LRU-ish chunk cache: generates chunks on demand, clears wholesale when
/// over capacity (a new camera region invalidates old chunks anyway).
pub struct ChunkCache {
    chunks: HashMap<(i32, i32), Chunk>,
    capacity: usize,
}

impl ChunkCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn contains(&self, cx: i32, cy: i32) -> bool {
        self.chunks.contains_key(&(cx, cy))
    }

    pub fn get(&mut self, world: &WorldGen, tx: i32, ty: i32) -> &Chunk {
        let cx = tx.div_euclid(CHUNK_SIZE);
        let cy = ty.div_euclid(CHUNK_SIZE);
        if !self.chunks.contains_key(&(cx, cy)) {
            if self.chunks.len() >= self.capacity {
                self.chunks.clear();
            }
            let chunk = world.generate_chunk(cx, cy);
            self.chunks.insert((cx, cy), chunk);
        }
        &self.chunks[&(cx, cy)]
    }
}

/// Tile kind at a world tile coordinate (generates the chunk on demand).
pub fn tile_at(world: &WorldGen, cache: &mut ChunkCache, tx: i32, ty: i32) -> TileKind {
    let chunk = cache.get(world, tx, ty);
    chunk.tiles[ty.rem_euclid(CHUNK_SIZE) as usize][tx.rem_euclid(CHUNK_SIZE) as usize].kind
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
            elevation,
            moisture,
            temperature,
            river,
        }
    }

    pub fn generate_chunk(&self, cx: i32, cy: i32) -> Chunk {
        let mut tiles = [[Tile { kind: TileKind::DeepWater }; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
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
                    other => other,
                };
                // Rivers: where a separate low-frequency noise forms a ridge and
                // the land is low, carve a crossable (wadeable) stream. Only
                // cuts walkable lowlands, never deep water or high peaks.
                if kind.walkable() && e < 0.18 && (self.river.get([wx, wy]) as f32).abs() < 0.045 {
                    kind = TileKind::ShallowWater;
                }
                tiles[ty as usize][tx as usize].kind = kind;
            }
        }
        Chunk { cx, cy, tiles }
    }

    fn classify(&self, elevation: f32, moisture: f32, temperature: f32) -> TileKind {
        if elevation < -0.30 {
            if elevation < -0.45 {
                TileKind::DeepWater
            } else {
                TileKind::Water
            }
        } else if elevation < -0.05 {
            TileKind::Sand
        } else if temperature < -0.25 {
            TileKind::Snow
        } else if elevation > 0.45 {
            TileKind::Stone
        } else if moisture > 0.35 {
            if elevation < 0.15 {
                TileKind::Swamp
            } else {
                TileKind::Forest
            }
        } else {
            TileKind::Grass
        }
    }
}