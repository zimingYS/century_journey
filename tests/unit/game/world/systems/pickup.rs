use super::*;
use crate::game::inventory::events::InventoryFeedbackEvent;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::player::components::PlayerLifecycle;
use crate::shared::item_id::ItemId;

#[test]
fn stage_seven_pickup_moves_drop_into_empty_inventory() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<InventoryFeedbackEvent>()
        .add_systems(Update, pickup_system);
    app.world_mut().spawn((
        Player,
        PlayerLifecycle::default(),
        InventoryState::default(),
        Transform::from_xyz(0.0, 70.0, 0.0),
    ));
    let mut dropped = DroppedItem::new(ItemStack::new(ItemId::item("century_journey:stick"), 3));
    dropped.pickup_delay = 0.0;
    let drop_entity = app
        .world_mut()
        .spawn((dropped, Transform::from_xyz(0.5, 70.0, 0.0)))
        .id();

    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&InventoryState>()
            .single(app.world())
            .unwrap()
            .hotbar
            .get_stack(0)
            .map(|stack| stack.count),
        Some(3)
    );
    assert!(app.world().get_entity(drop_entity).is_err());
}
