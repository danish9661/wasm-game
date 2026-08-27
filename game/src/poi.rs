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

/// Candidate village sites, further out than ruins so hamlets feel like their
/// own settlements rather than backyard camps.
const VILLAGE_CANDIDATES: [(i32, i32); 12] = [
    (30, -18),
    (-28, 22),
    (34, 14),
    (-32, -10),
    (20, 30),
    (-22, -30),
    (40, -8),
    (-38, 16),
    (26, -32),
    (8, 38),
    (-40, -22),
    (38, 30),
];

/// Deterministic village sites for a seed (up to `count`). Each is placed on the
/// first walkable candidate so hamlets never spawn in the sea.
pub fn village_sites(seed: u32, count: usize, mut is_walkable: impl FnMut(i32, i32) -> bool) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let mut h = seed.wrapping_mul(40503).wrapping_add(0x9e37);
    let mut i = 0;
    while out.len() < count && i < VILLAGE_CANDIDATES.len() * 2 {
        h = h.wrapping_mul(1664525).wrapping_add(1013904223);
        let (tx, ty) = VILLAGE_CANDIDATES[(h >> 16) as usize % VILLAGE_CANDIDATES.len()];
        if is_walkable(tx, ty) && !out.contains(&(tx, ty)) {
            out.push((tx, ty));
        }
        i += 1;
    }
    // Fallback: if the fixed candidates all landed in water/mountain for this
    // seed, spiral out from the origin and grab the first walkable tiles so a
    // world ALWAYS has at least one village to spawn into (otherwise the player
    // would be dropped into the wild).
    let mut r: i32 = 1;
    while out.len() < count && r < 240 {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue; // ring only
                }
                let (tx, ty) = (dx, dy);
                if is_walkable(tx, ty) && !out.contains(&(tx, ty)) {
                    out.push((tx, ty));
                    if out.len() >= count {
                        return out;
                    }
                }
            }
        }
        r += 1;
    }
    out
}

/// A stable, vaguely fantasy village name derived from the site coordinates so
/// the same world always names its hamlets the same way.
pub fn village_name(tx: i32, ty: i32) -> String {
    const PRE: &[&str] = &["Briar", "Oak", "Stone", "Moor", "Fen", "Hollow", "Ash", "Thorn", "Mist", "Grey"];
    const SUF: &[&str] = &["dale", "ford", "wick", "mere", "bury", "hollow", "stead", "reach", "croft", "end"];
    let h = ((tx as u32).wrapping_mul(73856093) ^ (ty as u32).wrapping_mul(19349663)).wrapping_add(0x51ab);
    let p = PRE[(h >> 4) as usize % PRE.len()];
    let s = SUF[(h >> 12) as usize % SUF.len()];
    format!("{}{}", p, s)
}

/// Candidate town sites, kept far from spawn (and from the villages) so the
/// city reads as a distant destination you travel to rather than a backyard.
const TOWN_CANDIDATES: [(i32, i32); 8] = [
    (0, 80),
    (80, 0),
    (-80, 0),
    (0, -80),
    (60, 60),
    (-60, -60),
    (60, -60),
    (-60, 60),
];

/// Deterministic town site for a seed (a single city per world). Picks the first
/// walkable candidate; a spiral scan guards against all candidates hitting water.
pub fn town_site(seed: u32, mut is_walkable: impl FnMut(i32, i32) -> bool) -> (i32, i32) {
    let mut h = seed.wrapping_mul(0x2545F491).wrapping_add(0x9e37);
    for _ in 0..TOWN_CANDIDATES.len() * 2 {
        h = h.wrapping_mul(1664525).wrapping_add(1013904223);
        let (tx, ty) = TOWN_CANDIDATES[(h >> 16) as usize % TOWN_CANDIDATES.len()];
        if is_walkable(tx, ty) {
            return (tx, ty);
        }
    }
    let (cx, cy) = TOWN_CANDIDATES[0];
    for r in 1i32..48 {
        for dx in -r..=r {
            for dy in -r..=r {
                if dx.abs().max(dy.abs()) != r {
                    continue;
                }
                if is_walkable(cx + dx, cy + dy) {
                    return (cx + dx, cy + dy);
                }
            }
        }
    }
    (cx, cy)
}

/// A stable, more "industrial" name for the town (old-world flavor).
pub fn town_name(tx: i32, ty: i32) -> String {
    const PRE: &[&str] = &["Old", "Ash", "Iron", "Cog", "Rust", "Vale", "Stone", "North", "East", "Grand"];
    const SUF: &[&str] = &["ford", "haven", "burgh", "ton", "gate", "hollow", "cross", "port", "reach", "field"];
    let h = ((tx as u32).wrapping_mul(0x85EBCA6B) ^ (ty as u32).wrapping_mul(0xC2B2AE35)).wrapping_add(0x1234);
    let p = PRE[(h >> 4) as usize % PRE.len()];
    let s = SUF[(h >> 12) as usize % SUF.len()];
    format!("{}{}", p, s)
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