use super::*;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::shared::item_id::ItemId;

#[test]
fn range_insert_never_uses_equipment_slots() {
    let mut inventory = SurvivalInventory::default();
    for index in 0..SurvivalInventory::BACKPACK_SIZE {
        inventory.set_stack(
            index,
            ItemStack::new(
                ItemId::item(format!("century_journey:full_{index}")),
                ItemStack::MAX_STACK_SIZE,
            ),
        );
    }

    let incoming = ItemStack::single(ItemId::item("century_journey:overflow"));
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
