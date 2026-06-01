pub mod format;
pub mod region;
pub mod level;
pub mod system;

use bevy::prelude::*;
use crate::core::state::app_state::AppState;
use crate::player::components::Player;
use crate::voxel::registry::BlockRegistry;
use crate::world::save::system::{AutoSaveTimer, LoadQueue, SaveConfig, SaveQueue};
use crate::world::storage::WorldStorage;

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) { app
        .insert_resource(SaveConfig::default())
        .insert_resource(SaveQueue::default())
        .insert_resource(LoadQueue::default())
        .init_resource::<AutoSaveTimer>()
        .add_systems(
            PostUpdate,
            (
                system::process_save_queue_system,
                system::process_load_queue_system,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            PostUpdate,
            system::auto_save_on_unload_system
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update,(
            save_load_keybind_system,
        ));
    }
}

/// 临时世界保存功能
/// F5 保存世界 / F9 加载世界
pub fn save_load_keybind_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    world_storage: Res<WorldStorage>,
    block_registry: Res<BlockRegistry>,
    save_config: Res<SaveConfig>,
    time_of_day: Res<crate::world::time::TimeOfDay>,
    player_query: Query<&Transform, With<Player>>,
) {
    // F5 — 保存
    if keyboard.just_pressed(KeyCode::F5) {
        let spawn_pos = player_query.single().map(|t| t.translation).unwrap_or(Vec3::ZERO);
        if let Err(e) = system::save_entire_world(
            &save_config.world_name,
            &world_storage,
            &block_registry,
            12345, // TODO: 从 WorldGenerator 获取
            spawn_pos,
            time_of_day.current_time,
        ) {
            log::error!("保存世界失败: {e}");
        } else {
            log::info!("世界已保存！");
        }
    }

    // F9 — 加载（注意：加载需要重启世界状态，此处仅加载元数据做演示）
    if keyboard.just_pressed(KeyCode::F9) {
        match level::load_level(&save_config.world_name) {
            Ok(level) => {
                log::info!(
                    "世界元数据已加载: seed={}, spawn={:?}",
                    level.seed,
                    level.spawn_position
                );
            }
            Err(e) => {
                log::error!("加载世界失败: {e}");
            }
        }
    }
}