use crate::items::{Inventory, ItemKind};

/// Sell price: how much Gold the merchant pays for one unit of `item`.
pub fn sell_price(item: ItemKind) -> u32 {
    match item {
        ItemKind::Food => 1,
        ItemKind::Herb => 2,
        ItemKind::Wood => 1,
        ItemKind::Stone => 1,
        ItemKind::Iron => 3,
        ItemKind::Gem => 5,
        ItemKind::IronPlate => 8,
        ItemKind::Gold => 0, // can't sell gold for gold
        ItemKind::Fragment | ItemKind::Map => 0, // quest items can't be sold
    }
}

/// Buy price: how much Gold the merchant charges for one unit of `item`.
pub fn buy_price(item: ItemKind) -> u32 {
    match item {
        ItemKind::Food => 2,
        ItemKind::Herb => 4,
        ItemKind::Wood => 2,
        ItemKind::Stone => 2,
        ItemKind::Iron => 6,
        ItemKind::Gem => 10,
        ItemKind::IronPlate => 15,
        ItemKind::Gold => 0,
        ItemKind::Fragment | ItemKind::Map => 0,
    }
}

/// Whether the merchant will buy this item (non-zero sell price).
pub fn can_sell(item: ItemKind) -> bool {
    sell_price(item) > 0
}

/// Whether the merchant will sell this item (non-zero buy price).
pub fn can_buy(item: ItemKind) -> bool {
    buy_price(item) > 0
}

/// Sell one unit of `item` from `inv`, gaining Gold. Returns the Gold earned
/// (0 if the item can't be sold or the inventory is empty).
pub fn sell(inv: &mut Inventory, item: ItemKind) -> u32 {
    let price = sell_price(item);
    if price == 0 || inv.count(item) == 0 {
        return 0;
    }
    inv.remove(item, 1);
    inv.add(ItemKind::Gold, price);
    price
}

/// Buy one unit of `item` into `inv`, spending Gold. Returns true on success
/// (and deducts Gold + adds item), false if the player can't afford it.
pub fn buy(inv: &mut Inventory, item: ItemKind) -> bool {
    let price = buy_price(item);
    if price == 0 || inv.count(ItemKind::Gold) < price {
        return false;
    }
    inv.remove(ItemKind::Gold, price);
    inv.add(item, 1);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sell_gives_gold_and_removes_item() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Food, 3);
        let earned = sell(&mut inv, ItemKind::Food);
        assert_eq!(earned, 1, "Food sells for 1 gold");
        assert_eq!(inv.count(ItemKind::Food), 2);
        assert_eq!(inv.count(ItemKind::Gold), 1);
    }

    #[test]
    fn sell_returns_zero_when_empty() {
        let mut inv = Inventory::new();
        assert_eq!(sell(&mut inv, ItemKind::Food), 0);
    }

    #[test]
    fn buy_deducts_gold_and_adds_item() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Gold, 10);
        assert!(buy(&mut inv, ItemKind::Iron));
        assert_eq!(inv.count(ItemKind::Iron), 1);
        assert_eq!(inv.count(ItemKind::Gold), 4); // 10 - 6
    }

    #[test]
    fn buy_fails_without_gold() {
        let mut inv = Inventory::new();
        assert!(!buy(&mut inv, ItemKind::Gem));
        assert_eq!(inv.count(ItemKind::Gem), 0);
    }

    #[test]
    fn cannot_sell_quest_items() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Fragment, 3);
        assert_eq!(sell(&mut inv, ItemKind::Fragment), 0);
        assert_eq!(inv.count(ItemKind::Fragment), 3);
    }

    #[test]
    fn prices_are_consistent() {
        // Every buyable item has a sell price (the merchant takes a cut).
        for item in [
            ItemKind::Food, ItemKind::Herb, ItemKind::Wood, ItemKind::Stone,
            ItemKind::Iron, ItemKind::Gem, ItemKind::IronPlate,
        ] {
            assert!(sell_price(item) > 0, "{item:?} should be sellable");
            assert!(buy_price(item) > sell_price(item), "{item:?} should cost more to buy than sell");
        }
    }
}
