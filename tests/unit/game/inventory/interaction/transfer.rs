use super::*;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::shared::item_id::ItemId;

fn item(path: &str, count: u32) -> ItemStack {
    ItemStack::new(ItemId::item(format!("century_journey:{path}")), count)
}

fn fill_backpack(inventory: &mut SurvivalInventory) {
    for index in 0..SurvivalInventory::BACKPACK_SIZE {
        inventory.set_stack(
            index,
            item(&format!("full_{index}"), ItemStack::MAX_STACK_SIZE),
        );
    }
}

#[test]
fn range_insert_never_uses_equipment_slots() {
    let mut inventory = SurvivalInventory::default();
    fill_backpack(&mut inventory);

    let incoming = item("overflow", 1);
    let result = insert_into_range(
        &mut inventory,
        incoming,
        0..SurvivalInventory::BACKPACK_SIZE,
    );

    assert!(matches!(result, InventoryInsertResult::Full(_)));
    assert!(
        inventory
            .get_stack(SurvivalInventory::equipment_index(0))
            .is_none()
    );
}

#[test]
fn range_insert_merges_into_existing_stack_before_using_empty_slot() {
    let mut inventory = SurvivalInventory::default();
    inventory.set_stack(0, item("stone", 60));

    let result = insert_into_range(&mut inventory, item("stone", 10), 0..2);

    assert!(matches!(result, InventoryInsertResult::AllInserted));
    assert_eq!(inventory.get_stack(0), Some(&item("stone", 64)));
    assert_eq!(inventory.get_stack(1), Some(&item("stone", 6)));
}

#[test]
fn range_insert_reports_partial_when_only_existing_stack_has_space() {
    let mut inventory = SurvivalInventory::default();
    fill_backpack(&mut inventory);
    inventory.set_stack(0, item("stone", 60));

    let result = insert_into_range(
        &mut inventory,
        item("stone", 10),
        0..SurvivalInventory::BACKPACK_SIZE,
    );

    let InventoryInsertResult::Partial(remaining) = result else {
        panic!("expected a partial insertion");
    };
    assert_eq!(remaining, item("stone", 6));
    assert_eq!(inventory.get_stack(0), Some(&item("stone", 64)));
}
