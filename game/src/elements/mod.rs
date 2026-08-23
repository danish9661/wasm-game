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
