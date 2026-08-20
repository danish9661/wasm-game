fn main() {
    let w = game::world::WorldGen::new(1337);
    use game::world::TileKind as K;
    let mut cache = game::world::ChunkCache::new(256);
    let mut best: Option<(i32, i32, i32)> = None;
    let mut count = 0;
    for ty in -150..150 { for tx in -150..150 {
        let k = game::world::tile_at(&w, &mut cache, tx, ty);
        if k != K::Swamp { continue; }
        if game::enemy::spawner_on(tx, ty, k).is_some() {
            count += 1;
            let d = tx.abs() + ty.abs();
            if best.map_or(true, |b| d < b.2) { best = Some((tx, ty, d)); }
        }
    }}
    println!("spawners in 300x300: {count}, nearest: {best:?}");
}
