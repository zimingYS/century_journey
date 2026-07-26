use bevy::prelude::*;

use crate::engine::asset::AssetPlugin;
use crate::engine::task::TaskPlugin;

/// Engine 层插件聚合入口。
///
/// 只负责注册通用基础设施，不包含 Content、Game 或 Client 的玩法知识。
pub struct EnginePluginGroup;

impl Plugin for EnginePluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins((AssetPlugin, TaskPlugin));
    }
}
