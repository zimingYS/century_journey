//! 处理口渴值、饮品使用和脱水伤害规则。
//!
//! 口渴与饥饿是两条彼此独立的生存轴：饥饿靠食物恢复、口渴靠饮品恢复，
//! 二者按时间与动作消耗各自推进，仅在玩家死亡/重生时同步重置；本模块不引入
//! 饱和度等二级缓冲，所有衰减与恢复都直接作用在 `Thirst.current` 上。

use crate::content::item::ItemRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::InventoryState;
use crate::game::inventory::container::InventoryContainer;
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::PlayerLifecycle;
use crate::game::player::survival::events::{DamageEvent, DamageSource, DrinkConsumedEvent};
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 玩家口渴值
#[derive(Component, Debug, Clone)]
pub struct Thirst {
    /// 当前口渴值。
    pub current: f32,
    /// 口渴值上限。
    pub max: f32,
}

impl Default for Thirst {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
        }
    }
}

impl Thirst {
    /// 返回供 HUD 使用且约束在零到一之间的口渴比例。
    pub fn fraction(&self) -> f32 {
        if !self.current.is_finite() || !self.max.is_finite() || self.max <= 0.0 {
            return 0.0;
        }
        (self.current / self.max).clamp(0.0, 1.0)
    }
    /// 判断玩家是否已进入脱水伤害区间。
    pub fn is_dehydrated(&self) -> bool {
        self.current <= 0.0
    }
    /// 判断当前口渴值是否已达到上限。
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }
    /// 饮用物品并恢复口渴值。
    pub fn drink(&mut self, amount: f32) {
        if amount.is_finite() && amount > 0.0 {
            self.current = (self.current + amount).min(self.max);
        }
    }
    /// 应用活动消耗或自然流逝，直接扣减口渴值并夹紧到零。
    pub fn exhaust(&mut self, amount: f32) {
        if !amount.is_finite() || amount <= 0.0 {
            return;
        }
        self.current = (self.current - amount).max(0.0);
    }
}

/// 饮品必须连续使用达到该时长后才会真正消耗。
pub const DRINK_USE_DURATION_SECONDS: f32 = 1.6;
/// 口渴随时间的自然消耗速率（每秒）；满口渴 20 点约 8 分钟耗尽。
const THIRST_PASSIVE_DRAIN_PER_SECOND: f32 = 1.0 / 480.0;

/// 口渴消耗：随时间自然流逝，冲刺与跳跃额外加速。
pub fn thirst_drain_system(
    time: Res<Time<Fixed>>,
    actions: Res<PlayerActionState>,
    gamemode: Res<PlayerGameMode>,
    mut query: Query<(&mut Thirst, &PlayerLifecycle), With<Player>>,
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

    for (mut thirst, lifecycle) in &mut query {
        if !lifecycle.is_alive() || gamemode.is_creative() {
            continue;
        }
        let mut drain = THIRST_PASSIVE_DRAIN_PER_SECOND * dt;
        if sprinting {
            drain += 0.05 * dt;
        }
        if jumped {
            drain += 0.03;
        }
        thirst.exhaust(drain);
    }
}

/// 使用当前快捷栏中的饮品。
pub fn use_drink_system(
    time: Res<Time<Fixed>>,
    actions: Res<PlayerActionState>,
    gamemode: Res<PlayerGameMode>,
    item_registry: Option<Res<ItemRegistry>>,
    mut query: Query<
        (
            Entity,
            &mut Thirst,
            &PlayerLifecycle,
            &mut DrinkUseState,
            &mut InventoryState,
        ),
        With<Player>,
    >,
    mut consumed_writer: MessageWriter<DrinkConsumedEvent>,
) {
    let Ok((player, mut thirst, lifecycle, mut drink_use, mut inventory)) = query.single_mut()
    else {
        return;
    };

    if !actions.pressed(PlayerAction::Use)
        || !gamemode.is_survival()
        || !lifecycle.is_alive()
        || thirst.is_full()
    {
        drink_use.cancel();
        return;
    }

    let Some(item_registry) = item_registry else {
        drink_use.cancel();
        return;
    };
    let active_index = inventory.hotbar.active_index;
    let Some(active_stack) = inventory.hotbar.get_stack(active_index) else {
        drink_use.cancel();
        return;
    };
    let drink_item = active_stack.item.clone();
    let Some(drink) = item_registry
        .get(&active_stack.item)
        .and_then(|definition| definition.drink_data())
        .copied()
    else {
        drink_use.cancel();
        return;
    };

    if !drink_use.matches(&drink_item, active_index) {
        drink_use.start(drink_item.clone(), active_index);
    }
    drink_use.advance(time.delta_secs());
    if drink_use.elapsed_seconds() < DRINK_USE_DURATION_SECONDS {
        return;
    }

    let consumed = inventory
        .hotbar
        .get_stack_mut(active_index)
        .filter(|stack| stack.item == drink_item)
        .and_then(|stack| stack.take(1))
        .is_some();
    if !consumed {
        drink_use.cancel();
        return;
    }

    thirst.drink(drink.thirst);
    if inventory
        .hotbar
        .get_stack(active_index)
        .is_some_and(crate::game::inventory::item::stack::ItemStack::is_empty)
    {
        inventory.hotbar.clear_slot(active_index);
    }
    drink_use.cancel();
    consumed_writer.write(DrinkConsumedEvent {
        player,
        item: drink_item,
    });
}

/// 脱水伤害（每 4 秒，口渴归零）。
pub fn dehydration_damage_system(
    mut timer: Local<f32>,
    time: Res<Time<Fixed>>,
    query: Query<(Entity, &Thirst, &PlayerLifecycle), With<Player>>,
    gamemode: Res<PlayerGameMode>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    *timer -= time.delta_secs();
    if *timer > 0.0 {
        return;
    }
    *timer = 4.0;

    for (entity, thirst, lifecycle) in &query {
        if lifecycle.is_alive() && thirst.is_dehydrated() && gamemode.is_survival() {
            damage_writer.write(DamageEvent {
                target: entity,
                amount: 1.0,
                source: DamageSource::Dehydration,
            });
        }
    }
}

/// 跟踪一次持续饮品使用动作；只有动作完成后才消耗物品。
#[derive(Component, Debug, Clone, Default)]
pub struct DrinkUseState {
    /// 正在使用中的饮品 ID；`None` 表示当前未在使用。
    item: Option<ItemId>,
    /// 饮品所在快捷栏槽位，用于切换槽位时取消旧动作。
    hotbar_slot: usize,
    /// 使用动作已连续保持的秒数。
    elapsed_seconds: f32,
}
impl DrinkUseState {
    /// 开始跟踪指定快捷栏槽位中的饮品。
    pub fn start(&mut self, item: ItemId, hotbar_slot: usize) {
        self.item = Some(item);
        self.hotbar_slot = hotbar_slot;
        self.elapsed_seconds = 0.0;
    }

    /// 取消当前使用动作并清空累计时长。
    pub fn cancel(&mut self) {
        *self = Self::default();
    }

    /// 判断是否正在跟踪饮品使用动作。
    pub fn is_active(&self) -> bool {
        self.item.is_some()
    }

    /// 判断当前动作是否仍对应同一物品和快捷栏槽位。
    pub fn matches(&self, item: &ItemId, hotbar_slot: usize) -> bool {
        self.item.as_ref() == Some(item) && self.hotbar_slot == hotbar_slot
    }

    /// 使用有效的正数固定步时长推进动作。
    pub fn advance(&mut self, delta_seconds: f32) {
        if delta_seconds.is_finite() && delta_seconds > 0.0 {
            self.elapsed_seconds += delta_seconds;
        }
    }

    /// 返回动作已连续保持的秒数。
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    /// 返回相对指定总时长、约束在零到一之间的进度。
    pub fn progress(&self, duration_seconds: f32) -> f32 {
        if duration_seconds <= 0.0 {
            return 1.0;
        }
        (self.elapsed_seconds / duration_seconds).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/survival/thirst.rs"]
mod tests;
