use game::enemy::astar;
fn main() {
    let mut is_blocked = |tx: i32, ty: i32| tx == 3 && (0..=2).contains(&ty);
    let path = astar((2, 1), (4, 1), &mut is_blocked);
    println!("path: {path:?}");
}
