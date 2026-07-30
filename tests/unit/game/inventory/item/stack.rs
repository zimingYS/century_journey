use super::*;

#[test]
fn tool_instance_loses_durability_and_breaks() {
    let mut tool = ItemStack::single(ItemId::item("century_journey:test_pickaxe"));

    assert_eq!(
        tool.damage_tool(1, 3),
        ToolDamageResult::Damaged { remaining: 2 }
    );
    assert_eq!(tool.durability(), Some(2));
    assert_eq!(
        tool.damage_tool(1, 3),
        ToolDamageResult::Damaged { remaining: 1 }
    );
    assert_eq!(tool.damage_tool(1, 3), ToolDamageResult::Broken);
    assert!(tool.is_empty());
}

#[test]
fn different_instance_data_never_merges() {
    let mut used = ItemStack::single(ItemId::item("century_journey:test_pickaxe"));
    used.instance.durability = Some(4);
    let unused = ItemStack::single(ItemId::item("century_journey:test_pickaxe"));

    assert!(!used.is_same_item(&unused));
    assert!(!used.can_merge(&unused));
}
