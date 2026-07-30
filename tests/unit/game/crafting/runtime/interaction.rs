use super::*;
use crate::game::inventory::container::InventoryContainer;
use crate::shared::item_id::ItemId;

#[test]
fn two_players_and_two_workbenches_are_fully_isolated() {
    let mut app = App::new();
    app.init_resource::<RecipeRegistry>()
        .init_resource::<ItemTagIndex>()
        .init_resource::<WorldContainers>()
        .add_message::<SlotInteractionEvent>()
        .add_systems(Update, crafting_interaction_system);

    let (first_container, second_container) = {
        let mut containers = app.world_mut().resource_mut::<WorldContainers>();
        (
            containers
                .ensure_at(IVec3::new(1, 2, 3), ContainerKind::Workbench)
                .unwrap(),
            containers
                .ensure_at(IVec3::new(9, 2, 3), ContainerKind::Workbench)
                .unwrap(),
        )
    };
    let first_player_id = PlayerId::new(1);
    let second_player_id = PlayerId::new(2);
    let first_item = ItemId::item("test:first_player_item");
    let second_item = ItemId::item("test:second_player_item");

    let mut first_inventory = InventoryState::default();
    first_inventory
        .cursor
        .set_stack(ItemStack::single(first_item.clone()));
    let mut second_inventory = InventoryState::default();
    second_inventory
        .cursor
        .set_stack(ItemStack::single(second_item.clone()));

    let first_player = app
        .world_mut()
        .spawn((
            first_player_id,
            first_inventory,
            PlayerCrafting::default(),
            ActiveCrafting::workbench(IVec3::new(1, 2, 3), first_container),
        ))
        .id();
    app.world_mut().spawn((
        second_player_id,
        second_inventory,
        PlayerCrafting::default(),
        ActiveCrafting::workbench(IVec3::new(9, 2, 3), second_container),
    ));

    app.world_mut().write_message(SlotInteractionEvent {
        player_id: first_player_id,
        container_id: Some(first_container),
        kind: SlotKind::Container(ContainerKind::Workbench),
        index: 0,
        action: SlotAction::LeftClick,
    });
    app.world_mut().write_message(SlotInteractionEvent {
        player_id: second_player_id,
        container_id: Some(second_container),
        kind: SlotKind::Container(ContainerKind::Workbench),
        index: 0,
        action: SlotAction::LeftClick,
    });
    app.update();

    let containers = app.world().resource::<WorldContainers>();
    assert_eq!(
        containers
            .workbench(first_container)
            .and_then(|workbench| workbench.get_stack(0))
            .map(|stack| &stack.item),
        Some(&first_item)
    );
    assert_eq!(
        containers
            .workbench(second_container)
            .and_then(|workbench| workbench.get_stack(0))
            .map(|stack| &stack.item),
        Some(&second_item)
    );
    assert_ne!(first_container, second_container);

    let mut inventories = app.world_mut().query::<(&PlayerId, &InventoryState)>();
    for (player_id, inventory) in inventories.iter(app.world()) {
        assert!(
            !inventory.cursor.has_item(),
            "cursor was not consumed for {player_id:?}"
        );
    }

    let cross_item = ItemId::item("test:cross_container_attempt");
    app.world_mut()
        .get_mut::<InventoryState>(first_player)
        .unwrap()
        .cursor
        .set_stack(ItemStack::single(cross_item.clone()));
    app.world_mut().write_message(SlotInteractionEvent {
        player_id: first_player_id,
        container_id: Some(second_container),
        kind: SlotKind::Container(ContainerKind::Workbench),
        index: 1,
        action: SlotAction::LeftClick,
    });
    app.update();

    assert!(
        app.world()
            .resource::<WorldContainers>()
            .workbench(second_container)
            .unwrap()
            .get_stack(1)
            .is_none()
    );
    assert_eq!(
        app.world()
            .get::<InventoryState>(first_player)
            .unwrap()
            .cursor
            .stack()
            .map(|stack| &stack.item),
        Some(&cross_item)
    );
}
