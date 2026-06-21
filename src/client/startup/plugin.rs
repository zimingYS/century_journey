use bevy::prelude::*;

use crate::client::startup::setup::setup;

/// 客户端启动 Plugin — 注册所有 Startup 系统。
///
/// 负责：光源、相机设置、UI 根节点等一次性初始化。
pub struct ClientStartupPlugin;

impl Plugin for ClientStartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}
