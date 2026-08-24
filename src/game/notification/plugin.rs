//! 组装通知消息的注册。

use super::components::PlayerNotification;
use bevy::prelude::*;

/// 通知消息插件：只注册跨层消息，展示由客户端负责。
pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerNotification>();
    }
}
