//! 组装权威世界状态与区块运行时资源。

use crate::game::world::state::authoritative::WorldState;
use crate::game::world::state::chunk_runtime::ChunkRuntime;
use bevy::prelude::*;

/// 组装权威世界数据与区块运行时状态资源。
///
/// 这是世界领域最底层的基础状态，其余子领域插件都在其上读写；无渲染的
/// 测试/服务端入口通过 [`crate::game::world::state::HeadlessWorldPlugin`]
/// 复用同一份装配，避免两处重复初始化。
pub struct WorldStatePlugin;

impl Plugin for WorldStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldState>()
            .init_resource::<ChunkRuntime>();
    }
}
