use super::*;

#[test]
fn stage_seven_reload_keeps_inventory_durability_stats_and_respawn_point() {
    let mut inventory = InventoryState::default();
    let mut tool = ItemStack::single(ItemId::item("century_journey:test_tool"));
    tool.instance.durability = Some(23);
    inventory.hotbar.set_stack(2, tool);
    inventory
        .survival
        .set_stack(4, ItemStack::new(ItemId::block("century_journey:dirt"), 17));
    inventory.hotbar.active_index = 2;
    let item_registry = ItemRegistry::default();

    let data = PlayerSaveData::from_runtime(
        Vec3::new(3.0, 72.0, -5.0),
        Quat::from_rotation_y(0.5),
        -0.25,
        &PlayerGameMode {
            mode: GameMode::Survival,
        },
        &inventory,
        &item_registry,
        13.5,
        7.25,
        3.0,
        Vec3::new(8.0, 71.0, 4.0),
    );
    let restored = data.restore_inventory();

    assert_eq!(data.game_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(data.health, 13.5);
    assert_eq!(data.hunger, 7.25);
    assert_eq!(data.saturation, 3.0);
    assert_eq!(data.respawn_point(), Vec3::new(8.0, 71.0, 4.0));
    assert_eq!(data.position, [3.0, 72.0, -5.0]);
    assert_eq!(data.camera_pitch, -0.25);
    assert_eq!(restored.hotbar.active_index, 2);
    assert_eq!(
        restored.hotbar.get_stack(2).map(|stack| stack.count),
        Some(1)
    );
    assert_eq!(
        restored.hotbar.get_stack(2).and_then(ItemStack::durability),
        Some(23)
    );
    assert_eq!(
        restored.survival.get_stack(4).map(|stack| stack.count),
        Some(17)
    );
}

#[test]
fn item_runtime_ids_are_remapped_by_unique_identifier() {
    let wood = crate::shared::identifier::Identifier::new("century_journey", "wood");
    let stone = crate::shared::identifier::Identifier::new("century_journey", "stone");
    let mut saved_registry = ItemRegistry::default();
    saved_registry
        .register(crate::content::item::definition::ItemDefinition::from_block(&wood, "Wood"));
    saved_registry
        .register(crate::content::item::definition::ItemDefinition::from_block(&stone, "Stone"));

    let mut inventory = InventoryState::default();
    inventory
        .hotbar
        .set_stack(0, ItemStack::new(ItemId::new(stone.clone()), 5));
    let data = PlayerSaveData::from_runtime(
        Vec3::ZERO,
        Quat::IDENTITY,
        0.0,
        &PlayerGameMode::default(),
        &inventory,
        &saved_registry,
        20.0,
        20.0,
        5.0,
        Vec3::ZERO,
    );
    assert_eq!(data.hotbar[0].runtime_id, Some(1));

    let mut current_registry = ItemRegistry::default();
    current_registry
        .register(crate::content::item::definition::ItemDefinition::from_block(&stone, "Stone"));
    current_registry
        .register(crate::content::item::definition::ItemDefinition::from_block(&wood, "Wood"));
    let restored = data.restore_inventory_with_registry(&current_registry);

    let stack = restored.hotbar.get_stack(0).unwrap();
    assert_eq!(stack.item, ItemId::new(stone));
    assert_eq!(stack.count, 5);
    assert_eq!(current_registry.runtime_id(&stack.item), Some(0));
}
