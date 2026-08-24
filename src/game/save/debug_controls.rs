//! 消费开发阶段的存档诊断命令。
//!
//! 快捷键由 Client 采集；本模块只拥有存档语义，不能直接读取键盘或界面状态。

use bevy::prelude::*;

use crate::content::block::registry::BlockRegistry;
use crate::game::notification::{NotificationLevel, PlayerNotification};
use crate::game::player::identity::Player;
use crate::game::save;
use crate::game::save::world::metadata::io;
use crate::game::save::{SaveConfig, SaveQueue, SaveWorker};
use crate::game::world::state::WorldState;

/// 开发环境可提交的存档诊断命令。
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveDebugCommand {
    /// 立即等待后台队列并保存完整世界。
    SaveWorld,
    /// 读取并记录当前世界的元数据，不替换运行中状态。
    InspectWorldMetadata,
}

/// 处理开发阶段的存档诊断命令。
///
/// 该系统不使用时间源；每条消息只在收到的渲染帧执行一次。
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_save_debug_commands_system(
    mut commands: MessageReader<SaveDebugCommand>,
    world_state: Res<WorldState>,
    block_registry: Res<BlockRegistry>,
    save_config: Res<SaveConfig>,
    simulation_clock: Res<crate::game::world::time::WorldSimulationClock>,
    player_query: Query<&Transform, With<Player>>,
    world_generator: Res<crate::game::world::generation::generator::WorldGenerator>,
    mut save_queue: ResMut<SaveQueue>,
    mut save_worker: ResMut<SaveWorker>,
    mut notifications: MessageWriter<PlayerNotification>,
) {
    for command in commands.read().copied() {
        match command {
            SaveDebugCommand::SaveWorld => {
                if let Err(error) = save::flush_save_queue(
                    &save_config.world_name,
                    &mut save_queue,
                    &mut save_worker,
                ) {
                    log::error!("[世界] 等待后台区块保存失败: {error}");
                    notifications.write(PlayerNotification {
                        text: format!("保存世界失败：{error}"),
                        level: NotificationLevel::Warning,
                    });
                    continue;
                }
                let spawn_pos = player_query
                    .single()
                    .map(|transform| transform.translation)
                    .unwrap_or(Vec3::ZERO);
                if let Err(error) = save::save_entire_world(
                    &save_config.world_name,
                    &world_state,
                    &block_registry,
                    world_generator.seed as u64,
                    world_generator.generation_version,
                    &simulation_clock,
                    spawn_pos,
                ) {
                    log::error!("[世界] 保存世界失败: {error}");
                    notifications.write(PlayerNotification {
                        text: format!("保存世界失败：{error}"),
                        level: NotificationLevel::Warning,
                    });
                } else {
                    log::info!("[世界] 世界已保存！");
                    notifications.write(PlayerNotification {
                        text: "世界已保存".to_owned(),
                        level: NotificationLevel::Success,
                    });
                }
            }
            SaveDebugCommand::InspectWorldMetadata => {
                match io::load_level(&save_config.world_name) {
                    Ok(level) => log::info!(
                        "[世界] 世界元数据已加载: seed={}, spawn={:?}",
                        level.seed,
                        level.spawn_position
                    ),
                    Err(error) => log::error!("[世界] 加载世界失败: {error}"),
                }
            }
        }
    }
}
