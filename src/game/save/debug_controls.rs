//! 提供开发阶段使用的存档快捷键控制。

use crate::content::block::registry::BlockRegistry;
use crate::game::player::identity::Player;
use crate::game::save;
use crate::game::save::world::metadata::io;
use crate::game::save::{SaveConfig, SaveQueue, SaveWorker};
use crate::game::world::state::WorldState;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::{KeyCode, Query, Res, ResMut, Transform, With};

/// 处理开发阶段使用的 F5 保存和 F9 加载快捷键。
///
/// F9 当前只读取并输出世界元数据，不负责重新构建运行中的世界。
pub(super) fn save_load_keybind_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    context: Res<crate::shared::states::InputContextState>,
    world_state: Res<WorldState>,
    block_registry: Res<BlockRegistry>,
    save_config: Res<SaveConfig>,
    simulation_clock: Res<crate::game::world::time::WorldSimulationClock>,
    player_query: Query<&Transform, With<Player>>,
    world_generator: Res<crate::game::world::generation::generator::WorldGenerator>,
    mut save_queue: ResMut<SaveQueue>,
    mut save_worker: ResMut<SaveWorker>,
) {
    if !context.active().allows_gameplay() {
        return;
    }
    // F5 — 保存
    if keyboard.just_pressed(KeyCode::F5) {
        if let Err(error) =
            save::flush_save_queue(&save_config.world_name, &mut save_queue, &mut save_worker)
        {
            log::error!("[世界] 等待后台区块保存失败: {error}");
            return;
        }
        let spawn_pos = player_query
            .single()
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);
        if let Err(e) = save::save_entire_world(
            &save_config.world_name,
            &world_state,
            &block_registry,
            world_generator.seed as u64,
            world_generator.generation_version,
            &simulation_clock,
            spawn_pos,
        ) {
            log::error!("[世界] 保存世界失败: {e}");
        } else {
            log::info!("[世界] 世界已保存！");
        }
    }

    // F9 — 加载（注意：加载需要重启世界状态，此处仅加载元数据做演示）
    if keyboard.just_pressed(KeyCode::F9) {
        match io::load_level(&save_config.world_name) {
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
