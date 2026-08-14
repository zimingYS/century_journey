use super::*;
use crate::content::item::definition::presentation::{AnimationConfig, HeldRenderDefinition};
use crate::content::item::definition::{DrinkData, ItemCategory, ItemDefinition};
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;
use std::time::Duration;

#[derive(Resource, Default)]
struct DrinkEventCount(usize);

fn count_drink_events(
    mut reader: MessageReader<DrinkConsumedEvent>,
    mut count: ResMut<DrinkEventCount>,
) {
    count.0 += reader.read().count();
}

fn run_fixed_step(app: &mut App) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_millis(750));
    app.world_mut().run_schedule(FixedUpdate);
}

#[test]
fn drink_is_consumed_only_after_the_use_animation_duration() {
    let bottle = ItemId::item("century_journey:water_bottle");
    let mut registry = ItemRegistry::default();
    registry.register(ItemDefinition {
        identifier: Identifier::parse("century_journey:water_bottle").unwrap(),
        display_name: "水瓶".into(),
        category: ItemCategory::Consumable,
        max_stack: 1,
        tags: vec!["drink".into()],
        icon: default(),
        model: None,
        placeable_block: None,
        tool: None,
        food: None,
        drink: Some(DrinkData { thirst: 8.0 }),
        held_renderer: HeldRenderDefinition::default(),
        animations: AnimationConfig::default(),
    });

    let mut inventory = InventoryState::default();
    inventory.hotbar.set_stack(0, ItemStack::new(bottle, 1));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(registry)
        .init_resource::<PlayerActionState>()
        .init_resource::<PlayerGameMode>()
        .init_resource::<DrinkEventCount>()
        .add_message::<DrinkConsumedEvent>()
        .add_systems(FixedUpdate, (use_drink_system, count_drink_events).chain());
    let player = app
        .world_mut()
        .spawn((
            Player,
            Thirst {
                current: 10.0,
                max: 20.0,
            },
            PlayerLifecycle::default(),
            DrinkUseState::default(),
            inventory,
        ))
        .id();
    app.world_mut()
        .resource_mut::<PlayerActionState>()
        .update(true, [PlayerAction::Use]);

    // 两次固定步（合计 1.5s）不足以覆盖 DRINK_USE_DURATION_SECONDS（1.6s），不消耗。
    run_fixed_step(&mut app);
    run_fixed_step(&mut app);

    let thirst = app.world().get::<Thirst>(player).unwrap();
    assert_eq!(thirst.current, 10.0);
    assert!(
        app.world()
            .get::<DrinkUseState>(player)
            .unwrap()
            .is_active()
    );
    assert_eq!(app.world().resource::<DrinkEventCount>().0, 0);
    assert_eq!(
        app.world()
            .get::<InventoryState>(player)
            .unwrap()
            .hotbar
            .get_stack(0)
            .map(|stack| stack.count),
        Some(1)
    );

    // 第三次固定步累计 2.25s > 1.6s，触发消耗。
    run_fixed_step(&mut app);

    let thirst = app.world().get::<Thirst>(player).unwrap();
    assert_eq!(thirst.current, 18.0);
    assert_eq!(app.world().resource::<DrinkEventCount>().0, 1);
    // 物品用尽（count=0）后槽位被自动清空。
    assert!(
        app.world()
            .get::<InventoryState>(player)
            .unwrap()
            .hotbar
            .get_stack(0)
            .is_none()
    );
}

#[test]
fn releasing_use_cancels_drink_without_consuming_it() {
    let bottle = ItemId::item("century_journey:water_bottle");
    let mut registry = ItemRegistry::default();
    registry.register(ItemDefinition {
        identifier: Identifier::parse("century_journey:water_bottle").unwrap(),
        display_name: "水瓶".into(),
        category: ItemCategory::Consumable,
        max_stack: 1,
        tags: vec!["drink".into()],
        icon: default(),
        model: None,
        placeable_block: None,
        tool: None,
        food: None,
        drink: Some(DrinkData { thirst: 8.0 }),
        held_renderer: HeldRenderDefinition::default(),
        animations: AnimationConfig::default(),
    });
    let mut inventory = InventoryState::default();
    inventory
        .hotbar
        .set_stack(0, ItemStack::new(bottle.clone(), 1));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(registry)
        .init_resource::<PlayerActionState>()
        .init_resource::<PlayerGameMode>()
        .add_message::<DrinkConsumedEvent>()
        .add_systems(FixedUpdate, use_drink_system);
    let player = app
        .world_mut()
        .spawn((
            Player,
            Thirst {
                current: 10.0,
                max: 20.0,
            },
            PlayerLifecycle::default(),
            DrinkUseState::default(),
            inventory,
        ))
        .id();

    app.world_mut()
        .resource_mut::<PlayerActionState>()
        .update(true, [PlayerAction::Use]);
    run_fixed_step(&mut app);
    app.world_mut()
        .resource_mut::<PlayerActionState>()
        .update(true, []);
    run_fixed_step(&mut app);

    assert_eq!(app.world().get::<Thirst>(player).unwrap().current, 10.0);
    assert!(
        !app.world()
            .get::<DrinkUseState>(player)
            .unwrap()
            .is_active()
    );
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
fn thirst_clamps_drink_and_ignores_invalid_exhaustion() {
    let mut thirst = Thirst {
        current: 19.0,
        max: 20.0,
    };

    thirst.drink(8.0);
    assert_eq!(thirst.current, 20.0);

    thirst.exhaust(f32::INFINITY);
    assert_eq!(thirst.current, 20.0);

    thirst.exhaust(-5.0);
    assert_eq!(thirst.current, 20.0);

    thirst.exhaust(f32::NAN);
    assert_eq!(thirst.current, 20.0);
}

#[test]
fn thirst_fraction_handles_clamp_and_invalid_inputs() {
    let mut thirst = Thirst {
        current: 10.0,
        max: 20.0,
    };
    assert_eq!(thirst.fraction(), 0.5);

    thirst.current = 0.0;
    assert_eq!(thirst.fraction(), 0.0);

    thirst.current = 30.0;
    assert_eq!(thirst.fraction(), 1.0);

    thirst.current = f32::NAN;
    assert_eq!(thirst.fraction(), 0.0);

    thirst.current = 10.0;
    thirst.max = 0.0;
    assert_eq!(thirst.fraction(), 0.0);
}
