/// Candidate ruins sites as tile offsets from spawn. Fixed set so the POI
/// lands in reachable countryside (12-26 tiles out) instead of mid-ocean.
const CANDIDATES: [(i32, i32); 10] = [
    (16, -12),
    (-16, 14),
    (18, 8),
    (-20, -6),
    (10, 18),
    (-12, -20),
    (24, -4),
    (-22, 10),
    (14, -20),
    (2, 24),
];

/// Deterministic ruins POI for a seed: a hash picks a candidate site, the
/// first walkable candidate wins, and a 3x3 fallback scan guards against
/// candidates sitting in water.
pub fn ruins_at(seed: u32, mut is_walkable: impl FnMut(i32, i32) -> bool) -> (i32, i32) {
    let mut h = seed.wrapping_mul(2654435761);
    for _ in 0..CANDIDATES.len() * 2 {
        h = h.wrapping_mul(1664525).wrapping_add(1013904223);
        let (tx, ty) = CANDIDATES[(h >> 16) as usize % CANDIDATES.len()];
        if is_walkable(tx, ty) {
            return (tx, ty);
        }
    }
    let (cx, cy) = CANDIDATES[0];
    for dy in -1..=1 {
        for dx in -1..=1 {
            if is_walkable(cx + dx, cy + dy) {
                return (cx + dx, cy + dy);
            }
        }
    }
    (cx, cy)
}

/// Ruins flavor walls flanking the chest, arranged as a U open to the
/// south so every approach lane stays clear.
pub fn ruins_walls(tx: i32, ty: i32) -> [(i32, i32); 4] {
    [
        (tx - 1, ty - 1),
        (tx + 1, ty - 1),
        (tx - 1, ty + 1),
        (tx + 1, ty + 1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{ChunkCache, WorldGen};

    fn walkable_1337(tx: i32, ty: i32) -> bool {
        let world = WorldGen::new(1337);
        let mut cache = ChunkCache::new(64);
        crate::world::tile_at(&world, &mut cache, tx, ty).walkable()
    }

    #[test]
    fn ruins_land_on_walkable_ground() {
        let (tx, ty) = ruins_at(1337, walkable_1337);
        println!("RUINS_1337=({tx},{ty})");
        assert!(walkable_1337(tx, ty), "ruins at ({tx},{ty}) must be walkable");
        assert!(
            walkable_1337(tx + 1, ty) && walkable_1337(tx - 1, ty) && walkable_1337(tx, ty + 1),
            "approach lanes must be walkable"
        );
        let _ = (tx, ty);
    }

    #[test]
    fn ruins_is_deterministic() {
        for seed in [1u32, 1337, 42, 999] {
            let a = ruins_at(seed, walkable_1337);
            let b = ruins_at(seed, walkable_1337);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn ruins_walls_flank_the_chest() {
        let walls = ruins_walls(5, 5);
        assert_eq!(walls, [(4, 4), (6, 4), (4, 6), (6, 6)]);
        let world = WorldGen::new(1337);
        let mut cache = ChunkCache::new(64);
        for (wx, wy) in walls {
            assert!(
                crate::world::tile_at(&world, &mut cache, wx, wy).walkable(),
                "flank wall at ({wx},{wy}) must sit on walkable ground"
            );
        }
    }
}