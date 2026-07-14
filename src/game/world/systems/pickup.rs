use crate::game::inventory::events::InventoryFeedbackEvent;
use crate::game::inventory::insert;
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::{Player, PlayerLifecycle};
use crate::game::world::entity::dropped_item::{DroppedItem, despawn_dropped_item};
use bevy::prelude::*;

/// 拾取范围半径
const PICKUP_RANGE: f32 = 2.0;

/// 自动拾取系统
/// 每帧检测玩家范围内所有掉落物，尝试插入玩家背包。
/// 成功则删除掉落物实体，失败则保留剩余物品
pub fn pickup_system(
    time: Res<Time>,
    player_query: Query<(&Transform, &PlayerLifecycle), With<Player>>,
    mut item_query: Query<(Entity, &Transform, &mut DroppedItem)>,
    mut inventory: ResMut<InventoryState>,
    mut commands: Commands,
    mut feedback_writer: MessageWriter<InventoryFeedbackEvent>,
    mut full_feedback_cooldown: Local<f32>,
) {
    *full_feedback_cooldown = (*full_feedback_cooldown - time.delta_secs()).max(0.0);
    let Ok((player_transform, lifecycle)) = player_query.single() else {
        return;
    };
    if !lifecycle.is_alive() {
        return;
    }
    let player_pos = player_transform.translation;

    for (entity, item_transform, mut dropped) in &mut item_query {
        // 刚生成的掉落物先等待一小段时间，避免玩家按 Q 后马上又捡回来。
        if !dropped.can_pickup() {
            continue;
        }

        // 距离检查
        if player_pos.distance(item_transform.translation) > PICKUP_RANGE {
            continue;
        }

        // 尝试插入背包（优先快捷栏，再背包）
        // 两步插入避免同时 borrow hotbar 和 survival
        let result = insert::insert_into_container(&mut inventory.hotbar, dropped.stack.clone());
        let result = match result {
            insert::InventoryInsertResult::AllInserted => result,
            insert::InventoryInsertResult::Partial(remaining)
            | insert::InventoryInsertResult::Full(remaining) => insert::insert_into_range(
                &mut inventory.survival,
                remaining,
                0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
            ),
        };

        match result {
            insert::InventoryInsertResult::AllInserted => {
                info!("Picked up {:?}", dropped.stack);
                despawn_dropped_item(&mut commands, entity);
            }
            insert::InventoryInsertResult::Partial(remaining) => {
                dropped.stack = remaining;
            }
            insert::InventoryInsertResult::Full(_) => {
                // 满载提示做节流，避免同一件地面物品每帧重复播放提示。
                if *full_feedback_cooldown <= 0.0 {
                    feedback_writer.write(InventoryFeedbackEvent::Full);
                    *full_feedback_cooldown = 1.25;
                }
            }
        }
    }
}

#[cfg(test)]
mod stage_seven_tests {
    use super::*;
    use crate::game::inventory::events::InventoryFeedbackEvent;
    use crate::game::inventory::item::stack::ItemStack;
    use crate::game::player::components::PlayerLifecycle;
    use crate::shared::item_id::ItemId;

    #[test]
    fn stage_seven_pickup_moves_drop_into_empty_inventory() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InventoryState>()
            .add_message::<InventoryFeedbackEvent>()
            .add_systems(Update, pickup_system);
        app.world_mut().spawn((
            Player,
            PlayerLifecycle::default(),
            Transform::from_xyz(0.0, 70.0, 0.0),
        ));
        let mut dropped =
            DroppedItem::new(ItemStack::new(ItemId::item("century_journey:stick"), 3));
        dropped.pickup_delay = 0.0;
        let drop_entity = app
            .world_mut()
            .spawn((dropped, Transform::from_xyz(0.5, 70.0, 0.0)))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .resource::<InventoryState>()
                .hotbar
                .get_stack(0)
                .map(|stack| stack.count),
            Some(3)
        );
        assert!(app.world().get_entity(drop_entity).is_err());
    }
}
