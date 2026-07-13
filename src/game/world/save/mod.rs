pub mod events;
pub mod format;
pub mod level;
pub mod player;
pub mod region;
pub mod system;

use crate::content::block::registry::BlockRegistry;
use crate::game::player::components::Player;
use crate::game::world::save::events::SaveDirtyEvent;
use crate::game::world::save::player::PlayerSaveManager;
use crate::game::world::save::system::{
    AutoSaveTimer, CachedBlockIdRemap, LoadQueue, SaveConfig, SaveQueue,
};
use crate::game::world::storage::WorldStorage;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveConfig::default())
            .insert_resource(SaveQueue::default())
            .insert_resource(LoadQueue::default())
            .init_resource::<AutoSaveTimer>()
            .init_resource::<CachedBlockIdRemap>()
            .init_resource::<PlayerSaveManager>()
            .add_message::<SaveDirtyEvent>()
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    system::cache_level_data_on_enter,
                    player::load_player_on_enter_system,
                )
                    .run_if(crate::app::flow::fresh_game_session),
            )
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
                system::auto_save_on_unload_system.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    save_load_keybind_system,
                    player::player_position_dirty_system,
                    player::inventory_dirty_tracking_system,
                    player::gamemode_dirty_tracking_system,
                    player::auto_save_player_system,
                    player::save_on_exit_system,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(Last, player::save_on_exit_system);
    }
}

/// 临时世界保存功能
/// F5 保存世界 / F9 加载世界
pub fn save_load_keybind_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    context: Res<crate::shared::states::InputContextState>,
    world_storage: Res<WorldStorage>,
    block_registry: Res<BlockRegistry>,
    save_config: Res<SaveConfig>,
    time_of_day: Res<crate::game::world::time::TimeOfDay>,
    player_query: Query<&Transform, With<Player>>,
    world_generator: Res<crate::game::world::generation::WorldGenerator>,
) {
    if !context.active().allows_gameplay() {
        return;
    }
    // F5 — 保存
    if keyboard.just_pressed(KeyCode::F5) {
        let spawn_pos = player_query
            .single()
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);
        if let Err(e) = system::save_entire_world(
            &save_config.world_name,
            &world_storage,
            &block_registry,
            world_generator.seed as u64,
            spawn_pos,
            time_of_day.current_time,
        ) {
            log::error!("[世界] 保存世界失败: {e}");
        } else {
            log::info!("[世界] 世界已保存！");
        }
    }

    // F9 — 加载（注意：加载需要重启世界状态，此处仅加载元数据做演示）
    if keyboard.just_pressed(KeyCode::F9) {
        match level::load_level(&save_config.world_name) {
            Ok(level) => {
                log::info!(
                    "[世界] 世界元数据已加载: seed={}, spawn={:?}",
                    level.seed,
                    level.spawn_position
                );
            }
            Err(e) => {
                log::error!("[世界] 加载世界失败: {e}");
            }
        }
    }
}
