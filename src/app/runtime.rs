use bevy::prelude::*;

use crate::app::plugin_group::AppPluginGroup;
use crate::client::plugin_group::ClientPluginGroup;
use crate::content::plugin_group::ContentPluginGroup;
use crate::engine::plugin_group::EnginePluginGroup;
use crate::game::plugin_group::GamePluginGroup;

/// 当前单机客户端的运行时总装配入口。
///
/// 具体层级通过各自的聚合插件注册，客户端应用只需要依赖这一层入口。
pub struct ClientRuntimePluginGroup;

impl Plugin for ClientRuntimePluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EnginePluginGroup,
            ContentPluginGroup,
            GamePluginGroup,
            AppPluginGroup,
            ClientPluginGroup,
        ));
    }
}
