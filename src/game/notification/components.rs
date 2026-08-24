//! 定义面向玩家的通用通知消息。
//!
//! Game 层各领域在值得让玩家知晓的事情发生时写入该消息，
//! 呈现形式（Toast、界面提示等）由 Client 层自行决定。

use bevy::prelude::*;

/// 通知级别，客户端据此选择强调色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationLevel {
    /// 常规信息。
    #[default]
    Info,
    /// 操作成功确认。
    Success,
    /// 需要注意的失败或异常。
    Warning,
}

/// 请求向玩家展示一条通知。
#[derive(Message, Debug, Clone)]
pub struct PlayerNotification {
    /// 面向玩家的中文通知文本。
    pub text: String,
    /// 通知级别。
    pub level: NotificationLevel,
}
