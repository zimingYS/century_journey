use super::*;

fn item(path: &str, count: u32) -> ItemStack {
    ItemStack::new(
        crate::shared::item_id::ItemId::item(format!("century_journey:{path}")),
        count,
    )
}

#[test]
fn closing_inventory_returns_cursor_stack_to_its_source() {
    let mut inventory = InventoryState::default();
    inventory
        .cursor
        .set_stack_with_source(item("stone", 12), CursorSource::Hotbar(3));

    let overflow = return_cursor_item(&mut inventory);

    assert!(overflow.is_none());
    assert!(!inventory.cursor.has_item());
    assert_eq!(inventory.hotbar.get_stack(3), Some(&item("stone", 12)));
}

#[test]
fn closing_inventory_preserves_overflow_when_every_storage_slot_is_full() {
    let mut inventory = InventoryState::default();
    for index in 0..HOTBAR_SIZE {
        inventory
            .hotbar
            .set_stack(index, item(&format!("hotbar_{index}"), 64));
    }
    for index in 0..SurvivalInventory::BACKPACK_SIZE {
        inventory
            .survival
            .set_stack(index, item(&format!("backpack_{index}"), 64));
    }
    inventory
        .cursor
        .set_stack_with_source(item("overflow", 7), CursorSource::Container(0));

    let overflow = return_cursor_item(&mut inventory);

    assert_eq!(overflow, Some(item("overflow", 7)));
    assert!(!inventory.cursor.has_item());
}

#[test]
fn closing_inventory_merges_then_uses_an_empty_slot_without_losing_count() {
    let mut inventory = InventoryState::default();
    inventory.hotbar.set_stack(2, item("stone", 60));
    inventory
        .cursor
        .set_stack_with_source(item("stone", 10), CursorSource::Hotbar(2));

    let overflow = return_cursor_item(&mut inventory);

    assert!(overflow.is_none());
    assert_eq!(inventory.hotbar.get_stack(2), Some(&item("stone", 64)));
    assert_eq!(inventory.hotbar.get_stack(0), Some(&item("stone", 6)));
}
