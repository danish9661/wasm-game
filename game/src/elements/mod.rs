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

/// Offline tooling hook: build every element into a flat vertex buffer so a
/// `bin` in this package can rasterize and save PNGs. Each vertex is
/// `x, y, r, g, b, a` (colors in `[0,1]`); every 3 consecutive vertices form
/// one triangle. Coordinates are centered on `(0,0)` — the caller crops to the
/// bounding box. Public so the `gen_pngs` binary (a separate crate) can call it.
pub fn preview_elements() -> Vec<(String, Vec<f32>)> {
    use prim::{rasterize, Part};
    let color = [0.72, 0.74, 0.80];
    let alpha = 1.0;
    let facing = (1.0, 0.0);
    let t = 0.0;
    let mut out: Vec<(String, Vec<f32>)> = Vec::new();
    let add = |out: &mut Vec<(String, Vec<f32>)>, name: &str, parts: Vec<Part>| {
        let mut verts: Vec<f32> = Vec::new();
        rasterize(&parts, &mut verts);
        out.push((name.to_string(), verts));
    };
    add(&mut out, "tree", tree::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "rock", rock::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "bush", bush::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "wall", wall::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "chest", chest::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "campfire", campfire::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "altar", altar::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "slime", slime::build(0.0, 0.0, color, alpha, facing, 0.0, t));
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
    ] {
        add(
            &mut out,
            &format!("weapon_{}", k.name().to_lowercase()),
            weapon::build(k, 0.0, 0.0, facing, 0.5, 0, true, alpha),
        );
    }
    add(&mut out, "block_shield", weapon::block_shield(0.0, 0.0, facing, alpha));
    add(&mut out, "bow_loosed", weapon::build(crate::weapons::WeaponKind::Bow, 0.0, 0.0, facing, 0.6, 0, false, alpha));
    add(&mut out, "mushroom", mushroom::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "crystal", crystal::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "flower", flower::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "grass_tuft", grass_tuft::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "fern", fern::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "ore", ore::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "fence", fence::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "torch", torch::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "anvil", anvil::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "bed", bed::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "well", well::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "sign", sign::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "barrel", barrel::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "totem", totem::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "rock_pile", rock_pile::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "statue", statue::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "lantern", lantern::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "brazier", brazier::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "crate_box", crate_box::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "pillar", pillar::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "bone_pile", bone_pile::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "cactus", cactus::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "vines", vines::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "lilypad", lilypad::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "reed", reed::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "rubble", rubble::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "ruin_tower", ruin_tower::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "house", house::build(0, 0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "cabin", house::build(1, 0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "hut", house::build(2, 0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "inn", house::build(3, 0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "barn", house::build(4, 0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "watchtower", house::build(5, 0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "skeleton", humanoid::build(0.0, 0.0, color, alpha, facing, 0.0, t, 0.0));
    add(&mut out, "goblin", humanoid::build(0.0, 0.0, color, alpha, facing, 0.0, t, 0.0));
    add(&mut out, "bat", bat::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "spider", spider::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "imp", imp::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "ogre", humanoid::build(0.0, 0.0, color, alpha, facing, 0.0, t, 0.0));
    add(&mut out, "wraith", wraith::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "stoneslinger", humanoid::build(0.0, 0.0, color, alpha, facing, 0.0, t, 0.0));
    add(&mut out, "colossus", colossus::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "scorpion_queen", scorpion_queen::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "toad_king", toad_king::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "brute", brute::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "stormcaller", stormcaller::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "ocean_leviathan", ocean_leviathan::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "wolf", wolf::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "archer", archer::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "raider", raider::build(0.0, 0.0, color, alpha, facing, 0.0, t));
    add(&mut out, "banner", banner::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "enchanting_table", enchanting_table::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "dungeon", dungeon::build(0.0, 0.0, color, alpha, facing, t));
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
