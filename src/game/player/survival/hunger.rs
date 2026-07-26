use crate::content::item::registry::registry::ItemRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::state::InventoryState;
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::PlayerLifecycle;
use crate::game::player::survival::events::{
    DamageEvent, DamageSource, FoodConsumedEvent, HealEvent,
};
use crate::game::player::survival::health::Health;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// Food must be used continuously for this long before it is consumed.
pub const FOOD_USE_DURATION_SECONDS: f32 = 1.6;

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
    time: Res<Time>,
    actions: Res<PlayerActionState>,
    gamemode: Res<PlayerGameMode>,
    item_registry: Option<Res<ItemRegistry>>,
    mut query: Query<
        (
            Entity,
            &mut Hunger,
            &PlayerLifecycle,
            &mut FoodUseState,
            &mut InventoryState,
        ),
        With<Player>,
    >,
    mut consumed_writer: MessageWriter<FoodConsumedEvent>,
) {
    let Ok((player, mut hunger, lifecycle, mut food_use, mut inventory)) = query.single_mut()
    else {
        return;
    };

    if !actions.pressed(PlayerAction::Use)
        || !gamemode.is_survival()
        || !lifecycle.is_alive()
        || hunger.is_full()
    {
        food_use.cancel();
        return;
    }

    let Some(item_registry) = item_registry else {
        food_use.cancel();
        return;
    };
    let active_index = inventory.hotbar.active_index;
    let Some(active_stack) = inventory.hotbar.get_stack(active_index) else {
        food_use.cancel();
        return;
    };
    let food_item = active_stack.item.clone();
    let Some(food) = item_registry
        .get(active_stack.item_id())
        .and_then(|definition| definition.food_data())
        .copied()
    else {
        food_use.cancel();
        return;
    };

    if !food_use.matches(&food_item, active_index) {
        food_use.start(food_item.clone(), active_index);
    }
    food_use.advance(time.delta_secs());
    if food_use.elapsed_seconds() < FOOD_USE_DURATION_SECONDS {
        return;
    }

    let consumed = inventory
        .hotbar
        .get_stack_mut(active_index)
        .filter(|stack| stack.item == food_item)
        .and_then(|stack| stack.take(1))
        .is_some();
    if !consumed {
        food_use.cancel();
        return;
    }

    hunger.eat(food.hunger, food.saturation);
    if inventory
        .hotbar
        .get_stack(active_index)
        .is_some_and(crate::game::inventory::item::stack::ItemStack::is_empty)
    {
        inventory.hotbar.clear_slot(active_index);
    }
    food_use.cancel();
    consumed_writer.write(FoodConsumedEvent {
        player,
        item: food_item,
    });
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
#[path = "../../../../tests/unit/game/player/systems/hunger.rs"]
mod stage_seven_tests;

/// 饥饿值
#[derive(Component, Debug, Clone)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
    pub saturation: f32,
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
            saturation: 5.0,
        }
    }
}

impl Hunger {
    pub fn fraction(&self) -> f32 {
        if !self.current.is_finite() || !self.max.is_finite() || self.max <= 0.0 {
            return 0.0;
        }
        (self.current / self.max).clamp(0.0, 1.0)
    }
    pub fn is_starving(&self) -> bool {
        self.current <= 0.0
    }
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// 食用物品并恢复饥饿与饱和度。
    pub fn eat(&mut self, hunger: f32, saturation: f32) {
        if hunger.is_finite() && hunger > 0.0 {
            self.current = (self.current + hunger).min(self.max);
        }
        if saturation.is_finite() && saturation > 0.0 {
            self.saturation = (self.saturation + saturation).min(self.current.max(0.0));
        }
    }
    /// 消耗, 优先从 saturation 扣除
    pub fn exhaust(&mut self, amount: f32) {
        if !amount.is_finite() || amount <= 0.0 {
            return;
        }
        if self.saturation > 0.0 {
            let d = amount.min(self.saturation);
            self.saturation -= d;
            let rest = amount - d;
            self.current = (self.current - rest).max(0.0);
        } else {
            self.current = (self.current - amount).max(0.0);
        }
    }
}

/// Tracks a food use action. The item is consumed only after the action completes.
#[derive(Component, Debug, Clone, Default)]
pub struct FoodUseState {
    item: Option<ItemId>,
    hotbar_slot: usize,
    elapsed_seconds: f32,
}

impl FoodUseState {
    pub fn start(&mut self, item: ItemId, hotbar_slot: usize) {
        self.item = Some(item);
        self.hotbar_slot = hotbar_slot;
        self.elapsed_seconds = 0.0;
    }

    pub fn cancel(&mut self) {
        *self = Self::default();
    }

    pub fn is_active(&self) -> bool {
        self.item.is_some()
    }

    pub fn matches(&self, item: &ItemId, hotbar_slot: usize) -> bool {
        self.item.as_ref() == Some(item) && self.hotbar_slot == hotbar_slot
    }

    pub fn advance(&mut self, delta_seconds: f32) {
        if delta_seconds.is_finite() && delta_seconds > 0.0 {
            self.elapsed_seconds += delta_seconds;
        }
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    pub fn progress(&self, duration_seconds: f32) -> f32 {
        if duration_seconds <= 0.0 {
            return 1.0;
        }
        (self.elapsed_seconds / duration_seconds).clamp(0.0, 1.0)
    }
}
