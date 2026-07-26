use bevy::prelude::*;

use crate::app::plugin::CorePlugin;

/// App 层插件聚合入口。
///
/// App 负责状态、菜单和应用流程，不在此复制 Game 或 Client 的业务逻辑。
pub struct AppPluginGroup;

impl Plugin for AppPluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins(CorePlugin);
    }
}
