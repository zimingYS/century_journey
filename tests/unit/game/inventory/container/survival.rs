use super::*;
use crate::shared::item_id::ItemId;

#[test]
fn equipment_and_dynamic_accessories_keep_stable_indices() {
    let mut inventory = SurvivalInventory::default();
    inventory.ensure_accessory_slots(8);

    let equipment = ItemStack::single(ItemId::item("century_journey:test_helmet"));
    let accessory = ItemStack::single(ItemId::item("century_journey:test_ring"));
    inventory.set_stack(SurvivalInventory::equipment_index(0), equipment.clone());
    inventory.set_stack(SurvivalInventory::accessory_index(7), accessory.clone());

    assert_eq!(inventory.slot_count(), 27 + 7 + 8);
    assert_eq!(
        inventory.get_stack(SurvivalInventory::equipment_index(0)),
        Some(&equipment)
    );
    assert_eq!(
        inventory.get_stack(SurvivalInventory::accessory_index(7)),
        Some(&accessory)
    );
}
