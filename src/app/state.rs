//! 注册跨层共享的顶层应用状态，不承载具体界面或游戏规则。

use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

/// 初始化主菜单、加载和游戏内等顶层状态机。
pub struct CoreStatePlugin;

impl Plugin for CoreStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}
