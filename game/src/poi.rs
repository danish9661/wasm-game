use crate::building::{Structure, StructureKind};
use crate::world::{ChunkCache, WorldGen, tile_at};

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
/// first walkable candidate so hamlets never spawn in the sea. Sites keep a
/// minimum separation (`SETTLEMENT_GAP`) so hamlet house-rings (radius 12)
/// can never overlap each other into merged roof blobs.
pub const SETTLEMENT_GAP: i32 = 26;

fn far_enough(out: &[(i32, i32)], tx: i32, ty: i32) -> bool {
    out.iter()
        .all(|(ox, oy)| (ox - tx).abs().max((oy - ty).abs()) >= SETTLEMENT_GAP)
}

pub fn village_sites(seed: u32, count: usize, mut is_walkable: impl FnMut(i32, i32) -> bool) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let mut h = seed.wrapping_mul(40503).wrapping_add(0x9e37);
    let mut i = 0;
    while out.len() < count && i < VILLAGE_CANDIDATES.len() * 2 {
        h = h.wrapping_mul(1664525).wrapping_add(1013904223);
        let (tx, ty) = VILLAGE_CANDIDATES[(h >> 16) as usize % VILLAGE_CANDIDATES.len()];
        if is_walkable(tx, ty) && !out.contains(&(tx, ty)) && far_enough(&out, tx, ty) {
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
                if is_walkable(tx, ty) && !out.contains(&(tx, ty)) && far_enough(&out, tx, ty) {
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

/// Full point-of-interest structure layout for a world seed: ruins chest +
/// flanking walls, village hamlets (sign, houses on the 7/12 ring, anvil,
/// well, portal), walled town (boundary, plaza, railway, buildings, cars),
/// and scattered dungeon entrances.
///
/// This is the SINGLE source of truth for world structures: the WASM client's
/// `reset_world` and the authoritative co-op `Simulation::new` both call it,
/// so single-player and multiplayer share one identical world for a seed.
/// NPCs, names, portal state and the town build-in animation stay client-side
/// (cosmetic); everything that blocks movement, gives light, or holds loot is
/// here.
pub fn poi_structures(seed: u32, world: &WorldGen, cache: &mut ChunkCache) -> Vec<Structure> {
    let mut structures = Vec::new();

    // Ruins: chest + flanking U walls.
    let ruins = ruins_at(seed, |tx, ty| tile_at(world, cache, tx, ty).walkable());
    structures.push(Structure { tx: ruins.0, ty: ruins.1, kind: StructureKind::Chest });
    for (wx, wy) in ruins_walls(ruins.0, ruins.1) {
        structures.push(Structure { tx: wx, ty: wy, kind: StructureKind::Wall });
    }

    // Villages.
    let sites = village_sites(seed, 3, |tx, ty| tile_at(world, cache, tx, ty).walkable());
    let first_village = sites.first().copied();
    let house_kinds = [StructureKind::House, StructureKind::Cabin, StructureKind::Hut];
    let special_kinds = [StructureKind::Inn, StructureKind::Barn, StructureKind::Watchtower, StructureKind::House];
    // Houses render oversized: ring at 7/12 keeps a clear spawn plaza.
    let ring: [(i32, i32); 12] = [
        (7, 0), (-7, 0), (0, 7), (0, -7),
        (7, 7), (-7, -7), (7, -7), (-7, 7),
        (12, 0), (-12, 0), (0, 12), (0, -12),
    ];
    for (vx, vy) in sites {
        structures.push(Structure { tx: vx, ty: vy, kind: StructureKind::Sign });
        for (i, (dx, dy)) in ring.iter().enumerate() {
            let (hx, hy) = (vx + dx, vy + dy);
            if tile_at(world, cache, hx, hy).walkable() {
                let kind = if i < 8 { house_kinds[i % 3] } else { special_kinds[(i - 8) % special_kinds.len()] };
                structures.push(Structure { tx: hx, ty: hy, kind });
            }
        }
        if tile_at(world, cache, vx + 2, vy).walkable() {
            structures.push(Structure { tx: vx + 2, ty: vy, kind: StructureKind::Anvil });
        }
        if tile_at(world, cache, vx - 2, vy).walkable() {
            structures.push(Structure { tx: vx - 2, ty: vy, kind: StructureKind::Well });
        }
    }

    // Village portal (first hamlet only).
    if let Some((fvx, fvy)) = first_village {
        let spots = [(fvx, fvy - 2), (fvx, fvy + 2), (fvx + 2, fvy), (fvx - 2, fvy)];
        if let Some(&(px, py)) = spots
            .iter()
            .find(|&&(x, y)| tile_at(world, cache, x, y).walkable())
        {
            structures.push(Structure { tx: px, ty: py, kind: StructureKind::Portal });
        }
    }

    // Town / city.
    let (tx0, ty0) = town_site(seed, |tx, ty| tile_at(world, cache, tx, ty).walkable());
    let r = 14;
    for x in (tx0 - r)..=(tx0 + r) {
        for y in (ty0 - r)..=(ty0 + r) {
            let edge = x == tx0 - r || x == tx0 + r || y == ty0 - r || y == ty0 + r;
            if !edge {
                continue;
            }
            let gate = ((x == tx0 - r || x == tx0 + r) && (y - ty0).abs() <= 1)
                || ((y == ty0 - r || y == ty0 + r) && (x - tx0).abs() <= 1);
            if gate {
                continue;
            }
            if tile_at(world, cache, x, y).walkable() {
                structures.push(Structure { tx: x, ty: y, kind: StructureKind::Wall });
            }
        }
    }
    structures.push(Structure { tx: tx0, ty: ty0, kind: StructureKind::Sign });
    let rail_y = ty0 + 4;
    for x in (tx0 - r + 1)..=(tx0 + r - 1) {
        if tile_at(world, cache, x, rail_y).walkable() {
            structures.push(Structure { tx: x, ty: rail_y, kind: StructureKind::Rail });
        }
    }
    structures.push(Structure { tx: tx0, ty: rail_y, kind: StructureKind::Train });
    let bkinds = [
        StructureKind::House,
        StructureKind::Cabin,
        StructureKind::Hut,
        StructureKind::Inn,
        StructureKind::Barn,
        StructureKind::Watchtower,
    ];
    let mut bi = 0usize;
    // Roomy 6-tile grid (house art runs ~200px tall — a 5-grid overlapped
    // roofs). Skips the plaza and the rail row.
    for gx in (tx0 - 10..=tx0 + 10).step_by(6) {
        for gy in (ty0 - 10..=ty0 + 10).step_by(6) {
            if (gx - tx0).abs() <= 2 && (gy - ty0).abs() <= 2 {
                continue;
            }
            if gy == rail_y {
                continue;
            }
            if tile_at(world, cache, gx, gy).walkable() {
                let k = bkinds[bi % bkinds.len()];
                bi += 1;
                structures.push(Structure { tx: gx, ty: gy, kind: k });
            }
        }
    }
    let mut h = ((tx0 as u32) ^ (ty0 as u32)).wrapping_mul(2654435761);
    for _ in 0..8 {
        h = h.wrapping_mul(1664525).wrapping_add(1013904223);
        let cx = tx0 - 10 + (((h as i32) % 21i32).abs());
        let cy = ty0 - 10 + (((h >> 8) as i32) % 21i32).abs();
        if (cx - tx0).abs() <= 1 && (cy - ty0).abs() <= 1 {
            continue;
        }
        if cy == rail_y {
            continue;
        }
        if tile_at(world, cache, cx, cy).walkable()
            && !structures.iter().any(|s| s.tx == cx && s.ty == cy)
        {
            structures.push(Structure { tx: cx, ty: cy, kind: StructureKind::Car });
        }
    }

    // Dungeon entrances, away from the spawn village and the ruins.
    {
        let start = first_village.unwrap_or((tx0, ty0));
        let sp = (start.0, start.1);
        let mut h = (seed ^ 0x9e37_79b9).wrapping_mul(2654435761);
        let mut placed = 0;
        for _ in 0..240 {
            h = h.wrapping_mul(1664525).wrapping_add(1013904223);
            let tx = sp.0 + ((h as i32) % 160) - 80;
            let ty = sp.1 + (((h >> 11) as i32) % 160) - 80;
            if (tx - sp.0).abs() < 30 && (ty - sp.1).abs() < 30 {
                continue;
            }
            if (tx - ruins.0).abs() < 12 && (ty - ruins.1).abs() < 12 {
                continue;
            }
            if tile_at(world, cache, tx, ty).walkable()
                && !structures.iter().any(|s| s.tx == tx && s.ty == ty)
            {
                structures.push(Structure { tx, ty, kind: StructureKind::Dungeon });
                placed += 1;
                if placed >= 5 {
                    break;
                }
            }
        }
    }

    // Spacing pass: drop house-kind structures that crowd another house-kind
    // within 4 tiles (neighbouring hamlet rings, town edges) or a scattered
    // decor home within 2. Merged oversized roofs read as one mud blob; a
    // missing house reads as a garden. Village houses come first in the list
    // so they win ties (spawn readability matters most).
    let mut kept: Vec<(i32, i32)> = Vec::new();
    structures.retain(|s| {
        let (tx, ty, kind) = (s.tx, s.ty, s.kind);
        if !matches!(
            kind,
            StructureKind::House
                | StructureKind::Cabin
                | StructureKind::Hut
                | StructureKind::Inn
                | StructureKind::Barn
                | StructureKind::Watchtower
        ) {
            return true;
        }
        if kept
            .iter()
            .any(|(ox, oy)| (ox - tx).abs().max((oy - ty).abs()) < 4)
        {
            return false;
        }
        for dx in -2..=2 {
            for dy in -2..=2 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (tx + dx, ty + dy);
                let tile = tile_at(world, cache, nx, ny);
                if matches!(
                    crate::building::decor_on(nx, ny, tile),
                    Some(StructureKind::House)
                        | Some(StructureKind::Cabin)
                        | Some(StructureKind::Hut)
                ) {
                    return false;
                }
            }
        }
        kept.push((tx, ty));
        true
    });

    structures
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
    fn ruins_is_deterministic() {        for seed in [1u32, 1337, 42, 999] {
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

    #[test]
    fn poi_layout_is_deterministic_and_complete() {
        use crate::building::StructureKind;
        let build = |seed| {
            let world = WorldGen::new(seed);
            let mut cache = ChunkCache::new(128);
            super::poi_structures(seed, &world, &mut cache)
        };
        let a = build(1337);
        let b = build(1337);
        assert_eq!(a.len(), b.len(), "same seed must yield same layout");
        assert!(a.iter().zip(b.iter()).all(|(x, y)| x.tx == y.tx && x.ty == y.ty && x.kind == y.kind));
        let has = |k: StructureKind| a.iter().any(|s| s.kind == k);
        assert!(has(StructureKind::Chest), "ruins chest present");
        assert!(has(StructureKind::House), "village houses present");
        assert!(has(StructureKind::Anvil), "village anvil present");
        assert!(has(StructureKind::Portal), "village portal present");
        assert!(has(StructureKind::Train), "town railway present");
        assert!(has(StructureKind::Dungeon), "dungeon entrances present");
        // No two structures share a tile.
        let mut seen = std::collections::HashSet::new();
        for s in &a {
            assert!(seen.insert((s.tx, s.ty, s.kind as u8)), "duplicate structure at ({},{})", s.tx, s.ty);
        }
        // House-kinds anywhere stand roomy: no two within 4 tiles, so
        // oversized roofs never collide. (Cars may park closer; they are low.)
        let houses: Vec<(i32, i32)> = a
            .iter()
            .filter(|s| {
                matches!(
                    s.kind,
                    StructureKind::House
                        | StructureKind::Cabin
                        | StructureKind::Hut
                        | StructureKind::Inn
                        | StructureKind::Barn
                        | StructureKind::Watchtower
                )
            })
            .map(|s| (s.tx, s.ty))
            .collect();
        assert!(!houses.is_empty(), "world must contain buildings");
        for (i, (ax, ay)) in houses.iter().enumerate() {
            for (bx, by) in &houses[i + 1..] {
                let d = (ax - bx).abs().max((ay - by).abs());
                assert!(d >= 4, "houses too close: ({ax},{ay}) vs ({bx},{by})");
            }
        }
    }
}