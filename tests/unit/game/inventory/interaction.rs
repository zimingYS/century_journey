use super::*;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::item::stack::ItemInstanceData;
use crate::shared::item_id::ItemId;

#[test]
fn wheel_transfer_moves_one_and_preserves_instance_data() {
    let item = ItemId::item("century_journey:test_tool");
    let mut source = SurvivalInventory::default();
    let mut dest = SurvivalInventory::default();
    source.set_stack(
        0,
        ItemStack::with_instance(
            item.clone(),
            2,
            ItemInstanceData {
                durability: Some(7),
            },
        ),
    );

    assert!(move_one_into_range(&mut source, &mut dest, 0, 0..1));
    assert_eq!(source.get_stack(0).unwrap().count, 1);
    assert_eq!(dest.get_stack(0).unwrap().count, 1);
    assert_eq!(dest.get_stack(0).unwrap().durability(), Some(7));
}

#[test]
fn wheel_pull_only_uses_matching_stacks() {
    let item = ItemId::item("century_journey:planks");
    let mut dest = SurvivalInventory::default();
    let mut source = SurvivalInventory::default();
    dest.set_stack(0, ItemStack::single(item.clone()));
    source.set_stack(0, ItemStack::new(item, 3));

    assert!(pull_one_matching(&mut dest, &mut source, 0, 0..1));
    assert_eq!(dest.get_stack(0).unwrap().count, 2);
    assert_eq!(source.get_stack(0).unwrap().count, 2);
}
