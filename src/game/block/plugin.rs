//! 组装 Game 层方块行为注册表与初始化系统。

use crate::game::block::behavior_registry::{BlockBehaviorRegistry, init_behavior_registry_system};
use bevy::prelude::*;

/// 组装方块行为注册表，并把内置行为绑定到内容声明的行为标识。
///
/// 行为实现（如 falling）在 `behaviors` 子模块，本插件只负责资源装配与
/// 首次启动时的内置行为注册。
pub struct BlockBehaviorPlugin;

impl Plugin for BlockBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockBehaviorRegistry>()
            .add_systems(Startup, init_behavior_registry_system);
    }
}
