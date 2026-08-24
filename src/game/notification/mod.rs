//! 面向玩家的通用通知消息。
//!
//! 本模块只定义消息语义：谁发生了值得告知玩家的事情，谁就写入
//! [`PlayerNotification`]；具体呈现形式由 Client 层决定。

pub mod components;
mod plugin;

pub use components::{NotificationLevel, PlayerNotification};
pub use plugin::NotificationPlugin;
