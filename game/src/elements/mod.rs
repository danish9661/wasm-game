//! Per-element artwork, isolated so each entity (tree, rock, person, …) lives
//! in its own file and can be replaced without touching the renderer.
//!
//! Every element returns a list of [`Part`]s. `prim::rasterize` is the single
//! place that turns parts into vertices — including the fake-2.5D dark "skirt".
//! When we later add a texture atlas (option B), only `prim` changes: a `Part`
//! already carries an optional `uv` rect, so element files don't need edits.

pub mod prim;

pub(crate) mod tree;
pub(crate) mod rock;
pub(crate) mod bush;
pub(crate) mod wall;
pub(crate) mod chest;
pub(crate) mod campfire;
pub(crate) mod altar;
pub(crate) mod arrow;
pub(crate) mod slime;
pub mod humanoid;
pub(crate) mod weapon;
pub(crate) mod guard;
pub(crate) mod golem;
pub(crate) mod hpbar;

// Harvestable resources
pub(crate) mod mushroom;
pub(crate) mod crystal;
pub(crate) mod flower;
pub(crate) mod grass_tuft;
pub(crate) mod fern;
pub(crate) mod ore;

// Buildable structures
pub(crate) mod fence;
pub(crate) mod torch;
pub(crate) mod anvil;
pub(crate) mod bed;
pub(crate) mod well;
pub(crate) mod spike;
pub(crate) mod farm_plot;
pub(crate) mod turret;
pub(crate) mod healing_totem;

    // Decorative props
    pub(crate) mod sign;
    pub(crate) mod house;
pub(crate) mod barrel;
pub(crate) mod totem;
pub(crate) mod rock_pile;
pub(crate) mod statue;
pub(crate) mod lantern;
pub(crate) mod brazier;
pub(crate) mod crate_box;
pub(crate) mod pillar;
pub(crate) mod bone_pile;
pub(crate) mod cactus;
pub(crate) mod vines;
pub(crate) mod lilypad;
pub(crate) mod reed;
pub(crate) mod rubble;
pub(crate) mod ruin_tower;
pub(crate) mod portal;

// Enemies (humanoid foes share the `humanoid` rig; only creatures keep bespoke art)
pub(crate) mod bat;
pub(crate) mod spider;
pub(crate) mod imp;
pub(crate) mod wraith;
pub(crate) mod colossus;
pub(crate) mod scorpion_queen;
pub(crate) mod toad_king;
pub(crate) mod brute;
pub(crate) mod stormcaller;
pub(crate) mod ocean_leviathan;
pub(crate) mod wolf;
pub(crate) mod archer;
pub(crate) mod raider;

// Structures with bespoke art
pub(crate) mod banner;
pub(crate) mod enchanting_table;
pub(crate) mod dungeon;
// Old-world vehicles (art extracted from the renderer dispatch)
pub(crate) mod car;
pub(crate) mod train;
pub(crate) mod rail;

/// Offline tooling hook: build every element into a flat vertex buffer so a
/// `bin` in this package can rasterize and save PNGs. Each vertex is
/// `x, y, r, g, b, a` (colors in `[0,1]`); every 3 consecutive vertices form
/// one triangle. Coordinates are centered on `(0,0)` — the caller crops to the
/// bounding box. Public so the `gen_pngs` binary (a separate crate) can call it.
pub fn preview_elements() -> Vec<(String, Vec<f32>)> {
    use prim::{rasterize, Part};
    let color = [0.72, 0.74, 0.80];
    // Representative in-game tints so previews match the live game (the pale
    // paint above is only the neutral-rig fallback for the player figure).
    use crate::building::StructureKind as SK;
    use crate::enemy::EnemyKind as EK;
    use crate::resources::ResourceKind as RK;
    let rk = |k: RK| k.color();
    let sk = |k: SK| k.color();
    let ek = |k: EK| k.color();
    let alpha = 1.0;
    let facing = (1.0, 0.0);
    let t = 0.0;
    let mut out: Vec<(String, Vec<f32>)> = Vec::new();
    let add = |out: &mut Vec<(String, Vec<f32>)>, name: &str, parts: Vec<Part>| {
        let mut verts: Vec<f32> = Vec::new();
        rasterize(&parts, &mut verts);
        out.push((name.to_string(), verts));
    };
    add(&mut out, "tree", tree::build(0.0, 0.0, rk(RK::Tree), alpha, facing, t));
    add(&mut out, "rock", rock::build(0.0, 0.0, rk(RK::Rock), alpha, facing, t));
    add(&mut out, "bush", bush::build(0.0, 0.0, rk(RK::Bush), alpha, facing, t));
    add(&mut out, "wall", wall::build(0.0, 0.0, sk(SK::Wall), alpha, facing, t));
    add(&mut out, "chest", chest::build(0.0, 0.0, sk(SK::Chest), alpha, facing, t));
    add(&mut out, "campfire", campfire::build(0.0, 0.0, sk(SK::Campfire), alpha, facing, t));
    add(&mut out, "altar", altar::build(0.0, 0.0, sk(SK::Altar), alpha, facing, t));
    add(&mut out, "slime", slime::build(0.0, 0.0, ek(EK::Slime), alpha, facing, 0.0, t));
    add(&mut out, "humanoid", humanoid::build(0.0, 0.0, color, alpha, facing, 0.0, t, 0.0));
    for k in [
        crate::weapons::WeaponKind::Sword,
        crate::weapons::WeaponKind::Axe,
        crate::weapons::WeaponKind::Spear,
        crate::weapons::WeaponKind::Hammer,
        crate::weapons::WeaponKind::Bow,
        crate::weapons::WeaponKind::Dagger,
        crate::weapons::WeaponKind::Crossbow,
        crate::weapons::WeaponKind::Mace,
        crate::weapons::WeaponKind::Scythe,
    ] {
        add(
            &mut out,
            &format!("weapon_{}", k.name().to_lowercase()),
            weapon::build(k, 0.0, 0.0, facing, 0.5, 0, true, alpha),
        );
    }
    add(&mut out, "block_shield", weapon::block_shield(0.0, 0.0, facing, alpha));
    add(&mut out, "bow_loosed", weapon::build(crate::weapons::WeaponKind::Bow, 0.0, 0.0, facing, 0.6, 0, false, alpha));
    add(&mut out, "mushroom", mushroom::build(0.0, 0.0, rk(RK::Mushroom), alpha, facing, t));
    add(&mut out, "crystal", crystal::build(0.0, 0.0, rk(RK::Crystal), alpha, facing, t));
    add(&mut out, "flower", flower::build(0.0, 0.0, rk(RK::Flower), alpha, facing, t));
    add(&mut out, "grass_tuft", grass_tuft::build(0.0, 0.0, rk(RK::GrassTuft), alpha, facing, t));
    add(&mut out, "fern", fern::build(0.0, 0.0, rk(RK::Fern), alpha, facing, t));
    add(&mut out, "ore", ore::build(0.0, 0.0, rk(RK::Ore), alpha, facing, t));
    add(&mut out, "fence", fence::build(0.0, 0.0, sk(SK::Fence), alpha, facing, t));
    add(&mut out, "torch", torch::build(0.0, 0.0, sk(SK::Torch), alpha, facing, t));
    add(&mut out, "anvil", anvil::build(0.0, 0.0, sk(SK::Anvil), alpha, facing, t));
    add(&mut out, "bed", bed::build(0.0, 0.0, sk(SK::Bed), alpha, facing, t));
    add(&mut out, "well", well::build(0.0, 0.0, sk(SK::Well), alpha, facing, t));
    add(&mut out, "sign", sign::build(0.0, 0.0, sk(SK::Sign), alpha, facing, t));
    // Barrel/crate take their wood tint from callers (in-game browns), so the
    // previews pass the StructureKind tints to match the game.
    add(&mut out, "barrel", barrel::build(0.0, 0.0, sk(SK::Barrel), alpha, facing, t));
    add(&mut out, "totem", totem::build(0.0, 0.0, sk(SK::Totem), alpha, facing, t));
    add(&mut out, "rock_pile", rock_pile::build(0.0, 0.0, sk(SK::RockPile), alpha, facing, t));
    add(&mut out, "statue", statue::build(0.0, 0.0, sk(SK::Statue), alpha, facing, t));
    add(&mut out, "lantern", lantern::build(0.0, 0.0, sk(SK::Lantern), alpha, facing, t));
    add(&mut out, "brazier", brazier::build(0.0, 0.0, sk(SK::Brazier), alpha, facing, t));
    add(&mut out, "crate_box", crate_box::build(0.0, 0.0, sk(SK::Crate), alpha, facing, t));
    add(&mut out, "pillar", pillar::build(0.0, 0.0, sk(SK::Pillar), alpha, facing, t));
    add(&mut out, "bone_pile", bone_pile::build(0.0, 0.0, sk(SK::BonePile), alpha, facing, t));
    add(&mut out, "cactus", cactus::build(0.0, 0.0, sk(SK::Cactus), alpha, facing, t));
    add(&mut out, "vines", vines::build(0.0, 0.0, sk(SK::Vines), alpha, facing, t));
    add(&mut out, "lilypad", lilypad::build(0.0, 0.0, sk(SK::Lilypad), alpha, facing, t));
    add(&mut out, "reed", reed::build(0.0, 0.0, sk(SK::Reed), alpha, facing, t));
    add(&mut out, "rubble", rubble::build(0.0, 0.0, sk(SK::Rubble), alpha, facing, t));
    add(&mut out, "ruin_tower", ruin_tower::build(0.0, 0.0, sk(SK::RuinTower), alpha, facing, t));
    add(&mut out, "house", house::build(0, 0.0, 0.0, sk(SK::House), alpha, facing, t));
    add(&mut out, "cabin", house::build(1, 0.0, 0.0, sk(SK::Cabin), alpha, facing, t));
    add(&mut out, "hut", house::build(2, 0.0, 0.0, sk(SK::Hut), alpha, facing, t));
    add(&mut out, "inn", house::build(3, 0.0, 0.0, sk(SK::Inn), alpha, facing, t));
    add(&mut out, "barn", house::build(4, 0.0, 0.0, sk(SK::Barn), alpha, facing, t));
    add(&mut out, "watchtower", house::build(5, 0.0, 0.0, sk(SK::Watchtower), alpha, facing, t));
    add(&mut out, "skeleton", humanoid::build_variant(0.0, 0.0, ek(EK::Skeleton), alpha, facing, 0.0, t, 0.0, humanoid::Variant::Skeleton));
    add(&mut out, "goblin", humanoid::build_variant(0.0, 0.0, ek(EK::Goblin), alpha, facing, 0.0, t, 0.0, humanoid::Variant::Goblin));
    add(&mut out, "bat", bat::build(0.0, 0.0, ek(EK::Bat), alpha, facing, 0.0, t));
    add(&mut out, "spider", spider::build(0.0, 0.0, ek(EK::Spider), alpha, facing, 0.0, t));
    add(&mut out, "imp", imp::build(0.0, 0.0, ek(EK::Imp), alpha, facing, 0.0, t));
    add(&mut out, "ogre", humanoid::build_variant(0.0, 0.0, ek(EK::Ogre), alpha, facing, 0.0, t, 0.0, humanoid::Variant::Ogre));
    add(&mut out, "wraith", wraith::build(0.0, 0.0, ek(EK::Wraith), alpha, facing, 0.0, t));
    add(&mut out, "stoneslinger", humanoid::build_variant(0.0, 0.0, ek(EK::Stoneslinger), alpha, facing, 0.0, t, 0.0, humanoid::Variant::Slinger));
    add(&mut out, "colossus", colossus::build(0.0, 0.0, ek(EK::Colossus), alpha, facing, 0.0, t));
    add(&mut out, "scorpion_queen", scorpion_queen::build(0.0, 0.0, ek(EK::ScorpionQueen), alpha, facing, 0.0, t));
    add(&mut out, "toad_king", toad_king::build(0.0, 0.0, ek(EK::ToadKing), alpha, facing, 0.0, t));
    add(&mut out, "brute", brute::build(0.0, 0.0, ek(EK::Brute), alpha, facing, 0.0, t));
    add(&mut out, "stormcaller", stormcaller::build(0.0, 0.0, ek(EK::Stormcaller), alpha, facing, 0.0, t));
    add(&mut out, "ocean_leviathan", ocean_leviathan::build(0.0, 0.0, ek(EK::OceanLeviathan), alpha, facing, 0.0, t));
    add(&mut out, "wolf", wolf::build(0.0, 0.0, ek(EK::Wolf), alpha, facing, 0.0, t));
    add(&mut out, "archer", archer::build(0.0, 0.0, ek(EK::Archer), alpha, facing, 0.0, t));
    add(&mut out, "raider", raider::build(0.0, 0.0, ek(EK::Raider), alpha, facing, 0.0, t));
    add(&mut out, "banner", banner::build(0.0, 0.0, sk(SK::Banner), alpha, facing, t));
    add(&mut out, "enchanting_table", enchanting_table::build(0.0, 0.0, sk(SK::EnchantingTable), alpha, facing, t));
    add(&mut out, "dungeon", dungeon::build(0.0, 0.0, sk(SK::Dungeon), alpha, facing, t));
    add(&mut out, "guard", guard::build(0.0, 0.0, crate::npc::NpcKind::Guard.color(), alpha, facing, 0.0, t, 0.0));
    add(&mut out, "golem", golem::build(0.0, 0.0, crate::npc::NpcKind::Golem.color(), alpha, facing, 0.0, t, 0.0));
    add(&mut out, "turret", turret::build(0.0, 0.0, sk(SK::Turret), alpha, facing, t));
    add(&mut out, "farm_plot", farm_plot::build(0.0, 0.0, sk(SK::FarmPlot), alpha, facing, t));
    add(&mut out, "healing_totem", healing_totem::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "portal", portal::build(0.0, 0.0, sk(SK::Portal), alpha, facing, t));
    add(&mut out, "spike", spike::build(0.0, 0.0, sk(SK::Spike), alpha, facing, t));
    add(&mut out, "car", car::build(0.0, 0.0, sk(SK::Car), alpha, facing, 16.0, 12.0));
    add(&mut out, "train", train::build(0.0, 0.0, sk(SK::Train), alpha, facing, 20.0, 16.0));
    add(&mut out, "rail", rail::build(0.0, 0.0, sk(SK::Rail), alpha, facing, 16.0, 6.0));
    out
}

#[cfg(test)]
mod tests {
    use super::prim::{rasterize, Part};

    fn verts(parts: Vec<Part>) -> Vec<f32> {
        let mut v = Vec::new();
        rasterize(&parts, &mut v);
        v
    }

    /// Every creature rig must respond to the walk cycle: stride (walk=1)
    /// poses must differ from rest (walk=0) at the same timestamp, or the
    /// entity glides instead of walking.
    #[test]
    fn every_creature_animates_its_stride() {
        let color = [0.72, 0.74, 0.80];
        let facing = (1.0, 0.0);
        let t = 0.7;
        let pairs: Vec<(&str, Vec<f32>, Vec<f32>)> = vec![
            ("slime", verts(super::slime::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::slime::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("bat", verts(super::bat::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::bat::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("spider", verts(super::spider::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::spider::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("imp", verts(super::imp::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::imp::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("wraith", verts(super::wraith::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::wraith::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("wolf", verts(super::wolf::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::wolf::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("archer", verts(super::archer::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::archer::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("raider", verts(super::raider::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::raider::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("brute", verts(super::brute::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::brute::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("stormcaller", verts(super::stormcaller::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::stormcaller::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("scorpion_queen", verts(super::scorpion_queen::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::scorpion_queen::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("toad_king", verts(super::toad_king::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::toad_king::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("ocean_leviathan", verts(super::ocean_leviathan::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::ocean_leviathan::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("colossus", verts(super::colossus::build(0.0, 0.0, color, 1.0, facing, 0.0, t)), verts(super::colossus::build(0.0, 0.0, color, 1.0, facing, 1.0, t))),
            ("golem", verts(super::golem::build(0.0, 0.0, color, 1.0, facing, 0.0, t, 0.0)), verts(super::golem::build(0.0, 0.0, color, 1.0, facing, 1.0, t, 0.0))),
            ("humanoid", verts(super::humanoid::build(0.0, 0.0, color, 1.0, facing, 0.0, t, 0.0)), verts(super::humanoid::build(0.0, 0.0, color, 1.0, facing, 1.0, t, 0.0))),
            ("guard", verts(super::guard::build(0.0, 0.0, color, 1.0, facing, 0.0, t, 0.0)), verts(super::guard::build(0.0, 0.0, color, 1.0, facing, 1.0, t, 0.0))),
            ("goblin", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, t, 0.0, super::humanoid::Variant::Goblin)), verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 1.0, t, 0.0, super::humanoid::Variant::Goblin))),
            ("skeleton", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, t, 0.0, super::humanoid::Variant::Skeleton)), verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 1.0, t, 0.0, super::humanoid::Variant::Skeleton))),
            ("ogre", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, t, 0.0, super::humanoid::Variant::Ogre)), verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 1.0, t, 0.0, super::humanoid::Variant::Ogre))),
            ("stoneslinger", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, t, 0.0, super::humanoid::Variant::Slinger)), verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 1.0, t, 0.0, super::humanoid::Variant::Slinger))),
        ];
        for (name, rest, stride) in pairs {
            // Flickering bits (toad tongue, storm wisps) may add/remove parts;
            // compare the shared prefix — the rig itself must still move.
            let n = rest.len().min(stride.len());
            assert!(n > 0, "{name} must emit geometry");
            let diffs = rest[..n].iter().zip(stride[..n].iter()).filter(|(a, b)| (*a - *b).abs() > 1e-4).count();
            assert!(diffs > 0, "{name} must change pose between rest and stride");
        }
    }

    /// Foe-identity guard: the humanoid variants must rasterize to distinct
    /// geometry (regression test for the ogre/stoneslinger/humanoid triple
    /// duplicate — byte-identical previews for distinct foes).
    #[test]
    fn humanoid_variants_are_distinct() {
        use super::humanoid::Variant;
        let color = [0.72, 0.74, 0.80];
        let facing = (1.0, 0.0);
        let vs = [
            ("civilian", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, 0.7, 0.0, Variant::Civilian))),
            ("goblin", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, 0.7, 0.0, Variant::Goblin))),
            ("skeleton", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, 0.7, 0.0, Variant::Skeleton))),
            ("ogre", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, 0.7, 0.0, Variant::Ogre))),
            ("slinger", verts(super::humanoid::build_variant(0.0, 0.0, color, 1.0, facing, 0.0, 0.7, 0.0, Variant::Slinger))),
        ];
        for i in 0..vs.len() {
            for j in (i + 1)..vs.len() {
                assert!(vs[i].1 != vs[j].1, "{} and {} rasterize identically", vs[i].0, vs[j].0);
            }
        }
    }

    /// Boss-mass guard: every boss must rasterize to at least the Brute's
    /// bbox area (regression test for the boss-hierarchy inversion where
    /// four of five bosses read as minions next to the common Brute).
    #[test]
    fn bosses_outmass_the_brute() {
        let color = [0.72, 0.74, 0.80];
        let facing = (1.0, 0.0);
        fn area(v: &[f32]) -> f32 {
            let (mut minx, mut maxx, mut miny, mut maxy) =
                (f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY);
            for c in v.chunks(6) {
                minx = minx.min(c[0]);
                maxx = maxx.max(c[0]);
                miny = miny.min(c[1]);
                maxy = maxy.max(c[1]);
            }
            (maxx - minx) * (maxy - miny)
        }
        let br = area(&verts(super::brute::build(0.0, 0.0, color, 1.0, facing, 0.0, 0.7)));
        assert!(br > 0.0, "brute must emit geometry");
        let bosses = [
            ("scorpion_queen", verts(super::scorpion_queen::build(0.0, 0.0, color, 1.0, facing, 0.0, 0.7))),
            ("toad_king", verts(super::toad_king::build(0.0, 0.0, color, 1.0, facing, 0.0, 0.7))),
            ("ocean_leviathan", verts(super::ocean_leviathan::build(0.0, 0.0, color, 1.0, facing, 0.0, 0.7))),
            ("stormcaller", verts(super::stormcaller::build(0.0, 0.0, color, 1.0, facing, 0.0, 0.7))),
            ("colossus", verts(super::colossus::build(0.0, 0.0, color, 1.0, facing, 0.0, 0.7))),
        ];
        for (name, v) in bosses {
            let a = area(&v);
            assert!(a >= br, "{name} bbox area {a:.0} must meet brute {br:.0}");
        }
    }

    fn bbox_center_x(v: &[f32]) -> f32 {
        let mut minx = f32::INFINITY;
        let mut maxx = f32::NEG_INFINITY;
        for c in v.chunks(6) {
            minx = minx.min(c[0]);
            maxx = maxx.max(c[0]);
        }
        (minx + maxx) / 2.0
    }

    /// Centering audit: symmetric elements built at cx=0 must be centered
    /// near x=0. Catches the classic `vquad(left-edge)` slip, where a part
    /// intended centered at C is drawn centered at C-hw (the house-roof bug:
    /// diamonds centered at cx while every wall sat half a width left).
    /// Tolerance absorbs gentle sway phases; facing is neutral (0,0).
    #[test]
    fn symmetric_elements_center_on_tile() {
        let color = [0.72, 0.74, 0.80];
        let facing = (0.0, 0.0);
        let t = 0.7;
        let items: Vec<(&str, Vec<f32>)> = vec![
            ("house", verts(super::house::build(0, 0.0, 0.0, color, 1.0, facing, t))),
            ("cabin", verts(super::house::build(1, 0.0, 0.0, color, 1.0, facing, t))),
            ("hut", verts(super::house::build(2, 0.0, 0.0, color, 1.0, facing, t))),
            ("barn", verts(super::house::build(4, 0.0, 0.0, color, 1.0, facing, t))),
            ("watchtower", verts(super::house::build(5, 0.0, 0.0, color, 1.0, facing, t))),
            ("pillar", verts(super::pillar::build(0.0, 0.0, color, 1.0, facing, t))),
            ("crate", verts(super::crate_box::build(0.0, 0.0, color, 1.0, facing, t))),
            ("chest", verts(super::chest::build(0.0, 0.0, color, 1.0, facing, t))),
            ("anvil", verts(super::anvil::build(0.0, 0.0, color, 1.0, facing, t))),
            ("well", verts(super::well::build(0.0, 0.0, color, 1.0, facing, t))),
            ("bed", verts(super::bed::build(0.0, 0.0, color, 1.0, facing, t))),
            ("statue", verts(super::statue::build(0.0, 0.0, color, 1.0, facing, t))),
            ("totem", verts(super::totem::build(0.0, 0.0, color, 1.0, facing, t))),
            ("ruin_tower", verts(super::ruin_tower::build(0.0, 0.0, color, 1.0, facing, t))),
            ("rock", verts(super::rock::build(0.0, 0.0, color, 1.0, facing, t))),
            ("slime", verts(super::slime::build(0.0, 0.0, color, 1.0, facing, 0.0, t))),
            ("barrel", verts(super::barrel::build(0.0, 0.0, color, 1.0, facing, t))),
            ("lantern", verts(super::lantern::build(0.0, 0.0, color, 1.0, facing, t))),
            ("brazier", verts(super::brazier::build(0.0, 0.0, color, 1.0, facing, t))),
            ("sign", verts(super::sign::build(0.0, 0.0, color, 1.0, facing, t))),
            ("altar", verts(super::altar::build(0.0, 0.0, color, 1.0, facing, t))),
            ("campfire", verts(super::campfire::build(0.0, 0.0, color, 1.0, facing, t))),
            ("spike", verts(super::spike::build(0.0, 0.0, color, 1.0, facing, t))),
            ("farm_plot", verts(super::farm_plot::build(0.0, 0.0, color, 1.0, facing, t))),
            ("turret", verts(super::turret::build(0.0, 0.0, color, 1.0, facing, t))),
            ("barrel", verts(super::barrel::build(0.0, 0.0, color, 1.0, facing, t))),
            ("rock_pile", verts(super::rock_pile::build(0.0, 0.0, color, 1.0, facing, t))),
            ("bone_pile", verts(super::bone_pile::build(0.0, 0.0, color, 1.0, facing, t))),
            ("rubble", verts(super::rubble::build(0.0, 0.0, color, 1.0, facing, t))),
            ("wall", verts(super::wall::build(0.0, 0.0, color, 1.0, facing, t))),
            ("torch", verts(super::torch::build(0.0, 0.0, color, 1.0, facing, t))),
            ("lantern", verts(super::lantern::build(0.0, 0.0, color, 1.0, facing, t))),
            ("brazier", verts(super::brazier::build(0.0, 0.0, color, 1.0, facing, t))),
            ("ore", verts(super::ore::build(0.0, 0.0, color, 1.0, facing, t))),
            ("tree", verts(super::tree::build(0.0, 0.0, color, 1.0, facing, t))),
            ("bush", verts(super::bush::build(0.0, 0.0, color, 1.0, facing, t))),
            ("mushroom", verts(super::mushroom::build(0.0, 0.0, color, 1.0, facing, t))),
            ("crystal", verts(super::crystal::build(0.0, 0.0, color, 1.0, facing, t))),
            ("flower", verts(super::flower::build(0.0, 0.0, color, 1.0, facing, t))),
            ("grass_tuft", verts(super::grass_tuft::build(0.0, 0.0, color, 1.0, facing, t))),
            ("fern", verts(super::fern::build(0.0, 0.0, color, 1.0, facing, t))),
            ("banner", verts(super::banner::build(0.0, 0.0, color, 1.0, facing, t))),
            ("enchanting_table", verts(super::enchanting_table::build(0.0, 0.0, color, 1.0, facing, t))),
            ("dungeon", verts(super::dungeon::build(0.0, 0.0, color, 1.0, facing, t))),
            ("altar", verts(super::altar::build(0.0, 0.0, color, 1.0, facing, t))),
        ];
        let mut bad = Vec::new();
        for (name, v) in items {
            let c = bbox_center_x(&v);
            if c.abs() > 6.0 {
                bad.push(format!("{name}:{c:.1}"));
            }
        }
        assert!(bad.is_empty(), "off-tile elements (vquad slip?): {}", bad.join(", "));
    }
}
