use crate::content::item::registry::registry::ItemRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::state::InventoryState;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::stats::{Health, Hunger};
use crate::game::player::components::{Player, PlayerLifecycle};
use crate::game::player::events::{DamageEvent, DamageSource, HealEvent};
use bevy::prelude::*;

/// 动作消耗系统：冲刺和跳跃会消耗饥饿值。
pub fn action_cost_system(
    time: Res<Time>,
    actions: Res<PlayerActionState>,
    mut query: Query<(&mut Hunger, &PlayerLifecycle), With<Player>>,
) {
    let dt = time.delta_secs();

    let sprinting = actions.pressed(PlayerAction::Sprint)
        && [
            PlayerAction::MoveForward,
            PlayerAction::MoveBackward,
            PlayerAction::MoveLeft,
            PlayerAction::MoveRight,
        ]
        .into_iter()
        .any(|action| actions.pressed(action));
    let jumped = actions.just_pressed(PlayerAction::Jump);

    for (mut hunger, lifecycle) in &mut query {
        if !lifecycle.is_alive() {
            continue;
        }
        if sprinting {
            hunger.exhaust(0.1 * dt);
        }
        if jumped {
            hunger.exhaust(0.05);
        }
    }
}

/// 使用当前快捷栏中的食物。
pub fn use_food_system(
    actions: Res<PlayerActionState>,
    gamemode: Res<PlayerGameMode>,
    item_registry: Option<Res<ItemRegistry>>,
    mut inventory: ResMut<InventoryState>,
    mut query: Query<(&mut Hunger, &PlayerLifecycle), With<Player>>,
) {
    if !actions.just_pressed(PlayerAction::Use) || !gamemode.is_survival() {
        return;
    }
    let Some(item_registry) = item_registry else {
        return;
    };
    let active_index = inventory.hotbar.active_index;
    let Some(food) = inventory
        .hotbar
        .get_stack(active_index)
        .and_then(|stack| item_registry.get(stack.item_id()))
        .and_then(|definition| definition.food_data())
        .copied()
    else {
        return;
    };

    let Ok((mut hunger, lifecycle)) = query.single_mut() else {
        return;
    };
    if !lifecycle.is_alive() || hunger.is_full() {
        return;
    }

    hunger.eat(food.hunger, food.saturation);
    if let Some(stack) = inventory.hotbar.get_stack_mut(active_index) {
        let _ = stack.take(1);
        if stack.is_empty() {
            inventory.hotbar.set_stack(
                active_index,
                crate::game::inventory::item::stack::ItemStack::empty(),
            );
        }
    }
}

/// 满饥饿自动回血 (每 4s, hunger >= 18)
pub fn natural_regeneration_system(
    mut timer: Local<f32>,
    time: Res<Time>,
    mut query: Query<(Entity, &Health, &mut Hunger, &PlayerLifecycle), With<Player>>,
    mut heal_writer: MessageWriter<HealEvent>,
) {
    *timer -= time.delta_secs();
    if *timer > 0.0 {
        return;
    }
    *timer = 4.0;

    for (entity, health, mut hunger, lifecycle) in &mut query {
        if lifecycle.is_alive() && hunger.current >= 18.0 && health.current < health.max {
            heal_writer.write(HealEvent {
                target: entity,
                amount: 1.0,
            });
            hunger.exhaust(0.25);
        }
    }
}

/// 饥饿伤害 (每 4s, hunger <= 0)
pub fn starvation_damage_system(
    mut timer: Local<f32>,
    time: Res<Time>,
    query: Query<(Entity, &Hunger, &PlayerLifecycle), With<Player>>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    *timer -= time.delta_secs();
    if *timer > 0.0 {
        return;
    }
    *timer = 4.0;

    for (entity, hunger, lifecycle) in &query {
        if lifecycle.is_alive() && hunger.is_starving() {
            damage_writer.write(DamageEvent {
                target: entity,
                amount: 1.0,
                source: DamageSource::Starvation,
            });
        }
    }
}

#[cfg(test)]
mod stage_seven_tests {
    use super::*;
    use crate::content::item::definition::{FoodData, ItemCategory, ItemDefinition};
    use crate::game::inventory::item::stack::ItemStack;
    use crate::shared::held_item::{AnimationConfig, HeldRenderDefinition};
    use crate::shared::identifier::Identifier;
    use crate::shared::item_id::ItemId;

    #[test]
    fn stage_seven_food_use_consumes_one_and_restores_hunger() {
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
        app.insert_resource(registry)
            .insert_resource(inventory)
            .init_resource::<PlayerActionState>()
            .init_resource::<PlayerGameMode>()
            .add_systems(Update, use_food_system);
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
            ))
            .id();
        app.world_mut()
            .resource_mut::<PlayerActionState>()
            .update(true, [PlayerAction::Use]);

        app.update();

        let hunger = app.world().get::<Hunger>(player).unwrap();
        assert_eq!(hunger.current, 14.0);
        assert_eq!(hunger.saturation, 2.4);
        assert_eq!(
            app.world()
                .resource::<InventoryState>()
                .hotbar
                .get_stack(0)
                .map(|stack| stack.count),
            Some(1)
        );
    }
}
