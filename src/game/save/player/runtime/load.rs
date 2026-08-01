//! 在进入世界时把玩家存档恢复到权威组件。

use bevy::prelude::{DetectChangesMut, Query, Res, ResMut, Transform, With};

use crate::content::item::ItemRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::{InventoryState, LocalInventoryMut};
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::{PlayerLifecycle, RespawnPoint};
use crate::game::player::movement::components::{PlayerAim, PlayerVelocity};
use crate::game::save::SaveConfig;
use crate::game::save::dirty::SaveDirtySource;
use crate::game::save::player::data::validation::validate_player_data;
use crate::game::save::player::{
    PlayerSaveData, PlayerSaveManager, player_save_path, read_player_data,
};

/// 进入游戏时加载并校验玩家存档。
///
/// 恢复只写入 Game 层权威组件；客户端相机在自己的表现同步阶段读取 PlayerAim。
/// 查询元组保证同一玩家的存档字段在一次恢复事务中共同更新。
#[allow(clippy::type_complexity)]
pub fn load_player_on_enter_system(
    save_config: Res<SaveConfig>,
    mut gamemode: ResMut<PlayerGameMode>,
    mut inventory: LocalInventoryMut,
    item_registry: Res<ItemRegistry>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut crate::game::player::survival::health::Health,
            &mut crate::game::player::survival::hunger::Hunger,
            &mut RespawnPoint,
            &mut PlayerLifecycle,
            &mut PlayerVelocity,
            &mut PlayerAim,
        ),
        With<Player>,
    >,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    save_manager.begin_session();
    let save_path = player_save_path(&save_config.world_name);
    let raw_data = if save_path.exists() {
        match read_player_data(&save_path) {
            Ok(data) => {
                log::info!("[存档系统] 从 {:?} 加载数据成功", save_path);
                data
            }
            Err(error) => {
                log::warn!("[存档系统] 从 {} 加载失败，已使用默认值", error);
                PlayerSaveData::default()
            }
        }
    } else {
        log::info!("[存档系统] 存档不存在，使用默认值创建玩家");
        PlayerSaveData::default()
    };

    let save_data = validate_player_data(&raw_data);
    *gamemode = save_data.restore_gamemode();

    replace_inventory_for_session(&mut inventory, &save_data, &item_registry);

    if let Ok((
        mut transform,
        mut health,
        mut hunger,
        mut respawn_point,
        mut lifecycle,
        mut velocity,
        mut aim,
    )) = player_query.single_mut()
    {
        *respawn_point = RespawnPoint(save_data.respawn_point());
        *transform = if save_data.health <= 0.0 {
            Transform::from_translation(respawn_point.0)
        } else {
            save_data.restore_transform()
        };
        save_manager.last_saved_position = transform.translation;

        health.current = if save_data.health <= 0.0 {
            health.max
        } else {
            save_data.health.clamp(0.0, health.max)
        };
        hunger.current = save_data.hunger.clamp(0.0, hunger.max);
        hunger.saturation = save_data.saturation.clamp(0.0, hunger.current);
        aim.pitch = save_data.camera_pitch();
        *lifecycle = PlayerLifecycle::default();
        *velocity = PlayerVelocity::default();
    }

    save_manager.set_dirty(SaveDirtySource::Position);
    inventory.set_changed();

    log::info!(
        "[存档系统] 玩家已生成，位置：{:?}，游戏模式：{}",
        save_data.position,
        save_data.gamemode
    );
}
/// 使用当前世界的存档整体替换库存，避免光标和最近物品等会话状态跨世界残留。
fn replace_inventory_for_session(
    inventory: &mut InventoryState,
    save_data: &PlayerSaveData,
    item_registry: &ItemRegistry,
) {
    *inventory = save_data.restore_inventory_with_registry(item_registry);
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/save/player/runtime/load.rs"]
mod tests;
