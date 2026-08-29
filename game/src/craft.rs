use crate::items::{Inventory, ItemKind};

/// A crafting recipe: a fixed set of input item counts consumed to produce one
/// output item. Recipes are pure data so they're trivially unit-testable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub output: ItemKind,
    pub inputs: &'static [(ItemKind, u32)],
}

/// Known recipes. The Anvil-only Iron Plate is the campaign crafting milestone.
pub const RECIPES: &[Recipe] = &[Recipe {
    output: ItemKind::IronPlate,
    inputs: &[(ItemKind::Iron, 2), (ItemKind::Stone, 4)],
}];

/// Look up the recipe that produces `output`, if any.
pub fn recipe_for(output: ItemKind) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.output == output)
}

/// Whether `inv` can currently pay for `r` (every input satisfied).
pub fn can_craft(inv: &Inventory, r: &Recipe) -> bool {
    r.inputs.iter().all(|(k, n)| inv.count(*k) >= *n)
}

/// Consume the inputs and add one output. Returns false (and changes nothing) if
/// the inventory can't pay, so a failed craft never eats resources.
pub fn craft(inv: &mut Inventory, r: &Recipe) -> bool {
    if !can_craft(inv, r) {
        return false;
    }
    for (k, n) in r.inputs {
        inv.remove(*k, *n);
    }
    inv.add(r.output, 1);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iron_plate_recipe_exists() {
        let r = recipe_for(ItemKind::IronPlate).expect("recipe present");
        assert_eq!(r.output, ItemKind::IronPlate);
        assert_eq!(r.inputs, &[(ItemKind::Iron, 2), (ItemKind::Stone, 4)]);
    }

    #[test]
    fn craft_consumes_inputs_and_yields_output() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Iron, 2);
        inv.add(ItemKind::Stone, 4);
        let r = recipe_for(ItemKind::IronPlate).unwrap();
        assert!(craft(&mut inv, r));
        assert_eq!(inv.count(ItemKind::IronPlate), 1);
        assert_eq!(inv.count(ItemKind::Iron), 0);
        assert_eq!(inv.count(ItemKind::Stone), 0);
    }

    #[test]
    fn craft_fails_without_materials() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Iron, 1); // too few iron, no stone
        let r = recipe_for(ItemKind::IronPlate).unwrap();
        assert!(!can_craft(&inv, r));
        assert!(!craft(&mut inv, r), "must not craft when short");
        assert_eq!(inv.count(ItemKind::Iron), 1, "inputs untouched on failure");
    }

    #[test]
    fn unknown_recipe_is_none() {
        assert!(recipe_for(ItemKind::Wood).is_none());
    }
}
