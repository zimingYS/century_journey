//! 通知 Toast UI：屏幕右上角弹出、自动淡出消失的通知堆叠。
//!
//! 消费 Game 层的 [`PlayerNotification`] 消息；同屏展示条数有上限，
//! 超出部分排队等待，队列溢出时丢弃最旧通知。

pub mod components;
mod plugin;
mod queue;
mod systems;

pub use plugin::ToastPlugin;
