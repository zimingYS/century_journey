use crate::content::item::ItemRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::LocalInventoryMut;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::{PlayerLifecycle, RespawnPoint};
use crate::game::player::movement::components::PlayerVelocity;
use crate::game::save::SaveConfig;
use crate::game::save::player::PlayerSaveManager;
use crate::shared::components::FpsCamera;
use bevy::camera::Camera3d;
use bevy::prelude::{Query, Res, ResMut, Time, Transform, With};

/// 进入游戏时加载玩家存档
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
        ),
        With<Player>,
    >,
    mut camera_query: Query<&mut FpsCamera, With<Camera3d>>,
    mut save_manager: ResMut<PlayerSaveManager>,
    _time: Res<Time>,
) {
    use crate::game::save::dirty::SaveDirtySource;
    use crate::game::save::player::data::validation::validate_player_data;
    use crate::game::save::player::{PlayerSaveData, player_save_path, read_player_data};
    use bevy::prelude::DetectChangesMut;

    let save_path = player_save_path(&save_config.world_name);
    let raw_data = if save_path.exists() {
        match read_player_data(&save_path) {
            Ok(data) => {
                log::info!("[存档系统] 从 {:?} 加载数据成功", save_path);
                data
            }
            Err(e) => {
                log::warn!("[存档系统] 从 {} 加载失败, 已使用默认值", e);
                PlayerSaveData::default()
            }
        }
    } else {
        log::info!("[存档系统] 存档不存在，正在已默认值创建");
        PlayerSaveData::default()
    };

    let save_data = validate_player_data(&raw_data);
    *gamemode = save_data.restore_gamemode();

    let restored = save_data.restore_inventory_with_registry(&item_registry);
    inventory.hotbar = restored.hotbar;
    inventory.survival = restored.survival;

    if let Ok((
        mut transform,
        mut health,
        mut hunger,
        mut respawn_point,
        mut lifecycle,
        mut velocity,
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
        *lifecycle = PlayerLifecycle::default();
        *velocity = PlayerVelocity::default();
    }

    if let Ok(mut fps_camera) = camera_query.single_mut() {
        fps_camera.set_pitch(save_data.camera_pitch());
    }

    save_manager.set_dirty(SaveDirtySource::Position);
    inventory.set_changed();

    log::info!(
        "[存档系统] 玩家已生成,位置:{:?},游戏模式:{}",
        save_data.position,
        save_data.gamemode
    );
}
