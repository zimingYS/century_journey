//! 组装区块流送资源和区块生命周期系统。

use super::{PlayerChunkCache, WorldStreamingConfig, manage_chunks_system};
use crate::shared::states::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{IntoScheduleConfigs, in_state};

/// 组装玩家附近区块的流送窗口和区块实体生命周期。
///
/// 该系统必须在生成任务之前运行，确保生成阶段读取到的区块实体和流送计划已存在。
pub(in crate::game::world) struct WorldStreamingPlugin;

impl Plugin for WorldStreamingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerChunkCache>()
            .init_resource::<WorldStreamingConfig>()
            .add_systems(
                Update,
                manage_chunks_system.run_if(in_state(AppState::InGame)),
            );
    }
}
