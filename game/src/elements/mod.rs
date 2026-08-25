//! Per-element artwork, isolated so each entity (tree, rock, person, …) lives
//! in its own file and can be replaced without touching the renderer.
//!
//! Every element returns a list of [`Part`]s. `prim::rasterize` is the single
//! place that turns parts into vertices — including the fake-2.5D dark "skirt".
//! When we later add a texture atlas (option B), only `prim` changes: a `Part`
//! already carries an optional `uv` rect, so element files don't need edits.

pub(crate) mod prim;

pub(crate) mod tree;
pub(crate) mod rock;
pub(crate) mod bush;
pub(crate) mod wall;
pub(crate) mod chest;
pub(crate) mod campfire;
pub(crate) mod altar;
pub(crate) mod arrow;
pub(crate) mod slime;
pub(crate) mod humanoid;
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

// Enemies
pub(crate) mod skeleton;
pub(crate) mod goblin;
pub(crate) mod bat;
pub(crate) mod spider;
pub(crate) mod imp;
pub(crate) mod ogre;
pub(crate) mod wraith;
pub(crate) mod stoneslinger;
pub(crate) mod colossus;
pub(crate) mod brute;
pub(crate) mod stormcaller;

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
    add(&mut out, "slime", slime::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "humanoid", humanoid::build(0.0, 0.0, color, alpha, facing, 0.0, t));
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
    add(&mut out, "skeleton", skeleton::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "goblin", goblin::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "bat", bat::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "spider", spider::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "imp", imp::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "ogre", ogre::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "wraith", wraith::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "stoneslinger", stoneslinger::build(0.0, 0.0, color, alpha, facing, t));
    add(&mut out, "colossus", colossus::build(0.0, 0.0, color, alpha, facing, t));
    out
}
