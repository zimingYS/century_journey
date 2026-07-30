//! 处理玩家附近掉落物的自动拾取。

use bevy::log::info;
use bevy::prelude::{Commands, Entity, Local, MessageWriter, Query, Res, Time, Transform, With};

use crate::game::inventory::events::InventoryFeedbackEvent;
use crate::game::inventory::interaction::transfer;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::PlayerLifecycle;
use crate::game::world::entity::dropped_item::{DroppedItem, despawn_dropped_item};

/// 玩家可自动拾取掉落物的半径。
const PICKUP_RANGE: f32 = 2.0;

/// 在固定步中为最近的存活玩家处理范围内掉落物。
///
/// 成功插入后删除实体；容量不足时保留剩余堆叠，并对“背包已满”反馈做节流。
pub fn pickup_system(
    time: Res<Time>,
    mut player_query: Query<(&Transform, &PlayerLifecycle, &mut InventoryState), With<Player>>,
    mut item_query: Query<(Entity, &Transform, &mut DroppedItem)>,
    mut commands: Commands,
    mut feedback_writer: MessageWriter<InventoryFeedbackEvent>,
    mut full_feedback_cooldown: Local<f32>,
) {
    *full_feedback_cooldown = (*full_feedback_cooldown - time.delta_secs()).max(0.0);
    for (entity, item_transform, mut dropped) in &mut item_query {
        // 新生成的掉落物先等待拾取冷却，避免玩家主动丢弃后立即捡回。
        if !dropped.can_pickup() {
            continue;
        }

        let Some((_, _, mut inventory)) = player_query
            .iter_mut()
            .filter(|(transform, lifecycle, _)| {
                lifecycle.is_alive()
                    && transform.translation.distance(item_transform.translation) <= PICKUP_RANGE
            })
            .min_by(|(left, _, _), (right, _, _)| {
                left.translation
                    .distance_squared(item_transform.translation)
                    .total_cmp(
                        &right
                            .translation
                            .distance_squared(item_transform.translation),
                    )
            })
        else {
            continue;
        };

        // 分两次借用库存，按“快捷栏、主背包”的稳定顺序插入。
        let result = transfer::insert_into_container(&mut inventory.hotbar, dropped.stack.clone());
        let result = match result {
            transfer::InventoryInsertResult::AllInserted => result,
            transfer::InventoryInsertResult::Partial(remaining)
            | transfer::InventoryInsertResult::Full(remaining) => transfer::insert_into_range(
                &mut inventory.survival,
                remaining,
                0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
            ),
        };

        match result {
            transfer::InventoryInsertResult::AllInserted => {
                info!("Picked up {:?}", dropped.stack);
                despawn_dropped_item(&mut commands, entity);
            }
            transfer::InventoryInsertResult::Partial(remaining) => {
                dropped.stack = remaining;
            }
            transfer::InventoryInsertResult::Full(_) => {
                // 满载提示按玩家交互节奏限频，避免同一实体在连续固定步中反复提示。
                if *full_feedback_cooldown <= 0.0 {
                    feedback_writer.write(InventoryFeedbackEvent::Full);
                    *full_feedback_cooldown = 1.25;
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/interaction/pickup.rs"]
mod tests;
