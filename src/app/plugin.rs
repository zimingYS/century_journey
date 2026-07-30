//! 组装 App 层的核心状态与流程插件。

use bevy::prelude::*;

use crate::app::flow::GameFlowPlugin;
use crate::app::state::CoreStatePlugin;

/// App 层核心插件入口。
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CoreStatePlugin, GameFlowPlugin));
    }
}
