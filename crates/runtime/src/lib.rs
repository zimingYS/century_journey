//! cj-runtime —— sim ⇄ ECS 适配层
//!
//! 规则：唯一允许把 simulation 接到 ECS 的地方（§6.1）。

pub mod plugin;
pub mod world_data;

pub use plugin::RuntimePlugin;
