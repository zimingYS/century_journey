use super::*;
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::item_id::ItemId;

#[test]
fn entering_world_replaces_transient_and_persisted_inventory_state() {
    let mut inventory = InventoryState::default();
    let stale_stack = ItemStack::new(ItemId::item("century_journey:stale_item"), 3);
    inventory.hotbar.set_stack(0, stale_stack.clone());
    inventory.cursor.set_stack(stale_stack.clone());
    inventory.add_recent_stack(stale_stack);
    inventory.creative.search_text = "old world".into();
    inventory.opened = true;

    replace_inventory_for_session(
        &mut inventory,
        &PlayerSaveData::default(),
        &ItemRegistry::default(),
    );

    assert!(inventory.hotbar.get_stack(0).is_none());
    assert!(!inventory.cursor.has_item());
    assert!(inventory.recent.items.is_empty());
    assert!(inventory.creative.search_text.is_empty());
    assert!(!inventory.opened);
}
