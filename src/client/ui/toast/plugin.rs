//! 组装 Toast 的资源与系统。

use bevy::prelude::*;

use super::queue::ToastQueue;
use super::systems::{push_toast_system, spawn_toast_root_system, update_toast_system};

/// 通知 Toast 插件：Startup 生成常驻堆叠容器，运行期消费通知消息。
pub struct ToastPlugin;

impl Plugin for ToastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToastQueue>()
            .add_systems(Startup, spawn_toast_root_system)
            .add_systems(Update, (push_toast_system, update_toast_system));
    }
}
