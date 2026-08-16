//! 组织 Game 层方块行为实现，不承担数据定义和渲染职责。

pub mod behavior_dispatch;
pub mod behavior_registry;
pub mod behaviors;
mod plugin;

pub use behavior_registry::{BlockBehaviorRegistry, init_behavior_registry_system};
pub use plugin::BlockBehaviorPlugin;
