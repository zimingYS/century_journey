use super::*;
use crate::content::item::definition::{FoodData, ItemCategory, ItemDefinition};
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::held_item::{AnimationConfig, HeldRenderDefinition};
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

#[derive(Resource, Default)]
struct FoodEventCount(usize);

fn count_food_events(
    mut reader: MessageReader<FoodConsumedEvent>,
    mut count: ResMut<FoodEventCount>,
) {
    count.0 += reader.read().count();
}

#[test]
fn food_is_consumed_only_after_the_use_animation_duration() {
    let apple = ItemId::item("century_journey:apple");
    let mut registry = ItemRegistry::default();
    registry.register(ItemDefinition {
        identifier: Identifier::parse("century_journey:apple").unwrap(),
        display_name: "野苹果".into(),
        category: ItemCategory::Consumable,
        max_stack: 64,
        tags: vec!["food".into()],
        icon: default(),
        model: None,
        placeable_block: None,
        tool: None,
        food: Some(FoodData {
            hunger: 4.0,
            saturation: 2.4,
        }),
        held_renderer: HeldRenderDefinition::default(),
        animations: AnimationConfig::default(),
    });

    let mut inventory = InventoryState::default();
    inventory.hotbar.set_stack(0, ItemStack::new(apple, 2));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)))
        .insert_resource(registry)
        .init_resource::<PlayerActionState>()
        .init_resource::<PlayerGameMode>()
        .init_resource::<FoodEventCount>()
        .add_message::<FoodConsumedEvent>()
        .add_systems(Update, (use_food_system, count_food_events).chain());
    app.world_mut()
        .resource_mut::<Time<bevy::time::Virtual>>()
        .set_max_delta(Duration::from_secs(2));
    let player = app
        .world_mut()
        .spawn((
            Player,
            Hunger {
                current: 10.0,
                max: 20.0,
                saturation: 0.0,
            },
            PlayerLifecycle::default(),
            FoodUseState::default(),
            inventory,
        ))
        .id();
    app.world_mut()
        .resource_mut::<PlayerActionState>()
        .update(true, [PlayerAction::Use]);

    app.update();
    app.update();

    let hunger = app.world().get::<Hunger>(player).unwrap();
    assert_eq!(hunger.current, 10.0);
    assert!(app.world().get::<FoodUseState>(player).unwrap().is_active());
    assert_eq!(app.world().resource::<FoodEventCount>().0, 0);
    assert_eq!(
        app.world()
            .get::<InventoryState>(player)
            .unwrap()
            .hotbar
            .get_stack(0)
            .map(|stack| stack.count),
        Some(2)
    );

    app.update();

    let hunger = app.world().get::<Hunger>(player).unwrap();
    assert_eq!(hunger.current, 14.0);
    assert_eq!(hunger.saturation, 2.4);
    assert_eq!(app.world().resource::<FoodEventCount>().0, 1);
    assert_eq!(
        app.world()
            .get::<InventoryState>(player)
            .unwrap()
            .hotbar
            .get_stack(0)
            .map(|stack| stack.count),
        Some(1)
    );
}

#[test]
fn releasing_use_cancels_food_without_consuming_it() {
    let apple = ItemId::item("century_journey:apple");
    let mut registry = ItemRegistry::default();
    registry.register(ItemDefinition {
        identifier: Identifier::parse("century_journey:apple").unwrap(),
        display_name: "Apple".into(),
        category: ItemCategory::Consumable,
        max_stack: 64,
        tags: vec!["food".into()],
        icon: default(),
        model: None,
        placeable_block: None,
        tool: None,
        food: Some(FoodData {
            hunger: 4.0,
            saturation: 2.4,
        }),
        held_renderer: HeldRenderDefinition::default(),
        animations: AnimationConfig::default(),
    });
    let mut inventory = InventoryState::default();
    inventory
        .hotbar
        .set_stack(0, ItemStack::new(apple.clone(), 2));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)))
        .insert_resource(registry)
        .init_resource::<PlayerActionState>()
        .init_resource::<PlayerGameMode>()
        .add_message::<FoodConsumedEvent>()
        .add_systems(Update, use_food_system);
    app.world_mut()
        .resource_mut::<Time<bevy::time::Virtual>>()
        .set_max_delta(Duration::from_secs(2));
    let player = app
        .world_mut()
        .spawn((
            Player,
            Hunger {
                current: 10.0,
                max: 20.0,
                saturation: 0.0,
            },
            PlayerLifecycle::default(),
            FoodUseState::default(),
            inventory,
        ))
        .id();

    app.world_mut()
        .resource_mut::<PlayerActionState>()
        .update(true, [PlayerAction::Use]);
    app.update();
    app.world_mut()
        .resource_mut::<PlayerActionState>()
        .update(true, []);
    app.update();

    assert_eq!(app.world().get::<Hunger>(player).unwrap().current, 10.0);
    assert!(!app.world().get::<FoodUseState>(player).unwrap().is_active());
    assert_eq!(
        app.world()
            .get::<InventoryState>(player)
            .unwrap()
            .hotbar
            .get_stack(0)
            .map(|stack| stack.count),
        Some(2)
    );
}
